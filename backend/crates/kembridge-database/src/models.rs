use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use bigdecimal::BigDecimal;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserAuthMethod {
    pub id: Uuid,
    pub user_id: Uuid,
    pub auth_type: String,
    pub wallet_address: String,
    pub chain: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserSession {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token_hash: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Transaction {
    pub id: Uuid,
    pub user_id: Uuid,
    
    // Cross-chain transaction details
    pub source_chain: String,
    pub destination_chain: String,
    pub source_token: String,
    pub destination_token: String,
    
    // Transaction amounts (using BigDecimal for PostgreSQL compatibility)
    pub amount_in: BigDecimal,
    pub amount_out: Option<BigDecimal>,
    pub exchange_rate: Option<BigDecimal>,
    
    // Fee structure
    pub bridge_fee_amount: BigDecimal,
    pub network_fee_amount: BigDecimal,
    pub quantum_protection_fee: BigDecimal,
    
    // Blockchain transaction identifiers
    pub source_tx_hash: Option<String>,
    pub destination_tx_hash: Option<String>,
    pub bridge_tx_hash: Option<String>,
    
    // Transaction state management
    pub status: String,
    pub status_history: serde_json::Value,
    
    // Quantum cryptography integration
    pub quantum_key_id: Option<Uuid>,
    pub encrypted_payload: Option<Vec<u8>>,
    pub encryption_metadata: serde_json::Value,
    
    // AI risk analysis (Phase 5.2.5)
    pub risk_score: BigDecimal,
    pub risk_factors: serde_json::Value,
    pub ai_analysis_version: String,
    
    // Temporal management
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub confirmed_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    
    // 1inch Fusion+ integration
    pub oneinch_order_id: Option<String>,
    pub oneinch_quote_id: Option<String>,
    pub fusion_metadata: serde_json::Value,
    
    // NEAR Chain Signatures integration
    pub near_chain_signature: serde_json::Value,
    pub near_account_id: Option<String>,
    
    // Virtual generated columns (read-only)
    pub transaction_value_usd: Option<BigDecimal>,
    pub processing_time_minutes: Option<i32>,
    pub transaction_category: Option<String>,
    pub bridge_direction: Option<String>,
    pub risk_category: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct QuantumKey {
    pub id: Uuid,
    pub user_id: Uuid,
    pub key_type: String,
    pub public_key: Vec<u8>,
    pub encrypted_private_key: Vec<u8>,
    pub algorithm: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AuditLog {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub action: String,
    pub resource_type: String,
    pub resource_id: Option<Uuid>,
    pub metadata: serde_json::Value,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub created_at: DateTime<Utc>,
}