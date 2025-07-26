// Phase 4.1: Ethereum Adapter - main implementation
use ethers::{
    providers::{Provider, Http, Middleware},
    types::{Address, U256, H256, TransactionReceipt, Transaction},
};
use std::sync::Arc;
use kembridge_crypto::QuantumKeyManager;

use super::{EthereumConfig, EthereumError, WalletInfo, TransactionStatus};

pub struct EthereumAdapter {
    provider: Arc<Provider<Http>>,
    chain_id: u64,
    quantum_manager: QuantumKeyManager,
    config: EthereumConfig,
}

impl EthereumAdapter {
    /// Create new Ethereum adapter for Sepolia
    pub async fn new(config: EthereumConfig) -> Result<Self, EthereumError> {
        // Validate configuration
        config.validate()?;

        let provider = Provider::<Http>::try_from(&config.rpc_url)
            .map_err(|e| EthereumError::ConnectionFailed(e.to_string()))?;
        
        let provider = Arc::new(provider);
        
        // Check connection to correct network
        let network_id = provider.get_chainid().await
            .map_err(|e| EthereumError::NetworkError(e.to_string()))?;
        
        if network_id.as_u64() != config.chain_id {
            return Err(EthereumError::InvalidNetwork {
                expected: config.chain_id,
                actual: network_id.as_u64(),
            });
        }

        tracing::info!(
            chain_id = config.chain_id,
            rpc_url = %config.rpc_url,
            "Ethereum adapter connected to Sepolia testnet"
        );

        Ok(Self {
            provider,
            chain_id: config.chain_id,
            quantum_manager: QuantumKeyManager::new(),
            config,
        })
    }

    /// Monitor ETH wallet balance
    pub async fn get_eth_balance(&self, address: Address) -> Result<U256, EthereumError> {
        let balance = self.provider.get_balance(address, None).await
            .map_err(|e| EthereumError::NetworkError(e.to_string()))?;
        
        tracing::debug!(
            address = %address,
            balance = %balance,
            "Retrieved ETH balance"
        );
        
        Ok(balance)
    }

    /// Get nonce for address
    pub async fn get_nonce(&self, address: Address) -> Result<U256, EthereumError> {
        let nonce = self.provider.get_transaction_count(address, None).await
            .map_err(|e| EthereumError::NetworkError(e.to_string()))?;
        
        Ok(nonce)
    }

    /// Estimate gas price
    pub async fn estimate_gas_price(&self) -> Result<U256, EthereumError> {
        let gas_price = self.provider.get_gas_price().await
            .map_err(|e| EthereumError::GasEstimationFailed(e.to_string()))?;
        
        // Apply multiplier for fast execution
        let multiplied_price = gas_price
            .checked_mul(U256::from((self.config.gas_price_multiplier * 100.0) as u64))
            .and_then(|p| p.checked_div(U256::from(100)))
            .unwrap_or(gas_price);
        
        Ok(multiplied_price)
    }

    /// Comprehensive wallet monitoring (so far without ERC-20)
    pub async fn get_wallet_info_basic(&self, address: Address) -> Result<WalletInfo, EthereumError> {
        let eth_balance = self.get_eth_balance(address).await?;
        let nonce = self.get_nonce(address).await?;
        
        // In Phase 4.1 without ERC-20 tokens for now
        let token_balances = Vec::new();
        
        Ok(WalletInfo {
            address,
            eth_balance,
            nonce,
            token_balances,
            last_updated: chrono::Utc::now(),
        })
    }

    /// Check transaction status without waiting
    pub async fn get_transaction_status(&self, tx_hash: H256) -> Result<TransactionStatus, EthereumError> {
        // Check if transaction is in the blockchain
        match self.provider.get_transaction(tx_hash).await {
            Ok(Some(tx)) => {
                // Transaction found, checking receipt
                match self.provider.get_transaction_receipt(tx_hash).await {
                    Ok(Some(receipt)) => {
                        let current_block = self.provider.get_block_number().await
                            .map_err(|e| EthereumError::NetworkError(e.to_string()))?;
                        
                        let confirmations = current_block
                            .saturating_sub(receipt.block_number.unwrap_or_default())
                            + 1;
                        
                        Ok(TransactionStatus::Confirmed {
                            receipt,
                            confirmations: confirmations.as_u64(),
                        })
                    },
                    Ok(None) => Ok(TransactionStatus::Pending { transaction: tx }),
                    Err(e) => Err(EthereumError::NetworkError(e.to_string())),
                }
            },
            Ok(None) => Ok(TransactionStatus::NotFound),
            Err(e) => Err(EthereumError::NetworkError(e.to_string())),
        }
    }

    /// Get provider information (for health checks)
    pub async fn get_provider_info(&self) -> Result<(u64, u64), EthereumError> {
        let chain_id = self.provider.get_chainid().await
            .map_err(|e| EthereumError::NetworkError(e.to_string()))?;
        
        let block_number = self.provider.get_block_number().await
            .map_err(|e| EthereumError::NetworkError(e.to_string()))?;
        
        Ok((chain_id.as_u64(), block_number.as_u64()))
    }

    /// Get and decrypt private key via quantum crypto
    /// TODO: Implement in Phase 4.3 with BridgeService integration
    async fn get_decrypted_private_key(
        &self,
        key_id: uuid::Uuid,
        user_id: uuid::Uuid,
    ) -> Result<Vec<u8>, EthereumError> {
        // TODO: Integration with QuantumService for key decryption in Phase 4.3
        tracing::warn!(
            key_id = %key_id,
            user_id = %user_id,
            "Using mock private key - implement quantum decryption in Phase 4.3"
        );
        
        // In the real implementation of Phase 4.3:
        // 1. Get encrypted key from quantum_keys table
        // 2. Use quantum crypto for decryption
        // 3. Return decrypted private key
        
        Err(EthereumError::QuantumCryptoError(
            "Quantum key decryption not implemented - Phase 4.3".to_string()
        ))
    }
}