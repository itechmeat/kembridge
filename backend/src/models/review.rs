// src/models/review.rs - Manual Review Workflow Models (Phase 5.2.4)
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use utoipa::ToSchema;
use std::fmt;

/// Review status for manual review queue
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ReviewStatus {
    /// Pending review by administrator
    Pending,
    /// Under review by an administrator
    InReview,
    /// Approved by administrator - transaction can proceed
    Approved,
    /// Rejected by administrator - transaction blocked
    Rejected,
    /// Escalated to higher level review
    Escalated,
    /// Automatically expired due to timeout
    Expired,
}

impl ReviewStatus {
    /// Check if review is in a final state
    pub fn is_final(&self) -> bool {
        matches!(self, ReviewStatus::Approved | ReviewStatus::Rejected | ReviewStatus::Expired)
    }

    /// Check if review is active (can be acted upon)
    pub fn is_active(&self) -> bool {
        matches!(self, ReviewStatus::Pending | ReviewStatus::InReview | ReviewStatus::Escalated)
    }
}

impl fmt::Display for ReviewStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReviewStatus::Pending => write!(f, "pending"),
            ReviewStatus::InReview => write!(f, "in_review"),
            ReviewStatus::Approved => write!(f, "approved"),
            ReviewStatus::Rejected => write!(f, "rejected"),
            ReviewStatus::Escalated => write!(f, "escalated"),
            ReviewStatus::Expired => write!(f, "expired"),
        }
    }
}

/// Priority level for manual review
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ReviewPriority {
    Low,
    Medium,
    High,
    Critical,
}

impl ReviewPriority {
    /// Get priority based on risk score
    pub fn from_risk_score(risk_score: f64) -> Self {
        match risk_score {
            s if s >= 0.95 => ReviewPriority::Critical,
            s if s >= 0.85 => ReviewPriority::High,
            s if s >= 0.75 => ReviewPriority::Medium,
            _ => ReviewPriority::Low,
        }
    }

    /// Get escalation timeout in hours based on priority
    pub fn escalation_timeout_hours(&self) -> i64 {
        match self {
            ReviewPriority::Critical => 2,   // 2 hours
            ReviewPriority::High => 6,       // 6 hours
            ReviewPriority::Medium => 24,    // 24 hours
            ReviewPriority::Low => 72,       // 72 hours
        }
    }
}

/// Manual review entry in the review queue
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ReviewQueueEntry {
    pub id: Uuid,
    pub transaction_id: Uuid,
    pub user_id: Uuid,
    pub risk_score: f64,
    pub status: ReviewStatus,
    pub priority: ReviewPriority,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub assigned_to: Option<Uuid>, // Admin user ID
    pub assigned_at: Option<DateTime<Utc>>,
    pub reviewed_by: Option<Uuid>, // Admin user ID who made final decision
    pub reviewed_at: Option<DateTime<Utc>>,
    pub review_reason: Option<String>,
    pub escalation_count: i32,
    pub last_escalated_at: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,
}

/// Request to add transaction to manual review queue
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateReviewRequest {
    pub transaction_id: Uuid,
    pub user_id: Uuid,
    pub risk_score: f64,
    pub priority: Option<ReviewPriority>,
    pub reason: String,
    pub metadata: Option<serde_json::Value>,
}

