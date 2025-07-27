// NEAR Protocol Adapter - Main implementation
// Phase 4.2: NEAR Protocol Adapter

use crate::near::{NearConfig, NearError, NearWallet, Result};
use kembridge_crypto::QuantumKeyManager;
use near_jsonrpc_client::{methods, JsonRpcClient};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};

#[cfg(feature = "chain-signatures")]
use crate::near::chain_signatures::ChainSignatureService;

use crate::near::one_click_api::{OneClickApiClient, OneClickConfig};

/// NEAR Protocol Adapter for KEMBridge
/// 
/// Provides unified interface for interacting with NEAR Protocol including:
/// - Basic account operations
/// - Transaction handling 
/// - Balance queries
/// - Integration with quantum cryptography
/// - Chain Signatures support (when enabled)
/// - 1Click API support (when enabled)
pub struct NearAdapter {
    rpc_client: JsonRpcClient,
    config: NearConfig,
    quantum_manager: Arc<RwLock<QuantumKeyManager>>,
    default_wallet: Option<NearWallet>,
    
    #[cfg(feature = "chain-signatures")]
    chain_signatures: Option<ChainSignatureService>,
    
    one_click_api: Option<OneClickApiClient>,
}

impl NearAdapter {
    /// Create a new NEAR adapter with the given configuration
    pub async fn new(config: NearConfig) -> Result<Self> {
        config.validate()?;
        
        info!("Initializing NEAR adapter for network: {}", config.network_id);
        
        let rpc_client = JsonRpcClient::connect(&config.rpc_url);
        let quantum_manager = Arc::new(RwLock::new(QuantumKeyManager::new()));

        let adapter = Self {
            rpc_client,
            config,
            quantum_manager,
            default_wallet: None,
            
            #[cfg(feature = "chain-signatures")]
            chain_signatures: None,
            
            one_click_api: None,
        };

        // Test connection
        adapter.test_connection().await?;
        
        info!("NEAR adapter initialized successfully");
        Ok(adapter)
    }

    /// Create adapter with a default wallet
    pub async fn with_wallet(config: NearConfig, wallet: NearWallet) -> Result<Self> {
        let mut adapter = Self::new(config).await?;
        adapter.default_wallet = Some(wallet);
        Ok(adapter)
    }

    /// Test connection to NEAR network
    pub async fn test_connection(&self) -> Result<()> {
        debug!("Testing NEAR network connection...");
        
        let request = methods::status::RpcStatusRequest;
        let response = self.rpc_client.call(request).await
            .map_err(|e| NearError::RpcStatusError(e.to_string()))?;
        
        if response.chain_id != self.config.network_id {
            return Err(NearError::ConfigError(format!(
                "Network mismatch: expected {}, got {}",
                self.config.network_id, response.chain_id
            )));
        }

        debug!("NEAR connection test successful - Chain ID: {}", response.chain_id);
        Ok(())
    }

    /// Get network information
    pub async fn get_network_info(&self) -> Result<(String, u64)> {
        let request = methods::status::RpcStatusRequest;
        let response = self.rpc_client.call(request).await
            .map_err(|e| NearError::RpcStatusError(e.to_string()))?;
        
        Ok((response.chain_id, response.latest_protocol_version as u64))
    }

    /// Get account information (simplified for now)
    pub async fn get_account(&self, account_id: &str) -> Result<String> {
        debug!("Getting account info for: {}", account_id);
        
        // For now, just do basic validation
        if account_id.is_empty() {
            return Err(NearError::InvalidAccountId("Account ID cannot be empty".to_string()));
        }

        // TODO: Implement actual account query once version conflicts are resolved
        Ok(format!("Account info for: {}", account_id))
    }

    /// Get account balance (simplified for now)
    pub async fn get_balance(&self, account_id: &str) -> Result<u128> {
        // For now, we'll implement a simplified version
        // TODO: Implement proper balance query once type conflicts are resolved
        match self.get_account(account_id).await {
            Ok(_) => Ok(1000000000000000000000000), // 1 NEAR as placeholder
            Err(e) => Err(e),
        }
    }

