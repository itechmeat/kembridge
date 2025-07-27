// src/routes/price_oracle.rs - Price Oracle API routes
use axum::{
    routing::{get, post, delete},
    Router,
};
use crate::{handlers::price_oracle, AppState};

/// Price Oracle API routes
pub fn price_oracle_routes() -> Router<AppState> {
    Router::new()
        // Price endpoints
        .route("/price", get(price_oracle::get_price))
        .route("/prices", get(price_oracle::get_multiple_prices))
        .route("/quote", post(price_oracle::get_price_quote))
        
        // System information
        .route("/supported", get(price_oracle::get_supported_symbols))
        .route("/health", get(price_oracle::get_provider_health))
        .route("/cache/stats", get(price_oracle::get_cache_stats))
        
        // Admin endpoints
        .route("/cache/clear", post(price_oracle::clear_cache))
        
        // Price alerts
        .route("/alerts", post(price_oracle::create_price_alert))
        .route("/alerts", get(price_oracle::get_user_alerts))
        .route("/alerts/{alert_id}", delete(price_oracle::delete_price_alert))
}