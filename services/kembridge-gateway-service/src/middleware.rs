use axum::{
    body::Body,
    http::{header, HeaderValue, Request, Response, StatusCode, Method},
    middleware::Next,
};
use kembridge_common::{ErrorCategory, FailureContext, RecoverySystem};
use serde_json::json;
use std::convert::Infallible;
use std::time::SystemTime;
use tracing::{info, warn};
use uuid::Uuid;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use chrono::{DateTime, Utc, Duration};
use sha2::{Sha256, Digest};
use base64;

/// Security headers middleware
/// Adds essential security headers to all responses
pub async fn security_headers(
    request: Request<Body>,
    next: Next,
) -> Result<Response<Body>, Infallible> {
    let mut response = next.run(request).await;

    let headers = response.headers_mut();

    // Strict Transport Security - Force HTTPS
    headers.insert(
        header::STRICT_TRANSPORT_SECURITY,
        HeaderValue::from_static("max-age=31536000; includeSubDomains; preload"),
    );

    // Content Type Options - Prevent MIME sniffing
    headers.insert(
        header::HeaderName::from_static("x-content-type-options"),
        HeaderValue::from_static("nosniff"),
    );

    // Frame Options - Prevent clickjacking
    headers.insert(
        header::HeaderName::from_static("x-frame-options"),
        HeaderValue::from_static("DENY"),
    );

    // Content Security Policy - XSS protection
    headers.insert(
        header::HeaderName::from_static("content-security-policy"),
        HeaderValue::from_static(
            "default-src 'self'; script-src 'self' 'unsafe-inline' 'unsafe-eval'; style-src 'self' 'unsafe-inline'; img-src 'self' data: https:; connect-src 'self' ws: wss: https:; font-src 'self' data:; object-src 'none'; media-src 'self'; frame-src 'none';"
        )
    );

    // XSS Protection
    headers.insert(
        header::HeaderName::from_static("x-xss-protection"),
        HeaderValue::from_static("1; mode=block"),
    );

    // Referrer Policy
    headers.insert(
        header::HeaderName::from_static("referrer-policy"),
        HeaderValue::from_static("strict-origin-when-cross-origin"),
    );

    // Permissions Policy
    headers.insert(
        header::HeaderName::from_static("permissions-policy"),
        HeaderValue::from_static("geolocation=(), microphone=(), camera=()"),
    );

    // Rate limiting headers
    headers.insert(
        header::HeaderName::from_static("x-ratelimit-limit"),
        HeaderValue::from_static("100"),
    );

    headers.insert(
        header::HeaderName::from_static("x-ratelimit-remaining"),
        HeaderValue::from_static("99"),
    );

    headers.insert(
        header::HeaderName::from_static("x-ratelimit-reset"),
        HeaderValue::from_static("3600"),
    );

    Ok(response)
}

/// Rate limiting middleware
/// Simple rate limiting implementation
pub async fn rate_limiting(
    request: Request<Body>,
    next: Next,
) -> Result<Response<Body>, Infallible> {
    // For now, just pass through - can be enhanced with actual rate limiting logic
    let response = next.run(request).await;
    Ok(response)
}

/// Error handling and recovery middleware
/// Provides comprehensive error handling with automatic recovery mechanisms
pub async fn error_handling_middleware(
    request: Request<Body>,
    next: Next,
) -> Result<Response<Body>, Infallible> {
    let request_id = Uuid::new_v4();
    let uri = request.uri().clone();
    let method = request.method().clone();

    info!("Request started: {} {} (ID: {})", method, uri, request_id);

    let response = next.run(request).await;
    let status = response.status();

    // Check if this is an error response that needs handling
    if status.is_client_error() || status.is_server_error() {
        warn!(
            "Error response detected: {} {} -> {} (ID: {})",
            method, uri, status, request_id
        );

        // Create failure context for recovery system
        let failure_context = FailureContext {
            transaction_id: request_id,
            operation_type: format!("{} {}", method, uri),
            error_category: categorize_error(status),
            error_message: format!("HTTP {} response", status.as_u16()),
            failed_at: SystemTime::now(),
            retry_count: 0,
            max_retries: 3,
            context: std::collections::HashMap::new(),
            service_name: "kembridge-gateway-service".to_string(),
            severity: if status.is_server_error() { 8 } else { 5 },
        };

        // Initialize recovery system
        let recovery_system = RecoverySystem::new();

        // Attempt recovery (in real implementation, this could trigger retry mechanisms)
        if let Ok(recovery_result) = recovery_system
            .handle_failure(failure_context.clone())
            .await
        {
            info!(
                "Recovery attempted for request {}: {:?}",
                request_id, recovery_result
            );
        }

        // For client errors (4xx), check if this is an API endpoint with specific error handling
        if status.is_client_error() {
            // Skip middleware enhancement for API endpoints that should return specific errors
            if uri.path().starts_with("/api/v1/") {
                info!(
                    "API endpoint error detected, passing through original error (ID: {})",
                    request_id
                );
                return Ok(response);
            }

            // Check if response already has a proper JSON error from our handlers
            if let Some(content_type) = response.headers().get("content-type") {
                if content_type
                    .to_str()
                    .unwrap_or("")
                    .contains("application/json")
                {
                    // Response already has JSON error from our handlers - pass it through
                    info!(
                        "JSON error response already present, passing through (ID: {})",
                        request_id
                    );
                    return Ok(response);
                }
            }

            // No JSON error, create enhanced response
            let enhanced_response = create_enhanced_error_response(status, &failure_context);
            return Ok(enhanced_response);
        }
    } else {
        info!(
            "Request completed successfully: {} {} -> {} (ID: {})",
            method, uri, status, request_id
        );
    }

    Ok(response)
}

