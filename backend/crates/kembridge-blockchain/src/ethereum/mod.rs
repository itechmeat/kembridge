// Phase 4.1: Ethereum Adapter Implementation
pub mod adapter;
pub mod config;
pub mod contracts;
pub mod error;
pub mod wallet;
pub mod events;

// Tests moved to tests/ directory

pub use adapter::EthereumAdapter;
pub use config::EthereumConfig;
pub use contracts::ERC20Contract;
pub use error::EthereumError;
pub use wallet::{WalletInfo, TokenBalance, TransactionStatus};
pub use events::{EthereumEventListener, BridgeEvent, EventListenerConfig};