// src/services/manual_review/database.rs - Database operations for manual review
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use tracing::{error, instrument, warn};
use uuid::Uuid;
use bigdecimal::FromPrimitive;

use crate::models::review::{
    ReviewQueueEntry, ReviewStatus, ReviewPriority, ReviewDecision, 
    ReviewQueueResponse, ReviewQueueQuery, ReviewError
};

/// Database operations for manual review system
pub struct ReviewDatabase {
    pool: PgPool,
}

impl ReviewDatabase {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Save review entry to database
    #[instrument(skip(self, review_entry))]
    pub async fn save_review_entry(&self, review_entry: &ReviewQueueEntry) -> Result<(), ReviewError> {
        sqlx::query!(
            r#"
            INSERT INTO review_queue (
                id, transaction_id, user_id, risk_score, status, priority,
                created_at, updated_at, expires_at, assigned_to, assigned_at,
                reviewed_by, reviewed_at, review_reason, escalation_count,
                last_escalated_at, metadata
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17
            )
            "#,
            review_entry.id,
            review_entry.transaction_id,
            review_entry.user_id,
            bigdecimal::BigDecimal::from_f64(review_entry.risk_score).unwrap_or_default(),
            match review_entry.status {
                ReviewStatus::Pending => "Pending",
                ReviewStatus::InReview => "InReview",
                ReviewStatus::Approved => "Approved",
                ReviewStatus::Rejected => "Rejected",
                ReviewStatus::Escalated => "Escalated",
                ReviewStatus::Expired => "Expired",
            },
            match review_entry.priority {
                ReviewPriority::Critical => "Critical",
                ReviewPriority::High => "High",
                ReviewPriority::Medium => "Medium",
                ReviewPriority::Low => "Low",
            },
            review_entry.created_at,
            review_entry.updated_at,
            review_entry.expires_at,
            review_entry.assigned_to,
            review_entry.assigned_at,
            review_entry.reviewed_by,
            review_entry.reviewed_at,
            review_entry.review_reason,
            review_entry.escalation_count,
            review_entry.last_escalated_at,
            review_entry.metadata
        )
        .execute(&self.pool)
        .await
        .map_err(|e| {
            error!("Failed to save review entry: {}", e);
            ReviewError::Database(e)
        })?;

        Ok(())
    }

    /// Assign review to admin user in database
    #[instrument(skip(self))]
    pub async fn assign_review(&self, review_id: Uuid, admin_user_id: Uuid) -> Result<ReviewQueueEntry, ReviewError> {
        let now = Utc::now();
        
        let affected_rows = sqlx::query!(
            r#"
            UPDATE review_queue 
            SET status = 'InReview', assigned_to = $2, assigned_at = $3, updated_at = $3
            WHERE id = $1 AND status = 'Pending'
            "#,
            review_id,
            admin_user_id,
            now
        )
        .execute(&self.pool)
        .await
        .map_err(|e| {
            error!("Failed to assign review: {}", e);
            ReviewError::Database(e)
        })?;

        if affected_rows.rows_affected() == 0 {
            return Err(ReviewError::NotFound(review_id));
        }

        // Fetch the updated record
        self.get_review_by_id(review_id).await
    }

    /// Update review status with decision
    #[instrument(skip(self, decision))]
    pub async fn update_review_status(&self, decision: &ReviewDecision) -> Result<(), ReviewError> {
        let mut tx = self.pool.begin().await
            .map_err(|e| ReviewError::Database(e))?;

        sqlx::query!(
            r#"
            UPDATE review_queue 
            SET status = $2, reviewed_by = $3, reviewed_at = $4, updated_at = $4
            WHERE id = $1
            "#,
            decision.review_id,
            decision.decision.to_string(),
            decision.reviewed_by,
            decision.reviewed_at
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            error!("Failed to update review status: {}", e);
            ReviewError::Database(e)
        })?;