/// Request to update review status
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateReviewRequest {
    pub status: ReviewStatus,
    pub reason: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

/// Response for review queue entry
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ReviewQueueResponse {
    pub review: ReviewQueueEntry,
    pub transaction_details: Option<TransactionSummary>,
    pub user_risk_profile: Option<UserRiskSummary>,
}

/// Summary of transaction for review context
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TransactionSummary {
    pub transaction_id: Uuid,
    pub user_id: Uuid,
    pub source_chain: String,
    pub destination_chain: String,
    pub source_token: String,
    pub destination_token: String,
    pub amount_in: f64,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub quantum_key_id: Option<String>,
}

/// Summary of user risk profile for review context
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UserRiskSummary {
    pub user_id: Uuid,
    pub overall_risk_score: f64,
    pub risk_level: String,
    pub transaction_count: u32,
    pub total_volume: f64,
    pub first_transaction: Option<DateTime<Utc>>,
    pub last_transaction: Option<DateTime<Utc>>,
    pub flags_count: u32,
}

/// Review decision result
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ReviewDecision {
    pub review_id: Uuid,
    pub transaction_id: Uuid,
    pub decision: ReviewStatus,
    pub reason: String,
    pub reviewed_by: Uuid,
    pub reviewed_at: DateTime<Utc>,
    pub metadata: Option<serde_json::Value>,
}

/// Query parameters for review queue listing
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct ReviewQueueQuery {
    /// Filter by review status
    pub status: Option<ReviewStatus>,
    /// Filter by priority level
    pub priority: Option<ReviewPriority>,
    /// Filter by assigned admin user
    pub assigned_to: Option<Uuid>,
    /// Only show expired reviews
    pub expired_only: Option<bool>,
    /// Pagination: page number (1-based)
    pub page: Option<u32>,
    /// Pagination: items per page (default 20, max 100)
    pub per_page: Option<u32>,
    /// Sort by field (created_at, updated_at, expires_at, risk_score)
    pub sort_by: Option<String>,
    /// Sort order (asc, desc)
    pub sort_order: Option<String>,
}

impl Default for ReviewQueueQuery {
    fn default() -> Self {
        Self {
            status: None,
            priority: None,
            assigned_to: None,
            expired_only: Some(false),
            page: Some(1),
            per_page: Some(20),
            sort_by: Some("created_at".to_string()),
            sort_order: Some("desc".to_string()),
        }
    }
}

/// Paginated response for review queue
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ReviewQueueListResponse {
    pub reviews: Vec<ReviewQueueResponse>,
    pub pagination: PaginationInfo,
    pub statistics: ReviewQueueStats,
}

/// Pagination information
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PaginationInfo {
    pub current_page: u32,
    pub per_page: u32,
    pub total_items: u64,
    pub total_pages: u32,
    pub has_next: bool,
    pub has_prev: bool,
}

/// Review queue statistics
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ReviewQueueStats {
    pub total_pending: u64,
    pub total_in_review: u64,
    pub total_escalated: u64,
    pub total_expired: u64,
    pub avg_resolution_time_hours: Option<f64>,
    pub critical_count: u64,
    pub high_priority_count: u64,
}

/// Notification for admin about review queue
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ReviewNotification {
    pub id: Uuid,
    pub notification_type: NotificationType,
    pub review_id: Uuid,
    pub transaction_id: Uuid,
    pub priority: ReviewPriority,
    pub message: String,
    pub created_at: DateTime<Utc>,
    pub sent_to: Vec<Uuid>, // Admin user IDs
    pub acknowledged_by: Vec<Uuid>, // Admin user IDs who acknowledged
}

/// Type of review notification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum NotificationType {
    /// New review added to queue
    NewReview,
    /// Review escalated due to timeout
    Escalated,
    /// Critical priority review added
    CriticalReview,
    /// Review queue backlog warning
    QueueBacklog,
    /// Review expired without decision
    ReviewExpired,
}

/// Error types for review workflow
#[derive(Debug, thiserror::Error)]
pub enum ReviewError {
    #[error("Review not found: {0}")]
    NotFound(Uuid),

    #[error("Review already finalized")]
    AlreadyFinalized,

    #[error("Invalid status transition: {from} -> {to}")]
    InvalidStatusTransition { from: ReviewStatus, to: ReviewStatus },

    #[error("Review assignment conflict: already assigned to {assigned_to}")]
    AssignmentConflict { assigned_to: Uuid },

    #[error("Unauthorized review action")]
    Unauthorized,

    #[error("Review expired")]
    Expired,

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}