use serde::{Deserialize, Serialize};
use bigdecimal::BigDecimal;
use uuid::Uuid;
use validator::Validate;

// Common blockchain types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockchainTransaction {
    pub hash: String,
    pub from: String,
    pub to: String,
    pub value: BigDecimal,
    pub gas_used: Option<u64>,
    pub gas_price: Option<BigDecimal>,
    pub status: TransactionStatus,
    pub confirmations: u64,
    pub block_number: Option<u64>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionStatus {
    Pending,
    Confirmed,
    Failed,
    Reverted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockInfo {
    pub number: u64,
    pub hash: String,
    pub parent_hash: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub transaction_count: u64,
    pub gas_used: u64,
    pub gas_limit: u64,
}

// Ethereum-specific types
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct EthereumTransactionRequest {
    #[validate(length(min = 42, max = 42))]
    pub to: String,
    
    pub value: BigDecimal,
    
    #[validate(range(min = 21000))]
    pub gas_limit: u64,
    
    pub gas_price: Option<BigDecimal>,
    pub max_fee_per_gas: Option<BigDecimal>,
    pub max_priority_fee_per_gas: Option<BigDecimal>,
    pub data: Option<String>,
    pub nonce: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EthereumBalance {
    pub address: String,
    pub balance_wei: BigDecimal,
    pub balance_eth: BigDecimal,
    pub block_number: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EthereumGasEstimate {
    pub gas_limit: u64,
    pub gas_price: BigDecimal,
    pub max_fee_per_gas: BigDecimal,
    pub max_priority_fee_per_gas: BigDecimal,
    pub total_cost_wei: BigDecimal,
    pub total_cost_eth: BigDecimal,
}

// NEAR-specific types
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct NearTransactionRequest {
    #[validate(length(min = 2, max = 64))]
    pub receiver_id: String,
    
    pub actions: Vec<NearAction>,
    
    #[validate(range(min = 10000000000000))] // 10 TGas minimum
    pub gas: u64,
    
    pub deposit: Option<BigDecimal>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NearAction {
    Transfer { amount: BigDecimal },
    FunctionCall {
        method_name: String,
        args: serde_json::Value,
        gas: u64,
        deposit: BigDecimal,
    },
    CreateAccount,
    DeleteAccount,
    AddKey {
        public_key: String,
        access_key: NearAccessKey,
    },
    DeleteKey {
        public_key: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NearAccessKey {
    pub nonce: u64,
    pub permission: NearAccessKeyPermission,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NearAccessKeyPermission {
    FullAccess,
    FunctionCall {
        allowance: Option<BigDecimal>,
        receiver_id: String,
        method_names: Vec<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NearAccount {
    pub account_id: String,
    pub balance: BigDecimal,
    pub storage_usage: u64,
    pub code_hash: String,
    pub block_height: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NearGasPrice {
    pub gas_price: BigDecimal,
    pub block_height: u64,
}

// Bridge-specific types
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CrossChainTransferRequest {
    pub transfer_id: Uuid,
    
    pub source_chain: ChainType,
    pub dest_chain: ChainType,
    
    #[validate(length(min = 2))]
    pub from_address: String,
    
    #[validate(length(min = 2))]
    pub to_address: String,
    
    pub token_address: String,
    pub amount: BigDecimal,
    
    pub user_id: Uuid,
    pub nonce: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChainType {
    Ethereum,
    Near,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossChainTransferStatus {
    pub transfer_id: Uuid,
    pub status: TransferStatus,
    pub source_tx_hash: Option<String>,
    pub dest_tx_hash: Option<String>,
    pub confirmations: u64,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransferStatus {
    Initiated,
    SourceConfirmed,
    Bridging,
    DestinationPending,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupportedToken {
    pub address: String,
    pub symbol: String,
    pub name: String,
    pub decimals: u8,
    pub chain: ChainType,
    pub bridge_contract: Option<String>,
    pub min_transfer_amount: BigDecimal,
    pub max_transfer_amount: BigDecimal,
    pub transfer_fee: BigDecimal,
}

// Chain signatures types (NEAR)
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ChainSignatureRequest {
    pub payload: String, // Hex-encoded payload to sign
    pub path: String,    // Derivation path
    pub key_version: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainSignatureResponse {
    pub signature: String,
    pub recovery_id: u8,
    pub big_r: String,
    pub big_s: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct SignatureVerificationRequest {
    pub message: String,
    pub signature: String,
    pub public_key: String,
    pub signature_type: SignatureType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SignatureType {
    Secp256k1,
    Ed25519,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignatureVerificationResponse {
    pub is_valid: bool,
    pub signer_address: Option<String>,
}

// Event monitoring types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockchainEvent {
    pub event_id: Uuid,
    pub chain: ChainType,
    pub block_number: u64,
    pub transaction_hash: String,
    pub event_type: String,
    pub contract_address: Option<String>,
    pub topics: Vec<String>,
    pub data: serde_json::Value,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventFilter {
    pub chain: ChainType,
    pub from_block: Option<u64>,
    pub to_block: Option<u64>,
    pub addresses: Option<Vec<String>>,
    pub topics: Option<Vec<String>>,
    pub limit: Option<u64>,
}

// Health monitoring types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockchainHealthStatus {
    pub chain: ChainType,
    pub is_healthy: bool,
    pub latest_block: u64,
    pub network_id: String,
    pub rpc_latency_ms: u64,
    pub connection_status: ConnectionStatus,
    pub last_error: Option<String>,
    pub last_check: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConnectionStatus {
    Connected,
    Connecting,
    Disconnected,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComprehensiveHealthResponse {
    pub service_status: String,
    pub uptime_seconds: u64,
    pub ethereum_health: BlockchainHealthStatus,
    pub near_health: BlockchainHealthStatus,
    pub database_health: DatabaseHealthStatus,
    pub redis_health: CacheHealthStatus,
    pub active_transfers: usize,
    pub processed_events_24h: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseHealthStatus {
    pub is_connected: bool,
    pub pool_size: u32,
    pub active_connections: u32,
    pub latency_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheHealthStatus {
    pub is_connected: bool,
    pub latency_ms: u64,
    pub memory_usage_mb: u64,
    pub keyspace_hits: u64,
    pub keyspace_misses: u64,
}