// src/routes/rate_limiting.rs - Rate Limiting Monitoring Routes
use axum::{
    routing::get,
    Router,
};

use crate::{
    handlers::rate_limiting::{
        get_rate_limit_dashboard,
        get_endpoint_rate_limits,
        get_top_violators,
        get_real_time_metrics,
        get_active_alerts,
    },
    state::AppState,
};

/// Create rate limiting monitoring routes
/// 
/// All routes require admin authentication for security
pub fn create_rate_limiting_routes() -> Router<AppState> {
    Router::new()
        // Dashboard overview with comprehensive statistics
        .route("/", get(get_rate_limit_dashboard))
        
        // Endpoint-specific statistics
        .route("/endpoints/{endpoint_class}", get(get_endpoint_rate_limits))
        
        // Security monitoring
        .route("/top-violators", get(get_top_violators))
        .route("/alerts", get(get_active_alerts))
        
        // Real-time monitoring
        .route("/real-time", get(get_real_time_metrics))
}