pub mod ethereum;
pub mod near;
pub mod types;

pub use ethereum::*;
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