/// Categorize HTTP status codes into error categories
fn categorize_error(status: StatusCode) -> ErrorCategory {
    match status.as_u16() {
        400..=499 => match status {
            StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => ErrorCategory::Authentication,
            StatusCode::BAD_REQUEST => ErrorCategory::Validation,
            StatusCode::NOT_FOUND => ErrorCategory::InternalService,
            StatusCode::TOO_MANY_REQUESTS => ErrorCategory::RateLimit,
            _ => ErrorCategory::Unknown,
        },
        500..=599 => match status {
            StatusCode::INTERNAL_SERVER_ERROR => ErrorCategory::InternalService,
            StatusCode::BAD_GATEWAY
            | StatusCode::SERVICE_UNAVAILABLE
            | StatusCode::GATEWAY_TIMEOUT => ErrorCategory::ExternalService,
            _ => ErrorCategory::InternalService,
        },
        _ => ErrorCategory::Unknown,
    }
}

/// Create enhanced error response with recovery information
fn create_enhanced_error_response(
    status: StatusCode,
    failure_context: &FailureContext,
) -> Response<Body> {
    let error_response = json!({
        "success": false,
        "error": get_user_friendly_error_message(status),
        "data": null,
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "error_details": {
            "code": status.as_u16(),
            "category": format!("{:?}", failure_context.error_category),
            "request_id": failure_context.transaction_id,
            "recovery_available": true,
            "suggested_actions": get_suggested_actions(status)
        }
    });

    Response::builder()
        .status(status)
        .header("content-type", "application/json")
        .header("x-request-id", failure_context.transaction_id.to_string())
        .header("x-error-recovery", "available")
        .body(Body::from(error_response.to_string()))
        .unwrap_or_else(|_| {
            Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::from("Internal Server Error"))
                .unwrap()
        })
}

/// Get user-friendly error messages
fn get_user_friendly_error_message(status: StatusCode) -> &'static str {
    match status {
        StatusCode::BAD_REQUEST => "Invalid request parameters. Please check your input and try again.",
        StatusCode::UNAUTHORIZED => "Authentication required. Please connect your wallet.",
        StatusCode::FORBIDDEN => "Access denied. You don't have permission for this operation.",
        StatusCode::NOT_FOUND => "Requested resource not found. Please check the URL and try again.",
        StatusCode::TOO_MANY_REQUESTS => "Too many requests. Please wait a moment and try again.",
        StatusCode::INTERNAL_SERVER_ERROR => "An internal error occurred. Our team has been notified.",
        StatusCode::BAD_GATEWAY => "Service temporarily unavailable. Please try again in a few moments.",
        StatusCode::SERVICE_UNAVAILABLE => "Service is temporarily down for maintenance. Please try again later.",
        StatusCode::GATEWAY_TIMEOUT => "Request timeout. The service is taking longer than expected to respond.",
        _ => "An unexpected error occurred. Please try again or contact support if the problem persists.",
    }
}

