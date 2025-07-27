// src/services/manual_review/statistics.rs - Statistics and analytics for manual review
use sqlx::PgPool;
use tracing::{error, instrument};

use crate::models::review::{ReviewQueueStats, ReviewError};

/// Statistics operations for manual review system
pub struct ReviewStatistics {
    pool: PgPool,
}

impl ReviewStatistics {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Get queue statistics from database
    #[instrument(skip(self))]
    pub async fn get_queue_statistics(&self) -> Result<ReviewQueueStats, ReviewError> {
        let stats_row = sqlx::query!(
            "SELECT * FROM get_review_queue_stats()"
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            error!("Failed to fetch queue statistics: {}", e);
            ReviewError::Database(e)
        })?;

        Ok(ReviewQueueStats {
            total_pending: stats_row.total_pending.unwrap_or(0) as u64,
            total_in_review: stats_row.total_in_review.unwrap_or(0) as u64,
            total_escalated: stats_row.total_escalated.unwrap_or(0) as u64,
            total_expired: stats_row.total_expired.unwrap_or(0) as u64,
            avg_resolution_time_hours: stats_row.avg_resolution_time_hours.map(|v| v.to_string().parse().unwrap_or(0.0)),
            critical_count: stats_row.critical_count.unwrap_or(0) as u64,
            high_priority_count: stats_row.high_priority_count.unwrap_or(0) as u64,
        })
    }

    /// Calculate queue health metrics
    pub fn calculate_queue_health(&self, stats: &ReviewQueueStats) -> QueueHealthMetrics {
        let total_active = stats.total_pending + stats.total_in_review;
        let total_completed = stats.total_escalated; // Simplified for demo
        
        let completion_rate = if total_active + total_completed > 0 {
            (total_completed as f64) / ((total_active + total_completed) as f64) * 100.0
        } else {
            0.0
        };

        let critical_ratio = if total_active > 0 {
            (stats.critical_count as f64) / (total_active as f64) * 100.0
        } else {
            0.0
        };

        let avg_resolution_hours = stats.avg_resolution_time_hours.unwrap_or(0.0);

        QueueHealthMetrics {
            completion_rate,
            critical_ratio,
            avg_resolution_hours,
            health_score: self.calculate_health_score(completion_rate, critical_ratio, avg_resolution_hours),
        }
    }

    fn calculate_health_score(&self, completion_rate: f64, critical_ratio: f64, avg_resolution_hours: f64) -> f64 {
        // Health score algorithm (0-100)
        let completion_score = completion_rate.min(100.0) * 0.4; // 40% weight
        let critical_score = (100.0 - critical_ratio.min(100.0)) * 0.3; // 30% weight (inverted)
        let resolution_score = if avg_resolution_hours <= 24.0 {
            100.0
        } else if avg_resolution_hours <= 72.0 {
            75.0
        } else {
            25.0
        } * 0.3; // 30% weight

        completion_score + critical_score + resolution_score
    }
}

/// Queue health metrics
#[derive(Debug, Clone)]
pub struct QueueHealthMetrics {
    pub completion_rate: f64,
    pub critical_ratio: f64,
    pub avg_resolution_hours: f64,
    pub health_score: f64,
}