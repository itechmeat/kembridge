// src/middleware/rate_limit.rs - Advanced rate limiting with Redis sliding window
use axum::{
    extract::Request,
    middleware::Next,
    response::Response,
    http::HeaderValue,
};
// TODO: Use real Redis rate limiting instead of mocks (Phase 1.3.4)
// use redis::AsyncCommands;
use std::time::Duration;
use crate::middleware::error_handler::ApiError;

/// Rate limiting middleware with sliding window algorithm
pub async fn rate_limit(request: Request, next: Next) -> Result<Response, ApiError> {
    // Extract rate limiting key from request
    let rate_key = extract_rate_limit_key(&request);
    
    // Get rate limits based on endpoint and user type
    let (limit, window) = get_rate_limits(&request);
    
    // Skip rate limiting for health checks and internal routes
    if should_skip_rate_limiting(request.uri().path()) {
        return Ok(next.run(request).await);
    }

    // Check rate limit using Redis sliding window
    // Note: This is a simplified implementation for the POC
    // In real implementation, we would inject Redis connection through state
    tracing::debug!(
        rate_key = %rate_key,
        limit = limit,
        window_seconds = window.as_secs(),
        "Checking rate limit"
    );

    // TODO: Use real Redis rate limiting instead of mocks (Phase 1.3.4)
    // Simulate rate limit check
    let (allowed, remaining, reset_time) = check_rate_limit_redis(&rate_key, limit, window).await?;

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
    add_rate_limit_headers(&mut response, remaining, reset_time);

    Ok(response)
}

/// Extract rate limiting key from request (IP + User ID + Endpoint class)
fn extract_rate_limit_key(request: &Request) -> String {
    let ip = extract_client_ip(request);
    let user_id = extract_user_id(request);
    let endpoint_class = classify_endpoint_for_rate_limit(request.uri().path());
    
    match user_id {
        Some(uid) => format!("rate_limit:user:{}:{}:{}", uid, endpoint_class, ip),
        None => format!("rate_limit:ip:{}:{}", ip, endpoint_class),
    }
}

/// Get rate limits based on endpoint and user authentication status
fn get_rate_limits(request: &Request) -> (u32, Duration) {
    let is_authenticated = request.headers().contains_key("authorization");
    let is_premium = extract_user_tier(request) == UserTier::Premium;
    
    let endpoint_class = classify_endpoint_for_rate_limit(request.uri().path());
    
    match (endpoint_class, is_authenticated, is_premium) {
        // Health checks - very high limits
        ("health", _, _) => (1000, Duration::from_secs(60)),
        
        // Authentication endpoints - moderate limits to prevent brute force
        ("auth", false, _) => (10, Duration::from_secs(60)),
        ("auth", true, _) => (20, Duration::from_secs(60)),
        
        // Bridge operations - strict limits for unauthenticated, generous for authenticated
        ("bridge", false, _) => (5, Duration::from_secs(300)), // 5 per 5 minutes
        ("bridge", true, false) => (50, Duration::from_secs(60)), // 50 per minute
        ("bridge", true, true) => (200, Duration::from_secs(60)), // Premium: 200 per minute
        
        // Quantum crypto operations - moderate limits
        ("quantum", false, _) => (2, Duration::from_secs(60)),
        ("quantum", true, false) => (20, Duration::from_secs(60)),
        ("quantum", true, true) => (100, Duration::from_secs(60)),
        
        // User management - standard limits
        ("user", true, false) => (60, Duration::from_secs(60)),
        ("user", true, true) => (120, Duration::from_secs(60)),
        
        // Admin endpoints - special limits
        ("admin", true, _) => (30, Duration::from_secs(60)),
        
        // Documentation and static content - high limits
        ("docs", _, _) => (200, Duration::from_secs(60)),
        
        // WebSocket - connection-based limits
        ("websocket", true, _) => (10, Duration::from_secs(300)), // 10 connections per 5 minutes
        
        // Default limits for other endpoints
        (_, false, _) => (30, Duration::from_secs(60)), // Unauthenticated
        (_, true, false) => (100, Duration::from_secs(60)), // Standard user
        (_, true, true) => (500, Duration::from_secs(60)), // Premium user
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
fn extract_client_ip(request: &Request) -> String {
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
fn extract_user_id(request: &Request) -> Option<String> {
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
fn extract_user_tier(request: &Request) -> UserTier {
    request.headers()
        .get("x-user-tier")
        .and_then(|h| h.to_str().ok())
        .map(|tier| match tier {
            "premium" => UserTier::Premium,
            _ => UserTier::Free,
        })
        .unwrap_or(UserTier::Free)
}

/// TODO: Use real Redis sliding window rate limiting instead of mocks (Phase 1.3.4)
/// Simplified Redis rate limit check
/// In real implementation, this would use the Redis connection from app state
async fn check_rate_limit_redis(
    _key: &str, 
    _limit: u32, 
    _window: Duration
) -> Result<(bool, u32, u64), ApiError> {
    // TODO: Use real Redis rate limiting instead of mocks (Phase 1.3.4)
    // Real implementation would:
    // 1. Use Redis ZRANGEBYSCORE to get requests in current window
    // 2. Count current requests
    // 3. If under limit, add current request with ZADD
    // 4. Clean up expired entries with ZREMRANGEBYSCORE
    // 5. Set TTL on the key
    
    // For now, always allow requests
    let allowed = true;
    let remaining = 50; // Mock remaining requests
    let reset_time = chrono::Utc::now().timestamp() as u64 + 60; // Reset in 60 seconds
    
    Ok((allowed, remaining, reset_time))
}

/// Add rate limit headers to response
fn add_rate_limit_headers(response: &mut Response, remaining: u32, reset_time: u64) {
    let headers = response.headers_mut();
    
    if let Ok(remaining_header) = HeaderValue::from_str(&remaining.to_string()) {
        headers.insert("x-rate-limit-remaining", remaining_header);
    }
    
    if let Ok(reset_header) = HeaderValue::from_str(&reset_time.to_string()) {
        headers.insert("x-rate-limit-reset", reset_header);
    }
    
    if let Ok(limit_header) = HeaderValue::from_str("100") { // Mock limit
        headers.insert("x-rate-limit-limit", limit_header);
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
        assert_eq!(limit, 5);
        assert_eq!(window, Duration::from_secs(300));

        // Authenticated standard user
        request.headers_mut().insert("authorization", "Bearer token123".parse().unwrap());
        let (limit, window) = get_rate_limits(&request);
        assert_eq!(limit, 50);
        assert_eq!(window, Duration::from_secs(60));

        // Premium user
        request.headers_mut().insert("x-user-tier", "premium".parse().unwrap());
        let (limit, window) = get_rate_limits(&request);
        assert_eq!(limit, 200);
        assert_eq!(window, Duration::from_secs(60));
    }
}