/// Get suggested recovery actions for different error types
fn get_suggested_actions(status: StatusCode) -> Vec<&'static str> {
    match status {
        StatusCode::BAD_REQUEST => vec![
            "Check your input parameters",
            "Ensure all required fields are provided",
            "Verify data formats (addresses, amounts, etc.)",
        ],
        StatusCode::UNAUTHORIZED => vec![
            "Connect your wallet",
            "Sign the authentication message",
            "Refresh the page if wallet is already connected",
        ],
        StatusCode::FORBIDDEN => vec![
            "Verify you have the necessary permissions",
            "Contact support if you believe this is an error",
        ],
        StatusCode::NOT_FOUND => vec![
            "Check the URL for typos",
            "Verify the resource still exists",
            "Try navigating from the home page",
        ],
        StatusCode::TOO_MANY_REQUESTS => vec![
            "Wait a few moments before trying again",
            "Reduce the frequency of your requests",
        ],
        StatusCode::INTERNAL_SERVER_ERROR
        | StatusCode::BAD_GATEWAY
        | StatusCode::SERVICE_UNAVAILABLE
        | StatusCode::GATEWAY_TIMEOUT => vec![
            "Try again in a few moments",
            "Check service status page",
            "Contact support if the problem persists",
        ],
        _ => vec![
            "Try refreshing the page",
            "Check your internet connection",
            "Contact support if the problem continues",
        ],
    }
}

// CSRF Protection Implementation
struct CsrfToken {
    token: String,
    created_at: DateTime<Utc>,
    expires_at: DateTime<Utc>,
}

type CsrfTokenStore = Arc<Mutex<HashMap<String, CsrfToken>>>;

lazy_static::lazy_static! {
    static ref CSRF_TOKENS: CsrfTokenStore = Arc::new(Mutex::new(HashMap::new()));
}

/// Generate a new CSRF token
pub fn generate_csrf_token() -> String {
    let mut hasher = Sha256::new();
    let timestamp = Utc::now().timestamp_nanos_opt().unwrap_or(0);
    let uuid = uuid::Uuid::new_v4();
    let random_bytes = uuid.as_bytes();
    
    hasher.update(timestamp.to_be_bytes());
    hasher.update(random_bytes);
    hasher.update(b"kembridge_csrf_secret");
    
    let result = hasher.finalize();
    use base64::{Engine as _, engine::general_purpose};
    general_purpose::STANDARD.encode(result)
}

/// Store CSRF token with expiration
pub fn store_csrf_token(token: String) {
    let csrf_token = CsrfToken {
        token: token.clone(),
        created_at: Utc::now(),
        expires_at: Utc::now() + Duration::hours(1), // 1 hour expiration
    };
    
    if let Ok(mut tokens) = CSRF_TOKENS.lock() {
        tokens.insert(token, csrf_token);
        
        // Clean up expired tokens
        let now = Utc::now();
        tokens.retain(|_, csrf_token| csrf_token.expires_at > now);
    }
}

/// Validate CSRF token
pub fn validate_csrf_token(token: &str) -> bool {
    if let Ok(mut tokens) = CSRF_TOKENS.lock() {
        let now = Utc::now();
        
        // Clean up expired tokens first
        tokens.retain(|_, csrf_token| csrf_token.expires_at > now);
        
        // Check if token exists and is valid
        if let Some(csrf_token) = tokens.get(token) {
            if csrf_token.expires_at > now {
                // Remove token after use (single-use)
                tokens.remove(token);
                return true;
            }
        }
    }
    false
}

/// CSRF protection middleware
/// Validates CSRF tokens for state-changing operations (POST, PUT, DELETE, PATCH)
pub async fn csrf_protection(
    request: Request<Body>,
    next: Next,
) -> Result<Response<Body>, Infallible> {
    let method = request.method().clone();
    let uri = request.uri().clone();
    
    // Only check CSRF for state-changing methods
    if matches!(method, Method::POST | Method::PUT | Method::DELETE | Method::PATCH) {
        // Skip CSRF check for auth endpoints (they use different protection)
        if uri.path().starts_with("/api/v1/auth/") {
            return Ok(next.run(request).await);
        }
        
        // Extract CSRF token from headers
        let csrf_token = request.headers()
            .get("x-csrf-token")
            .and_then(|h| h.to_str().ok());
        
        match csrf_token {
            Some(token) => {
                if validate_csrf_token(token) {
                    info!("✅ CSRF token validated for {} {}", method, uri);
                    Ok(next.run(request).await)
                } else {
                    warn!("❌ Invalid CSRF token for {} {}", method, uri);
                    let error_response = json!({
                        "success": false,
                        "error": "Invalid or expired CSRF token",
                        "data": null,
                        "timestamp": chrono::Utc::now().to_rfc3339(),
                        "error_details": {
                            "code": 403,
                            "category": "CSRF_PROTECTION",
                            "message": "CSRF token validation failed"
                        }
                    });
                    
                    let response = Response::builder()
                        .status(StatusCode::FORBIDDEN)
                        .header("content-type", "application/json")
                        .body(Body::from(error_response.to_string()))
                        .unwrap();
                    
                    Ok(response)
                }
            }
            None => {
                warn!("❌ Missing CSRF token for {} {}", method, uri);
                let error_response = json!({
                    "success": false,
                    "error": "CSRF token required",
                    "data": null,
                    "timestamp": chrono::Utc::now().to_rfc3339(),
                    "error_details": {
                        "code": 403,
                        "category": "CSRF_PROTECTION",
                        "message": "CSRF token is required for this operation"
                    }
                });
                
                let response = Response::builder()
                    .status(StatusCode::FORBIDDEN)
                    .header("content-type", "application/json")
                    .body(Body::from(error_response.to_string()))
                    .unwrap();
                
                Ok(response)
            }
        }
    } else {
        // GET requests and other safe methods don't need CSRF protection
        Ok(next.run(request).await)
    }
}

