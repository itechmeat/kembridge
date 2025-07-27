// src/services/manual_review.rs - Manual Review Queue Management Service (Phase 5.2.4)
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

/// Service for managing manual review queue
#[derive(Clone)]
pub struct ManualReviewService {
    db_pool: PgPool,
    escalation_enabled: bool,
    max_queue_size: u32,
    default_expiration_hours: i64,
}

impl ManualReviewService {
    /// Create new manual review service
    pub fn new(db_pool: PgPool) -> Self {
        Self {
            db_pool,
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
        
        // Create review entry in database (simplified in-memory for now)
        let review_entry = ReviewQueueEntry {
            id: review_id,
            transaction_id: request.transaction_id,
            user_id: request.user_id,
            risk_score: request.risk_score,
            status: ReviewStatus::Pending,
            priority: priority.clone(),
            created_at: now,
            updated_at: now,
            expires_at,
            assigned_to: None,
            assigned_at: None,
            reviewed_by: None,
            reviewed_at: None,
            review_reason: Some(request.reason.clone()),
            escalation_count: 0,
            last_escalated_at: None,
            metadata: request.metadata.unwrap_or_else(|| serde_json::json!({})),
        };

        info!(
            review_id = %review_id,
            transaction_id = %request.transaction_id,
            user_id = %request.user_id,
            risk_score = request.risk_score,
            priority = ?priority,
            expires_at = %expires_at,
            "Transaction added to manual review queue"
        );

        // TODO: Save to database
        // self.save_review_entry(&review_entry).await?;

        // Send notification for high priority reviews
        if matches!(priority, ReviewPriority::Critical | ReviewPriority::High) {
            if let Err(e) = self.send_review_notification(
                &review_entry,
                NotificationType::NewReview,
            ).await {
                warn!(review_id = %review_id, error = %e, "Failed to send review notification");
            }
        }

        Ok(review_entry)
    }

    /// Assign review to admin user
    #[instrument(skip(self))]
    pub async fn assign_review(
        &self,
        review_id: Uuid,
        admin_user_id: Uuid,
    ) -> Result<ReviewQueueEntry, ReviewError> {
        info!(
            review_id = %review_id,
            admin_user_id = %admin_user_id,
            "Assigning review to admin user"
        );

        // TODO: Implement database update
        // For now, return a mock entry
        let now = Utc::now();
        let review_entry = ReviewQueueEntry {
            id: review_id,
            transaction_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            risk_score: 0.75,
            status: ReviewStatus::InReview,
            priority: ReviewPriority::Medium,
            created_at: now - Duration::hours(1),
            updated_at: now,
            expires_at: now + Duration::hours(23),
            assigned_to: Some(admin_user_id),
            assigned_at: Some(now),
            reviewed_by: None,
            reviewed_at: None,
            review_reason: Some("High risk score requires manual review".to_string()),
            escalation_count: 0,
            last_escalated_at: None,
            metadata: serde_json::json!({}),
        };

        Ok(review_entry)
    }

    /// Make review decision (approve/reject)
    #[instrument(skip(self, request))]
    pub async fn make_review_decision(
        &self,
        review_id: Uuid,
        admin_user_id: Uuid,
        request: UpdateReviewRequest,
    ) -> Result<ReviewDecision, ReviewError> {
        let now = Utc::now();

        // Validate status transition
        if !matches!(request.status, ReviewStatus::Approved | ReviewStatus::Rejected) {
            return Err(ReviewError::InvalidStatusTransition {
                from: ReviewStatus::InReview,
                to: request.status,
            });
        }

        let decision = ReviewDecision {
            review_id,
            transaction_id: Uuid::new_v4(), // TODO: Get from database
            decision: request.status.clone(),
            reason: request.reason.unwrap_or_else(|| {
                match request.status {
                    ReviewStatus::Approved => "Approved by admin review".to_string(),
                    ReviewStatus::Rejected => "Rejected by admin review".to_string(),
                    _ => "Review decision made".to_string(),
                }
            }),
            reviewed_by: admin_user_id,
            reviewed_at: now,
            metadata: request.metadata,
        };

        info!(
            review_id = %review_id,
            admin_user_id = %admin_user_id,
            decision = %decision.decision,
            reason = %decision.reason,
            "Review decision made"
        );

        // TODO: Update database
        // self.update_review_status(&decision).await?;

        Ok(decision)
    }

    /// Get review queue with filtering and pagination
    #[instrument(skip(self, query))]
    pub async fn get_review_queue(
        &self,
        query: ReviewQueueQuery,
    ) -> Result<ReviewQueueListResponse, ReviewError> {
        let page = query.page.unwrap_or(1);
        let per_page = query.per_page.unwrap_or(20).min(100); // Cap at 100 items

        debug!(
            status_filter = ?query.status,
            priority_filter = ?query.priority,
            page = page,
            per_page = per_page,
            "Fetching review queue"
        );

        // TODO: Implement database query with filters
        // For now, return mock data
        let total_items = 42u64;
        let total_pages = ((total_items as f64) / (per_page as f64)).ceil() as u32;

        let mock_reviews = vec![
            self.create_mock_review_response(1).await,
            self.create_mock_review_response(2).await,
        ];

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
            reviews: mock_reviews,
            pagination,
            statistics,
        })
    }

    /// Get review queue statistics
    #[instrument(skip(self))]
    pub async fn get_queue_statistics(&self) -> Result<ReviewQueueStats, ReviewError> {
        // TODO: Implement real database queries
        Ok(ReviewQueueStats {
            total_pending: 15,
            total_in_review: 5,
            total_escalated: 2,
            total_expired: 3,
            avg_resolution_time_hours: Some(18.5),
            critical_count: 1,
            high_priority_count: 6,
        })
    }

    /// Check for reviews that need escalation
    #[instrument(skip(self))]
    pub async fn check_escalations(&self) -> Result<Vec<ReviewQueueEntry>, ReviewError> {
        if !self.escalation_enabled {
            return Ok(vec![]);
        }

        let now = Utc::now();
        debug!("Checking for reviews that need escalation");

        // TODO: Implement database query for reviews past escalation time
        // For now, return empty vector
        Ok(vec![])
    }

    /// Escalate review due to timeout
    #[instrument(skip(self))]
    pub async fn escalate_review(&self, review_id: Uuid) -> Result<ReviewQueueEntry, ReviewError> {
        let now = Utc::now();

        info!(review_id = %review_id, "Escalating review due to timeout");

        // TODO: Implement database update
        let escalated_review = ReviewQueueEntry {
            id: review_id,
            transaction_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            risk_score: 0.85,
            status: ReviewStatus::Escalated,
            priority: ReviewPriority::High,
            created_at: now - Duration::hours(25),
            updated_at: now,
            expires_at: now + Duration::hours(6), // Shorter timeout for escalated reviews
            assigned_to: None,
            assigned_at: None,
            reviewed_by: None,
            reviewed_at: None,
            review_reason: Some("Escalated due to timeout".to_string()),
            escalation_count: 1,
            last_escalated_at: Some(now),
            metadata: serde_json::json!({
                "escalated_at": now,
                "escalation_reason": "timeout"
            }),
        };

        // Send escalation notification
        if let Err(e) = self.send_review_notification(
            &escalated_review,
            NotificationType::Escalated,
        ).await {
            warn!(review_id = %review_id, error = %e, "Failed to send escalation notification");
        }

        Ok(escalated_review)
    }

    /// Send notification about review queue events
    #[instrument(skip(self, review_entry))]
    async fn send_review_notification(
        &self,
        review_entry: &ReviewQueueEntry,
        notification_type: NotificationType,
    ) -> Result<(), ReviewError> {
        let message = match notification_type {
            NotificationType::NewReview => {
                format!(
                    "New {} priority review added for transaction {}",
                    match review_entry.priority {
                        ReviewPriority::Critical => "CRITICAL",
                        ReviewPriority::High => "HIGH",
                        ReviewPriority::Medium => "MEDIUM",
                        ReviewPriority::Low => "LOW",
                    },
                    review_entry.transaction_id
                )
            }
            NotificationType::Escalated => {
                format!(
                    "Review {} escalated due to timeout (risk score: {:.2})",
                    review_entry.id,
                    review_entry.risk_score
                )
            }
            NotificationType::CriticalReview => {
                format!(
                    "CRITICAL: High-risk transaction {} requires immediate review",
                    review_entry.transaction_id
                )
            }
            NotificationType::QueueBacklog => {
                "Review queue backlog detected - immediate attention required".to_string()
            }
            NotificationType::ReviewExpired => {
                format!("Review {} expired without decision", review_entry.id)
            }
        };

        info!(
            review_id = %review_entry.id,
            notification_type = ?notification_type,
            message = %message,
            "Sending review notification"
        );

        // TODO: Implement actual notification system (email, Slack, etc.)
        // For now, just log the notification

        Ok(())
    }

    /// Create mock review response for testing
    pub async fn create_mock_review_response(&self, index: u32) -> ReviewQueueResponse {
        let review_id = Uuid::new_v4();
        let transaction_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let now = Utc::now();

        let review = ReviewQueueEntry {
            id: review_id,
            transaction_id,
            user_id,
            risk_score: 0.7 + (index as f64 * 0.1),
            status: if index % 2 == 0 { ReviewStatus::Pending } else { ReviewStatus::InReview },
            priority: if index == 1 { ReviewPriority::High } else { ReviewPriority::Medium },
            created_at: now - Duration::hours(index as i64),
            updated_at: now,
            expires_at: now + Duration::hours(24 - (index as i64)),
            assigned_to: if index % 2 == 1 { Some(Uuid::new_v4()) } else { None },
            assigned_at: if index % 2 == 1 { Some(now - Duration::minutes(30)) } else { None },
            reviewed_by: None,
            reviewed_at: None,
            review_reason: Some(format!("Risk score {} requires manual review", 0.7 + (index as f64 * 0.1))),
            escalation_count: 0,
            last_escalated_at: None,
            metadata: serde_json::json!({
                "source_chain": "ethereum",
                "destination_chain": "near",
                "amount": 1000.0 * index as f64
            }),
        };

        let transaction_details = Some(TransactionSummary {
            transaction_id,
            user_id,
            source_chain: "ethereum".to_string(),
            destination_chain: "near".to_string(),
            source_token: "ETH".to_string(),
            destination_token: "wETH".to_string(),
            amount_in: 1000.0 * index as f64,
            status: "pending_review".to_string(),
            created_at: now - Duration::hours(index as i64),
            quantum_key_id: Some(Uuid::new_v4().to_string()),
        });

        let user_risk_profile = Some(UserRiskSummary {
            user_id,
            overall_risk_score: 0.6 + (index as f64 * 0.05),
            risk_level: "medium".to_string(),
            transaction_count: index * 10,
            total_volume: 10000.0 * index as f64,
            first_transaction: Some(now - Duration::days(30)),
            last_transaction: Some(now - Duration::hours(index as i64)),
            flags_count: if index > 1 { index - 1 } else { 0 },
        });

        ReviewQueueResponse {
            review,
            transaction_details,
            user_risk_profile,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_review_priority_from_risk_score() {
        assert_eq!(ReviewPriority::from_risk_score(0.96), ReviewPriority::Critical);
        assert_eq!(ReviewPriority::from_risk_score(0.90), ReviewPriority::High);
        assert_eq!(ReviewPriority::from_risk_score(0.80), ReviewPriority::Medium);
        assert_eq!(ReviewPriority::from_risk_score(0.70), ReviewPriority::Low);
    }

    #[test]
    fn test_escalation_timeouts() {
        assert_eq!(ReviewPriority::Critical.escalation_timeout_hours(), 2);
        assert_eq!(ReviewPriority::High.escalation_timeout_hours(), 6);
        assert_eq!(ReviewPriority::Medium.escalation_timeout_hours(), 24);
        assert_eq!(ReviewPriority::Low.escalation_timeout_hours(), 72);
    }

    #[test]
    fn test_review_status_methods() {
        assert!(ReviewStatus::Approved.is_final());
        assert!(ReviewStatus::Rejected.is_final());
        assert!(ReviewStatus::Expired.is_final());
        assert!(!ReviewStatus::Pending.is_final());

        assert!(ReviewStatus::Pending.is_active());
        assert!(ReviewStatus::InReview.is_active());
        assert!(ReviewStatus::Escalated.is_active());
        assert!(!ReviewStatus::Approved.is_active());
    }
}