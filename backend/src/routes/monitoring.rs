// src/routes/monitoring.rs - Monitoring dashboard routes
use axum::{routing::get, Router};
use crate::{handlers::monitoring, state::AppState};

/// Monitoring dashboard routes
pub fn monitoring_routes() -> Router<AppState> {
    Router::new()
        .route("/stats", get(monitoring::get_dashboard_stats))
        .route("/events", get(monitoring::get_real_time_events))
        .route("/config", get(monitoring::get_monitoring_config))
        .route("/config", axum::routing::put(monitoring::update_monitoring_config))
        .route("/connections", get(monitoring::get_connection_details))
}