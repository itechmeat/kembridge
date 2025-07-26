// src/routes/admin.rs - Admin routes (Future implementation)
use axum::{
    routing::{get, post},
    Router,
};
use crate::AppState;

/// Create admin routes
/// These will be implemented later for system administration
pub fn create_routes() -> Router<AppState> {
    Router::new()
        // System statistics
        .route("/stats", get(crate::handlers::admin::get_system_stats))
        
        // User management
        .route("/users", get(crate::handlers::admin::list_users))
        .route("/users/{user_id}/ban", post(crate::handlers::admin::ban_user))
        
        // Transaction monitoring
        .route("/transactions", get(crate::handlers::admin::list_transactions))
        .route("/transactions/{tx_id}/review", post(crate::handlers::admin::review_transaction))
        
        // Risk management
        .route("/risk/thresholds", get(crate::handlers::admin::get_risk_thresholds))
        .route("/risk/thresholds", post(crate::handlers::admin::update_risk_thresholds))
}