        sqlx::query!(
            r#"
            INSERT INTO review_decisions (
                review_id, transaction_id, decision, reason, reviewed_by, reviewed_at, metadata
            ) VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
            decision.review_id,
            decision.transaction_id,
            decision.decision.to_string(),
            decision.reason,
            decision.reviewed_by,
            decision.reviewed_at,
            decision.metadata
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            error!("Failed to save review decision: {}", e);
            ReviewError::Database(e)
        })?;

        tx.commit().await
            .map_err(|e| ReviewError::Database(e))?;

        Ok(())
    }

    /// Get reviews from database with filtering
    #[instrument(skip(self, query))]
    pub async fn get_reviews_with_pagination(
        &self,
        query: &ReviewQueueQuery,
        page: u32,
        per_page: u32,
    ) -> Result<(Vec<ReviewQueueResponse>, u64), ReviewError> {
        let offset = ((page - 1) * per_page) as i64;
        let limit = per_page as i64;

        let rows = sqlx::query!(
            "SELECT * FROM review_queue ORDER BY created_at DESC LIMIT $1 OFFSET $2",
            limit,
            offset
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            error!("Failed to fetch reviews: {}", e);
            ReviewError::Database(e)
        })?;

        let total_count = sqlx::query_scalar!("SELECT COUNT(*) FROM review_queue")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| {
                error!("Failed to count reviews: {}", e);
                ReviewError::Database(e)
            })?
            .unwrap_or(0) as u64;

        let mut reviews = Vec::new();
        for row in rows {
            let review_entry = ReviewQueueEntry {
                id: row.id,
                transaction_id: row.transaction_id,
                user_id: row.user_id,
                risk_score: row.risk_score.to_string().parse().unwrap_or(0.0),
                status: match row.status.as_str() {
                    "Pending" => ReviewStatus::Pending,
                    "InReview" => ReviewStatus::InReview,
                    "Approved" => ReviewStatus::Approved,
                    "Rejected" => ReviewStatus::Rejected,
                    "Escalated" => ReviewStatus::Escalated,
                    "Expired" => ReviewStatus::Expired,
                    _ => ReviewStatus::Pending,
                },
                priority: match row.priority.as_str() {
                    "Critical" => ReviewPriority::Critical,
                    "High" => ReviewPriority::High,
                    "Medium" => ReviewPriority::Medium,
                    "Low" => ReviewPriority::Low,
                    _ => ReviewPriority::Medium,
                },
                created_at: row.created_at,
                updated_at: row.updated_at,
                expires_at: row.expires_at,
                assigned_to: row.assigned_to,
                assigned_at: row.assigned_at,
                reviewed_by: row.reviewed_by,
                reviewed_at: row.reviewed_at,
                review_reason: row.review_reason,
                escalation_count: row.escalation_count,
                last_escalated_at: row.last_escalated_at,
                metadata: row.metadata,
            };
            
            let response = ReviewQueueResponse {
                review: review_entry,
                transaction_details: None,
                user_risk_profile: None,
            };
            reviews.push(response);
        }

        Ok((reviews, total_count))
    }

    /// Get review by ID from database
    #[instrument(skip(self))]
    pub async fn get_review_by_id(&self, review_id: Uuid) -> Result<ReviewQueueEntry, ReviewError> {
        let row = sqlx::query!(
            "SELECT * FROM review_queue WHERE id = $1",
            review_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            error!("Failed to fetch review by ID: {}", e);
            ReviewError::Database(e)
        })?;

        match row {
            Some(row) => {
                Ok(ReviewQueueEntry {
                    id: row.id,
                    transaction_id: row.transaction_id,
                    user_id: row.user_id,
                    risk_score: row.risk_score.to_string().parse().unwrap_or(0.0),
                    status: match row.status.as_str() {
                        "Pending" => ReviewStatus::Pending,
                        "InReview" => ReviewStatus::InReview,
                        "Approved" => ReviewStatus::Approved,
                        "Rejected" => ReviewStatus::Rejected,
                        "Escalated" => ReviewStatus::Escalated,
                        "Expired" => ReviewStatus::Expired,
                        _ => ReviewStatus::Pending,
                    },
                    priority: match row.priority.as_str() {
                        "Critical" => ReviewPriority::Critical,
                        "High" => ReviewPriority::High,
                        "Medium" => ReviewPriority::Medium,
                        "Low" => ReviewPriority::Low,
                        _ => ReviewPriority::Medium,
                    },
                    created_at: row.created_at,
                    updated_at: row.updated_at,
                    expires_at: row.expires_at,
                    assigned_to: row.assigned_to,
                    assigned_at: row.assigned_at,
                    reviewed_by: row.reviewed_by,
                    reviewed_at: row.reviewed_at,
                    review_reason: row.review_reason,
                    escalation_count: row.escalation_count,
                    last_escalated_at: row.last_escalated_at,
                    metadata: row.metadata,
                })
            },
            None => Err(ReviewError::NotFound(review_id)),
        }
    }

    /// Escalate review in database
    #[instrument(skip(self))]
    pub async fn escalate_review(&self, review_id: Uuid) -> Result<ReviewQueueEntry, ReviewError> {
        let now = Utc::now();
        
        let affected_rows = sqlx::query!(
            r#"
            UPDATE review_queue 
            SET status = 'Escalated', 
                priority = 'High',
                escalation_count = escalation_count + 1,
                last_escalated_at = $2,
                updated_at = $3,
                expires_at = NOW() + INTERVAL '6 hours'
            WHERE id = $1
            "#,
            review_id,
            now,
            now
        )
        .execute(&self.pool)
        .await
        .map_err(|e| {
            error!("Failed to escalate review: {}", e);
            ReviewError::Database(e)
        })?;

        if affected_rows.rows_affected() == 0 {
            return Err(ReviewError::NotFound(review_id));
        }

        // Fetch the updated record
        self.get_review_by_id(review_id).await
    }

    /// Get expired reviews from database
    #[instrument(skip(self))]
    pub async fn get_expired_reviews(&self) -> Result<Vec<ReviewQueueEntry>, ReviewError> {
        let rows = sqlx::query!(
            "SELECT * FROM get_expired_reviews()"
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            error!("Failed to fetch expired reviews: {}", e);
            ReviewError::Database(e)
        })?;

        let mut expired_reviews = Vec::new();
        for row in rows {
            if let Some(id) = row.id {
                if let Ok(review) = self.get_review_by_id(id).await {
                    expired_reviews.push(review);
                }
            }
        }

        Ok(expired_reviews)
    }

}