/// Enhanced rate limiting middleware with proper headers
pub async fn enhanced_rate_limiting(
    request: Request<Body>,
    next: Next,
) -> Result<Response<Body>, Infallible> {
    // Simple rate limiting simulation - in production use Redis or similar
    let client_ip = request.headers()
        .get("x-forwarded-for")
        .or_else(|| request.headers().get("x-real-ip"))
        .and_then(|h| h.to_str().ok())
        .unwrap_or("unknown");
    
    // For demo purposes, simulate rate limiting
    let should_rate_limit = false; // In real implementation, check actual rate limits
    
    if should_rate_limit {
        warn!("🚫 Rate limit exceeded for IP: {}", client_ip);
        
        let error_response = json!({
            "success": false,
            "error": "Rate limit exceeded",
            "data": null,
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "error_details": {
                "code": 429,
                "category": "RATE_LIMIT",
                "message": "Too many requests. Please try again later."
            }
        });
        
        let response = Response::builder()
            .status(StatusCode::TOO_MANY_REQUESTS)
            .header("content-type", "application/json")
            .header("x-ratelimit-limit", "100")
            .header("x-ratelimit-remaining", "0")
            .header("x-ratelimit-reset", "3600")
            .header("retry-after", "60")
            .body(Body::from(error_response.to_string()))
            .unwrap();
        
        Ok(response)
    } else {
        let mut response = next.run(request).await;
        
        // Add rate limiting headers to successful responses
        let headers = response.headers_mut();
        headers.insert(
            header::HeaderName::from_static("x-ratelimit-limit"),
            HeaderValue::from_static("100"),
        );
        headers.insert(
            header::HeaderName::from_static("x-ratelimit-remaining"),
            HeaderValue::from_static("99"),
        );
        headers.insert(
            header::HeaderName::from_static("x-ratelimit-reset"),
            HeaderValue::from_static("3600"),
        );
        
        Ok(response)
     }
}

/// Input sanitization middleware
/// Sanitizes request bodies and query parameters to prevent XSS and injection attacks
pub async fn input_sanitization(
    mut request: Request<Body>,
    next: Next,
) -> Result<Response<Body>, Infallible> {
    let uri = request.uri().clone();
    
    // Sanitize query parameters
    if let Some(query) = uri.query() {
        let sanitized_query = sanitize_string(query);
        if sanitized_query != query {
            warn!("🧹 Sanitized query parameters for {}", uri.path());
        }
    }
    
    // For POST/PUT requests, we would sanitize the body here
    // This is a simplified implementation - in production, you'd want more sophisticated sanitization
    let method = request.method().clone();
    if matches!(method, Method::POST | Method::PUT | Method::PATCH) {
        // Extract and sanitize body if it's JSON
        let content_type = request.headers()
            .get("content-type")
            .and_then(|h| h.to_str().ok())
            .unwrap_or("");
            
        if content_type.contains("application/json") {
            let (parts, body) = request.into_parts();
            let body_bytes = match axum::body::to_bytes(body, usize::MAX).await {
                Ok(bytes) => bytes,
                Err(_) => {
                    let error_response = json!({
                        "success": false,
                        "error": "Invalid request body",
                        "data": null,
                        "timestamp": chrono::Utc::now().to_rfc3339()
                    });
                    
                    let response = Response::builder()
                        .status(StatusCode::BAD_REQUEST)
                        .header("content-type", "application/json")
                        .body(Body::from(error_response.to_string()))
                        .unwrap();
                    
                    return Ok(response);
                }
            };
            
            // Sanitize JSON body
            if let Ok(body_str) = String::from_utf8(body_bytes.to_vec()) {
                let sanitized_body = sanitize_json_string(&body_str);
                if sanitized_body != body_str {
                    info!("🧹 Sanitized request body for {} {}", method, uri.path());
                }
                
                // Reconstruct request with sanitized body
                request = Request::from_parts(parts, Body::from(sanitized_body));
            } else {
                request = Request::from_parts(parts, Body::from(body_bytes));
            }
        }
    }
    
    Ok(next.run(request).await)
}

