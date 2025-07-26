// src/routes/user.rs - User management routes (Phase 2.3 placeholder)
use axum::{
    routing::{get, put, delete},
    Router,
};
use crate::AppState;

/// Create user management routes
/// These will be fully implemented in Phase 2.3 - User Management
pub fn create_routes() -> Router<AppState> {
    Router::new()
        // Get user profile (Phase 2.3.1)
        .route("/profile", get(crate::handlers::user::get_profile))
        
        // Update user profile (Phase 2.3.5)
        .route("/profile", put(crate::handlers::user::update_profile))
        
        // Get user's wallets
        .route("/wallets", get(crate::handlers::user::get_wallets))
        
        // Add new wallet (Phase 2.3.4)
        .route("/wallets", put(crate::handlers::user::add_wallet))
        
        // Remove wallet
        .route("/wallets/{wallet_id}", delete(crate::handlers::user::remove_wallet))
        
        // Get user's risk profile
        .route("/risk-profile", get(crate::handlers::user::get_risk_profile))
}