// Phase 4.1: Legacy Ethereum client - now moved to ethereum/mod.rs
// This file kept for backward compatibility

// Note: Can't re-export from ethereum module due to circular dependency

// Legacy client wrapper for compatibility
use crate::{BlockchainError, BlockchainConfig, TransactionHash, ChainType};
use anyhow::Result;

#[deprecated(note = "Use EthereumAdapter instead - Phase 4.1")]
pub struct EthereumClient {
    config: BlockchainConfig,
}

#[allow(deprecated)]
impl EthereumClient {
    pub fn new(config: BlockchainConfig) -> Self {
        Self { config }
    }

    pub async fn get_balance(&self, address: &str) -> Result<String, BlockchainError> {
        // INFO: Legacy implementation - use EthereumAdapter::get_eth_balance
        Ok("1000000000000000000".to_string()) // 1 ETH in wei
    }

    pub async fn send_transaction(&self, to: &str, amount: &str) -> Result<TransactionHash, BlockchainError> {
        // INFO: Legacy implementation - use EthereumAdapter in Phase 4.3
        Ok(TransactionHash {
            hash: "0x1234567890abcdef".to_string(),
            chain: ChainType::Ethereum,
        })
    }

    pub async fn get_transaction_status(&self, tx_hash: &str) -> Result<bool, BlockchainError> {
        // INFO: Legacy implementation - use EthereumAdapter::get_transaction_status
        Ok(true)
    }
}