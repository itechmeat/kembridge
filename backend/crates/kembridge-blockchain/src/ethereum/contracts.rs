// Phase 4.1: ERC-20 contracts (base implementation)
use ethers::{
    providers::{Provider, Http},
    types::{Address, U256, H256},
    contract::Contract,
    abi::Abi,
};
use std::sync::Arc;

use super::EthereumError;

pub struct ERC20Contract {
    contract: Contract<Provider<Http>>,
    address: Address,
}

impl ERC20Contract {
    /// Create ERC-20 contract
    pub async fn new(
        provider: Arc<Provider<Http>>,
        address: Address,
    ) -> Result<Self, EthereumError> {
        // Base ERC-20 ABI for balance and transfer
        let abi_json = r#"[
            {
                "constant": true,
                "inputs": [{"name": "_owner", "type": "address"}],
                "name": "balanceOf",
                "outputs": [{"name": "balance", "type": "uint256"}],
                "type": "function"
            },
            {
                "constant": false,
                "inputs": [
                    {"name": "_to", "type": "address"},
                    {"name": "_value", "type": "uint256"}
                ],
                "name": "transfer",
                "outputs": [{"name": "", "type": "bool"}],
                "type": "function"
            },
            {
                "constant": true,
                "inputs": [],
                "name": "decimals",
                "outputs": [{"name": "", "type": "uint8"}],
                "type": "function"
            },
            {
                "constant": true,
                "inputs": [],
                "name": "symbol",
                "outputs": [{"name": "", "type": "string"}],
                "type": "function"
            }
        ]"#;
        
        let abi: Abi = serde_json::from_str(abi_json)
            .map_err(|e| EthereumError::ContractError(format!("Invalid ABI: {}", e)))?;
        
        let contract = Contract::new(address, abi, provider);
        
        Ok(Self { contract, address })
    }
    
    /// Get token balance
    pub async fn balance_of(&self, owner: Address) -> Result<U256, EthereumError> {
        let balance: U256 = self.contract
            .method::<_, U256>("balanceOf", owner)
            .map_err(|e| EthereumError::ContractError(format!("Method call failed: {}", e)))?
            .call()
            .await
            .map_err(|e| EthereumError::ContractError(format!("Balance query failed: {}", e)))?;
        
        tracing::debug!(
            contract = %self.address,
            owner = %owner,
            balance = %balance,
            "Retrieved ERC-20 balance"
        );
        
        Ok(balance)
    }
    
    /// Get token symbol
    pub async fn symbol(&self) -> Result<String, EthereumError> {
        let symbol: String = self.contract
            .method::<_, String>("symbol", ())
            .map_err(|e| EthereumError::ContractError(format!("Method call failed: {}", e)))?
            .call()
            .await
            .map_err(|e| EthereumError::ContractError(format!("Symbol query failed: {}", e)))?;
        
        Ok(symbol)
    }
    
    /// Get number of decimal places
    pub async fn decimals(&self) -> Result<u8, EthereumError> {
        let decimals: u8 = self.contract
            .method::<_, u8>("decimals", ())
            .map_err(|e| EthereumError::ContractError(format!("Method call failed: {}", e)))?
            .call()
            .await
            .map_err(|e| EthereumError::ContractError(format!("Decimals query failed: {}", e)))?;
        
        Ok(decimals)
    }
    
    /// Transfer tokens (requires wallet with private key)
    /// TODO: Integration with quantum-protected wallets in Phase 4.3
    pub async fn transfer_mock(
        &self,
        _to: Address,
        _amount: U256,
    ) -> Result<H256, EthereumError> {
        // TODO: Implement in Phase 4.3 with quantum-protected signing
        tracing::warn!("ERC-20 transfer not implemented - requires quantum wallet integration in Phase 4.3");
        
        Err(EthereumError::ContractError(
            "ERC-20 transfers require quantum wallet integration - Phase 4.3".to_string()
        ))
    }
}