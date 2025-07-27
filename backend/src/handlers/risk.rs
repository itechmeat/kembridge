// src/handlers/risk.rs - Risk Analysis HTTP Handlers (Phase 5.2.6)
use axum::{
    extract::{State, Path, Query},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use tracing::{info, warn, instrument};
use utoipa::{ToSchema, IntoParams};

use crate::{
    extractors::auth::AuthUser,
    models::risk::{UserRiskProfileResponse, RiskThresholds},
    services::{RiskIntegrationService, OperationDecision},
    state::AppState,
    middleware::error_handler::ApiError,
};

/// Request for user risk profile
#[derive(Debug, Deserialize, ToSchema, IntoParams)]
pub struct GetUserRiskProfileQuery {
    /// Number of days of history to include in the profile
    pub include_history_days: Option<i32>,
}

/// Response for risk profile endpoint
#[derive(Debug, Serialize, ToSchema)]
pub struct RiskProfileResponse {
    pub user_risk_profile: UserRiskProfileResponse,
    pub current_thresholds: RiskThresholds,
}

/// Request to update risk thresholds (admin only)
#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateRiskThresholdsRequest {
    pub low_threshold: f64,
    pub medium_threshold: f64,
    pub high_threshold: f64,
    pub auto_block_threshold: f64,
    pub manual_review_threshold: f64,
}

/// Response with current risk thresholds
#[derive(Debug, Serialize, ToSchema)]
pub struct RiskThresholdsResponse {
    pub thresholds: RiskThresholds,
    pub updated_by: Option<String>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Health check response for AI Engine
#[derive(Debug, Serialize, ToSchema)]
pub struct RiskEngineHealthResponse {
    pub ai_engine_healthy: bool,
    pub risk_analysis_enabled: bool,
    pub current_thresholds: RiskThresholds,
    pub checked_at: chrono::DateTime<chrono::Utc>,
}

/// Get user risk profile
#[utoipa::path(
    get,
    path = "/api/v1/risk/profile/{user_id}",
    params(
        ("user_id" = Uuid, Path, description = "User ID to get risk profile for"),
        GetUserRiskProfileQuery
    ),
    responses(
        (status = 200, description = "User risk profile retrieved successfully", body = RiskProfileResponse),
        (status = 400, description = "Invalid user ID or parameters"),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Access denied - can only access own profile unless admin"),
        (status = 404, description = "User not found"),
        (status = 500, description = "Risk analysis service unavailable"),
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Risk Analysis"
)]
#[instrument(skip(state), fields(user_id = %user_id))]
pub async fn get_user_risk_profile(
    State(state): State<AppState>,
    user: AuthUser,
    Path(user_id): Path<Uuid>,
    Query(query): Query<GetUserRiskProfileQuery>,
) -> Result<Json<RiskProfileResponse>, ApiError> {
    // Users can only access their own profile unless they're admin
    if user_id != user.user_id && !user.is_admin() {
        warn!(
            requester = %user.user_id,
            requested_user = %user_id,
            "Unauthorized attempt to access user risk profile"
        );
        return Err(ApiError::Authorization);
    }

    let risk_service = &state.risk_integration_service;
    
    let user_profile = risk_service
        .get_user_risk_profile(user_id, query.include_history_days)
        .await
        .map_err(|e| {
            warn!(user_id = %user_id, error = %e, "Failed to get user risk profile");
            ApiError::internal_server_error(format!("Risk analysis failed: {}", e))
        })?;

    let current_thresholds = risk_service.risk_thresholds();

    info!(
        user_id = %user_id,
        risk_score = user_profile.overall_risk_score,
        risk_level = ?user_profile.risk_level,
        "User risk profile retrieved"
    );

    Ok(Json(RiskProfileResponse {
        user_risk_profile: user_profile,
        current_thresholds,
    }))
}

/// Get current risk analysis thresholds
#[utoipa::path(
    get,
    path = "/api/v1/risk/thresholds",
    responses(
        (status = 200, description = "Risk thresholds retrieved successfully", body = RiskThresholdsResponse),
        (status = 401, description = "Authentication required"),
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Risk Analysis"
)]
#[instrument(skip(state))]
pub async fn get_risk_thresholds(
    State(state): State<AppState>,
    _user: AuthUser,
) -> Result<Json<RiskThresholdsResponse>, ApiError> {
    let risk_service = &state.risk_integration_service;
    let thresholds = risk_service.risk_thresholds().clone();

    Ok(Json(RiskThresholdsResponse {
        thresholds,
        updated_by: None, // TODO: Track who last updated thresholds
        updated_at: chrono::Utc::now(),
    }))
}

