// src/services/manual_review/mod.rs - Manual Review service modules

pub mod database;
pub mod statistics;
pub mod notifications;

use std::sync::Arc;
use uuid::Uuid;
use chrono::{DateTime, Utc, Duration};
use sqlx::PgPool;
use tracing::{info, warn, error, debug, instrument};

use crate::models::review::{
    ReviewQueueEntry, ReviewStatus, ReviewPriority, CreateReviewRequest,
    UpdateReviewRequest, ReviewQueueQuery, ReviewQueueListResponse,
    ReviewQueueResponse, ReviewDecision, PaginationInfo, ReviewQueueStats,
    ReviewNotification, NotificationType, ReviewError,
    TransactionSummary, UserRiskSummary,
};

use self::database::ReviewDatabase;
use self::statistics::{ReviewStatistics, QueueHealthMetrics};
use self::notifications::ReviewNotificationService;

/// Service for managing manual review queue
#[derive(Clone)]
pub struct ManualReviewService {
    database: Arc<ReviewDatabase>,
    statistics: Arc<ReviewStatistics>,
    notifications: Arc<ReviewNotificationService>,
    escalation_enabled: bool,
    max_queue_size: u32,
    default_expiration_hours: i64,
}

impl ManualReviewService {
    /// Create new manual review service
    pub fn new(db_pool: PgPool) -> Self {
        Self {
            database: Arc::new(ReviewDatabase::new(db_pool.clone())),
            statistics: Arc::new(ReviewStatistics::new(db_pool.clone())),
            notifications: Arc::new(ReviewNotificationService::new(true)),
            escalation_enabled: true,
            max_queue_size: 1000, // Configurable queue size limit
            default_expiration_hours: 72, // Default 72 hours expiration
        }
    }

    /// Add transaction to manual review queue
    #[instrument(skip(self, request))]
    pub async fn add_to_review_queue(
        &self,
        request: CreateReviewRequest,
    ) -> Result<ReviewQueueEntry, ReviewError> {
        let now = Utc::now();
        let priority = request.priority.unwrap_or_else(|| {
            ReviewPriority::from_risk_score(request.risk_score)
        });
        
        let expiration_hours = priority.escalation_timeout_hours();
        let expires_at = now + Duration::hours(expiration_hours);
        
        let review_id = Uuid::new_v4();
        
        let review_entry = ReviewQueueEntry {
            id: review_id,
            transaction_id: request.transaction_id,
            user_id: request.user_id,
            risk_score: request.risk_score,
            status: ReviewStatus::Pending,
            priority,
            created_at: now,
            updated_at: now,
            expires_at,
            assigned_to: None,
            assigned_at: None,
            reviewed_by: None,
            reviewed_at: None,
            review_reason: Some(request.reason),
            escalation_count: 0,
            last_escalated_at: None,
            metadata: request.metadata.unwrap_or_else(|| serde_json::json!({})),
        };

        info!(
            review_id = %review_id,
            transaction_id = %request.transaction_id,
            user_id = %request.user_id,
            risk_score = request.risk_score,
            priority = ?review_entry.priority,
            "Adding transaction to manual review queue"
        );

        // Save to database
        self.database.save_review_entry(&review_entry).await?;

        // Send notification
        if let Err(e) = self.notifications.send_review_notification(
            &review_entry,
            NotificationType::NewReview,
        ).await {
            warn!(review_id = %review_id, error = %e, "Failed to send creation notification");
        }

        Ok(review_entry)
    }

    /// Assign review to admin user
    #[instrument(skip(self))]
    pub async fn assign_review(&self, review_id: Uuid, admin_user_id: Uuid) -> Result<ReviewQueueEntry, ReviewError> {
        info!(review_id = %review_id, admin_user_id = %admin_user_id, "Assigning review to admin user");

        // Update database to assign review
        let review_entry = self.database.assign_review(review_id, admin_user_id).await?;

        // Send assignment notification
        if let Err(e) = self.notifications.send_review_notification(
            &review_entry,
            NotificationType::CriticalReview,
        ).await {
            warn!(review_id = %review_id, error = %e, "Failed to send assignment notification");
        }

        Ok(review_entry)
    }

