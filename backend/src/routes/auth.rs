// src/routes/auth.rs - Authentication routes (Phase 2 placeholder)
use axum::{
    routing::{get, post},
    Router,
};
use crate::AppState;

/// Create authentication routes
/// These will be fully implemented in Phase 2.1 - Web3 Authentication Service
pub fn create_routes() -> Router<AppState> {
    Router::new()
        // Generate nonce for Web3 signature (Phase 2.1.2)
        .route("/nonce", get(crate::handlers::auth::generate_nonce))
        
        // Verify Web3 wallet signature (Phase 2.1.6)
        .route("/verify", post(crate::handlers::auth::verify_wallet))
        
        // Refresh JWT token (Phase 2.2.8)
        .route("/refresh", post(crate::handlers::auth::refresh_token))
        
        // Logout (Phase 2.2.7)
        .route("/logout", post(crate::handlers::auth::logout))
}