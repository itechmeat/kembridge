// src/routes/bridge.rs - Cross-chain bridge routes (Phase 4 placeholder)
use axum::{
    routing::{get, post},
    Router,
};
use crate::AppState;

/// Create bridge operation routes
/// These will be fully implemented in Phase 4.3 - Basic Bridge Logic
pub fn create_routes() -> Router<AppState> {
    Router::new()
        // Get swap quote (Phase 6.3.5)
        .route("/quote", post(crate::handlers::bridge::get_quote))
        
        // Initiate cross-chain swap (Phase 4.3.4)
        .route("/swap", post(crate::handlers::bridge::initiate_swap))
        
        // Get transaction status (Phase 4.3.8)
        .route("/status/{transaction_id}", get(crate::handlers::bridge::get_swap_status))
        
        // Get user's transaction history
        .route("/history", get(crate::handlers::bridge::get_transaction_history))
}