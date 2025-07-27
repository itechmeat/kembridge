// Phase 4.1: Ethereum Adapter - main implementation
use ethers::{
    providers::{Provider, Http, Middleware},
    types::{Address, U256, H256, TransactionReceipt, Transaction},
};
use std::sync::{Arc, Mutex};
use kembridge_crypto::QuantumKeyManager;
use tokio::time::Duration;

use super::{EthereumConfig, EthereumError, WalletInfo, TransactionStatus, RealBridgeAdapter};

pub struct EthereumAdapter {
    provider: Arc<Provider<Http>>,
    chain_id: u64,
    quantum_manager: Arc<Mutex<QuantumKeyManager>>,
    config: EthereumConfig,
    bridge_adapter: Option<RealBridgeAdapter>,
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
            quantum_manager: Arc::new(Mutex::new(QuantumKeyManager::new())),
            config,
            bridge_adapter: None, // Will be initialized when bridge contract address is set
        })
    }
    
    // TODO (MOCK WARNING): check if mock 
    /// Initialize real bridge contract adapter (H4: Real Bridge Integration)
    pub async fn init_bridge_contract(&mut self, contract_address: Address) -> Result<(), EthereumError> {
        let bridge_adapter = RealBridgeAdapter::new(
            Arc::clone(&self.provider),
            contract_address
        ).await?;
        
        self.bridge_adapter = Some(bridge_adapter);
        
        tracing::info!(
            contract_address = %contract_address,
            "Real bridge contract adapter initialized"
        );
        
        Ok(())
    }
    
    /// Get bridge adapter reference
    pub fn bridge_adapter(&self) -> Option<&RealBridgeAdapter> {
        self.bridge_adapter.as_ref()
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

    /// Lock ETH tokens in bridge contract
    // TODO (MOCK WARNING): check if fallback 
    /// Requires bridge contract to be initialized - no fallbacks
    pub async fn lock_eth_tokens(
        &self,
        bridge_contract_address: Address,
        amount: U256,
        recipient_chain: &str,
        quantum_hash: &str,
        user_wallet: Address,
    ) -> Result<H256, EthereumError> {
        // TODO (MOCK WARNING): check if fallback 
        // Require real bridge adapter - no fallbacks
        let bridge_adapter = self.bridge_adapter.as_ref()
            .ok_or_else(|| EthereumError::ContractError(
                "Bridge contract not initialized. Real bridge contract address required.".to_string()
            ))?;
        
        tracing::info!("Using REAL bridge contract for ETH lock operation");
        
        // Get real private key from quantum key manager
        let private_key = {
            let mut quantum_manager = self.quantum_manager.lock()
                .map_err(|e| EthereumError::QuantumCryptoError(format!("Mutex lock failed: {}", e)))?;
            let user_key_id = quantum_manager.generate_ethereum_keypair(uuid::Uuid::new_v4())
                .map_err(|e| EthereumError::QuantumCryptoError(e.to_string()))?;
            quantum_manager.get_ethereum_private_key(user_key_id)
                .map_err(|e| EthereumError::QuantumCryptoError(e.to_string()))?
        };
        
        bridge_adapter.lock_eth_tokens(
            amount,
            recipient_chain,
            quantum_hash,
            user_wallet,
            &private_key
        ).await
    }

    /// Unlock ETH tokens from bridge contract
    // TODO (MOCK WARNING): check if fallback 
    /// Requires bridge contract to be initialized - no fallbacks
    pub async fn unlock_eth_tokens(
        &self,
        bridge_contract_address: Address,
        amount: U256,
        recipient: Address,
        near_tx_proof: &str,
        quantum_hash: &str,
    ) -> Result<H256, EthereumError> {
        // TODO (MOCK WARNING): check if fallback 
        // Require real bridge adapter - no fallbacks
        let bridge_adapter = self.bridge_adapter.as_ref()
            .ok_or_else(|| EthereumError::ContractError(
                "Bridge contract not initialized. Real bridge contract address required.".to_string()
            ))?;
        
        tracing::info!("Using REAL bridge contract for ETH unlock operation");
        
        // Get real admin private key from quantum key manager
        let admin_private_key = {
            let mut quantum_manager = self.quantum_manager.lock()
                .map_err(|e| EthereumError::QuantumCryptoError(format!("Mutex lock failed: {}", e)))?;
            let admin_key_id = quantum_manager.get_admin_key()
                .map_err(|e| EthereumError::QuantumCryptoError(e.to_string()))?;
            quantum_manager.get_ethereum_private_key(admin_key_id)
                .map_err(|e| EthereumError::QuantumCryptoError(e.to_string()))?
        };
        
        bridge_adapter.unlock_eth_tokens(
            recipient,
            amount,
            "near", // Source chain
            quantum_hash,
            &admin_private_key
        ).await
    }

    /// Get and decrypt private key via quantum crypto
    /// TODO (feat): Implement in Phase 4.3 with BridgeService integration (P2.1)
    async fn get_decrypted_private_key(
        &self,
        key_id: uuid::Uuid,
        user_id: uuid::Uuid,
    ) -> Result<Vec<u8>, EthereumError> {
        // TODO (feat): Integration with QuantumService for key decryption in Phase 4.3 (P1)
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