// Phase 4.1: Ethereum-specific error types
use ethers::types::{U256, H256};

#[derive(Debug, thiserror::Error)]
pub enum EthereumError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Invalid network - expected {expected}, got {actual}")]
    InvalidNetwork { expected: u64, actual: u64 },

    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),

    #[error("Wallet error: {0}")]
    WalletError(String),

    #[error("Insufficient funds - available: {available}, required: {required}")]
    InsufficientFunds { available: U256, required: U256 },

    #[error("Signing failed: {0}")]
    SigningFailed(String),

    #[error("Transaction failed: {0}")]
    TransactionFailed(String),

    #[error("Invalid transaction: {0}")]
    InvalidTransaction(String),

    #[error("Confirmation failed: {0}")]
    ConfirmationFailed(String),

    #[error("Transaction not found: {0}")]
    TransactionNotFound(H256),

    #[error("Transaction reverted - hash: {tx_hash}, gas used: {gas_used}")]
    TransactionReverted { tx_hash: H256, gas_used: U256 },

    #[error("Contract error: {0}")]
    ContractError(String),

    #[error("Gas estimation failed: {0}")]
    GasEstimationFailed(String),

    #[error("Quantum crypto error: {0}")]
    QuantumCryptoError(String),
}