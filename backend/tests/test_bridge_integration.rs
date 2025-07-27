// H4: Bridge Contract Integration Test
use ethers::types::{Address, U256};
use kembridge_blockchain::ethereum::{EthereumAdapter, EthereumConfig, RealBridgeAdapter};
use std::str::FromStr;

#[tokio::test]
async fn test_bridge_adapter_initialization() -> Result<(), Box<dyn std::error::Error>> {
    // Test Ethereum adapter creation
    let config = EthereumConfig::sepolia();
    let mut adapter = EthereumAdapter::new(config).await?;
    
    println!("✅ Ethereum adapter initialized successfully");
    
    // Test with placeholder contract address
    let contract_address = Address::from_str("0x0000000000000000000000000000000000000000")?;
    
    // This will fail since it's a zero address, but tests the flow
    let result = adapter.init_bridge_contract(contract_address).await;
    
    match result {
        Ok(_) => println!("✅ Bridge contract initialized (unexpected for zero address)"),
        Err(e) => println!("❌ Expected error for zero address: {}", e),
    }
    
    // Test bridge adapter access
    let bridge_adapter = adapter.bridge_adapter();
    assert!(bridge_adapter.is_none(), "Bridge adapter should be None after failed init");
    
    println!("✅ Bridge adapter integration test completed");
    
    Ok(())
}

#[tokio::test]
async fn test_mock_bridge_operations() -> Result<(), Box<dyn std::error::Error>> {
    let config = EthereumConfig::sepolia();
    let adapter = EthereumAdapter::new(config).await?;
    
    println!("✅ Testing mock bridge operations");
    
    let contract_address = Address::from_str("0x1234567890123456789012345678901234567890")?;
    let amount = U256::from(1_000_000_000_000_000u64); // 0.001 ETH
    let user_wallet = Address::from_str("0x742d35cc6634c0532925a3b8d1d1cc4d35a4b4d4")?;
    let quantum_hash = "test_quantum_hash_123";
    
    // Test lock operation (will use mock since no real bridge contract)
    let lock_tx = adapter.lock_eth_tokens(
        contract_address,
        amount,
        "near",
        quantum_hash,
        user_wallet,
    ).await?;
    
    println!("✅ Mock lock transaction: {}", lock_tx);
    
    // Test unlock operation (will use mock since no real bridge contract)
    let unlock_tx = adapter.unlock_eth_tokens(
        contract_address,
        amount,
        user_wallet,
        "proof_placeholder",
        quantum_hash,
    ).await?;
    
    println!("✅ Mock unlock transaction: {}", unlock_tx);
    
    println!("✅ Mock bridge operations test completed");
    
    Ok(())
}

/// Integration test for when real contract address is available
/// This test will be updated once contract is deployed
#[tokio::test]
#[ignore] // Ignore until real contract is deployed
async fn test_real_bridge_integration() -> Result<(), Box<dyn std::error::Error>> {
    // TODO: Update this test with real deployed contract address
    let real_contract_address = std::env::var("BRIDGE_CONTRACT_ADDRESS")
        .unwrap_or_else(|_| "0x0000000000000000000000000000000000000000".to_string());
    
    if real_contract_address == "0x0000000000000000000000000000000000000000" {
        println!("⏭️  Skipping real bridge test - no contract address provided");
        return Ok(());
    }
    
    let config = EthereumConfig::sepolia();
    let mut adapter = EthereumAdapter::new(config).await?;
    
    let contract_address = Address::from_str(&real_contract_address)?;
    
    // Initialize with real contract
    adapter.init_bridge_contract(contract_address).await?;
    println!("✅ Real bridge contract initialized: {}", contract_address);
    
    // Test bridge adapter is available
    let bridge_adapter = adapter.bridge_adapter();
    assert!(bridge_adapter.is_some(), "Bridge adapter should be available");
    
    // Test reading contract stats
    if let Some(bridge) = bridge_adapter {
        let stats = bridge.get_bridge_stats().await?;
        println!("✅ Contract stats - Balance: {}, Locked: {}, Unlocked: {}, Active: {}", 
                stats.0, stats.1, stats.2, stats.3);
        
        let owner = bridge.get_owner().await?;
        println!("✅ Contract owner: {}", owner);
        
        let balance = bridge.get_contract_balance().await?;
        println!("✅ Contract balance: {}", balance);
    }
    
    println!("✅ Real bridge integration test completed");
    
    Ok(())
}

/// Test bridge contract constants and ABI
#[tokio::test]
async fn test_bridge_constants_and_abi() -> Result<(), Box<dyn std::error::Error>> {
    use kembridge_blockchain::ethereum::{get_bridge_abi, BridgeConstants};
    
    // Test ABI loading
    let abi = get_bridge_abi();
    println!("✅ Bridge ABI loaded with {} functions/events", abi.len());
    
    // Test constants
    println!("✅ Min lock amount: {} wei", BridgeConstants::MIN_LOCK_AMOUNT);
    println!("✅ Max lock amount: {} wei", BridgeConstants::MAX_LOCK_AMOUNT);
    
    // Test function selectors
    println!("✅ Lock tokens selector: {:?}", BridgeConstants::LOCK_TOKENS_SELECTOR);
    println!("✅ Unlock tokens selector: {:?}", BridgeConstants::UNLOCK_TOKENS_SELECTOR);
    println!("✅ Get bridge stats selector: {:?}", BridgeConstants::GET_BRIDGE_STATS_SELECTOR);
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_constants() {
        use kembridge_blockchain::ethereum::BridgeConstants;
        
        // Validate constants make sense
        assert!(BridgeConstants::MIN_LOCK_AMOUNT > 0);
        assert!(BridgeConstants::MAX_LOCK_AMOUNT > BridgeConstants::MIN_LOCK_AMOUNT);
        assert_eq!(BridgeConstants::MIN_LOCK_AMOUNT, 1_000_000_000_000_000); // 0.001 ETH
        assert_eq!(BridgeConstants::MAX_LOCK_AMOUNT, 10_000_000_000_000_000_000); // 10 ETH
    }
}