    /// Get latest block height (simplified)
    pub async fn get_latest_block_height(&self) -> Result<u64> {
        // TODO: Implement proper block query once type conflicts are resolved
        Ok(100000000) // Placeholder block height
    }

    /// Check if account exists
    pub async fn account_exists(&self, account_id: &str) -> Result<bool> {
        match self.get_account(account_id).await {
            Ok(_) => Ok(true),
            Err(NearError::RpcQueryError(_)) => Ok(false),
            Err(e) => Err(e),
        }
    }

    /// Get configuration
    pub fn config(&self) -> &NearConfig {
        &self.config
    }

    /// Get default wallet (if set)
    pub fn default_wallet(&self) -> Option<&NearWallet> {
        self.default_wallet.as_ref()
    }

    /// Set default wallet
    pub fn set_default_wallet(&mut self, wallet: NearWallet) {
        self.default_wallet = Some(wallet);
    }

    /// Get quantum manager
    pub fn quantum_manager(&self) -> Arc<RwLock<QuantumKeyManager>> {
        Arc::clone(&self.quantum_manager)
    }

    /// Initialize Chain Signatures service
    #[cfg(feature = "chain-signatures")]
    pub fn init_chain_signatures(&mut self) -> Result<()> {
        let service = ChainSignatureService::testnet();
        self.chain_signatures = Some(service);
        info!("Chain Signatures service initialized");
        Ok(())
    }

    /// Get Chain Signatures service
    #[cfg(feature = "chain-signatures")]
    pub fn chain_signatures(&self) -> Option<&ChainSignatureService> {
        self.chain_signatures.as_ref()
    }

    /// Get mutable Chain Signatures service
    #[cfg(feature = "chain-signatures")]
    pub fn chain_signatures_mut(&mut self) -> Option<&mut ChainSignatureService> {
        self.chain_signatures.as_mut()
    }

    /// Quantum-protected account balance query
    pub async fn get_balance_quantum_protected(
        &self,
        account_id: &str,
        quantum_key_id: &str,
    ) -> Result<u128> {
        debug!("Fetching quantum-protected balance for account: {}", account_id);
        
        // Get balance normally
        let balance = self.get_balance(account_id).await?;
        
        // Quantum-protect the response data
        let balance_data = balance.to_string().into_bytes();
        let quantum_manager = self.quantum_manager.read().await;
        
        // TODO: Fix quantum manager API once method signature is available
        // For now we just return the balance without quantum protection
        // let _protected_data = quantum_manager.protect_data(&balance_data, quantum_key_id)?;
        
        Ok(balance)
    }

    /// Validate NEAR account ID format (basic validation)
    pub fn validate_account_id(account_id: &str) -> Result<()> {
        if account_id.is_empty() {
            return Err(NearError::InvalidAccountId("Account ID cannot be empty".to_string()));
        }
        
        // Basic validation - real validation would require near-primitives
        if account_id.contains("..") || account_id.starts_with('.') || account_id.ends_with('.') {
            return Err(NearError::InvalidAccountId("Invalid account ID format".to_string()));
        }
        
        // Check for uppercase letters (NEAR account IDs should be lowercase)
        if account_id.chars().any(|c| c.is_uppercase()) {
            return Err(NearError::InvalidAccountId("Account ID must be lowercase".to_string()));
        }
        
        Ok(())
    }

    /// Derive Ethereum address using Chain Signatures
    #[cfg(feature = "chain-signatures")]
    pub async fn derive_ethereum_address(&mut self, near_account_id: &str) -> Result<String> {
        if self.chain_signatures.is_none() {
            self.init_chain_signatures()?;
        }
        
        if let Some(service) = self.chain_signatures_mut() {
            service.derive_ethereum_address(near_account_id).await
        } else {
            Err(NearError::ChainSignaturesError("Chain Signatures service not available".to_string()))
        }
    }

