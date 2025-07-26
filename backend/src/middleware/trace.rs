// src/middleware/trace.rs - Comprehensive observability with Rust 1.88+ features
use axum::http::{Request, Response};
use tracing::{info_span, Span, Level};
use uuid::Uuid;
use std::time::Instant;

/// Create comprehensive span for HTTP requests with modern Rust patterns
pub fn make_span<B>(request: &Request<B>) -> Span {
    let request_id = request
        .headers()
        .get("x-request-id")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    // Extract user context if available
    let user_context = extract_user_context(request);
    
    // Extract client information
    let client_info = extract_client_info(request);

    info_span!(
        "http_request",
        // Request identification
        method = %request.method(),
        uri = %request.uri(),
        version = ?request.version(),
        request_id = %request_id,
        
        // User context
        user_id = user_context.user_id,
        wallet_address = user_context.wallet_address,
        session_id = user_context.session_id,
        
        // Client information
        user_agent = client_info.user_agent,
        client_ip = client_info.client_ip,
        forwarded_for = client_info.forwarded_for,
        
        // Security context
        quantum_signature_present = request.headers().contains_key("x-quantum-signature"),
        auth_method = extract_auth_method(request),
        
        // Request characteristics
        content_type = request.headers()
            .get("content-type")
            .and_then(|h| h.to_str().ok())
            .unwrap_or("unknown"),
        content_length = request.headers()
            .get("content-length")
            .and_then(|h| h.to_str().ok())
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(0),
        
        // Timing
        started_at = %chrono::Utc::now().to_rfc3339(),
        
        // Placeholder fields that will be filled during the request lifecycle
        status_code = tracing::field::Empty,
        response_time_ms = tracing::field::Empty,
        response_size = tracing::field::Empty,
        error_code = tracing::field::Empty,
    )
}

/// Called when request starts processing
pub fn on_request<B>(request: &Request<B>, span: &Span) {
    // Record additional request context that wasn't available at span creation
    span.record("request_start_time", &Instant::now().elapsed().as_nanos());
    
    // Log request initiation with appropriate level based on endpoint
    let log_level = determine_log_level(request.uri().path());
    
    match log_level {
        Level::DEBUG => tracing::debug!("🔄 Processing request"),
        Level::INFO => tracing::info!("🔄 Processing request"),
        _ => tracing::trace!("🔄 Processing request"),
    }

    // Record metrics (use cloned values for static lifetime requirement)
    let method = request.method().as_str().to_string();
    let endpoint = classify_endpoint(request.uri().path()).to_string();
    
    metrics::counter!("http_requests_total", 
        "method" => method,
        "endpoint" => endpoint
    ).increment(1);
}

/// Called when response is ready
pub fn on_response<B>(response: &Response<B>, latency: std::time::Duration, span: &Span) {
    let status = response.status();
    let response_time_ms = latency.as_millis();
    
    // Record span fields
    span.record("status_code", status.as_u16());
    span.record("response_time_ms", response_time_ms);
    
    // Record response size if available
    if let Some(content_length) = response.headers()
        .get("content-length")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.parse::<u64>().ok()) 
    {
        span.record("response_size", content_length);
    }

    // Log completion with contextual emoji and level
    let (emoji, level, message) = match status.as_u16() {
        200..=299 => ("✅", Level::INFO, "Request completed successfully"),
        300..=399 => ("🔄", Level::INFO, "Request redirected"),
        400..=499 => ("⚠️", Level::WARN, "Client error"),
        500..=599 => ("❌", Level::ERROR, "Server error"),
        _ => ("🤔", Level::WARN, "Unusual status code"),
    };

    // Use Rust 1.88+ let chains for elegant conditional logging
    if let Some(error_header) = response.headers().get("x-error-code")
        && let Ok(error_code) = error_header.to_str() {
        span.record("error_code", error_code);
    }

    match level {
        Level::ERROR => tracing::error!(
            "{} {} - status: {}, latency: {}ms",
            emoji, message, status, response_time_ms
        ),
        Level::WARN => tracing::warn!(
            "{} {} - status: {}, latency: {}ms", 
            emoji, message, status, response_time_ms
        ),
        Level::INFO => tracing::info!(
            "{} {} - status: {}, latency: {}ms",
            emoji, message, status, response_time_ms
        ),
        _ => tracing::debug!(
            "{} {} - status: {}, latency: {}ms",
            emoji, message, status, response_time_ms
        ),
    }

    // Record metrics with owned strings for static lifetime requirement
    let method = response.headers()
        .get("x-original-method")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("unknown")
        .to_string();
    let status_class_hist = status_class(status.as_u16()).to_string();
    let endpoint = response.headers()
        .get("x-endpoint-class")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("unknown")
        .to_string();
    
    metrics::histogram!("http_request_duration_ms",
        "method" => method,
        "status_class" => status_class_hist,
        "endpoint" => endpoint
    ).record(response_time_ms as f64);

    // Record metrics with owned strings for static lifetime requirement
    let status_code = status.as_str().to_string();
    let status_class = status_class(status.as_u16()).to_string();
    
    metrics::counter!("http_responses_total",
        "status_code" => status_code,
        "status_class" => status_class
    ).increment(1);
}

