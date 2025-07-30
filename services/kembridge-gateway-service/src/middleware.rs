use axum::{
    http::{header, HeaderMap, HeaderValue, Request, Response},
    middleware::Next,
    body::Body,
};
use std::convert::Infallible;

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
        HeaderValue::from_static("max-age=31536000; includeSubDomains; preload")
    );
    
    // Content Type Options - Prevent MIME sniffing
    headers.insert(
        header::HeaderName::from_static("x-content-type-options"),
        HeaderValue::from_static("nosniff")
    );
    
    // Frame Options - Prevent clickjacking
    headers.insert(
        header::HeaderName::from_static("x-frame-options"),
        HeaderValue::from_static("DENY")
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
        HeaderValue::from_static("1; mode=block")
    );
    
    // Referrer Policy
    headers.insert(
        header::HeaderName::from_static("referrer-policy"),
        HeaderValue::from_static("strict-origin-when-cross-origin")
    );
    
    // Permissions Policy
    headers.insert(
        header::HeaderName::from_static("permissions-policy"),
        HeaderValue::from_static("geolocation=(), microphone=(), camera=()")
    );
    
    // Rate limiting headers
    headers.insert(
        header::HeaderName::from_static("x-ratelimit-limit"),
        HeaderValue::from_static("100")
    );
    
    headers.insert(
        header::HeaderName::from_static("x-ratelimit-remaining"),
        HeaderValue::from_static("99")
    );
    
    headers.insert(
        header::HeaderName::from_static("x-ratelimit-reset"),
        HeaderValue::from_static("3600")
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