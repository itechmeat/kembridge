// src/handlers/auth.rs - Web3 Authentication handlers
use axum::{
    extract::{Query, State}, 
    response::Json, 
    http::StatusCode
};
use kembridge_auth::{NonceRequest, NonceResponse, AuthRequest, AuthResponse};
use crate::AppState;

/// Generate nonce for Web3 signature
pub async fn generate_nonce(
    Query(request): Query<NonceRequest>,
    State(state): State<AppState>
) -> Result<Json<NonceResponse>, StatusCode> {
    match state.auth_service
        .generate_nonce(&request.wallet_address, request.chain_type)
        .await 
    {
        Ok(response) => Ok(Json(response)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Verify Web3 wallet signature and issue JWT
pub async fn verify_wallet(
    State(state): State<AppState>,
    Json(request): Json<AuthRequest>
) -> Result<Json<AuthResponse>, StatusCode> {
    match state.auth_service
        .verify_wallet_signature(request)
        .await 
    {
        Ok(response) => Ok(Json(response)),
        Err(_) => Err(StatusCode::UNAUTHORIZED),
    }
}

/// Refresh JWT token (placeholder - Phase 2.2.8)
pub async fn refresh_token(State(_state): State<AppState>) -> Result<Json<serde_json::Value>, StatusCode> {
    Ok(Json(serde_json::json!({
        "message": "JWT refresh token logic will be implemented in Phase 2.2",
        "implementation_phase": "2.2"
    })))
}

/// Logout (placeholder - Phase 2.2.7)
pub async fn logout(State(_state): State<AppState>) -> Result<Json<serde_json::Value>, StatusCode> {
    Ok(Json(serde_json::json!({
        "message": "Logout functionality will be implemented in Phase 2.2",
        "implementation_phase": "2.2"
    })))
}