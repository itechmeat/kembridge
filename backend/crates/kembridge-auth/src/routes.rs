// Route definitions for Web3 authentication

use axum::{
    routing::{get, post},
    Router,
};
use crate::{AuthService, handlers::*};

pub fn create_auth_routes() -> Router<AuthService> {
    Router::new()
        .route("/health", get(health_check_handler))
        .route("/nonce", post(generate_nonce_handler))
        .route("/verify-wallet", post(verify_wallet_handler))
}

pub fn create_auth_routes_with_prefix() -> Router<AuthService> {
    Router::new()
        .nest("/api/auth", create_auth_routes())
}