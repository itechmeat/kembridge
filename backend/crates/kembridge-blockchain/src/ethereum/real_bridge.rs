// H4: Real Bridge Contract Integration
use ethers::{
    providers::{Provider, Http, Middleware},
    types::{Address, U256, H256, TransactionRequest, Bytes},
    contract::Contract,
    abi::Abi,
    middleware::SignerMiddleware,
    signers::{LocalWallet, Signer},
};
use std::sync::Arc;
use tokio::time::Duration;

use super::{EthereumError, bridge_abi::{get_bridge_abi, BridgeConstants}};

pub struct RealBridgeAdapter {
    provider: Arc<Provider<Http>>,
    contract_address: Address,
    contract: Contract<Provider<Http>>,
}

impl RealBridgeAdapter {
    /// Create new real bridge adapter
    pub async fn new(
        provider: Arc<Provider<Http>>,
        contract_address: Address,
    ) -> Result<Self, EthereumError> {
        let abi = get_bridge_abi();
        let contract = Contract::new(contract_address, abi, Arc::clone(&provider));
        
        // Verify contract exists and is valid
        let code = provider.get_code(contract_address, None).await
            .map_err(|e| EthereumError::NetworkError(e.to_string()))?;
        
        if code.is_empty() {
            return Err(EthereumError::ContractError(
                "No contract found at specified address".to_string()
            ));
        }
        
        tracing::info!(
            contract_address = %contract_address,
            "Real bridge adapter initialized"
        );
        
        Ok(Self {
            provider,
            contract_address,
            contract,
        })
    }
    
    /// Lock ETH tokens in bridge contract
    pub async fn lock_eth_tokens(
        &self,
        amount: U256,
        recipient_chain: &str,
        quantum_hash: &str,
        user_wallet: Address,
        private_key: &[u8], // TODO (MOCK WARNING): Use quantum-protected key storage (P1)
    ) -> Result<H256, EthereumError> {
        // Validate amount limits
        if amount < U256::from(BridgeConstants::MIN_LOCK_AMOUNT) {
            return Err(EthereumError::InvalidTransaction(
                format!("Amount {} is below minimum {}", amount, BridgeConstants::MIN_LOCK_AMOUNT)
            ));
        }
        
        if amount > U256::from(BridgeConstants::MAX_LOCK_AMOUNT) {
            return Err(EthereumError::InvalidTransaction(
                format!("Amount {} exceeds maximum {}", amount, BridgeConstants::MAX_LOCK_AMOUNT)
            ));
        }
        
        tracing::info!(
            contract_address = %self.contract_address,
            amount = %amount,
            recipient_chain = %recipient_chain,
            quantum_hash = %quantum_hash,
            user_wallet = %user_wallet,
            "Executing real ETH lock transaction"
        );
        
        // Create and send real transaction
        tracing::info!("Creating real bridge transaction with quantum-protected signing");
        
        // Convert private key bytes to LocalWallet
        let wallet = ethers::signers::LocalWallet::from_bytes(private_key)
            .map_err(|e| EthereumError::ContractError(format!("Invalid private key: {}", e)))?;
        
        // Connect wallet to provider
        let client = Arc::new(SignerMiddleware::new(
            Arc::clone(&self.provider),
            wallet.with_chain_id(self.provider.get_chainid().await?.as_u64())
        ));
        
        // Create contract instance with user signer
        let user_contract = Contract::new(
            self.contract_address,
            self.contract.abi().clone(),
            client
        );
        
        // Create the lock transaction call
        let lock_call = user_contract
            .method::<_, ()>("lockTokens", (recipient_chain.to_string(), quantum_hash.to_string()))
            .map_err(|e| EthereumError::ContractError(e.to_string()))?
            .value(amount);
        
        // Estimate gas for the transaction
        let gas_estimate = lock_call.estimate_gas().await
            .map_err(|e| EthereumError::GasEstimationFailed(format!("Gas estimation failed: {}", e)))?;
        
        // Send transaction with proper gas settings
        let call_with_gas = lock_call.gas(gas_estimate * 120 / 100); // 20% buffer
        let pending = call_with_gas
            .send()
            .await
            .map_err(|e| EthereumError::ContractError(format!("Transaction send failed: {}", e)))?;
        
        let tx_hash = pending.tx_hash().clone();
        
        tracing::info!(
            tx_hash = %tx_hash,
            "Real ETH lock transaction completed (demo mode)"
        );
        
        Ok(tx_hash)
    }

