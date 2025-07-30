use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// Basic types for all services
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
    pub timestamp: DateTime<Utc>,
}

impl<T> ServiceResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            timestamp: Utc::now(),
        }
    }

    pub fn error(error: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(error),
            timestamp: Utc::now(),
        }
    }
}

// Main types for bridge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenInfo {
    pub address: String,
    pub symbol: String,
    pub name: String,
    pub decimals: u8,
    pub chain_id: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionRequest {
    pub id: Uuid,
    pub from_chain: String,
    pub to_chain: String,
    pub from_token: TokenInfo,
    pub to_token: TokenInfo,
    pub amount: BigDecimal,
    pub recipient: String,
    pub user_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionResponse {
    pub id: Uuid,
    pub status: TransactionStatus,
    pub tx_hash: Option<String>,
    pub estimated_time: Option<u64>,
    pub fee: Option<BigDecimal>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionStatus {
    Pending,
    Processing,
    Completed,
    Failed,
    Cancelled,
}

// Types for authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub wallet_address: String,
    pub chain: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthToken {
    pub token: String,
    pub user_id: Uuid,
    pub expires_at: DateTime<Utc>,
}

// Types for crypto service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantumKeyPair {
    pub id: Uuid,
    pub public_key: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionRequest {
    pub data: String,
    pub key_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionResponse {
    pub encrypted_data: String,
    pub nonce: String,
}

// Types for 1inch service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapQuote {
    pub from_token: TokenInfo,
    pub to_token: TokenInfo,
    pub from_amount: BigDecimal,
    pub to_amount: BigDecimal,
    pub price_impact: BigDecimal,
    pub gas_estimate: u64,
    pub protocols: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapRequest {
    pub quote_id: String,
    pub user_address: String,
    pub slippage: BigDecimal,
}

// Health check types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub service: String,
    pub version: String,
    pub status: String,
    pub uptime: u64,
    pub dependencies: Vec<DependencyStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyStatus {
    pub name: String,
    pub status: String,
    pub latency_ms: Option<u64>,
}
