// src/middleware/rate_limit.rs - Advanced rate limiting with Redis sliding window
use axum::{
    extract::State,
    middleware::Next,
    response::Response,
    http::{Request, HeaderValue},
    body::Body,
};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use uuid::Uuid;
use crate::{middleware::error_handler::ApiError, state::AppState, constants::*};

/// Rate limiting middleware with sliding window algorithm and Redis
pub async fn rate_limit_with_state(
    State(app_state): State<AppState>,
    request: Request<Body>,
    next: Next,
) -> Result<Response, ApiError> {
    // Extract rate limiting key from request
    let rate_key = extract_rate_limit_key(&request);
    
    // Get rate limits based on endpoint and user type
    let (limit, window) = get_rate_limits(&request);
    
    // Skip rate limiting for health checks and internal routes
    if should_skip_rate_limiting(request.uri().path()) {
        return Ok(next.run(request).await);
    }

    // Check rate limit using RateLimitService
    tracing::debug!(
        rate_key = %rate_key,
        limit = limit,
        window_seconds = window.as_secs(),
        "Checking rate limit with RateLimitService"
    );

    let endpoint_class = classify_endpoint_for_rate_limit(request.uri().path());
    let user_id = extract_user_id(&request);
    let ip_address = extract_client_ip(&request);

    // Use RateLimitService for comprehensive rate limiting with statistics
    let result = app_state.rate_limit_service.check_rate_limit(
        &rate_key,
        limit,
        window,
        endpoint_class,
        user_id.as_deref(),
        &ip_address,
    ).await?;

    let allowed = result.allowed;
    let remaining = result.remaining;
    let reset_time = result.reset_time;

    if !allowed {
        tracing::warn!(
            rate_key = %rate_key,
            limit = limit,
            "Rate limit exceeded"
        );

        return Err(ApiError::RateLimit { 
            limit, 
            window: window.as_secs() 
        });
    }

    // Run the request
    let mut response = next.run(request).await;

    // Add rate limit headers to response
    add_rate_limit_headers(&mut response, limit, remaining, reset_time);

    Ok(response)
}

/// TODO (MOCK WARNING): Legacy rate limiting middleware (fallback)
/// Deprecated: Use rate_limit_with_state instead
pub async fn rate_limit(request: Request<Body>, next: Next) -> Result<Response, ApiError> {
    // For backward compatibility, skip rate limiting if no Redis available
    tracing::warn!("Using legacy rate limiting without Redis - requests not actually limited");
    Ok(next.run(request).await)
}

/// Extract rate limiting key from request (IP + User ID + Endpoint class)
fn extract_rate_limit_key(request: &Request<Body>) -> String {
    let ip = extract_client_ip(request);
    let user_id = extract_user_id(request);
    let endpoint_class = classify_endpoint_for_rate_limit(request.uri().path());
    
    match user_id {
        Some(uid) => format!("rate_limit:user:{}:{}:{}", uid, endpoint_class, ip),
        None => format!("rate_limit:ip:{}:{}", ip, endpoint_class),
    }
}

