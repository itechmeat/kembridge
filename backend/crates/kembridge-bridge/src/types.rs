use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeTransaction {
    pub id: Uuid,
    pub from_chain: String,
    pub to_chain: String,
    pub from_address: String,
    pub to_address: String,
    pub amount: String,
    pub token_address: String,
    pub status: TransactionStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionStatus {
    Pending,
    Processing,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeRequest {
    pub from_chain: String,
    pub to_chain: String,
    pub from_address: String,
    pub to_address: String,
    pub amount: String,
    pub token_address: String,
}