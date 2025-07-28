// src/routes/admin.rs - Admin routes (Future implementation)
use axum::{
    routing::{get, post, put},
    Router,
};
use crate::state::AppState;

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
        
        // Manual review queue management (Phase 5.2.4)
        .route("/review/queue", get(crate::handlers::manual_review::get_review_queue))
        .route("/review/queue", post(crate::handlers::manual_review::add_to_review_queue))
        .route("/review/{review_id}", get(crate::handlers::manual_review::get_review_details))
        .route("/review/{review_id}/assign", put(crate::handlers::manual_review::assign_review))
        .route("/review/{review_id}/decision", put(crate::handlers::manual_review::make_review_decision))
        .route("/review/{review_id}/escalate", put(crate::handlers::manual_review::escalate_review))
        .route("/review/check-escalations", post(crate::handlers::manual_review::check_escalations))
}