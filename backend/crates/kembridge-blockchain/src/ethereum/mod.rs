// Phase 4.1: Ethereum Adapter Implementation
pub mod adapter;
pub mod config;
pub mod contracts;
pub mod error;
pub mod wallet;

// Tests moved to tests/ directory

pub use adapter::EthereumAdapter;
pub use config::EthereumConfig;
pub use contracts::ERC20Contract;
pub use error::EthereumError;
pub use wallet::{WalletInfo, TokenBalance, TransactionStatus};