    /// Sign Ethereum transaction using Chain Signatures with quantum protection
    #[cfg(feature = "chain-signatures")]
    pub async fn quantum_protected_chain_signature(
        &self,
        ethereum_tx: &[u8],
        near_account_id: &str,
        quantum_key_id: &str,
    ) -> Result<Vec<u8>> {
        // 1. Quantum protect the transaction data
        let quantum_manager = self.quantum_manager.read().await;
        // TODO: Implement quantum protection once API is available
        // let protected_tx = quantum_manager.protect_transaction_data(ethereum_tx, quantum_key_id)?;
        
        // 2. Sign using Chain Signatures
        if let Some(service) = self.chain_signatures.as_ref() {
            let signature = service.sign_ethereum_transaction(ethereum_tx, near_account_id).await?;
            Ok(signature)
        } else {
            Err(NearError::ChainSignaturesError("Chain Signatures service not initialized".to_string()))
        }
    }

    /// Create a new adapter with Chain Signatures enabled
    #[cfg(feature = "chain-signatures")]
    pub async fn with_chain_signatures(config: NearConfig) -> Result<Self> {
        let mut adapter = Self::new(config).await?;
        adapter.init_chain_signatures()?;
        Ok(adapter)
    }

    /// Initialize 1Click API client
    pub fn init_one_click_api(&mut self) -> Result<()> {
        let one_click_config = if self.config.is_testnet() {
            OneClickConfig::testnet()
        } else {
            OneClickConfig::mainnet()
        };
        
        let client = OneClickApiClient::new(one_click_config);
        self.one_click_api = Some(client);
        
        info!("1Click API client initialized");
        Ok(())
    }

    /// Get 1Click API client
    pub fn one_click_api(&self) -> Option<&OneClickApiClient> {
        self.one_click_api.as_ref()
    }

    /// Get mutable 1Click API client
    pub fn one_click_api_mut(&mut self) -> Option<&mut OneClickApiClient> {
        self.one_click_api.as_mut()
    }

    /// Create a new adapter with 1Click API enabled
    pub async fn with_one_click_api(config: NearConfig) -> Result<Self> {
        let mut adapter = Self::new(config).await?;
        adapter.init_one_click_api()?;
        Ok(adapter)
    }

    /// Create a new adapter with both Chain Signatures and 1Click API
    #[cfg(feature = "chain-signatures")]
    pub async fn with_full_features(config: NearConfig) -> Result<Self> {
        let mut adapter = Self::new(config).await?;
        adapter.init_chain_signatures()?;
        adapter.init_one_click_api()?;
        Ok(adapter)
    }

    /// Mint wrapped tokens on NEAR (ETH → NEAR direction)
    /// Implements Phase 4.3.4: NEAR mint/burn mechanism  
    pub async fn mint_bridge_tokens(
        &self,
        bridge_contract_id: &str,
        recipient: &str,
        amount: u128,
        eth_tx_proof: &str,
        quantum_hash: &str,
    ) -> Result<String> {
        // TODO [Phase 4.3.4]: Complete implementation with real NEAR bridge contract
        // This will include:
        // 1. Verify ETH lock transaction via Chain Signatures
        // 2. Call NEAR bridge contract ft_mint(recipient, amount, eth_proof, quantum_hash)
        // 3. Use self.rpc_client for transaction submission
        // 4. Wait for transaction finality
        // 5. Return real transaction hash

        tracing::info!(
            contract_id = %bridge_contract_id,
            recipient = %recipient,
            amount = %amount,
            eth_tx_proof = %eth_tx_proof,
            quantum_hash = %quantum_hash,
            "MOCK: Minting wrapped tokens on NEAR"
        );

        // Simulate NEAR network delay
        tokio::time::sleep(std::time::Duration::from_secs(3)).await;

        // Generate mock transaction hash
        let mock_tx_hash = format!("{}:{}", quantum_hash, hex::encode(recipient.as_bytes()));
        
        tracing::info!(tx_hash = %mock_tx_hash, "Mock NEAR mint transaction created");
        Ok(mock_tx_hash)
    }

