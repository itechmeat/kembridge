// src/routes/fusion_plus.rs - 1inch Fusion+ Cross-Chain Routes

use axum::{
    routing::{get, post},
    Router,
};

use crate::{
    handlers::fusion_plus,
    state::AppState,
};

/// Create Fusion+ cross-chain routes
pub fn create_fusion_plus_routes() -> Router<AppState> {
    Router::new()
        // Cross-chain quote endpoints
        .route("/quote", get(fusion_plus::get_cross_chain_quote))
        
        // Order management endpoints
        .route("/build-order", post(fusion_plus::build_cross_chain_order))
        .route("/submit-order", post(fusion_plus::submit_cross_chain_order))
        
        // Order tracking endpoints
        .route("/orders/active", get(fusion_plus::get_active_cross_chain_orders))
        .route("/orders/{order_hash}", get(fusion_plus::get_cross_chain_order_by_hash))
        
        // Utility endpoints
        .route("/escrow-factory/{chain_id}", get(fusion_plus::get_escrow_factory))
}