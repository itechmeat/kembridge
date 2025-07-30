// src/middleware/csrf.rs - CSRF Protection middleware
use axum::{extract::{Request, State}, middleware::Next, response::Response, http::{HeaderValue, Method}};
use crate::{middleware::error_handler::ApiError, state::AppState};
use uuid::Uuid;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// CSRF Protection middleware
/// Validates CSRF tokens for state-changing operations
pub async fn csrf_protection(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Result<Response, ApiError> {
    let method = request.method();
    let path = request.uri().path();
    
    // Skip CSRF protection for safe methods and public endpoints
    if should_skip_csrf_protection(method, path) {
        return Ok(next.run(request).await);
    }
    
    // Check for CSRF token in headers
    let csrf_token = request.headers()
        .get("x-csrf-token")
        .or_else(|| request.headers().get("csrf-token"))
        .and_then(|h| h.to_str().ok());
    
    match csrf_token {
        Some(token) => {
            // Validate CSRF token
            if !validate_csrf_token(&state, token).await {
                tracing::warn!("Invalid CSRF token: {}", token);
                return Err(ApiError::forbidden("Invalid CSRF token"));
            }
        }
        None => {
            // For authenticated requests, CSRF token is required
            if request.headers().contains_key("authorization") {
                tracing::warn!("Missing CSRF token for authenticated request to {}", path);
                return Err(ApiError::forbidden("CSRF token required"));
            }
        }
    }
    
    Ok(next.run(request).await)
}

/// Check if CSRF protection should be skipped
fn should_skip_csrf_protection(method: &Method, path: &str) -> bool {
    // Skip for safe HTTP methods
    if matches!(method, &Method::GET | &Method::HEAD | &Method::OPTIONS) {
        return true;
    }
    
    // Skip for public endpoints
    match path {
        "/health" | "/ready" | "/metrics" => true,
        path if path.starts_with("/api/v1/auth") => true,
        path if path.starts_with("/docs") => true,
        path if path.starts_with("/static") => true,
        _ => false,
    }
}

/// Validate CSRF token
async fn validate_csrf_token(state: &AppState, token: &str) -> bool {
    // For now, implement a simple token validation
    // In production, this would check against stored tokens in Redis
    
    // Basic format validation
    if token.len() < 32 || !token.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
        return false;
    }
    
    // For demo purposes, accept any properly formatted token
    // In production, validate against stored session tokens
    true
}

/// Generate CSRF token for client
pub async fn generate_csrf_token(state: &AppState, user_id: Option<&str>) -> String {
    let token = Uuid::new_v4().to_string();
    
    // In production, store token in Redis with expiration
    // For now, return the generated token
    
    tracing::debug!("Generated CSRF token for user: {:?}", user_id);
    token
}

/// CSRF token response
pub async fn get_csrf_token(
    State(state): State<AppState>,
) -> Result<axum::Json<serde_json::Value>, ApiError> {
    let token = generate_csrf_token(&state, None).await;
    
    Ok(axum::Json(serde_json::json!({
        "csrf_token": token,
        "expires_in": 3600 // 1 hour
    })))
}