    /// Make review decision
    #[instrument(skip(self, decision))]
    pub async fn make_review_decision(&self, decision: ReviewDecision) -> Result<(), ReviewError> {
        info!(
            review_id = %decision.review_id,
            decision = ?decision.decision,
            reviewed_by = %decision.reviewed_by,
            "Processing review decision"
        );

        // Update database with decision
        self.database.update_review_status(&decision).await?;

        // Create a review entry for notification (simplified)
        let review_entry = ReviewQueueEntry {
            id: decision.review_id,
            transaction_id: decision.transaction_id,
            user_id: Uuid::new_v4(), // Would be fetched from DB in real implementation
            risk_score: 0.0,
            status: decision.decision.clone(),
            priority: ReviewPriority::Medium,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            expires_at: chrono::Utc::now(),
            assigned_to: Some(decision.reviewed_by),
            assigned_at: Some(decision.reviewed_at),
            reviewed_by: Some(decision.reviewed_by),
            reviewed_at: Some(decision.reviewed_at),
            review_reason: Some(decision.reason.clone()),
            escalation_count: 0,
            last_escalated_at: None,
            metadata: decision.metadata.clone().unwrap_or_else(|| serde_json::json!({})),
        };

        // Send completion notification
        if let Err(e) = self.notifications.send_review_notification(
            &review_entry,
            NotificationType::NewReview,
        ).await {
            warn!(review_id = %decision.review_id, error = %e, "Failed to send completion notification");
        }

        Ok(())
    }

    /// Get specific review by ID
    #[instrument(skip(self))]
    pub async fn get_review_by_id(&self, review_id: Uuid) -> Result<ReviewQueueEntry, ReviewError> {
        debug!(review_id = %review_id, "Fetching review by ID");
        self.database.get_review_by_id(review_id).await
    }

    /// Get review queue with pagination and filtering
    #[instrument(skip(self, query))]
    pub async fn get_review_queue(
        &self,
        query: ReviewQueueQuery,
        page: u32,
        per_page: u32,
    ) -> Result<ReviewQueueListResponse, ReviewError> {
        debug!(page = page, per_page = per_page, "Fetching review queue");

        // Get reviews from database with filters
        let (reviews, total_items) = self.database.get_reviews_with_pagination(&query, page, per_page).await?;
        let total_pages = ((total_items as f64) / (per_page as f64)).ceil() as u32;

        let pagination = PaginationInfo {
            current_page: page,
            per_page,
            total_items,
            total_pages,
            has_next: page < total_pages,
            has_prev: page > 1,
        };

        let statistics = self.get_queue_statistics().await?;

        Ok(ReviewQueueListResponse {
            reviews,
            pagination,
            statistics,
        })
    }

    /// Get queue statistics
    #[instrument(skip(self))]
    pub async fn get_queue_statistics(&self) -> Result<ReviewQueueStats, ReviewError> {
        self.statistics.get_queue_statistics().await
    }

    /// Get queue health metrics
    #[instrument(skip(self))]
    pub async fn get_queue_health(&self) -> Result<QueueHealthMetrics, ReviewError> {
        let stats = self.get_queue_statistics().await?;
        Ok(self.statistics.calculate_queue_health(&stats))
    }

    /// Check for expired reviews and escalate
    #[instrument(skip(self))]
    pub async fn check_expired_reviews(&self) -> Result<Vec<ReviewQueueEntry>, ReviewError> {
        if !self.escalation_enabled {
            return Ok(vec![]);
        }

        info!("Checking for expired reviews");

        // Get expired reviews from database
        let expired_reviews = self.database.get_expired_reviews().await?;

        for review in &expired_reviews {
            if let Err(e) = self.escalate_review(review.id).await {
                error!(review_id = %review.id, error = %e, "Failed to escalate expired review");
            }
        }

        Ok(expired_reviews)
    }

    /// Escalate review due to timeout
    #[instrument(skip(self))]
    pub async fn escalate_review(&self, review_id: Uuid) -> Result<ReviewQueueEntry, ReviewError> {
        info!(review_id = %review_id, "Escalating review due to timeout");

        // Update database to escalate review
        let escalated_review = self.database.escalate_review(review_id).await?;

        // Send escalation notification
        if let Err(e) = self.notifications.send_escalation_alert(&escalated_review).await {
            warn!(review_id = %review_id, error = %e, "Failed to send escalation notification");
        }

        Ok(escalated_review)
    }
}
