// src/middleware/security_headers.rs - Security headers middleware
use axum::{extract::Request, middleware::Next, response::Response};
use axum::http::HeaderValue;
use crate::middleware::error_handler::ApiError;

/// Security headers middleware
/// Adds security-related HTTP headers to all responses
pub async fn security_headers(
    request: Request,
    next: Next,
) -> Result<Response, ApiError> {
    let mut response = next.run(request).await;
    let headers = response.headers_mut();
    
    // Content Security Policy
    headers.insert(
        "content-security-policy",
        HeaderValue::from_static(
            "default-src 'self'; \
             script-src 'self' 'unsafe-inline' 'unsafe-eval'; \
             style-src 'self' 'unsafe-inline'; \
             img-src 'self' data: https:; \
             font-src 'self' data:; \
             connect-src 'self' ws: wss: https: http://localhost:* http://127.0.0.1:*; \
             frame-ancestors 'none'; \
             base-uri 'self'; \
             form-action 'self'"
        )
    );
    
    // X-Frame-Options
    headers.insert(
        "x-frame-options",
        HeaderValue::from_static("DENY")
    );
    
    // X-Content-Type-Options
    headers.insert(
        "x-content-type-options",
        HeaderValue::from_static("nosniff")
    );
    
    // X-XSS-Protection
    headers.insert(
        "x-xss-protection",
        HeaderValue::from_static("1; mode=block")
    );
    
    // Referrer-Policy
    headers.insert(
        "referrer-policy",
        HeaderValue::from_static("strict-origin-when-cross-origin")
    );
    
    // Permissions-Policy
    headers.insert(
        "permissions-policy",
        HeaderValue::from_static(
            "camera=(), microphone=(), geolocation=(), payment=(), usb=()"
        )
    );
    
    // Strict-Transport-Security (HSTS)
    headers.insert(
        "strict-transport-security",
        HeaderValue::from_static("max-age=31536000; includeSubDomains; preload")
    );
    
    // Cross-Origin-Embedder-Policy
    headers.insert(
        "cross-origin-embedder-policy",
        HeaderValue::from_static("require-corp")
    );
    
    // Cross-Origin-Opener-Policy
    headers.insert(
        "cross-origin-opener-policy",
        HeaderValue::from_static("same-origin")
    );
    
    // Cross-Origin-Resource-Policy
    headers.insert(
        "cross-origin-resource-policy",
        HeaderValue::from_static("same-origin")
    );
    
    // Remove server information
    headers.remove("server");
    headers.remove("x-powered-by");
    
    // Cache-Control for sensitive endpoints
    let path = request.uri().path();
    if is_sensitive_endpoint(path) {
        headers.insert(
            "cache-control",
            HeaderValue::from_static("no-store, no-cache, must-revalidate, private")
        );
        headers.insert(
            "pragma",
            HeaderValue::from_static("no-cache")
        );
        headers.insert(
            "expires",
            HeaderValue::from_static("0")
        );
    }
    
    Ok(response)
}

/// Check if endpoint contains sensitive data
fn is_sensitive_endpoint(path: &str) -> bool {
    match path {
        path if path.starts_with("/api/v1/auth") => true,
        path if path.starts_with("/api/v1/user") => true,
        path if path.starts_with("/api/v1/admin") => true,
        path if path.starts_with("/api/v1/quantum") => true,
        path if path.contains("token") => true,
        path if path.contains("key") => true,
        path if path.contains("secret") => true,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_is_sensitive_endpoint() {
        assert!(is_sensitive_endpoint("/api/v1/auth/login"));
        assert!(is_sensitive_endpoint("/api/v1/user/profile"));
        assert!(is_sensitive_endpoint("/api/v1/admin/users"));
        assert!(is_sensitive_endpoint("/api/v1/quantum/sign"));
        assert!(is_sensitive_endpoint("/api/v1/csrf-token"));
        assert!(!is_sensitive_endpoint("/health"));
        assert!(!is_sensitive_endpoint("/api/v1/bridge/quote"));
    }
}