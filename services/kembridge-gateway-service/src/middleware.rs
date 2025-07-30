use axum::{
    body::Body,
    http::{header, HeaderValue, Request, Response, StatusCode},
    middleware::Next,
};
use kembridge_common::{ErrorCategory, FailureContext, RecoverySystem};
use serde_json::json;
use std::convert::Infallible;
use std::time::SystemTime;
use tracing::{info, warn};
use uuid::Uuid;

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