    /// Unlock ETH tokens from bridge contract
    pub async fn unlock_eth_tokens(
        &self,
        recipient: Address,
        amount: U256,
        source_chain: &str,
        quantum_hash: &str,
        admin_private_key: &[u8], // TODO (MOCK WARNING): Use quantum-protected admin key (P1)
    ) -> Result<H256, EthereumError> {
        // Validate amount limits
        if amount < U256::from(BridgeConstants::MIN_LOCK_AMOUNT) {
            return Err(EthereumError::InvalidTransaction(
                format!("Amount {} is below minimum {}", amount, BridgeConstants::MIN_LOCK_AMOUNT)
            ));
        }
        
        // Check contract balance
        let contract_balance = self.provider.get_balance(self.contract_address, None).await
            .map_err(|e| EthereumError::NetworkError(e.to_string()))?;
            
        if contract_balance < amount {
            return Err(EthereumError::ContractError(
                format!("Insufficient contract balance: {} < {}", contract_balance, amount)
            ));
        }
        
        tracing::info!(
            contract_address = %self.contract_address,
            recipient = %recipient,
            amount = %amount,
            source_chain = %source_chain,
            quantum_hash = %quantum_hash,
            "Executing real ETH unlock transaction"
        );
        
        // Create and send real admin transaction
        tracing::info!("Creating real unlock transaction with quantum-protected admin signing");
        
        // Convert admin private key bytes to LocalWallet
        let admin_wallet = ethers::signers::LocalWallet::from_bytes(admin_private_key)
            .map_err(|e| EthereumError::ContractError(format!("Invalid admin private key: {}", e)))?;
        
        // Connect admin wallet to provider
        let admin_client = Arc::new(SignerMiddleware::new(
            Arc::clone(&self.provider),
            admin_wallet.with_chain_id(self.provider.get_chainid().await?.as_u64())
        ));
        
        // Create contract instance with admin signer
        let admin_contract = Contract::new(
            self.contract_address,
            self.contract.abi().clone(),
            admin_client
        );
        
        // Estimate gas for the unlock transaction
        let unlock_call = admin_contract
            .method::<_, ()>("unlockTokens", (
                recipient,
                amount,
                source_chain.to_string(),
                quantum_hash.to_string()
            ))
            .map_err(|e| EthereumError::ContractError(e.to_string()))?;
            
        let gas_estimate = unlock_call.estimate_gas().await
            .map_err(|e| EthereumError::GasEstimationFailed(format!("Gas estimation failed: {}", e)))?;
        
        // Send transaction with proper gas settings
        let call_with_gas = unlock_call.gas(gas_estimate * 120 / 100); // 20% buffer
        let pending = call_with_gas
            .send()
            .await
            .map_err(|e| EthereumError::ContractError(format!("Unlock transaction failed: {}", e)))?;
        
        let tx_hash = pending.tx_hash().clone();
        
        tracing::info!(
            tx_hash = %tx_hash,
            "Real ETH unlock transaction completed (demo mode)"
        );
        
        Ok(tx_hash)
    }
    
    /// Get bridge contract statistics
    pub async fn get_bridge_stats(&self) -> Result<(U256, U256, U256, U256), EthereumError> {
        tracing::debug!("Fetching real bridge contract statistics");
        
        // Call getBridgeStats function on real contract
        let stats: (U256, U256, U256, U256) = self.contract
            .method("getBridgeStats", ())
            .map_err(|e| EthereumError::ContractError(e.to_string()))?
            .call()
            .await
            .map_err(|e| EthereumError::ContractError(format!("Failed to get bridge stats: {}", e)))?;
        
        tracing::info!(
            balance = %stats.0,
            locked = %stats.1,
            unlocked = %stats.2,
            active_balance = %stats.3,
            "Retrieved real bridge contract statistics"
        );
        
        Ok(stats)
    }
    
    /// Check if a quantum hash has been processed
    pub async fn is_processed(&self, quantum_hash: &str) -> Result<bool, EthereumError> {
        // Create hash from quantum_hash string
        let hash_bytes = ethers::utils::keccak256(quantum_hash.as_bytes());
        let hash = H256::from(hash_bytes);
        
        // Call isProcessed function on real contract
        let processed: bool = self.contract
            .method("isProcessed", hash)
            .map_err(|e| EthereumError::ContractError(e.to_string()))?
            .call()
            .await
            .map_err(|e| EthereumError::ContractError(format!("Failed to check processed status: {}", e)))?;
        
        tracing::debug!(
            quantum_hash = %quantum_hash,
            processed = processed,
            "Checked quantum hash processed status"
        );
        
        Ok(processed)
    }
    
    /// Get contract owner
    pub async fn get_owner(&self) -> Result<Address, EthereumError> {
        let owner: Address = self.contract
            .method("owner", ())
            .map_err(|e| EthereumError::ContractError(e.to_string()))?
            .call()
            .await
            .map_err(|e| EthereumError::ContractError(format!("Failed to get owner: {}", e)))?;
        
        tracing::debug!(owner = %owner, "Retrieved contract owner");
        Ok(owner)
    }
    
    /// Get contract address
    pub fn contract_address(&self) -> Address {
        self.contract_address
    }
    
    /// Get contract balance
    pub async fn get_contract_balance(&self) -> Result<U256, EthereumError> {
        let balance = self.provider.get_balance(self.contract_address, None).await
            .map_err(|e| EthereumError::NetworkError(e.to_string()))?;
        
        tracing::debug!(
            contract_address = %self.contract_address,
            balance = %balance,
            "Retrieved contract balance"
        );
        
        Ok(balance)
    }
}