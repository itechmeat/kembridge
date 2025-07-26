// Phase 4.1: Ethereum Adapter Integration Tests
use kembridge_blockchain::{EthereumAdapter, EthereumConfig, EthereumError, WalletInfo};
use ethers::types::Address;
use std::str::FromStr;

#[test]
fn test_ethereum_config_validation() {
    let valid_config = EthereumConfig::sepolia();
    assert!(valid_config.validate().is_ok());

    let invalid_config = EthereumConfig {
        rpc_url: "http://insecure.endpoint.com".to_string(),
        chain_id: 1, // mainnet instead of sepolia
        gas_price_multiplier: 1.2,
        confirmation_blocks: 2,
        max_retry_attempts: 3,
    };
    
    assert!(invalid_config.validate().is_err());
}

#[test]
fn test_ethereum_config_default() {
    let config = EthereumConfig::default();
    assert_eq!(config.chain_id, 11155111); // Sepolia
    assert!(config.rpc_url.starts_with("https://"));
}

#[tokio::test]
async fn test_ethereum_adapter_with_invalid_rpc() {
    let config = EthereumConfig {
        rpc_url: "https://invalid.rpc.endpoint.com".to_string(),
        chain_id: 11155111,
        gas_price_multiplier: 1.2,
        confirmation_blocks: 2,
        max_retry_attempts: 3,
    };

    // This should fail to connect to invalid RPC
    let result = EthereumAdapter::new(config).await;
    assert!(result.is_err());
}

#[test]
fn test_wallet_info_creation() {
    let address = Address::from_str("0x742d35Cc6564C0532d3c6B4a5d9C6A5c4a1e3a8b").unwrap();
    let wallet_info = WalletInfo {
        address,
        eth_balance: ethers::types::U256::from(1000000000000000000u64), // 1 ETH
        nonce: ethers::types::U256::from(5),
        token_balances: vec![],
        last_updated: chrono::Utc::now(),
    };

    assert_eq!(wallet_info.address, address);
    assert_eq!(wallet_info.eth_balance, ethers::types::U256::from(1000000000000000000u64));
    assert_eq!(wallet_info.nonce, ethers::types::U256::from(5));
}

#[test]
fn test_ethereum_error_types() {
    let error = EthereumError::ConnectionFailed("Test error".to_string());
    assert!(error.to_string().contains("Connection failed"));

    let insufficient_funds_error = EthereumError::InsufficientFunds {
        available: ethers::types::U256::from(500),
        required: ethers::types::U256::from(1000),
    };
    assert!(insufficient_funds_error.to_string().contains("Insufficient funds"));
}