// Phase 4.1: Real blockchain adapters implementation
#![allow(unused_variables, dead_code)] // Temporary for Phase 4.1

pub mod ethereum;
pub mod ethereum_legacy;
pub mod near;
pub mod types;

// Re-export Ethereum adapter
pub use ethereum::{EthereumAdapter, EthereumConfig, EthereumError, WalletInfo, TokenBalance, TransactionStatus, ERC20Contract};

// Keep legacy exports for compatibility  
pub use ethereum_legacy::EthereumClient;
pub use near::*;
pub use types::*;

#[derive(Debug, thiserror::Error)]
pub enum BlockchainError {
    #[error("RPC error: {0}")]
    RpcError(String),
    #[error("Transaction failed: {0}")]
    TransactionFailed(String),
    #[error("Invalid address: {0}")]
    InvalidAddress(String),
    #[error("Insufficient gas: {0}")]
    InsufficientGas(String),
}