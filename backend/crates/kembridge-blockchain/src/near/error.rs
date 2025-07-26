// NEAR-specific error types
// Phase 4.2: NEAR Protocol Adapter

use thiserror::Error;
use near_jsonrpc_client::errors::{JsonRpcError, RpcTransportError};

#[derive(Debug, Error)]
pub enum NearError {
    #[error("NEAR RPC error: {0}")]
    RpcError(#[from] JsonRpcError<RpcTransportError>),
    
    #[error("NEAR RPC status error: {0}")]
    RpcStatusError(String),
    
    #[error("NEAR RPC query error: {0}")]
    RpcQueryError(String),

    #[error("NEAR crypto error: {0}")]
    CryptoError(String),

    #[error("NEAR configuration error: {0}")]
    ConfigError(String),

    #[error("NEAR wallet error: {0}")]
    WalletError(String),

    #[error("Chain Signatures error: {0}")]
    ChainSignaturesError(String),

    #[error("1Click API error: {0}")]
    OneClickApiError(String),

    #[error("Network connection error: {0}")]
    NetworkError(#[from] reqwest::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Quantum crypto integration error: {0}")]
    QuantumError(String),

    #[error("Invalid account ID: {0}")]
    InvalidAccountId(String),

    #[error("Transaction error: {0}")]
    TransactionError(String),
}