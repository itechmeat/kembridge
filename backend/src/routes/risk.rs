// src/routes/risk.rs - Risk Analysis Routes (Phase 5.2.6)
use axum::{
    routing::{get, put, post},
    Router,
};

use crate::handlers::risk::{
    get_user_risk_profile,
    get_risk_thresholds,
    update_risk_thresholds,
    get_risk_engine_health,
    test_risk_analysis,
};
use crate::state::AppState;

/// Create risk analysis routes
pub fn create_routes() -> Router<AppState> {
    Router::new()
        // User risk profile endpoints
        .route("/profile/:user_id", get(get_user_risk_profile))
        
        // Risk threshold management
        .route("/thresholds", get(get_risk_thresholds))
        .route("/thresholds", put(update_risk_thresholds))
        
        // Health and monitoring
        .route("/health", get(get_risk_engine_health))
        
        // Development and testing endpoints
        .route("/test", post(test_risk_analysis))
}