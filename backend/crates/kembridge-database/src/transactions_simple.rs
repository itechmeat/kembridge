// src/transactions_simple.rs - Simple Transaction Database Operations with Real-time Risk Score Updates (Phase 5.2.5)
use sqlx::PgPool;
use uuid::Uuid;
use bigdecimal::BigDecimal;
use serde_json::Value;
use anyhow::Result;
use chrono::{DateTime, Utc};

/// Simple service for transaction database operations with real-time risk score updates
pub struct TransactionService {
    pool: PgPool,
}

impl TransactionService {
    /// Create new transaction service
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Update transaction risk score in real-time (Phase 5.2.5)
    pub async fn update_risk_score(
        &self,
        transaction_id: Uuid,
        risk_score: BigDecimal,
        risk_factors: Option<Value>,
        ai_analysis_version: Option<String>,
    ) -> Result<()> {
        let risk_factors_json = risk_factors.unwrap_or_else(|| Value::Object(serde_json::Map::new()));
        let ai_version = ai_analysis_version.unwrap_or_else(|| "v1.0".to_string());

        sqlx::query!(
            r#"
            UPDATE transactions 
            SET risk_score = $1::numeric, 
                risk_factors = $2::jsonb, 
                ai_analysis_version = $3,
                updated_at = NOW()
            WHERE id = $4
            "#,
            risk_score,
            risk_factors_json,
            ai_version,
            transaction_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get transaction risk score
    pub async fn get_risk_score(&self, transaction_id: Uuid) -> Result<Option<BigDecimal>> {
        let result = sqlx::query_scalar!(
            "SELECT risk_score FROM transactions WHERE id = $1",
            transaction_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(result.flatten())
    }

    /// Get transactions by risk score range
    pub async fn get_transactions_by_risk_range(
        &self,
        min_risk_score: BigDecimal,
        max_risk_score: BigDecimal,
        limit: Option<i64>,
    ) -> Result<Vec<Uuid>> {
        let transactions = sqlx::query_scalar!(
            r#"
            SELECT id 
            FROM transactions 
            WHERE risk_score >= $1::numeric AND risk_score <= $2::numeric
            ORDER BY risk_score DESC, created_at ASC
            LIMIT $3
            "#,
            min_risk_score,
            max_risk_score,
            limit.unwrap_or(50)
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(transactions)
    }

    /// Get high risk transactions requiring review
    pub async fn get_high_risk_transactions(
        &self,
        min_risk_score: BigDecimal,
        limit: Option<i64>,
    ) -> Result<Vec<Uuid>> {
        let transactions = sqlx::query_scalar!(
            r#"
            SELECT id 
            FROM transactions 
            WHERE risk_score >= $1::numeric 
                AND status IN ('pending', 'validating', 'locked')
            ORDER BY risk_score DESC, created_at ASC
            LIMIT $2
            "#,
            min_risk_score,
            limit.unwrap_or(50)
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(transactions)
    }

    /// Get risk statistics (simple version)
    pub async fn get_risk_statistics(&self) -> Result<RiskStatistics> {
        let stats = sqlx::query!(
            r#"
            SELECT 
                COUNT(*) as total_transactions,
                AVG(risk_score) as avg_risk_score,
                COUNT(*) FILTER (WHERE risk_score > 0.7) as high_risk_count,
                COUNT(*) FILTER (WHERE risk_score > 0.3 AND risk_score <= 0.7) as medium_risk_count,
                COUNT(*) FILTER (WHERE risk_score <= 0.3) as low_risk_count,
                MAX(risk_score) as max_risk_score,
                MIN(risk_score) as min_risk_score
            FROM transactions 
            WHERE created_at >= NOW() - INTERVAL '24 hours'
            "#,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(RiskStatistics {
            total_transactions: stats.total_transactions.unwrap_or(0) as u64,
            avg_risk_score: stats.avg_risk_score.unwrap_or(BigDecimal::from(0)),
            high_risk_count: stats.high_risk_count.unwrap_or(0) as u64,
            medium_risk_count: stats.medium_risk_count.unwrap_or(0) as u64,
            low_risk_count: stats.low_risk_count.unwrap_or(0) as u64,
            max_risk_score: stats.max_risk_score.unwrap_or(BigDecimal::from(0)),
            min_risk_score: stats.min_risk_score.unwrap_or(BigDecimal::from(0)),
        })
    }

    /// Get risk score history for a transaction (Phase 5.2.5)
    pub async fn get_risk_score_history(&self, transaction_id: Uuid) -> Result<Vec<RiskScoreHistoryEntry>> {
        let history = sqlx::query!(
            r#"
            SELECT 
                id,
                transaction_id,
                old_risk_score,
                new_risk_score,
                COALESCE(risk_factors, '{}'::jsonb) as risk_factors,
                ai_analysis_version,
                created_at as change_timestamp
            FROM risk_score_history 
            WHERE transaction_id = $1
            ORDER BY created_at ASC
            "#,
            transaction_id
        )
        .fetch_all(&self.pool)
        .await?;

        let entries = history.into_iter().map(|row| {
            RiskScoreHistoryEntry {
                id: row.id,
                transaction_id: row.transaction_id,
                old_risk_score: row.old_risk_score,
                new_risk_score: row.new_risk_score,
                risk_factors: row.risk_factors.expect("risk_factors should not be null due to COALESCE"),
                ai_analysis_version: row.ai_analysis_version,
                change_timestamp: row.change_timestamp,
            }
        }).collect();

        Ok(entries)
    }

    /// Get risk score trends over time (Phase 5.2.5)
    pub async fn get_risk_score_trends(&self, days: i32) -> Result<Vec<RiskTrendEntry>> {
        let trends = sqlx::query!(
            r#"
            SELECT 
                DATE_TRUNC('day', created_at) as trend_date,
                AVG(risk_score) as avg_risk_score,
                COUNT(*) as transaction_count,
                COUNT(*) FILTER (WHERE risk_score > 0.7) as high_risk_count,
                MAX(risk_score) as max_risk_score,
                MIN(risk_score) as min_risk_score
            FROM transactions 
            WHERE created_at >= NOW() - INTERVAL '1 day' * $1
            GROUP BY DATE_TRUNC('day', created_at)
            ORDER BY trend_date ASC
            "#,
            days as f64
        )
        .fetch_all(&self.pool)
        .await?;

        let entries = trends.into_iter().map(|row| {
            RiskTrendEntry {
                date: row.trend_date.unwrap_or_else(|| Utc::now()),
                avg_risk_score: row.avg_risk_score.unwrap_or(BigDecimal::from(0)),
                transaction_count: row.transaction_count.unwrap_or(0) as u64,
                high_risk_count: row.high_risk_count.unwrap_or(0) as u64,
                max_risk_score: row.max_risk_score.unwrap_or(BigDecimal::from(0)),
                min_risk_score: row.min_risk_score.unwrap_or(BigDecimal::from(0)),
            }
        }).collect();

        Ok(entries)
    }

    /// Get optimized risk analytics for dashboard (Phase 5.2.5)
    pub async fn get_risk_analytics_summary(&self, hours: i32) -> Result<RiskAnalyticsSummary> {
        let summary = sqlx::query!(
            r#"
            SELECT 
                COUNT(*) as total_transactions,
                AVG(risk_score) as avg_risk_score,
                COUNT(*) FILTER (WHERE risk_score > 0.7) as high_risk_count,
                COUNT(*) FILTER (WHERE risk_score > 0.3 AND risk_score <= 0.7) as medium_risk_count,
                COUNT(*) FILTER (WHERE risk_score <= 0.3) as low_risk_count,
                MAX(risk_score) as max_risk_score,
                MIN(risk_score) as min_risk_score,
                COUNT(*) FILTER (WHERE status = 'completed') as completed_count,
                COUNT(*) FILTER (WHERE status = 'failed') as failed_count,
                COUNT(*) FILTER (WHERE status = 'pending') as pending_count,
                SUM(amount_in) as total_volume,
                COUNT(DISTINCT user_id) as unique_users,
                COUNT(*) FILTER (WHERE risk_score > 0.7 AND status = 'completed') as high_risk_completed,
                COUNT(*) FILTER (WHERE risk_score > 0.7 AND status = 'failed') as high_risk_failed
            FROM transactions 
            WHERE created_at >= NOW() - INTERVAL '1 hour' * $1
            "#,
            hours as f64
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(RiskAnalyticsSummary {
            total_transactions: summary.total_transactions.unwrap_or(0) as u64,
            avg_risk_score: summary.avg_risk_score.unwrap_or(BigDecimal::from(0)),
            high_risk_count: summary.high_risk_count.unwrap_or(0) as u64,
            medium_risk_count: summary.medium_risk_count.unwrap_or(0) as u64,
            low_risk_count: summary.low_risk_count.unwrap_or(0) as u64,
            max_risk_score: summary.max_risk_score.unwrap_or(BigDecimal::from(0)),
            min_risk_score: summary.min_risk_score.unwrap_or(BigDecimal::from(0)),
            completed_count: summary.completed_count.unwrap_or(0) as u64,
            failed_count: summary.failed_count.unwrap_or(0) as u64,
            pending_count: summary.pending_count.unwrap_or(0) as u64,
            total_volume: summary.total_volume.unwrap_or(BigDecimal::from(0)),
            unique_users: summary.unique_users.unwrap_or(0) as u64,
            high_risk_completed: summary.high_risk_completed.unwrap_or(0) as u64,
            high_risk_failed: summary.high_risk_failed.unwrap_or(0) as u64,
            time_range_hours: hours,
        })
    }

    /// Get risk score distribution for charts (Phase 5.2.5)
    pub async fn get_risk_score_distribution(&self, hours: i32) -> Result<Vec<RiskScoreDistribution>> {
        let distribution = sqlx::query!(
            r#"
            SELECT 
                CASE 
                    WHEN risk_score <= 0.2 THEN 'very_low'
                    WHEN risk_score <= 0.4 THEN 'low'
                    WHEN risk_score <= 0.6 THEN 'medium'
                    WHEN risk_score <= 0.8 THEN 'high'
                    ELSE 'very_high'
                END as risk_level,
                COUNT(*) as count,
                AVG(risk_score) as avg_score,
                SUM(amount_in) as total_volume
            FROM transactions 
            WHERE created_at >= NOW() - INTERVAL '1 hour' * $1
            GROUP BY 
                CASE 
                    WHEN risk_score <= 0.2 THEN 'very_low'
                    WHEN risk_score <= 0.4 THEN 'low'
                    WHEN risk_score <= 0.6 THEN 'medium'
                    WHEN risk_score <= 0.8 THEN 'high'
                    ELSE 'very_high'
                END
            ORDER BY avg_score ASC
            "#,
            hours as f64
        )
        .fetch_all(&self.pool)
        .await?;

        let entries = distribution.into_iter().map(|row| {
            RiskScoreDistribution {
                risk_level: row.risk_level.unwrap_or_else(|| "unknown".to_string()),
                count: row.count.unwrap_or(0) as u64,
                avg_score: row.avg_score.unwrap_or(BigDecimal::from(0)),
                total_volume: row.total_volume.unwrap_or(BigDecimal::from(0)),
            }
        }).collect();

        Ok(entries)
    }

    /// Get top risky transactions for monitoring (Phase 5.2.5)
    pub async fn get_top_risky_transactions(&self, limit: i64) -> Result<Vec<TopRiskyTransaction>> {
        let transactions = sqlx::query!(
            r#"
            SELECT 
                t.id,
                t.user_id,
                t.source_chain,
                t.destination_chain,
                t.amount_in,
                t.risk_score,
                t.status,
                t.created_at,
                eth_auth.wallet_address as ethereum_address,
                near_auth.wallet_address as near_account_id
            FROM transactions t
            JOIN users u ON t.user_id = u.id
            LEFT JOIN user_auth_methods eth_auth ON t.user_id = eth_auth.user_id AND eth_auth.chain_type = 'ethereum'
            LEFT JOIN user_auth_methods near_auth ON t.user_id = near_auth.user_id AND near_auth.chain_type = 'near'
            WHERE t.risk_score > 0.6
            ORDER BY t.risk_score DESC, t.created_at DESC
            LIMIT $1
            "#,
            limit
        )
        .fetch_all(&self.pool)
        .await?;

        let entries = transactions.into_iter().map(|row| {
            TopRiskyTransaction {
                id: row.id,
                user_id: row.user_id,
                source_chain: row.source_chain,
                destination_chain: row.destination_chain,
                amount_in: row.amount_in,
                risk_score: row.risk_score.unwrap_or(BigDecimal::from(0)),
                status: row.status,
                created_at: row.created_at.expect("created_at should not be null"),
                ethereum_address: row.ethereum_address,
                near_account_id: row.near_account_id,
            }
        }).collect();

        Ok(entries)
    }
}

/// Simple risk statistics for analytics and monitoring
#[derive(Debug, Clone)]
pub struct RiskStatistics {
    pub total_transactions: u64,
    pub avg_risk_score: BigDecimal,
    pub high_risk_count: u64,
    pub medium_risk_count: u64,
    pub low_risk_count: u64,
    pub max_risk_score: BigDecimal,
    pub min_risk_score: BigDecimal,
}

/// Risk score history entry for transaction analytics (Phase 5.2.5)
#[derive(Debug, Clone)]
pub struct RiskScoreHistoryEntry {
    pub id: Uuid,
    pub transaction_id: Uuid,
    pub old_risk_score: Option<BigDecimal>,
    pub new_risk_score: BigDecimal,
    pub risk_factors: Value,
    pub ai_analysis_version: Option<String>,
    pub change_timestamp: DateTime<Utc>,
}

/// Risk trend entry for time-series analytics (Phase 5.2.5)
#[derive(Debug, Clone)]
pub struct RiskTrendEntry {
    pub date: DateTime<Utc>,
    pub avg_risk_score: BigDecimal,
    pub transaction_count: u64,
    pub high_risk_count: u64,
    pub max_risk_score: BigDecimal,
    pub min_risk_score: BigDecimal,
}

/// Comprehensive risk analytics summary (Phase 5.2.5)
#[derive(Debug, Clone)]
pub struct RiskAnalyticsSummary {
    pub total_transactions: u64,
    pub avg_risk_score: BigDecimal,
    pub high_risk_count: u64,
    pub medium_risk_count: u64,
    pub low_risk_count: u64,
    pub max_risk_score: BigDecimal,
    pub min_risk_score: BigDecimal,
    pub completed_count: u64,
    pub failed_count: u64,
    pub pending_count: u64,
    pub total_volume: BigDecimal,
    pub unique_users: u64,
    pub high_risk_completed: u64,
    pub high_risk_failed: u64,
    pub time_range_hours: i32,
}

/// Risk score distribution for charts (Phase 5.2.5)
#[derive(Debug, Clone)]
pub struct RiskScoreDistribution {
    pub risk_level: String,
    pub count: u64,
    pub avg_score: BigDecimal,
    pub total_volume: BigDecimal,
}

/// Top risky transactions for monitoring (Phase 5.2.5)
#[derive(Debug, Clone)]
pub struct TopRiskyTransaction {
    pub id: Uuid,
    pub user_id: Uuid,
    pub source_chain: String,
    pub destination_chain: String,
    pub amount_in: BigDecimal,
    pub risk_score: BigDecimal,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub ethereum_address: Option<String>,
    pub near_account_id: Option<String>,
}