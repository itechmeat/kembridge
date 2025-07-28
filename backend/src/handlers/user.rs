// src/handlers/user.rs - User management handlers (Phase 2.3 implementation)
use axum::{extract::{State, Path}, response::Json, http::StatusCode};
use serde_json::{json, Value};
use crate::state::AppState;
use crate::extractors::auth::AuthUser;
use crate::models::user::{UserProfileResponse, UpdateUserRequest, AddWalletRequest};
use validator::Validate;
use crate::middleware::error_handler::ApiError;

/// Get user profile (Phase 2.3.1) - Now fully implemented
pub async fn get_profile(
    auth_user: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<UserProfileResponse>, ApiError> {
    let profile = state.user_service
        .get_user_profile_response(auth_user.user_id)
        .await?;
    
    Ok(Json(profile))
}

/// Update user profile (Phase 2.3.5) - Now fully implemented
pub async fn update_profile(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Json(request): Json<UpdateUserRequest>,
) -> Result<Json<UserProfileResponse>, ApiError> {
    let updated_user = state.user_service
        .update_user(auth_user.user_id, request)
        .await?;
    
    // Return full profile response after update
    let profile = state.user_service
        .get_user_profile_response(updated_user.id)
        .await?;
    
    Ok(Json(profile))
}

/// Get user's wallets - Now fully implemented
pub async fn get_wallets(
    auth_user: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<Value>, ApiError> {
    let wallets = state.user_service
        .get_user_wallets(auth_user.user_id)
        .await?;
    
    Ok(Json(json!({
        "wallets": wallets,
        "total": wallets.len()
    })))
}

/// Add new wallet (Phase 2.3.4) - Now fully implemented
pub async fn add_wallet(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Json(request): Json<AddWalletRequest>,
) -> Result<Json<Value>, ApiError> {
    state.user_service
        .add_wallet(auth_user.user_id, request)
        .await?;
    
    Ok(Json(json!({
        "message": "Wallet added successfully",
        "success": true
    })))
}

/// Remove wallet - Now fully implemented
pub async fn remove_wallet(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Path(wallet_address): Path<String>,
) -> Result<Json<Value>, ApiError> {
    state.user_service
        .remove_wallet(auth_user.user_id, &wallet_address)
        .await?;
    
    Ok(Json(json!({
        "wallet_address": wallet_address,
        "message": "Wallet removed successfully",
        "success": true
    })))
}

/// Soft delete user account (Phase 2.3.6) - Implemented
pub async fn delete_profile(
    auth_user: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<Value>, ApiError> {
    state.user_service
        .soft_delete_user(auth_user.user_id)
        .await?;
    
    Ok(Json(json!({
        "message": "User account deleted successfully",
        "success": true
    })))
}

/// Set primary wallet
pub async fn set_primary_wallet(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Path(wallet_address): Path<String>,
) -> Result<Json<Value>, ApiError> {
    state.user_service
        .set_primary_wallet(auth_user.user_id, &wallet_address)
        .await?;
    
    Ok(Json(json!({
        "wallet_address": wallet_address,
        "message": "Primary wallet updated successfully",
        "success": true
    })))
}

/// Get user's risk profile
pub async fn get_risk_profile(
    auth_user: AuthUser,
    State(_state): State<AppState>,
) -> Result<Json<Value>, StatusCode> {
    Ok(Json(json!({
        "user_id": auth_user.user_id,
        "risk_score": 0.0,
        "risk_level": "unknown",
        "factors": [],
        "last_updated": chrono::Utc::now().to_rfc3339(),
        "message": "Risk profiling will be implemented in Phase 5.2 - Integration with Bridge Service",
        "implementation_phase": "5.2"
    })))
}