// src/middleware/rate_limit.rs - Advanced rate limiting with Redis sliding window
use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
    http::HeaderValue,
};
use redis::AsyncCommands;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use uuid::Uuid;
use crate::{middleware::error_handler::ApiError, state::AppState};

/// Rate limiting middleware with sliding window algorithm and Redis
pub async fn rate_limit_with_state(
    State(app_state): State<AppState>,
    request: Request,
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

    // Check rate limit using Redis sliding window
    tracing::debug!(
        rate_key = %rate_key,
        limit = limit,
        window_seconds = window.as_secs(),
        "Checking rate limit with Redis"
    );

    // Real Redis rate limiting implementation using regular ConnectionManager
    let (allowed, remaining, reset_time) = check_rate_limit_redis_simple(
        &app_state.redis, 
        &rate_key, 
        limit, 
        window
    ).await?;

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
pub async fn rate_limit(request: Request, next: Next) -> Result<Response, ApiError> {
    // For backward compatibility, skip rate limiting if no Redis available
    tracing::warn!("Using legacy rate limiting without Redis - requests not actually limited");
    Ok(next.run(request).await)
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

/// Real Redis sliding window rate limiting implementation using ConnectionManager
async fn check_rate_limit_redis_simple(
    redis_manager: &redis::aio::ConnectionManager,
    key: &str,
    limit: u32,
    window: Duration,
) -> Result<(bool, u32, u64), ApiError> {
    use redis::AsyncCommands;
    
    let mut conn = redis_manager.clone();
    
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    let window_start = now - window.as_secs();
    let window_end = now;
    
    // Lua script for atomic sliding window rate limiting
    let script = r#"
        local key = KEYS[1]
        local window_start = tonumber(ARGV[1])
        local now = tonumber(ARGV[2])
        local limit = tonumber(ARGV[3])
        local window_seconds = tonumber(ARGV[4])
        local request_id = ARGV[5]
        
        -- Remove expired entries
        redis.call('ZREMRANGEBYSCORE', key, '-inf', window_start)
        
        -- Count current requests
        local current_count = redis.call('ZCARD', key)
        
        -- Check rate limit
        if current_count >= limit then
            return {0, 0, now + window_seconds}
        end
        
        -- Add current request
        redis.call('ZADD', key, now, request_id)
        
        -- Set TTL
        redis.call('EXPIRE', key, window_seconds)
        
        local remaining = limit - current_count - 1
        return {1, remaining, now + window_seconds}
    "#;
    
    let request_id = format!("{}:{}", now, Uuid::new_v4());
    
    // Execute script using ConnectionManager
    let result: Vec<u64> = redis::cmd("EVAL")
        .arg(script)
        .arg(1) // Number of keys
        .arg(key)
        .arg(window_start)
        .arg(now)
        .arg(limit)
        .arg(window.as_secs())
        .arg(&request_id)
        .query_async(&mut conn)
        .await
        .map_err(|e| ApiError::Internal(format!("Redis script execution failed: {}", e)))?;
    
    let allowed = result[0] == 1;
    let remaining = result[1] as u32;
    let reset_time = result[2];
    
    tracing::debug!(
        key = %key,
        allowed = allowed,
        limit = limit,
        remaining = remaining,
        "Rate limit check completed"
    );
    
    Ok((allowed, remaining, reset_time))
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