/// Update risk analysis thresholds (admin only)
#[utoipa::path(
    put,
    path = "/api/v1/risk/thresholds",
    request_body = UpdateRiskThresholdsRequest,
    responses(
        (status = 200, description = "Risk thresholds updated successfully", body = RiskThresholdsResponse),
        (status = 400, description = "Invalid threshold values"),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Admin access required"),
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Risk Analysis"
)]
#[instrument(skip(state, request), fields(admin_user = %user.user_id))]
pub async fn update_risk_thresholds(
    State(state): State<AppState>,
    user: AuthUser,
    Json(request): Json<UpdateRiskThresholdsRequest>,
) -> Result<Json<RiskThresholdsResponse>, ApiError> {
    // Only admin users can update risk thresholds
    if !user.is_admin() {
        warn!(user_id = %user.user_id, "Unauthorized attempt to update risk thresholds");
        return Err(ApiError::Authorization);
    }

    // Validate threshold values
    if request.low_threshold < 0.0 || request.low_threshold > 1.0 ||
       request.medium_threshold < 0.0 || request.medium_threshold > 1.0 ||
       request.high_threshold < 0.0 || request.high_threshold > 1.0 ||
       request.auto_block_threshold < 0.0 || request.auto_block_threshold > 1.0 ||
       request.manual_review_threshold < 0.0 || request.manual_review_threshold > 1.0 {
        return Err(ApiError::bad_request("Threshold values must be between 0.0 and 1.0"));
    }

    if request.low_threshold >= request.medium_threshold ||
       request.medium_threshold >= request.high_threshold {
        return Err(ApiError::bad_request("Thresholds must be in ascending order: low < medium < high"));
    }

    let new_thresholds = RiskThresholds {
        low_threshold: request.low_threshold,
        medium_threshold: request.medium_threshold,
        high_threshold: request.high_threshold,
        auto_block_threshold: request.auto_block_threshold,
        manual_review_threshold: request.manual_review_threshold,
    };

    // Update thresholds in the service
    state.risk_integration_service.update_risk_thresholds(new_thresholds.clone());

    info!(
        admin_user = %user.user_id,
        new_thresholds = ?new_thresholds,
        "Risk thresholds updated"
    );

    Ok(Json(RiskThresholdsResponse {
        thresholds: new_thresholds,
        updated_by: Some(user.user_id.to_string()),
        updated_at: chrono::Utc::now(),
    }))
}

/// Check AI Engine health and risk analysis status
#[utoipa::path(
    get,
    path = "/api/v1/risk/health",
    responses(
        (status = 200, description = "Risk engine health status", body = RiskEngineHealthResponse),
        (status = 401, description = "Authentication required"),
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Risk Analysis"
)]
#[instrument(skip(state))]
pub async fn get_risk_engine_health(
    State(state): State<AppState>,
    _user: AuthUser,
) -> Result<Json<RiskEngineHealthResponse>, ApiError> {
    let risk_service = &state.risk_integration_service;
    
    let ai_engine_healthy = risk_service.health_check().await;
    let current_thresholds = risk_service.risk_thresholds();

    info!(
        ai_engine_healthy = ai_engine_healthy,
        "Risk engine health check completed"
    );

    Ok(Json(RiskEngineHealthResponse {
        ai_engine_healthy,
        risk_analysis_enabled: state.config.enable_ai_risk_analysis,
        current_thresholds,
        checked_at: chrono::Utc::now(),
    }))
}

/// Test risk analysis with mock data (development only)
#[utoipa::path(
    post,
    path = "/api/v1/risk/test",
    responses(
        (status = 200, description = "Risk analysis test completed"),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Only available in development environment"),
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Risk Analysis"
)]
#[instrument(skip(state))]
pub async fn test_risk_analysis(
    State(state): State<AppState>,
    user: AuthUser,
) -> Result<Json<serde_json::Value>, ApiError> {
    // Only allow in development environment
    if !matches!(state.config.environment, crate::config::Environment::Development) {
        return Err(ApiError::Authorization);
    }

    let risk_service = &state.risk_integration_service;
    
    // Create a mock swap operation for testing
    let mock_swap = kembridge_bridge::SwapOperation {
        swap_id: Uuid::new_v4(),
        user_id: user.user_id,
        from_chain: "ethereum".to_string(),
        to_chain: "near".to_string(),
        amount: 1_000_000_000_000_000_000, // 1 ETH in wei
        recipient: "test.testnet".to_string(),
        status: kembridge_bridge::SwapStatus::Initialized,
        quantum_key_id: None,
        eth_tx_hash: None,
        near_tx_hash: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        expires_at: chrono::Utc::now() + chrono::Duration::minutes(30),
    };

    match risk_service.analyze_bridge_risk(&mock_swap).await {
        Ok(risk_response) => {
            let is_admin = user.is_admin();
            let decision = risk_service.should_allow_operation_with_user(&risk_response, Some(is_admin));
            
            // Log the decision for audit trail
            risk_service.log_risk_decision(&mock_swap, &risk_response, &decision, Some(is_admin));
            
            info!(
                user_id = %user.user_id,
                is_admin = is_admin,
                risk_score = risk_response.risk_score,
                decision = ?decision,
                "Risk analysis test completed"
            );

            Ok(Json(serde_json::json!({
                "test_completed": true,
                "mock_swap_id": mock_swap.swap_id,
                "risk_analysis": {
                    "risk_score": risk_response.risk_score,
                    "risk_level": risk_response.risk_level,
                },
                "decision": {
                    "allowed": decision.is_allowed(),
                    "requires_review": decision.requires_review(),
                    "blocked": decision.is_blocked(),
                    "risk_score": decision.risk_score(),
                    "reason": decision.reason(),
                },
                "test_context": {
                    "is_admin_user": is_admin,
                    "admin_bypass_enabled": state.config.risk_admin_bypass_enabled,
                    "thresholds": risk_service.risk_thresholds(),
                }
            })))
        }
        Err(e) => {
            warn!(user_id = %user.user_id, error = %e, "Risk analysis test failed");
            Ok(Json(serde_json::json!({
                "test_completed": false,
                "error": e.to_string(),
                "fallback_behavior": "Risk analysis disabled or AI Engine unavailable"
            })))
        }
    }
}