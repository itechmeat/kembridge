// src/handlers/manual_review.rs - Manual Review Workflow Handlers (Phase 5.2.4)
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use uuid::Uuid;
use utoipa::ToSchema;
use serde::{Deserialize, Serialize};
use tracing::{info, error, instrument};

use crate::{
    state::AppState,
    extractors::auth::AuthUser,
    models::review::{
        CreateReviewRequest, UpdateReviewRequest, ReviewQueueQuery,
        ReviewQueueListResponse, ReviewQueueResponse, ReviewDecision,
    },
};

/// Response for manual review endpoints
#[derive(Debug, Serialize, ToSchema)]
pub struct ManualReviewApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub message: String,
}

impl<T> ManualReviewApiResponse<T> {
    pub fn success(data: T, message: impl Into<String>) -> Self {
        Self {
            success: true,
            data: Some(data),
            message: message.into(),
        }
    }

    pub fn error(message: impl Into<String>) -> ManualReviewApiResponse<()> {
        ManualReviewApiResponse {
            success: false,
            data: None,
            message: message.into(),
        }
    }
}

/// Add transaction to manual review queue
#[utoipa::path(
    post,
    path = "/api/v1/admin/review/queue",
    tags = ["Manual Review"],
    summary = "Add transaction to manual review queue",
    description = "Adds a transaction to the manual review queue for administrator evaluation",
    request_body = CreateReviewRequest,
    responses(
        (status = 201, description = "Transaction added to review queue", body = ManualReviewApiResponse<ReviewQueueResponse>),
        (status = 400, description = "Invalid request"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    )
)]
#[instrument(skip(state, request))]
pub async fn add_to_review_queue(
    State(state): State<AppState>,
    _user: AuthUser, // TODO: In production, use AdminAuth extractor
    Json(request): Json<CreateReviewRequest>,
) -> Result<Json<ManualReviewApiResponse<ReviewQueueResponse>>, StatusCode> {
    info!(
        transaction_id = %request.transaction_id,
        user_id = %request.user_id,
        risk_score = request.risk_score,
        "Adding transaction to manual review queue"
    );

    match state.manual_review_service.add_to_review_queue(request).await {
        Ok(review_entry) => {
            let response = ReviewQueueResponse {
                review: review_entry,
                transaction_details: None, // TODO: Fetch from database
                user_risk_profile: None,   // TODO: Fetch from risk service
            };

            Ok(Json(ManualReviewApiResponse::success(
                response,
                "Transaction added to review queue successfully"
            )))
        }
        Err(e) => {
            error!(error = %e, "Failed to add transaction to review queue");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get review queue with filtering and pagination
#[utoipa::path(
    get,
    path = "/api/v1/admin/review/queue",
    tags = ["Manual Review"],
    summary = "Get review queue",
    description = "Retrieve the manual review queue with filtering and pagination options",
    params(
        ("status" = Option<String>, Query, description = "Filter by review status"),
        ("priority" = Option<String>, Query, description = "Filter by priority level"),
        ("page" = Option<u32>, Query, description = "Page number (1-based)"),
        ("per_page" = Option<u32>, Query, description = "Items per page (max 100)")
    ),
    responses(
        (status = 200, description = "Review queue retrieved", body = ManualReviewApiResponse<ReviewQueueListResponse>),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    )
)]
#[instrument(skip(state, query))]
pub async fn get_review_queue(
    State(state): State<AppState>,
    _user: AuthUser, // TODO: In production, use AdminAuth extractor
    Query(query): Query<ReviewQueueQuery>,
) -> Result<Json<ManualReviewApiResponse<ReviewQueueListResponse>>, StatusCode> {
    info!(
        status_filter = ?query.status,
        priority_filter = ?query.priority,
        page = ?query.page,
        per_page = ?query.per_page,
        "Fetching review queue"
    );

    match state.manual_review_service.get_review_queue(query).await {
        Ok(queue_response) => {
            Ok(Json(ManualReviewApiResponse::success(
                queue_response,
                "Review queue retrieved successfully"
            )))
        }
        Err(e) => {
            error!(error = %e, "Failed to fetch review queue");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Assign review to admin user
#[utoipa::path(
    put,
    path = "/api/v1/admin/review/{review_id}/assign",
    tags = ["Manual Review"],
    summary = "Assign review to admin",
    description = "Assign a pending review to an administrator for evaluation",
    params(
        ("review_id" = Uuid, Path, description = "Review queue entry ID")
    ),
    responses(
        (status = 200, description = "Review assigned successfully", body = ManualReviewApiResponse<ReviewQueueResponse>),
        (status = 404, description = "Review not found"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    )
)]
#[instrument(skip(state))]
pub async fn assign_review(
    State(state): State<AppState>,
    user: AuthUser, // Admin user who is taking the review
    Path(review_id): Path<Uuid>,
) -> Result<Json<ManualReviewApiResponse<ReviewQueueResponse>>, StatusCode> {
    info!(
        review_id = %review_id,
        admin_user_id = %user.user_id,
        "Assigning review to admin user"
    );

    match state.manual_review_service.assign_review(review_id, user.user_id).await {
        Ok(review_entry) => {
            let response = ReviewQueueResponse {
                review: review_entry,
                transaction_details: None, // TODO: Fetch from database
                user_risk_profile: None,   // TODO: Fetch from risk service
            };

            Ok(Json(ManualReviewApiResponse::success(
                response,
                "Review assigned successfully"
            )))
        }
        Err(e) => {
            error!(error = %e, review_id = %review_id, "Failed to assign review");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Make review decision (approve/reject)
#[utoipa::path(
    put,
    path = "/api/v1/admin/review/{review_id}/decision",
    tags = ["Manual Review"],
    summary = "Make review decision",
    description = "Make a final decision on a review (approve or reject)",
    params(
        ("review_id" = Uuid, Path, description = "Review queue entry ID")
    ),
    request_body = UpdateReviewRequest,
    responses(
        (status = 200, description = "Review decision made", body = ManualReviewApiResponse<ReviewDecision>),
        (status = 400, description = "Invalid status transition"),
        (status = 404, description = "Review not found"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    )
)]
#[instrument(skip(state, request))]
pub async fn make_review_decision(
    State(state): State<AppState>,
    user: AuthUser, // Admin user making the decision
    Path(review_id): Path<Uuid>,
    Json(request): Json<UpdateReviewRequest>,
) -> Result<Json<ManualReviewApiResponse<ReviewDecision>>, StatusCode> {
    info!(
        review_id = %review_id,
        admin_user_id = %user.user_id,
        decision = %request.status,
        "Making review decision"
    );

    match state.manual_review_service.make_review_decision(review_id, user.user_id, request).await {
        Ok(decision) => {
            Ok(Json(ManualReviewApiResponse::success(
                decision,
                "Review decision made successfully"
            )))
        }
        Err(e) => {
            error!(error = %e, review_id = %review_id, "Failed to make review decision");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get specific review details
#[utoipa::path(
    get,
    path = "/api/v1/admin/review/{review_id}",
    tags = ["Manual Review"],
    summary = "Get review details",
    description = "Get detailed information about a specific review",
    params(
        ("review_id" = Uuid, Path, description = "Review queue entry ID")
    ),
    responses(
        (status = 200, description = "Review details retrieved", body = ManualReviewApiResponse<ReviewQueueResponse>),
        (status = 404, description = "Review not found"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    )
)]
#[instrument(skip(state))]
pub async fn get_review_details(
    State(state): State<AppState>,
    _user: AuthUser, // TODO: In production, use AdminAuth extractor
    Path(review_id): Path<Uuid>,
) -> Result<Json<ManualReviewApiResponse<ReviewQueueResponse>>, StatusCode> {
    info!(review_id = %review_id, "Fetching review details");

    // TODO: Implement get_review_by_id method in ManualReviewService
    // For now, return a mock response
    let mock_response = state.manual_review_service.create_mock_review_response(1).await;

    Ok(Json(ManualReviewApiResponse::success(
        mock_response,
        "Review details retrieved successfully"
    )))
}

/// Escalate review manually
#[utoipa::path(
    put,
    path = "/api/v1/admin/review/{review_id}/escalate",
    tags = ["Manual Review"],
    summary = "Escalate review",
    description = "Manually escalate a review to higher priority",
    params(
        ("review_id" = Uuid, Path, description = "Review queue entry ID")
    ),
    responses(
        (status = 200, description = "Review escalated", body = ManualReviewApiResponse<ReviewQueueResponse>),
        (status = 404, description = "Review not found"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    )
)]
#[instrument(skip(state))]
pub async fn escalate_review(
    State(state): State<AppState>,
    _user: AuthUser, // TODO: In production, use AdminAuth extractor
    Path(review_id): Path<Uuid>,
) -> Result<Json<ManualReviewApiResponse<ReviewQueueResponse>>, StatusCode> {
    info!(review_id = %review_id, "Manually escalating review");

    match state.manual_review_service.escalate_review(review_id).await {
        Ok(review_entry) => {
            let response = ReviewQueueResponse {
                review: review_entry,
                transaction_details: None, // TODO: Fetch from database
                user_risk_profile: None,   // TODO: Fetch from risk service
            };

            Ok(Json(ManualReviewApiResponse::success(
                response,
                "Review escalated successfully"
            )))
        }
        Err(e) => {
            error!(error = %e, review_id = %review_id, "Failed to escalate review");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Check for reviews that need escalation (cron job endpoint)
#[utoipa::path(
    post,
    path = "/api/v1/admin/review/check-escalations",
    tags = ["Manual Review"],
    summary = "Check escalations",
    description = "Check for reviews that need escalation due to timeout (for automated processes)",
    responses(
        (status = 200, description = "Escalation check completed", body = ManualReviewApiResponse<Vec<Uuid>>),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    )
)]
#[instrument(skip(state))]
pub async fn check_escalations(
    State(state): State<AppState>,
    _user: AuthUser, // TODO: In production, use AdminAuth extractor
) -> Result<Json<ManualReviewApiResponse<Vec<Uuid>>>, StatusCode> {
    info!("Checking for reviews that need escalation");

    match state.manual_review_service.check_escalations().await {
        Ok(escalated_reviews) => {
            let escalated_ids: Vec<Uuid> = escalated_reviews.iter().map(|r| r.id).collect();
            let escalated_count = escalated_ids.len();
            
            info!(escalated_count = escalated_count, "Escalation check completed");

            Ok(Json(ManualReviewApiResponse::success(
                escalated_ids,
                format!("Found {} reviews requiring escalation", escalated_count)
            )))
        }
        Err(e) => {
            error!(error = %e, "Failed to check escalations");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}