/// Get rate limits based on endpoint and user authentication status
fn get_rate_limits<T>(request: &Request<T>) -> (u32, Duration) {
    let is_authenticated = request.headers().contains_key("authorization");
    let is_premium = extract_user_tier(request) == UserTier::Premium;
    
    let endpoint_class = classify_endpoint_for_rate_limit(request.uri().path());
    
    match (endpoint_class, is_authenticated, is_premium) {
        // Health checks - very high limits
        ("health", _, _) => (RATE_LIMIT_HEALTH_LIMIT, Duration::from_secs(RATE_LIMIT_HEALTH_WINDOW_SEC)),
        
        // Authentication endpoints - moderate limits to prevent brute force
        ("auth", false, _) => (RATE_LIMIT_AUTH_UNAUTH_LIMIT, Duration::from_secs(RATE_LIMIT_AUTH_WINDOW_SEC)),
        ("auth", true, _) => (RATE_LIMIT_AUTH_AUTH_LIMIT, Duration::from_secs(RATE_LIMIT_AUTH_WINDOW_SEC)),
        
        // Bridge operations - strict limits for unauthenticated, generous for authenticated
        ("bridge", false, _) => (RATE_LIMIT_BRIDGE_UNAUTH_LIMIT, Duration::from_secs(RATE_LIMIT_BRIDGE_UNAUTH_WINDOW_SEC)),
        ("bridge", true, false) => (RATE_LIMIT_BRIDGE_AUTH_LIMIT, Duration::from_secs(RATE_LIMIT_BRIDGE_WINDOW_SEC)),
        ("bridge", true, true) => (RATE_LIMIT_BRIDGE_PREMIUM_LIMIT, Duration::from_secs(RATE_LIMIT_BRIDGE_WINDOW_SEC)),
        
        // Quantum crypto operations - moderate limits
        ("quantum", false, _) => (RATE_LIMIT_QUANTUM_UNAUTH_LIMIT, Duration::from_secs(RATE_LIMIT_QUANTUM_WINDOW_SEC)),
        ("quantum", true, false) => (RATE_LIMIT_QUANTUM_AUTH_LIMIT, Duration::from_secs(RATE_LIMIT_QUANTUM_WINDOW_SEC)),
        ("quantum", true, true) => (RATE_LIMIT_QUANTUM_PREMIUM_LIMIT, Duration::from_secs(RATE_LIMIT_QUANTUM_WINDOW_SEC)),
        
        // User management - standard limits
        ("user", true, false) => (RATE_LIMIT_USER_AUTH_LIMIT, Duration::from_secs(RATE_LIMIT_USER_WINDOW_SEC)),
        ("user", true, true) => (RATE_LIMIT_USER_PREMIUM_LIMIT, Duration::from_secs(RATE_LIMIT_USER_WINDOW_SEC)),
        
        // Admin endpoints - special limits
        ("admin", true, _) => (RATE_LIMIT_ADMIN_LIMIT, Duration::from_secs(RATE_LIMIT_ADMIN_WINDOW_SEC)),
        
        // Documentation and static content - high limits
        ("docs", _, _) => (RATE_LIMIT_DOCS_LIMIT, Duration::from_secs(RATE_LIMIT_DOCS_WINDOW_SEC)),
        
        // WebSocket - connection-based limits
        ("websocket", true, _) => (RATE_LIMIT_WEBSOCKET_LIMIT, Duration::from_secs(RATE_LIMIT_WEBSOCKET_WINDOW_SEC)),
        
        // Default limits for other endpoints
        (_, false, _) => (RATE_LIMIT_DEFAULT_UNAUTH_LIMIT, Duration::from_secs(RATE_LIMIT_DEFAULT_WINDOW_SEC)),
        (_, true, false) => (RATE_LIMIT_DEFAULT_AUTH_LIMIT, Duration::from_secs(RATE_LIMIT_DEFAULT_WINDOW_SEC)),
        (_, true, true) => (RATE_LIMIT_DEFAULT_PREMIUM_LIMIT, Duration::from_secs(RATE_LIMIT_DEFAULT_WINDOW_SEC)),
    }
}

/// Check if rate limiting should be skipped for this path
fn should_skip_rate_limiting(path: &str) -> bool {
    matches!(path, "/metrics" | "/ready")
}

/// Classify endpoint for rate limiting purposes
fn classify_endpoint_for_rate_limit(path: &str) -> &'static str {
    match path {
        "/health" | "/ready" | "/metrics" => "health",
        path if path.starts_with("/api/v1/auth") => "auth",
        path if path.starts_with("/api/v1/bridge") => "bridge",
        path if path.starts_with("/api/v1/quantum") => "quantum",
        path if path.starts_with("/api/v1/user") => "user",
        path if path.starts_with("/api/v1/admin") => "admin",
        path if path.starts_with("/docs") => "docs",
        "/ws" => "websocket",
        _ => "general",
    }
}

/// Extract client IP address with proxy support
fn extract_client_ip(request: &Request<Body>) -> String {
    // Check X-Forwarded-For header first (proxy support)
    if let Some(forwarded) = request.headers()
        .get("x-forwarded-for")
        .and_then(|h| h.to_str().ok()) 
    {
        // Take the first IP in the chain
        if let Some(first_ip) = forwarded.split(',').next() {
            return first_ip.trim().to_string();
        }
    }
    
    // Check X-Real-IP header
    if let Some(real_ip) = request.headers()
        .get("x-real-ip")
        .and_then(|h| h.to_str().ok()) 
    {
        return real_ip.to_string();
    }
    
    // Fallback to "unknown" - in real deployment, connection info would be available
    "unknown".to_string()
}

