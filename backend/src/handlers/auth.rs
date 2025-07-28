// src/handlers/auth.rs - Web3 Authentication handlers
use axum::{
    extract::{Query, State}, 
    response::Json, 
    http::StatusCode
};
use kembridge_auth::{NonceRequest, NonceResponse, AuthRequest, AuthResponse};
use crate::state::AppState;

/// Generate nonce for Web3 signature
pub async fn generate_nonce(
    Query(request): Query<NonceRequest>,
    State(state): State<AppState>
) -> Result<Json<NonceResponse>, StatusCode> {
    tracing::info!("🔑 Nonce Handler: Generating nonce for wallet: {}", request.wallet_address);
    tracing::debug!("🔑 Nonce Handler: Request details: chain_type={:?}", request.chain_type);
    
    match state.auth_service
        .generate_nonce(&request.wallet_address, request.chain_type)
        .await 
    {
        Ok(response) => {
            tracing::info!("✅ Nonce Handler: Nonce generated successfully");
            Ok(Json(response))
        },
        Err(err) => {
            tracing::error!("❌ Nonce Handler: Failed to generate nonce: {:?}", err);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        },
    }
}

/// Verify Web3 wallet signature and issue JWT
pub async fn verify_wallet(
    State(state): State<AppState>,
    Json(request): Json<AuthRequest>
) -> Result<Json<AuthResponse>, StatusCode> {
    tracing::info!("🔐 Auth Handler: Verifying wallet signature for address: {}", request.wallet_address);
    tracing::debug!("🔐 Auth Handler: Request details: chain_type={:?}, signature_len={}, nonce_len={}", 
        request.chain_type, request.signature.len(), request.nonce.len());
    
    match state.auth_service
        .verify_wallet_signature(request)
        .await 
    {
        Ok(response) => {
            tracing::info!("✅ Auth Handler: Authentication successful for user_id: {}", response.user_id);
            Ok(Json(response))
        },
        Err(err) => {
            tracing::error!("❌ Auth Handler: Authentication failed: {:?}", err);
            Err(StatusCode::UNAUTHORIZED)
        },
    }
}

/// Refresh JWT token 
pub async fn refresh_token(
    State(state): State<AppState>,
    auth_user: crate::extractors::auth::AuthUser,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // For now, generate a new token with the same claims
    // In production, you might want to implement proper refresh token rotation
    match state.auth_service.jwt_manager
        .generate_token(
            auth_user.user_id, 
            &auth_user.wallet_address, 
            auth_user.chain_type
        )
        .await 
    {
        Ok(new_token) => {
            Ok(Json(serde_json::json!({
                "access_token": new_token,
                "token_type": "Bearer",
                "expires_in": 24 * 60 * 60, // 24 hours
                "message": "Token refreshed successfully"
            })))
        },
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Logout - invalidate session
pub async fn logout(
    State(state): State<AppState>,
    auth_user: crate::extractors::auth::AuthUser,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Mark session as expired in the database
    let result = sqlx::query(
        r#"
        UPDATE user_sessions 
        SET expires_at = NOW() 
        WHERE user_id = $1 AND jwt_token_hash = $2
        "#,
    )
    .bind(auth_user.user_id)
    .bind(&auth_user.session_id) // Assuming session_id contains token hash
    .execute(&state.db)
    .await;

    match result {
        Ok(_) => {
            Ok(Json(serde_json::json!({
                "message": "Logged out successfully",
                "user_id": auth_user.user_id,
                "timestamp": chrono::Utc::now().to_rfc3339()
            })))
        },
        Err(e) => {
            tracing::error!("Failed to logout user {}: {}", auth_user.user_id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}