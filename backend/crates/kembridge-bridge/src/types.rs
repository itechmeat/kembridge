use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc, Duration};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapOperation {
    pub swap_id: Uuid,
    pub user_id: Uuid,
    pub from_chain: String,
    pub to_chain: String,
    pub amount: u128,
    pub recipient: String,
    pub status: SwapStatus,
    pub quantum_key_id: Option<String>,
    pub eth_tx_hash: Option<String>,
    pub near_tx_hash: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum SwapStatus {
    Initialized,
    EthLocking,
    EthLocked,
    NearMinting,
    NearMinted,
    Completed,
    Failed,
    Cancelled,
    Timeout,
    RolledBack,
}

impl std::fmt::Display for SwapStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SwapStatus::Initialized => write!(f, "initialized"),
            SwapStatus::EthLocking => write!(f, "eth_locking"),
            SwapStatus::EthLocked => write!(f, "eth_locked"),
            SwapStatus::NearMinting => write!(f, "near_minting"),
            SwapStatus::NearMinted => write!(f, "near_minted"),
            SwapStatus::Completed => write!(f, "completed"),
            SwapStatus::Failed => write!(f, "failed"),
            SwapStatus::Cancelled => write!(f, "cancelled"),
            SwapStatus::Timeout => write!(f, "timeout"),
            SwapStatus::RolledBack => write!(f, "rolled_back"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapInitResponse {
    pub swap_id: Uuid,
    pub status: SwapStatus,
    pub estimated_time: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapResult {
    pub swap_id: Uuid,
    pub eth_tx_hash: Option<String>,
    pub near_tx_hash: Option<String>,
    pub status: SwapStatus,
    pub quantum_key_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EthereumLockResult {
    pub transaction_hash: String,
    pub confirmed: bool,
    pub quantum_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NearMintResult {
    pub transaction_hash: String,
    pub confirmed: bool,
    pub quantum_hash: String,
}

// Legacy types for backward compatibility
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