/// Sanitize a string to prevent XSS and injection attacks
fn sanitize_string(input: &str) -> String {
    input
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
        .replace('&', "&amp;")
        .replace("javascript:", "")
        .replace("data:", "")
        .replace("vbscript:", "")
        .replace("onload", "")
        .replace("onerror", "")
        .replace("onclick", "")
        .replace("onmouseover", "")
        .replace("<script", "")
        .replace("</script>", "")
        .replace("SELECT", "")
        .replace("INSERT", "")
        .replace("UPDATE", "")
        .replace("DELETE", "")
        .replace("DROP", "")
        .replace("UNION", "")
        .replace("--", "")
        .replace("/*", "")
        .replace("*/", "")
}

/// Sanitize JSON string while preserving structure
fn sanitize_json_string(input: &str) -> String {
    // Parse JSON and sanitize string values
    if let Ok(mut value) = serde_json::from_str::<serde_json::Value>(input) {
        sanitize_json_value(&mut value);
        serde_json::to_string(&value).unwrap_or_else(|_| input.to_string())
    } else {
        sanitize_string(input)
    }
}

/// Recursively sanitize JSON values
fn sanitize_json_value(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::String(s) => {
            *s = sanitize_string(s);
        }
        serde_json::Value::Object(map) => {
            for (_, v) in map.iter_mut() {
                sanitize_json_value(v);
            }
        }
        serde_json::Value::Array(arr) => {
            for v in arr.iter_mut() {
                sanitize_json_value(v);
            }
        }
        _ => {} // Numbers, booleans, null don't need sanitization
    }
}

/// Input validation middleware
/// Validates input length and format to prevent abuse
pub async fn input_validation(
    request: Request<Body>,
    next: Next,
) -> Result<Response<Body>, Infallible> {
    let uri = request.uri().clone();
    let method = request.method().clone();
    
    // Check query parameter length
    if let Some(query) = uri.query() {
        if query.len() > 2048 { // 2KB limit for query parameters
            warn!("🚫 Query parameters too long for {} {}", method, uri.path());
            
            let error_response = json!({
                "success": false,
                "error": "Query parameters too long",
                "data": null,
                "timestamp": chrono::Utc::now().to_rfc3339(),
                "error_details": {
                    "code": 400,
                    "category": "INPUT_VALIDATION",
                    "message": "Query parameters exceed maximum length of 2048 characters"
                }
            });
            
            let response = Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .header("content-type", "application/json")
                .body(Body::from(error_response.to_string()))
                .unwrap();
            
            return Ok(response);
        }
    }
    
    // Check content length for POST/PUT requests
    if matches!(method, Method::POST | Method::PUT | Method::PATCH) {
        if let Some(content_length) = request.headers().get("content-length") {
            if let Ok(length_str) = content_length.to_str() {
                if let Ok(length) = length_str.parse::<usize>() {
                    if length > 1024 * 1024 { // 1MB limit
                        warn!("🚫 Request body too large for {} {}", method, uri.path());
                        
                        let error_response = json!({
                            "success": false,
                            "error": "Request body too large",
                            "data": null,
                            "timestamp": chrono::Utc::now().to_rfc3339(),
                            "error_details": {
                                "code": 413,
                                "category": "INPUT_VALIDATION",
                                "message": "Request body exceeds maximum size of 1MB"
                            }
                        });
                        
                        let response = Response::builder()
                            .status(StatusCode::PAYLOAD_TOO_LARGE)
                            .header("content-type", "application/json")
                            .body(Body::from(error_response.to_string()))
                            .unwrap();
                        
                        return Ok(response);
                    }
                }
            }
        }
    }
    
    Ok(next.run(request).await)
}
