// NEAR Protocol configuration
// Phase 4.2: NEAR Protocol Adapter

use crate::near::{NearError, Result};
use serde::{Deserialize, Serialize};

/// NEAR Protocol configuration for different networks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NearConfig {
    pub network_id: String,
    pub rpc_url: String,
    pub helper_url: Option<String>,
    pub explorer_url: Option<String>,
    pub wallet_url: Option<String>,
}

impl NearConfig {
    /// Create configuration for NEAR testnet
    pub fn testnet() -> Self {
        Self {
            network_id: "testnet".to_string(),
            rpc_url: std::env::var("NEAR_TESTNET_RPC_URL")
                .unwrap_or_else(|_| "https://rpc.testnet.near.org".to_string()),
            helper_url: Some("https://helper.testnet.near.org".to_string()),
            explorer_url: Some("https://testnet.nearblocks.io".to_string()),
            wallet_url: Some("https://testnet.mynearwallet.com".to_string()),
        }
    }

    /// Create configuration for NEAR mainnet
    pub fn mainnet() -> Self {
        Self {
            network_id: "mainnet".to_string(),
            rpc_url: std::env::var("NEAR_MAINNET_RPC_URL")
                .unwrap_or_else(|_| "https://rpc.mainnet.near.org".to_string()),
            helper_url: Some("https://helper.mainnet.near.org".to_string()),
            explorer_url: Some("https://nearblocks.io".to_string()),
            wallet_url: Some("https://app.mynearwallet.com".to_string()),
        }
    }

    /// Create custom configuration
    pub fn custom(network_id: String, rpc_url: String) -> Self {
        Self {
            network_id,
            rpc_url,
            helper_url: None,
            explorer_url: None,
            wallet_url: None,
        }
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<()> {
        if self.network_id.is_empty() {
            return Err(NearError::ConfigError("Network ID cannot be empty".to_string()));
        }

        if self.rpc_url.is_empty() {
            return Err(NearError::ConfigError("RPC URL cannot be empty".to_string()));
        }

        // Basic URL validation
        if !self.rpc_url.starts_with("http://") && !self.rpc_url.starts_with("https://") {
            return Err(NearError::ConfigError("RPC URL must start with http:// or https://".to_string()));
        }

        Ok(())
    }

    /// Check if this is a testnet configuration
    pub fn is_testnet(&self) -> bool {
        self.network_id == "testnet"
    }

    /// Check if this is a mainnet configuration
    pub fn is_mainnet(&self) -> bool {
        self.network_id == "mainnet"
    }
}