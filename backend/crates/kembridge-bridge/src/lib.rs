// TODO: This is mock implementation - Phase 4.3 will add real bridge logic
#![allow(unused_variables)]

pub mod types;
pub mod executor;

pub use types::*;
pub use executor::*;

#[derive(Debug, thiserror::Error)]
pub enum BridgeError {
    #[error("Bridge operation failed: {0}")]
    OperationFailed(String),
    #[error("Insufficient funds: required {required}, available {available}")]
    InsufficientFunds { required: String, available: String },
    #[error("Unsupported chain: {chain}")]
    UnsupportedChain { chain: String },
    #[error("Transaction failed: {0}")]
    TransactionFailed(String),
}