// Phase 4.3: Basic Bridge Logic - Full implementation
#![allow(unused_variables)]

pub mod types;
pub mod executor;
pub mod service;
pub mod swap_engine;
pub mod state_machine;
pub mod validation;
pub mod timeout_manager;
pub mod api;
pub mod event_handler;

pub use types::*;
pub use executor::*;
pub use service::*;
pub use swap_engine::*;
pub use state_machine::*;
pub use validation::*;
pub use timeout_manager::*;
pub use api::*;
pub use event_handler::*;

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
    #[error("Invalid amount format")]
    InvalidAmount,
    #[error("Unauthorized access")]
    Unauthorized,
    #[error("Atomic operation failed")]
    AtomicOperationFailed,
    #[error("Quantum integrity violation")]
    QuantumIntegrityViolation,
    #[error("Invalid state transition from {from:?} to {to:?}")]
    InvalidStateTransition { from: SwapStatus, to: SwapStatus },
    #[error("Swap operation not found")]
    SwapNotFound,
    #[error("Operation timeout")]
    OperationTimeout,
    #[error("Validation error: {0}")]
    ValidationError(String),
    #[error("Quantum crypto error: {0}")]
    QuantumCryptoError(String),
    #[error("Blockchain error: {0}")]
    BlockchainError(String),
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),
    #[error("Authentication error: {0}")]
    AuthError(String),
}