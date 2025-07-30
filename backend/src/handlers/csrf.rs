// src/handlers/csrf.rs - CSRF token handlers
use axum::{extract::State, Json};
use serde_json::{json, Value};
use crate::{middleware::error_handler::ApiError, state::AppState, middleware::csrf::generate_csrf_token};

/// Get CSRF token endpoint
/// Returns a new CSRF token for client-side requests
pub async fn get_csrf_token(
    State(state): State<AppState>,
) -> Result<Json<Value>, ApiError> {
    let token = generate_csrf_token(&state, None).await;
    
    Ok(Json(json!({
        "csrf_token": token,
        "expires_in": 3600, // 1 hour
        "usage": "Include this token in X-CSRF-Token header for state-changing requests"
    })))
}

/// Validate CSRF token endpoint
/// Allows clients to validate their CSRF token
pub async fn validate_csrf_token(
    State(_state): State<AppState>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, ApiError> {
    let token = payload.get("token")
        .and_then(|t| t.as_str())
        .ok_or_else(|| ApiError::bad_request("Missing token field"))?;
    
    // Basic validation
    let is_valid = token.len() >= 32 && 
                   token.chars().all(|c| c.is_ascii_alphanumeric() || c == '-');
    
    Ok(Json(json!({
        "valid": is_valid,
        "message": if is_valid { "Token is valid" } else { "Token is invalid" }
    })))
}