// Helper structures and functions

#[derive(Debug, Default)]
struct UserContext {
    user_id: Option<String>,
    wallet_address: Option<String>, 
    session_id: Option<String>,
}

#[derive(Debug, Default)]
struct ClientInfo {
    user_agent: Option<String>,
    client_ip: Option<String>,
    forwarded_for: Option<String>,
}

fn extract_user_context<B>(request: &Request<B>) -> UserContext {
    UserContext {
        user_id: request.headers()
            .get("x-user-id")
            .and_then(|h| h.to_str().ok())
            .map(|s| s.to_string()),
        wallet_address: request.headers()
            .get("x-wallet-address") 
            .and_then(|h| h.to_str().ok())
            .map(|s| s.to_string()),
        session_id: request.headers()
            .get("x-session-id")
            .and_then(|h| h.to_str().ok())
            .map(|s| s.to_string()),
    }
}

fn extract_client_info<B>(request: &Request<B>) -> ClientInfo {
    ClientInfo {
        user_agent: request.headers()
            .get("user-agent")
            .and_then(|h| h.to_str().ok())
            .map(|s| s.to_string()),
        client_ip: request.headers()
            .get("x-real-ip")
            .and_then(|h| h.to_str().ok())
            .map(|s| s.to_string()),
        forwarded_for: request.headers()
            .get("x-forwarded-for")
            .and_then(|h| h.to_str().ok())
            .map(|s| s.to_string()),
    }
}

fn extract_auth_method<B>(request: &Request<B>) -> Option<&'static str> {
    if request.headers().contains_key("x-quantum-signature") {
        Some("quantum")
    } else if let Some(auth_header) = request.headers().get("authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if auth_str.starts_with("Bearer ") {
                Some("jwt")
            } else if auth_str.starts_with("Basic ") {
                Some("basic")
            } else {
                Some("unknown")
            }
        } else {
            Some("malformed")
        }
    } else {
        None
    }
}

fn determine_log_level(path: &str) -> Level {
    match path {
        // Health checks and metrics - trace level
        "/health" | "/ready" | "/metrics" => Level::TRACE,
        
        // Static assets and docs - debug level  
        path if path.starts_with("/docs") || path.starts_with("/static") => Level::DEBUG,
        
        // API endpoints - info level
        path if path.starts_with("/api/") => Level::INFO,
        
        // WebSocket connections - info level
        "/ws" => Level::INFO,
        
        // Everything else - debug level
        _ => Level::DEBUG,
    }
}

fn classify_endpoint(path: &str) -> &'static str {
    match path {
        "/health" | "/ready" | "/metrics" => "health",
        path if path.starts_with("/api/v1/auth") => "auth",
        path if path.starts_with("/api/v1/bridge") => "bridge", 
        path if path.starts_with("/api/v1/quantum") => "quantum",
        path if path.starts_with("/api/v1/user") => "user",
        path if path.starts_with("/api/v1/admin") => "admin",
        path if path.starts_with("/docs") => "docs",
        "/ws" => "websocket",
        _ => "other",
    }
}

fn status_class(status_code: u16) -> &'static str {
    match status_code {
        100..=199 => "1xx",
        200..=299 => "2xx", 
        300..=399 => "3xx",
        400..=499 => "4xx",
        500..=599 => "5xx",
        _ => "unknown",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::{Method, Uri};

    #[test]
    fn test_classify_endpoint() {
        assert_eq!(classify_endpoint("/health"), "health");
        assert_eq!(classify_endpoint("/api/v1/auth/login"), "auth");
        assert_eq!(classify_endpoint("/api/v1/bridge/swap"), "bridge");
        assert_eq!(classify_endpoint("/api/v1/quantum/generate"), "quantum");
        assert_eq!(classify_endpoint("/docs/swagger"), "docs");
        assert_eq!(classify_endpoint("/unknown"), "other");
    }

    #[test] 
    fn test_status_class() {
        assert_eq!(status_class(200), "2xx");
        assert_eq!(status_class(404), "4xx");
        assert_eq!(status_class(500), "5xx");
        assert_eq!(status_class(999), "unknown");
    }

    #[test]
    fn test_determine_log_level() {
        assert_eq!(determine_log_level("/health"), Level::TRACE);
        assert_eq!(determine_log_level("/api/v1/bridge/swap"), Level::INFO);
        assert_eq!(determine_log_level("/docs/swagger"), Level::DEBUG);
        assert_eq!(determine_log_level("/unknown"), Level::DEBUG);
    }

    #[test]
    fn test_extract_auth_method() {
        let mut request = Request::builder()
            .method(Method::GET)
            .uri("/test")
            .body(())
            .unwrap();

        // No auth header
        assert_eq!(extract_auth_method(&request), None);

        // JWT Bearer token
        request.headers_mut().insert("authorization", "Bearer token123".parse().unwrap());
        assert_eq!(extract_auth_method(&request), Some("jwt"));

        // Quantum signature
        request.headers_mut().insert("x-quantum-signature", "sig123".parse().unwrap());
        assert_eq!(extract_auth_method(&request), Some("quantum"));
    }
}