/// Extract user ID from JWT token in authorization header
fn extract_user_id(request: &Request<Body>) -> Option<String> {
    request.headers()
        .get("x-user-id")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string())
}

#[derive(Debug, PartialEq)]
enum UserTier {
    Free,
    Premium,
}

/// Extract user tier from headers (set by auth middleware)
fn extract_user_tier<T>(request: &Request<T>) -> UserTier {
    request.headers()
        .get("x-user-tier")
        .and_then(|h| h.to_str().ok())
        .map(|tier| match tier {
            "premium" => UserTier::Premium,
            _ => UserTier::Free,
        })
        .unwrap_or(UserTier::Free)
}


/// Add rate limit headers to response
fn add_rate_limit_headers(response: &mut Response, limit: u32, remaining: u32, reset_time: u64) {
    let headers = response.headers_mut();
    
    if let Ok(limit_header) = HeaderValue::from_str(&limit.to_string()) {
        headers.insert("x-rate-limit-limit", limit_header);
    }
    
    if let Ok(remaining_header) = HeaderValue::from_str(&remaining.to_string()) {
        headers.insert("x-rate-limit-remaining", remaining_header);
    }
    
    if let Ok(reset_header) = HeaderValue::from_str(&reset_time.to_string()) {
        headers.insert("x-rate-limit-reset", reset_header);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::{Method, Uri};

    #[test]
    fn test_classify_endpoint_for_rate_limit() {
        assert_eq!(classify_endpoint_for_rate_limit("/health"), "health");
        assert_eq!(classify_endpoint_for_rate_limit("/api/v1/auth/login"), "auth");
        assert_eq!(classify_endpoint_for_rate_limit("/api/v1/bridge/swap"), "bridge");
        assert_eq!(classify_endpoint_for_rate_limit("/api/v1/quantum/generate"), "quantum");
        assert_eq!(classify_endpoint_for_rate_limit("/docs/swagger"), "docs");
        assert_eq!(classify_endpoint_for_rate_limit("/unknown"), "general");
    }

    #[test]
    fn test_should_skip_rate_limiting() {
        assert!(should_skip_rate_limiting("/metrics"));
        assert!(should_skip_rate_limiting("/ready"));
        assert!(!should_skip_rate_limiting("/health"));
        assert!(!should_skip_rate_limiting("/api/v1/bridge/swap"));
    }

    #[test]
    fn test_extract_user_tier() {
        let mut request = Request::builder()
            .method(Method::GET)
            .uri("/test")
            .body(())
            .unwrap();

        // No tier header
        assert_eq!(extract_user_tier(&request), UserTier::Free);

        // Premium tier
        request.headers_mut().insert("x-user-tier", "premium".parse().unwrap());
        assert_eq!(extract_user_tier(&request), UserTier::Premium);

        // Unknown tier defaults to free
        request.headers_mut().insert("x-user-tier", "unknown".parse().unwrap());
        assert_eq!(extract_user_tier(&request), UserTier::Free);
    }

    #[test]
    fn test_get_rate_limits() {
        let mut request = Request::builder()
            .method(Method::POST)
            .uri("/api/v1/bridge/swap")
            .body(())
            .unwrap();

        // Unauthenticated bridge request
        let (limit, window) = get_rate_limits(&request);
        assert_eq!(limit, RATE_LIMIT_BRIDGE_UNAUTH_LIMIT);
        assert_eq!(window, Duration::from_secs(RATE_LIMIT_BRIDGE_UNAUTH_WINDOW_SEC));

        // Authenticated standard user
        request.headers_mut().insert("authorization", "Bearer token123".parse().unwrap());
        let (limit, window) = get_rate_limits(&request);
        assert_eq!(limit, RATE_LIMIT_BRIDGE_AUTH_LIMIT);
        assert_eq!(window, Duration::from_secs(RATE_LIMIT_BRIDGE_WINDOW_SEC));

        // Premium user
        request.headers_mut().insert("x-user-tier", "premium".parse().unwrap());
        let (limit, window) = get_rate_limits(&request);
        assert_eq!(limit, RATE_LIMIT_BRIDGE_PREMIUM_LIMIT);
        assert_eq!(window, Duration::from_secs(RATE_LIMIT_BRIDGE_WINDOW_SEC));
    }
}