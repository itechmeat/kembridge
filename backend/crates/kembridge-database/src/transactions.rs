// src/transactions.rs - Transaction Database Operations with Real-time Risk Score Updates (Phase 5.2.5)
use sqlx::PgPool;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use bigdecimal::BigDecimal;
use serde_json::Value;
use anyhow::Result;

use crate::models::Transaction;

/// Service for transaction database operations with real-time risk score updates
pub struct TransactionService {
    pool: PgPool,
}

impl TransactionService {
    /// Create new transaction service
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Create a new transaction with initial risk score
    pub async fn create_transaction(
        &self,
        user_id: Uuid,
        source_chain: String,
        destination_chain: String,
        source_token: String,
        destination_token: String,
        amount_in: BigDecimal,
        expected_amount_out: Option<BigDecimal>,
        quantum_key_id: Option<Uuid>,
        expires_in_hours: i32,
        initial_risk_score: BigDecimal,
        risk_factors: Value,
    ) -> Result<Uuid> {
        let transaction_id = sqlx::query_scalar!(
            r#"
            SELECT create_bridge_transaction(
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10
            ) as "transaction_id!"
            "#,
            user_id,
            source_chain,
            destination_chain,
            source_token,
            destination_token,
            amount_in,
            expected_amount_out,
            quantum_key_id,
            expires_in_hours,
            None::<String> // oneinch_quote_id
        )
        .fetch_one(&self.pool)
        .await?;

        // Update with provided risk score and factors
        if initial_risk_score != BigDecimal::from(0) || risk_factors != Value::Null {
            self.update_risk_score(transaction_id, initial_risk_score, Some(risk_factors), Some("initial_assessment".to_string())).await?;
        }

        Ok(transaction_id)
    }

    /// Update transaction risk score in real-time (Phase 5.2.5)
    pub async fn update_risk_score(
        &self,
        transaction_id: Uuid,
        risk_score: BigDecimal,
        risk_factors: Option<Value>,
        ai_analysis_version: Option<String>,
    ) -> Result<()> {
        let mut query = sqlx::QueryBuilder::new(
            "UPDATE transactions SET risk_score = "
        );
        query.push_bind(risk_score);
        query.push(", updated_at = NOW()");

        if let Some(factors) = risk_factors {
            query.push(", risk_factors = ");
            query.push_bind(factors);
        }

        if let Some(version) = ai_analysis_version {
            query.push(", ai_analysis_version = ");
            query.push_bind(version);
        }

        query.push(" WHERE id = ");
        query.push_bind(transaction_id);

        query.build().execute(&self.pool).await?;

        Ok(())
    }

