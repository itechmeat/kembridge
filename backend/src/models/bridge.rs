// src/models/bridge.rs - Bridge operation data models (Phase 4 placeholder)
use serde::{Deserialize, Serialize};
use validator::Validate;
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct BridgeSwapRequest {
    #[validate(length(min = 1, max = 20))]
    pub source_chain: String,

    #[validate(length(min = 1, max = 20))]
    pub destination_chain: String,

    #[validate(length(min = 1, max = 10))]
    pub source_token: String,

    #[validate(length(min = 1, max = 10))]
    pub destination_token: String,

    #[validate(range(min = 0.000001))]
    pub amount: f64,

    pub quantum_key_id: Option<Uuid>,

    #[validate(range(min = 1, max = 168))] // 1 hour to 1 week
    pub expires_in_hours: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BridgeSwapResponse {
    pub transaction_id: Uuid,
    pub status: TransactionStatus,
    pub estimated_completion: DateTime<Utc>,
    pub bridge_fee: f64,
    pub network_fee: f64,
    pub quantum_protection_fee: f64,
    pub exchange_rate: f64,
    pub expected_output: f64,
    pub risk_score: f64,
    pub quantum_protected: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TransactionStatus {
    Pending,
    Validating,
    Locked,
    Processing,
    Confirming,
    Confirmed,
    Completed,
    Failed,
    Cancelled,
    Expired,
    Refunded,
}