// src/handlers/rate_limiting.rs - Rate Limiting Monitoring API Endpoints
use axum::{
    extract::{State, Query, Path},
    response::Json,
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use utoipa::ToSchema;

use crate::{
    state::AppState,
    extractors::auth::AdminAuth,
    services::{RateLimitStats, ViolatorInfo, AlertCondition},
};

/// Query parameters for rate limiting statistics
#[derive(Debug, Deserialize, ToSchema, utoipa::IntoParams)]
pub struct RateLimitStatsQuery {
    /// Time window in hours for statistics (default: 24)
    #[serde(default = "default_time_window")]
    pub time_window_hours: u32,
    
    /// Specific endpoint class to filter by
    pub endpoint_class: Option<String>,
}

fn default_time_window() -> u32 {
    24
}

/// Rate limiting dashboard response
#[derive(Debug, Serialize, ToSchema)]
pub struct RateLimitDashboard {
    pub overview: RateLimitOverview,
    pub endpoint_stats: Vec<RateLimitStats>,
    pub top_violators: Vec<ViolatorInfo>,
    pub active_alerts: Vec<AlertCondition>,
    pub timestamp: DateTime<Utc>,
}

/// Rate limiting overview statistics
#[derive(Debug, Serialize, ToSchema)]
pub struct RateLimitOverview {
    pub total_requests: u64,
    pub total_blocked: u64,
    pub overall_violation_rate: f64,
    pub active_endpoints: u32,
    pub unique_ips: u32,
    pub alerts_count: u32,
}

/// Real-time rate limiting metrics
#[derive(Debug, Serialize, ToSchema)]
pub struct RealTimeMetrics {
    pub current_rps: f64, // requests per second
    pub current_blocks_per_minute: u32,
    pub active_rate_limits: u32,
    pub redis_memory_usage: Option<u64>,
    pub timestamp: DateTime<Utc>,
}

/// Get comprehensive rate limiting dashboard
/// 
/// Returns overview statistics, endpoint breakdown, top violators, and active alerts
#[utoipa::path(
    get,
    path = "/api/v1/monitoring/rate-limits",
    tags = ["Rate Limiting"],
    security(("bearer_auth" = [])),
    params(RateLimitStatsQuery),
    responses(
        (status = 200, description = "Rate limiting dashboard data", body = RateLimitDashboard),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - Admin access required"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_rate_limit_dashboard(
    State(state): State<AppState>,
    Query(query): Query<RateLimitStatsQuery>,
    _admin: AdminAuth,
) -> Result<Json<RateLimitDashboard>, StatusCode> {
    tracing::info!(
        time_window = query.time_window_hours,
        endpoint_class = ?query.endpoint_class,
        "Getting rate limiting dashboard"
    );

    // Get comprehensive statistics
    let endpoint_stats = match state.rate_limit_service.get_comprehensive_stats().await {
        Ok(stats) => stats,
        Err(e) => {
            tracing::error!(error = %e, "Failed to get comprehensive rate limit stats");
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Calculate overview metrics
    let total_requests: u64 = endpoint_stats.iter().map(|s| s.total_requests).sum();
    let total_blocked: u64 = endpoint_stats.iter().map(|s| s.blocked_requests).sum();
    let overall_violation_rate = if total_requests > 0 {
        (total_blocked as f64) / (total_requests as f64)
    } else {
        0.0
    };

    // Get top violators across all endpoints
    let top_violators = match state.rate_limit_service.get_top_violators("all").await {
        Ok(violators) => violators,
        Err(e) => {
            tracing::warn!(error = %e, "Failed to get top violators");
            Vec::new()
        }
    };

    // Check for active alerts
    let mut active_alerts = Vec::new();
    for stats in &endpoint_stats {
        match state.rate_limit_service.check_alert_conditions(&stats.endpoint_class).await {
            Ok(mut alerts) => active_alerts.append(&mut alerts),
            Err(e) => {
                tracing::warn!(
                    endpoint_class = %stats.endpoint_class,
                    error = %e,
                    "Failed to check alert conditions"
                );
            }
        }
    }

    let overview = RateLimitOverview {
        total_requests,
        total_blocked,
        overall_violation_rate,
        active_endpoints: endpoint_stats.len() as u32,
        unique_ips: top_violators.len() as u32,
        alerts_count: active_alerts.len() as u32,
    };

    let dashboard = RateLimitDashboard {
        overview,
        endpoint_stats,
        top_violators,
        active_alerts,
        timestamp: Utc::now(),
    };

    Ok(Json(dashboard))
}

/// Get rate limiting statistics for a specific endpoint class
/// 
/// Returns detailed statistics for the specified endpoint class
#[utoipa::path(
    get,
    path = "/api/v1/monitoring/rate-limits/endpoints/{endpoint_class}",
    tags = ["Rate Limiting"],
    security(("bearer_auth" = [])),
    params(
        ("endpoint_class" = String, Path, description = "Endpoint class (e.g., 'bridge', 'auth', 'quantum')")
    ),
    responses(
        (status = 200, description = "Endpoint rate limiting statistics", body = RateLimitStats),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - Admin access required"),
        (status = 404, description = "Endpoint class not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_endpoint_rate_limits(
    State(state): State<AppState>,
    Path(endpoint_class): Path<String>,
    _admin: AdminAuth,
) -> Result<Json<RateLimitStats>, StatusCode> {
    tracing::info!(endpoint_class = %endpoint_class, "Getting endpoint rate limit stats");

    match state.rate_limit_service.get_endpoint_stats(&endpoint_class).await {
        Ok(stats) => Ok(Json(stats)),
        Err(e) => {
            tracing::error!(
                endpoint_class = %endpoint_class,
                error = %e,
                "Failed to get endpoint rate limit stats"
            );
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get top rate limiting violators
/// 
/// Returns list of IP addresses and users with the highest violation rates
#[utoipa::path(
    get,
    path = "/api/v1/monitoring/rate-limits/top-violators",
    tags = ["Rate Limiting"],
    security(("bearer_auth" = [])),
    params(RateLimitStatsQuery),
    responses(
        (status = 200, description = "Top rate limiting violators", body = Vec<ViolatorInfo>),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - Admin access required"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_top_violators(
    State(state): State<AppState>,
    Query(query): Query<RateLimitStatsQuery>,
    _admin: AdminAuth,
) -> Result<Json<Vec<ViolatorInfo>>, StatusCode> {
    tracing::info!("Getting top rate limit violators");

    let endpoint_class = query.endpoint_class.as_deref().unwrap_or("all");
    
    match state.rate_limit_service.get_top_violators(endpoint_class).await {
        Ok(violators) => Ok(Json(violators)),
        Err(e) => {
            tracing::error!(error = %e, "Failed to get top violators");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get real-time rate limiting metrics
/// 
/// Returns current system performance and rate limiting activity
#[utoipa::path(
    get,
    path = "/api/v1/monitoring/rate-limits/real-time",
    tags = ["Rate Limiting"],
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Real-time rate limiting metrics", body = RealTimeMetrics),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - Admin access required"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_real_time_metrics(
    State(state): State<AppState>,
    _admin: AdminAuth,
) -> Result<Json<RealTimeMetrics>, StatusCode> {
    tracing::debug!("Getting real-time rate limit metrics");

    // Get current statistics
    let endpoint_stats = match state.rate_limit_service.get_comprehensive_stats().await {
        Ok(stats) => stats,
        Err(e) => {
            tracing::error!(error = %e, "Failed to get real-time stats");
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Calculate real-time metrics
    let total_requests: u64 = endpoint_stats.iter().map(|s| s.total_requests).sum();
    let total_blocked: u64 = endpoint_stats.iter().map(|s| s.blocked_requests).sum();
    
    // Simple approximation of current RPS (in real implementation, this would be more sophisticated)
    let current_rps = total_requests as f64 / 3600.0; // rough estimate based on hourly data
    let current_blocks_per_minute = (total_blocked / 60) as u32; // rough estimate
    
    let metrics = RealTimeMetrics {
        current_rps,
        current_blocks_per_minute,
        active_rate_limits: endpoint_stats.len() as u32,
        redis_memory_usage: None, // TODO: Get from Redis INFO command
        timestamp: Utc::now(),
    };

    Ok(Json(metrics))
}

/// Get active alerts for rate limiting violations
/// 
/// Returns current alert conditions that require attention
#[utoipa::path(
    get,
    path = "/api/v1/monitoring/rate-limits/alerts",
    tags = ["Rate Limiting"],
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Active rate limiting alerts", body = Vec<AlertCondition>),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - Admin access required"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_active_alerts(
    State(state): State<AppState>,
    _admin: AdminAuth,
) -> Result<Json<Vec<AlertCondition>>, StatusCode> {
    tracing::info!("Getting active rate limit alerts");

    // Get endpoint classes to check
    let endpoint_classes = vec![
        "health", "auth", "bridge", "quantum", "user", "admin", "docs", "websocket", "general"
    ];

    let mut all_alerts = Vec::new();
    
    for endpoint_class in endpoint_classes {
        match state.rate_limit_service.check_alert_conditions(endpoint_class).await {
            Ok(mut alerts) => all_alerts.append(&mut alerts),
            Err(e) => {
                tracing::warn!(
                    endpoint_class = endpoint_class,
                    error = %e,
                    "Failed to check alert conditions"
                );
            }
        }
    }

    Ok(Json(all_alerts))
}