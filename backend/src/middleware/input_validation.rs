// src/middleware/input_validation.rs - Input validation and sanitization middleware
use axum::{extract::Request, middleware::Next, response::Response, body::Body};
use crate::middleware::error_handler::ApiError;
use regex::Regex;
use std::sync::OnceLock;

/// Input validation middleware
/// Sanitizes and validates input to prevent XSS, SQL injection, and other attacks
pub async fn input_validation(
    mut request: Request<Body>,
    next: Next,
) -> Result<Response, ApiError> {
    let method = request.method();
    let path = request.uri().path();
    
    // Skip validation for safe methods and certain endpoints
    if should_skip_validation(method, path) {
        return Ok(next.run(request).await);
    }
    
    // Validate query parameters
    if let Some(query) = request.uri().query() {
        if !validate_query_string(query) {
            tracing::warn!("Malicious query string detected: {}", query);
            return Err(ApiError::bad_request("Invalid query parameters"));
        }
    }
    
    // Validate headers
    for (name, value) in request.headers().iter() {
        if let Ok(value_str) = value.to_str() {
            if !validate_header_value(value_str) {
                tracing::warn!("Malicious header detected: {}: {}", name, value_str);
                return Err(ApiError::bad_request("Invalid header value"));
            }
        }
    }
    
    Ok(next.run(request).await)
}

/// Check if validation should be skipped
fn should_skip_validation(method: &axum::http::Method, path: &str) -> bool {
    // Skip for safe HTTP methods
    if matches!(method, &axum::http::Method::GET | &axum::http::Method::HEAD | &axum::http::Method::OPTIONS) {
        return true;
    }
    
    // Skip for certain endpoints
    match path {
        "/health" | "/ready" | "/metrics" => true,
        path if path.starts_with("/docs") => true,
        path if path.starts_with("/static") => true,
        _ => false,
    }
}

/// Validate query string for malicious patterns
fn validate_query_string(query: &str) -> bool {
    static SQL_INJECTION_PATTERNS: OnceLock<Vec<Regex>> = OnceLock::new();
    let patterns = SQL_INJECTION_PATTERNS.get_or_init(|| {
        vec![
            Regex::new(r"(?i)(union|select|insert|update|delete|drop|create|alter|exec|execute)").unwrap(),
            Regex::new(r"(?i)(script|javascript|vbscript|onload|onerror|onclick)").unwrap(),
            Regex::new(r"(?i)(<script|</script|<iframe|</iframe)").unwrap(),
            Regex::new(r"(?i)(eval\(|expression\(|url\()").unwrap(),
            Regex::new(r"(?i)(--|\/\*|\*\/|;\s*drop|;\s*delete)").unwrap(),
        ]
    });
    
    // Check for SQL injection and XSS patterns
    for pattern in patterns {
        if pattern.is_match(query) {
            return false;
        }
    }
    
    // Check for excessive length
    if query.len() > 2048 {
        return false;
    }
    
    true
}

/// Validate header value for malicious patterns
fn validate_header_value(value: &str) -> bool {
    static MALICIOUS_PATTERNS: OnceLock<Vec<Regex>> = OnceLock::new();
    let patterns = MALICIOUS_PATTERNS.get_or_init(|| {
        vec![
            Regex::new(r"(?i)<script").unwrap(),
            Regex::new(r"(?i)javascript:").unwrap(),
            Regex::new(r"(?i)vbscript:").unwrap(),
            Regex::new(r"(?i)data:text/html").unwrap(),
            Regex::new(r"\x00|\x0a|\x0d").unwrap(), // Null bytes and CRLF
        ]
    });
    
    // Check for malicious patterns
    for pattern in patterns {
        if pattern.is_match(value) {
            return false;
        }
    }
    
    // Check for excessive length
    if value.len() > 8192 {
        return false;
    }
    
    true
}

/// Sanitize string input to prevent XSS
pub fn sanitize_string(input: &str) -> String {
    input
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
        .replace('&', "&amp;")
        .chars()
        .take(1000) // Limit length
        .collect()
}

/// Validate numeric input
pub fn validate_numeric_input(input: &str) -> bool {
    static NUMERIC_PATTERN: OnceLock<Regex> = OnceLock::new();
    let pattern = NUMERIC_PATTERN.get_or_init(|| {
        Regex::new(r"^[0-9]+(\.[0-9]+)?$").unwrap()
    });
    
    pattern.is_match(input) && input.len() <= 50
}

/// Validate wallet address format
pub fn validate_wallet_address(address: &str) -> bool {
    static ETH_ADDRESS_PATTERN: OnceLock<Regex> = OnceLock::new();
    static NEAR_ADDRESS_PATTERN: OnceLock<Regex> = OnceLock::new();
    
    let eth_pattern = ETH_ADDRESS_PATTERN.get_or_init(|| {
        Regex::new(r"^0x[a-fA-F0-9]{40}$").unwrap()
    });
    
    let near_pattern = NEAR_ADDRESS_PATTERN.get_or_init(|| {
        Regex::new(r"^[a-z0-9_-]+\.(testnet|near)$|^[a-f0-9]{64}$").unwrap()
    });
    
    eth_pattern.is_match(address) || near_pattern.is_match(address)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_sql_injection_detection() {
        assert!(!validate_query_string("'; DROP TABLE users; --"));
        assert!(!validate_query_string("' OR '1'='1"));
        assert!(!validate_query_string("UNION SELECT * FROM users"));
        assert!(validate_query_string("search=normal_query"));
    }
    
    #[test]
    fn test_xss_detection() {
        assert!(!validate_query_string("<script>alert('xss')</script>"));
        assert!(!validate_query_string("javascript:alert('xss')"));
        assert!(validate_query_string("normal_parameter=value"));
    }
    
    #[test]
    fn test_sanitize_string() {
        assert_eq!(sanitize_string("<script>alert('xss')</script>"), "&lt;script&gt;alert(&#x27;xss&#x27;)&lt;/script&gt;");
        assert_eq!(sanitize_string("normal text"), "normal text");
    }
    
    #[test]
    fn test_validate_numeric_input() {
        assert!(validate_numeric_input("123.45"));
        assert!(validate_numeric_input("0.1"));
        assert!(!validate_numeric_input("abc"));
        assert!(!validate_numeric_input("123.45.67"));
    }
    
    #[test]
    fn test_validate_wallet_address() {
        assert!(validate_wallet_address("0x1234567890123456789012345678901234567890"));
        assert!(validate_wallet_address("alice.near"));
        assert!(validate_wallet_address("test.testnet"));
        assert!(!validate_wallet_address("invalid_address"));
    }
}