    /// Get transaction by ID
    pub async fn get_transaction(&self, transaction_id: Uuid) -> Result<Option<Transaction>> {
        let transaction = sqlx::query_as!(
            Transaction,
            r#"
            SELECT 
                id, user_id, source_chain, destination_chain,
                source_token, destination_token, amount_in, amount_out, exchange_rate,
                bridge_fee_amount, network_fee_amount, quantum_protection_fee,
                source_tx_hash, destination_tx_hash, bridge_tx_hash,
                status, status_history, quantum_key_id, encrypted_payload, 
                encryption_metadata, risk_score, risk_factors, ai_analysis_version,
                created_at, updated_at, confirmed_at, completed_at, expires_at,
                oneinch_order_id, oneinch_quote_id, fusion_metadata,
                near_chain_signature, near_account_id,
                transaction_value_usd, processing_time_minutes,
                transaction_category, bridge_direction, risk_category
            FROM transactions 
            WHERE id = $1
            "#,
            transaction_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(transaction)
    }

    /// Get transactions by user with risk score filtering
    pub async fn get_user_transactions(
        &self,
        user_id: Uuid,
        min_risk_score: Option<BigDecimal>,
        max_risk_score: Option<BigDecimal>,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<Transaction>> {
        let mut query = sqlx::QueryBuilder::new(
            r#"
            SELECT 
                id, user_id, source_chain, destination_chain,
                source_token, destination_token, amount_in, amount_out, exchange_rate,
                bridge_fee_amount, network_fee_amount, quantum_protection_fee,
                source_tx_hash, destination_tx_hash, bridge_tx_hash,
                status, status_history, quantum_key_id, encrypted_payload, 
                encryption_metadata, risk_score, risk_factors, ai_analysis_version,
                created_at, updated_at, confirmed_at, completed_at, expires_at,
                oneinch_order_id, oneinch_quote_id, fusion_metadata,
                near_chain_signature, near_account_id,
                transaction_value_usd, processing_time_minutes,
                transaction_category, bridge_direction, risk_category
            FROM transactions 
            WHERE user_id = 
            "#
        );
        query.push_bind(user_id);

        if let Some(min_risk) = min_risk_score {
            query.push(" AND risk_score >= ");
            query.push_bind(min_risk);
        }

        if let Some(max_risk) = max_risk_score {
            query.push(" AND risk_score <= ");
            query.push_bind(max_risk);
        }

        query.push(" ORDER BY created_at DESC");

        if let Some(limit_val) = limit {
            query.push(" LIMIT ");
            query.push_bind(limit_val);
        }

        if let Some(offset_val) = offset {
            query.push(" OFFSET ");
            query.push_bind(offset_val);
        }

        let transactions = query
            .build_query_as::<Transaction>()
            .fetch_all(&self.pool)
            .await?;

        Ok(transactions)
    }

    /// Get transactions requiring manual review based on risk score
    pub async fn get_transactions_for_review(
        &self,
        min_risk_score: BigDecimal,
        limit: Option<i64>,
    ) -> Result<Vec<Transaction>> {
        let transactions = sqlx::query_as!(
            Transaction,
            r#"
            SELECT 
                id, user_id, source_chain, destination_chain,
                source_token, destination_token, amount_in, amount_out, exchange_rate,
                bridge_fee_amount, network_fee_amount, quantum_protection_fee,
                source_tx_hash, destination_tx_hash, bridge_tx_hash,
                status, status_history, quantum_key_id, encrypted_payload, 
                encryption_metadata, risk_score, risk_factors, ai_analysis_version,
                created_at, updated_at, confirmed_at, completed_at, expires_at,
                oneinch_order_id, oneinch_quote_id, fusion_metadata,
                near_chain_signature, near_account_id,
                transaction_value_usd, processing_time_minutes,
                transaction_category, bridge_direction, risk_category
            FROM transactions 
            WHERE risk_score >= $1 
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

    /// Update transaction status with risk score update
    pub async fn update_status_with_risk(
        &self,
        transaction_id: Uuid,
        new_status: String,
        reason: Option<String>,
        tx_hash: Option<String>,
        risk_score: Option<BigDecimal>,
        metadata: Option<Value>,
    ) -> Result<bool> {
        let result = sqlx::query_scalar!(
            "SELECT update_transaction_status($1, $2, $3, $4, $5, $6) as updated",
            transaction_id,
            new_status,
            reason,
            tx_hash,
            risk_score,
            metadata.unwrap_or_else(|| Value::Object(serde_json::Map::new()))
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(result.unwrap_or(false))
    }

    /// Get risk score history for analytics (Phase 5.2.5)
    pub async fn get_risk_score_history(
        &self,
        transaction_id: Uuid,
    ) -> Result<Vec<RiskScoreHistoryEntry>> {
        // Extract risk score updates from audit logs
        let history = sqlx::query_as!(
            RiskScoreHistoryEntry,
            r#"
            SELECT 
                event_data->>'old_risk_score' as "old_risk_score",
                event_data->>'new_risk_score' as "new_risk_score",
                event_data->'risk_factors' as "risk_factors!",
                created_at
            FROM audit_logs 
            WHERE event_data->>'transaction_id' = $1 
                AND event_type = 'risk_score_updated'
            ORDER BY created_at ASC
            "#,
            transaction_id.to_string()
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(history)
    }

    /// Get risk statistics for analytics
    pub async fn get_risk_statistics(&self, time_range_hours: Option<i32>) -> Result<RiskStatistics> {
        let hours = time_range_hours.unwrap_or(24);
        
        let stats = sqlx::query!(
            r#"
            SELECT 
                COUNT(*) as total_transactions,
                AVG(risk_score) as avg_risk_score,
                COUNT(*) FILTER (WHERE risk_category = 'HIGH') as high_risk_count,
                COUNT(*) FILTER (WHERE risk_category = 'MEDIUM') as medium_risk_count,
                COUNT(*) FILTER (WHERE risk_category = 'LOW') as low_risk_count,
                COUNT(*) FILTER (WHERE status = 'failed' AND risk_score > 0.7) as high_risk_failures,
                MAX(risk_score) as max_risk_score,
                MIN(risk_score) as min_risk_score
            FROM transactions 
            WHERE created_at >= NOW() - INTERVAL '1 hour' * $1
            "#,
            hours
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(RiskStatistics {
            total_transactions: stats.total_transactions.unwrap_or(0) as u64,
            avg_risk_score: stats.avg_risk_score.unwrap_or(BigDecimal::from(0)),
            high_risk_count: stats.high_risk_count.unwrap_or(0) as u64,
            medium_risk_count: stats.medium_risk_count.unwrap_or(0) as u64,
            low_risk_count: stats.low_risk_count.unwrap_or(0) as u64,
            high_risk_failures: stats.high_risk_failures.unwrap_or(0) as u64,
            max_risk_score: stats.max_risk_score.unwrap_or(BigDecimal::from(0)),
            min_risk_score: stats.min_risk_score.unwrap_or(BigDecimal::from(0)),
            time_range_hours: hours,
        })
    }
}

/// Risk score history entry for analytics
#[derive(Debug, Clone)]
pub struct RiskScoreHistoryEntry {
    pub old_risk_score: Option<String>,
    pub new_risk_score: Option<String>,
    pub risk_factors: Value,
    pub created_at: DateTime<Utc>,
}

/// Risk statistics for analytics and monitoring
#[derive(Debug, Clone)]
pub struct RiskStatistics {
    pub total_transactions: u64,
    pub avg_risk_score: BigDecimal,
    pub high_risk_count: u64,
    pub medium_risk_count: u64,
    pub low_risk_count: u64,
    pub high_risk_failures: u64,
    pub max_risk_score: BigDecimal,
    pub min_risk_score: BigDecimal,
    pub time_range_hours: i32,
}