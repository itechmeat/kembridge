// Phase 4.1: Ethereum configuration for Sepolia testnet
use super::EthereumError;

#[derive(Debug, Clone)]
pub struct EthereumConfig {
    pub rpc_url: String,
    pub chain_id: u64,
    pub gas_price_multiplier: f64,
    pub confirmation_blocks: u64,
    pub max_retry_attempts: u32,
}

impl EthereumConfig {
    /// Configuration for Sepolia testnet
    pub fn sepolia() -> Self {
        Self {
            rpc_url: std::env::var("ETHEREUM_RPC_URL")
                .unwrap_or_else(|_| "https://sepolia.infura.io/v3/YOUR_INFURA_KEY".to_string()),
            chain_id: 11155111, // Sepolia Chain ID
            gas_price_multiplier: 1.2, // 20% buffer for fast execution
            confirmation_blocks: 2, // Fast confirmations in testnet
            max_retry_attempts: 3,
        }
    }

    /// Configuration validation
    pub fn validate(&self) -> Result<(), EthereumError> {
        if self.chain_id != 11155111 {
            return Err(EthereumError::InvalidNetwork {
                expected: 11155111,
                actual: self.chain_id,
            });
        }

        if !self.rpc_url.starts_with("https://") {
            return Err(EthereumError::InvalidConfiguration(
                "RPC URL must use HTTPS".to_string()
            ));
        }

        if self.gas_price_multiplier < 1.0 || self.gas_price_multiplier > 5.0 {
            return Err(EthereumError::InvalidConfiguration(
                "Gas price multiplier must be between 1.0 and 5.0".to_string()
            ));
        }

        Ok(())
    }
}

impl Default for EthereumConfig {
    fn default() -> Self {
        Self::sepolia()
    }
}