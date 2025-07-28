// src/routes/user.rs - User management routes (Phase 2.3 implementation)
use axum::{
    routing::{get, put, delete, post},
    Router,
};
use crate::state::AppState;

/// Create user management routes - Now fully implemented
pub fn create_routes() -> Router<AppState> {
    Router::new()
        // Get user profile (Phase 2.3.1) ✅
        .route("/profile", get(crate::handlers::user::get_profile))
        
        // Update user profile (Phase 2.3.5) ✅
        .route("/profile", put(crate::handlers::user::update_profile))
        
        // Soft delete user profile (Phase 2.3.6) ✅
        .route("/profile", delete(crate::handlers::user::delete_profile))
        
        // Get user's wallets ✅
        .route("/wallets", get(crate::handlers::user::get_wallets))
        
        // Add new wallet (Phase 2.3.4) ✅
        .route("/wallets", post(crate::handlers::user::add_wallet))
        
        // Remove wallet ✅
        .route("/wallets/{wallet_address}", delete(crate::handlers::user::remove_wallet))
        
        // Set primary wallet ✅
        .route("/wallets/{wallet_address}/primary", put(crate::handlers::user::set_primary_wallet))
        
        // Get user's risk profile
        .route("/risk-profile", get(crate::handlers::user::get_risk_profile))
}