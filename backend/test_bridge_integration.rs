// Test BridgeIntegrationService with real API calls
use std::env;
use std::sync::Arc;
use kembridge_backend::services::BridgeIntegrationService;
use kembridge_backend::oneinch::OneinchService;
use kembridge_backend::oneinch::OneinchBridgeIntegration;
use kembridge_backend::services::BridgeService;
use kembridge_backend::constants::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    // Get API key from environment
    let api_key = env::var("ONEINCH_API_KEY")
        .map_err(|_| "ONEINCH_API_KEY not set in environment")?;
    
    println!("🔧 Testing BridgeIntegrationService with real API key...");
    
    // Create OneinchService
    let oneinch_service = Arc::new(OneinchService::new(
        api_key.clone(),
        ONEINCH_ETHEREUM_CHAIN_ID,
    ));
    
    // Create OneinchBridgeIntegration
    let oneinch_bridge = Arc::new(OneinchBridgeIntegration::new(
        api_key.clone(),
        oneinch_service.clone(),
    ));
    
    // Create BridgeService
    let bridge_service = Arc::new(BridgeService::new(
        oneinch_bridge.clone(),
        None, // No NEAR integration for this test
    ));
    
    // Create BridgeIntegrationService
    let bridge_integration = BridgeIntegrationService::new(bridge_service);
    
    // Test 1: Get bridge quote
    println!("\n🌉 Testing bridge quote...");
    match bridge_integration.get_bridge_quote(
        "1",           // Ethereum mainnet
        "42161",       // Arbitrum One
        "0xA0b86a33E6441fE7c29b0A2E25D07E7d44C72d1D", // DAI
        "0xA0b86a33E6441fE7c29b0A2E25D07E7d44C72d1D", // DAI
        "1000000000000000000", // 1 DAI
    ).await {
        Ok(quote) => println!("✅ Bridge quote successful: {}", quote),
        Err(e) => println!("❌ Bridge quote failed: {}", e),
    }
    
    // Test 2: Get single chain swap quote
    println!("\n🔄 Testing single chain swap quote...");
    match bridge_integration.get_single_chain_swap_quote(
        "1",           // Ethereum mainnet
        "0xA0b86a33E6441fE7c29b0A2E25D07E7d44C72d1D", // DAI
        "0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE", // ETH
        "1000000000000000000", // 1 DAI
    ).await {
        Ok(quote) => println!("✅ Single chain swap quote successful: {}", quote),
        Err(e) => println!("❌ Single chain swap quote failed: {}", e),
    }
    
    // Test 3: Check bridge service availability
    println!("\n🔍 Testing bridge service availability...");
    match bridge_integration.is_bridge_service_available("1inch").await {
        Ok(available) => println!("✅ Bridge service availability: {}", available),
        Err(e) => println!("❌ Bridge service availability check failed: {}", e),
    }
    
    println!("\n🎉 All tests completed!");
    Ok(())
}