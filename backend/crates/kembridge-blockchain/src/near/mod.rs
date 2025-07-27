// NEAR Protocol module structure for KEMBridge
// Phase 4.2: NEAR Protocol Adapter

pub mod adapter;
pub mod config;
pub mod error;
pub mod wallet;
pub mod events;

// Advanced features (will be implemented progressively)
#[cfg(feature = "chain-signatures")]
pub mod chain_signatures;

#[cfg(feature = "one-click")]
pub mod one_click_api;

// Always include one_click_api for now (testing)
#[cfg(not(feature = "one-click"))]
pub mod one_click_api;

// Re-exports for easy access
pub use adapter::NearAdapter;
pub use config::NearConfig;
pub use error::NearError;
pub use wallet::NearWallet;

// Advanced features re-exports
#[cfg(feature = "chain-signatures")]
pub use chain_signatures::ChainSignatureService;

pub use one_click_api::{
    OneClickApiClient, OneClickConfig, QuoteRequest, QuoteResponse, SwapStatus,
    OptimizationCriteria, OptimizationTarget, QuoteVariation, SwapExecution, ExecutionStatus
};

pub use events::{NearEventListener, NearBridgeEvent, NearEventListenerConfig};

// Type aliases for convenience
pub type Result<T> = std::result::Result<T, NearError>;