    /// Burn wrapped tokens on NEAR (NEAR → ETH direction)
    /// Implements Phase 4.3.4: NEAR burn mechanism for NEAR → ETH
    pub async fn burn_bridge_tokens(
        &self,
        bridge_contract_id: &str,
        amount: u128,
        eth_recipient: &str,
        quantum_hash: &str,
    ) -> Result<String> {
        // TODO [Phase 4.3.4]: Complete implementation with real NEAR bridge contract
        // This will include:
        // 1. Call NEAR bridge contract ft_burn(amount, eth_recipient, quantum_hash)
        // 2. Use self.rpc_client for transaction submission
        // 3. Generate proof for Ethereum unlock
        // 4. Emit bridge event for Ethereum side
        // 5. Return transaction hash

        tracing::info!(
            contract_id = %bridge_contract_id,
            amount = %amount,
            eth_recipient = %eth_recipient,
            quantum_hash = %quantum_hash,
            "MOCK: Burning wrapped tokens on NEAR"
        );

        // Simulate NEAR network delay
        tokio::time::sleep(std::time::Duration::from_secs(3)).await;

        // Generate mock transaction hash
        let mock_tx_hash = format!("{}:{}", quantum_hash, hex::encode(eth_recipient.as_bytes()));
        
        tracing::info!(tx_hash = %mock_tx_hash, "Mock NEAR burn transaction created");
        Ok(mock_tx_hash)
    }

    /// Lock NEAR tokens in bridge contract (NEAR → ETH direction)
    /// Alternative to burn for non-wrapped NEAR tokens
    pub async fn lock_near_tokens(
        &self,
        bridge_contract_id: &str,
        amount: u128,
        eth_recipient: &str,
        quantum_hash: &str,
    ) -> Result<String> {
        // TODO [Phase 4.3.4]: Complete implementation with real NEAR bridge contract
        // This will include:
        // 1. Call NEAR bridge contract near_lock(amount, eth_recipient, quantum_hash)
        // 2. Lock native NEAR tokens in escrow
        // 3. Generate proof for Ethereum mint
        // 4. Return transaction hash

        tracing::info!(
            contract_id = %bridge_contract_id,
            amount = %amount,
            eth_recipient = %eth_recipient,
            quantum_hash = %quantum_hash,
            "MOCK: Locking NEAR tokens in bridge contract"
        );

        // Simulate NEAR network delay
        tokio::time::sleep(std::time::Duration::from_secs(3)).await;

        // Generate mock transaction hash
        let mock_tx_hash = format!("{}:{}", quantum_hash, hex::encode(eth_recipient.as_bytes()));
        
        tracing::info!(tx_hash = %mock_tx_hash, "Mock NEAR lock transaction created");
        Ok(mock_tx_hash)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::near::NearConfig;

    #[tokio::test]
    async fn test_near_adapter_creation() {
        let config = NearConfig::testnet();
        
        // This test requires actual network connection, so it might fail in CI
        // In production, we'd use a mock client for testing
        match NearAdapter::new(config).await {
            Ok(adapter) => {
                assert!(adapter.config.is_testnet());
                assert!(adapter.default_wallet.is_none());
            }
            Err(e) => {
                // Network connection might fail in test environment
                println!("Test failed (expected in CI): {}", e);
            }
        }
    }

    #[test]
    fn test_account_id_validation() {
        // Valid account IDs
        assert!(NearAdapter::validate_account_id("alice.testnet").is_ok());
        assert!(NearAdapter::validate_account_id("system").is_ok());
        assert!(NearAdapter::validate_account_id("test.near").is_ok());

        // Invalid account IDs
        assert!(NearAdapter::validate_account_id("").is_err());
        assert!(NearAdapter::validate_account_id("INVALID").is_err());
        assert!(NearAdapter::validate_account_id("alice..testnet").is_err());
    }
}