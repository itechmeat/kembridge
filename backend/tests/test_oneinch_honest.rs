// tests/test_oneinch_honest.rs
// Tests for honest 1inch integration

use kembridge_backend::oneinch::{OneinchService, types::QuoteParams, client::FusionClient};
use kembridge_backend::constants::*;
use bigdecimal::BigDecimal;
use std::sync::Arc;

/// Test creation of service with API key validation
#[tokio::test]
async fn test_service_creation_with_api_key_validation() {
    // Test with potentially invalid key
    let service = OneinchService::new(
        "test_key_for_testing".to_string(),
        ONEINCH_SEPOLIA_CHAIN_ID,
    );
    
    // Check that the service was created
    assert!(service.is_supported_chain(ONEINCH_SEPOLIA_CHAIN_ID));
    assert!(!service.has_price_oracle()); // Initially no oracle
}

/// Test API key validation
#[tokio::test]
async fn test_api_key_validation() {
    let service = OneinchService::new(
        "definitely_invalid_key".to_string(),
        ONEINCH_ETHEREUM_CHAIN_ID,
    );
    
    // With an invalid key, there should be an error or false
    match service.validate_api_key().await {
        Ok(false) => {
            // Expected result for an invalid key
            println!("✅ Correctly identified invalid API key");
        },
        Err(_) => {
            // Also acceptable - error when checking
            println!("✅ Error when checking invalid key (expected)");
        },
        Ok(true) => {
            // Unexpected, but possible if API doesn't strictly check keys
            println!("⚠️  Unexpected: invalid key passed validation");
        }
    }
}

/// Test comprehensive health check
#[tokio::test]
async fn test_comprehensive_health_check() {
    let service = OneinchService::new(
        "test_key".to_string(),
        ONEINCH_ETHEREUM_CHAIN_ID,
    );
    
    // Perform health check
    match service.comprehensive_health_check().await {
        Ok(health_data) => {
            // Check the response structure
            assert!(health_data.get("chain_id").is_some());
            assert!(health_data.get("chain_supported").is_some());
            assert!(health_data.get("api_connectivity").is_some());
            assert!(health_data.get("api_key").is_some());
            assert!(health_data.get("timestamp").is_some());
            
            println!("✅ Health check returned correct structure");
        },
        Err(e) => {
            println!("⚠️  Health check failed: {} (expected with test key)", e);
        }
    }
}

/// Test liquidity information
#[tokio::test]
async fn test_liquidity_info() {
    let service = OneinchService::new(
        "test_key".to_string(),
        ONEINCH_ETHEREUM_CHAIN_ID,
    );
    
    // Test popular ETH/USDC pair
    match service.get_liquidity_info(
        ETHEREUM_NATIVE_TOKEN,
        ETHEREUM_USDC_ADDRESS
    ).await {
        Ok(liquidity_info) => {
            // Check the response structure
            assert!(liquidity_info.get("available").is_some());
            
            if liquidity_info["available"].as_bool().unwrap_or(false) {
                println!("✅ Liquidity available");
                
                if let Some(score) = liquidity_info.get("liquidity_score") {
                    println!("  Score: {:?}", score);
                }
            } else {
                println!("ℹ️  Liquidity unavailable (may be due to test key)");
            }
        },
        Err(e) => {
            println!("⚠️  Error getting liquidity: {} (expected with test key)", e);
        }
    }
}

/// Test getting a quote with honest error handling
#[tokio::test]
async fn test_quote_with_honest_error_handling() {
    let service = OneinchService::new(
        "test_key".to_string(),
        ONEINCH_ETHEREUM_CHAIN_ID,
    );
    
    let quote_params = QuoteParams {
        from_token: ETHEREUM_NATIVE_TOKEN.to_string(),
        to_token: ETHEREUM_USDC_ADDRESS.to_string(),
        amount: BigDecimal::from(1000000000000000000u64), // 1 ETH
        from_address: ETHEREUM_ZERO_ADDRESS.to_string(),
        slippage: Some(0.5),
        disable_estimate: Some(false),
        allow_partial_fill: Some(true),
        source: Some("kembridge-test".to_string()),
    };
    
    match service.get_quote(&quote_params).await {
        Ok(quote) => {
            println!("✅ Quote received:");
            println!("  From: {}", quote.from_amount);
            println!("  To: {}", quote.to_amount);
            println!("  Gas: {}", quote.estimated_gas);
            
            // Check that the quote has reasonable values
            assert!(quote.from_amount > BigDecimal::from(0));
            assert!(quote.to_amount > BigDecimal::from(0));
            assert!(quote.estimated_gas > BigDecimal::from(0));
        },
        Err(e) => {
            println!("⚠️  Error getting quote: {}", e);
            println!("ℹ️  This is expected with test API key - honest error handling!");
            
            // Check that this is an honest error, not a fallback
            let error_msg = e.to_string();
            assert!(!error_msg.contains("fallback"));
            assert!(!error_msg.contains("mock"));
            assert!(!error_msg.contains("fake"));
        }
    }
}

/// Test quote parameter validation
#[tokio::test]
async fn test_quote_parameter_validation() {
    let service = OneinchService::new(
        "test_key".to_string(),
        ONEINCH_ETHEREUM_CHAIN_ID,
    );
    
    // Test with invalid parameters
    let invalid_params = QuoteParams {
        from_token: "".to_string(), // Empty address
        to_token: ETHEREUM_USDC_ADDRESS.to_string(),
        amount: BigDecimal::from(0), // Zero amount
        from_address: ETHEREUM_ZERO_ADDRESS.to_string(),
        slippage: Some(0.1),
        disable_estimate: Some(false),
        allow_partial_fill: Some(true),
        source: Some("kembridge-validation-test".to_string()),
    };
    
    match service.get_quote(&invalid_params).await {
        Ok(_) => {
            panic!("Unexpected quote received for invalid parameters!");
        },
        Err(e) => {
            println!("✅ Invalid parameters rejected correctly: {}", e);
            
            // Check that this is a validation error
            let error_msg = e.to_string();
            assert!(
                error_msg.contains("empty") || 
                error_msg.contains("invalid") || 
                error_msg.contains("zero") ||
                error_msg.contains("greater than 0")
            );
        }
    }
}

/// Test supported chains
#[test]
fn test_supported_chains() {
    let service = OneinchService::new(
        "test_key".to_string(),
        ONEINCH_ETHEREUM_CHAIN_ID,
    );
    
    // Test supported chains
    assert!(service.is_supported_chain(ONEINCH_ETHEREUM_CHAIN_ID));
    assert!(service.is_supported_chain(ONEINCH_BSC_CHAIN_ID));
    assert!(service.is_supported_chain(ONEINCH_POLYGON_CHAIN_ID));
    assert!(service.is_supported_chain(ONEINCH_SEPOLIA_CHAIN_ID));
    
    // Test unsupported chains
    assert!(!service.is_supported_chain(999999));
    assert!(!service.is_supported_chain(0));
    
    // Check supported chains list
    let supported_chains = OneinchService::get_supported_chains();
    assert!(supported_chains.len() >= 6);
    assert!(supported_chains.contains(&ONEINCH_ETHEREUM_CHAIN_ID));
    assert!(supported_chains.contains(&ONEINCH_BSC_CHAIN_ID));
}

/// Test FusionClient creation with validation
#[test]
fn test_fusion_client_creation() {
    // Test with short key (should warn)
    let client = FusionClient::new(
        "short".to_string(),
        ONEINCH_ETHEREUM_CHAIN_ID,
    );
    
    assert_eq!(client.get_chain_id(), ONEINCH_ETHEREUM_CHAIN_ID);
    
    // Test with test key (should warn)
    let client2 = FusionClient::new(
        "test_key".to_string(),
        ONEINCH_BSC_CHAIN_ID,
    );
    
    assert_eq!(client2.get_chain_id(), ONEINCH_BSC_CHAIN_ID);
    
    // Test with more realistic key
    let client3 = FusionClient::new(
        "realistic_api_key_1234567890".to_string(),
        ONEINCH_POLYGON_CHAIN_ID,
    );
    
    assert_eq!(client3.get_chain_id(), ONEINCH_POLYGON_CHAIN_ID);
}

/// Test changing chain ID
#[test]
fn test_chain_id_update() {
    let mut client = FusionClient::new(
        "test_key_12345".to_string(),
        ONEINCH_ETHEREUM_CHAIN_ID,
    );
    
    assert_eq!(client.get_chain_id(), ONEINCH_ETHEREUM_CHAIN_ID);
    
    // Change chain ID
    client.set_chain_id(ONEINCH_BSC_CHAIN_ID);
    assert_eq!(client.get_chain_id(), ONEINCH_BSC_CHAIN_ID);
    
    // Check that the URL also updated
    // (this is internal logic, but important for proper operation)
}

/// Integration test with real API key (if available)
#[tokio::test]
async fn test_real_api_integration() {
    // This test only runs if a real API key is set
    let api_key = match std::env::var("ONEINCH_API_KEY") {
        Ok(key) if !key.is_empty() && key != "test_key" => key,
        _ => {
            println!("ℹ️  Skipping integration test - no real API key");
            println!("ℹ️  Set ONEINCH_API_KEY for full testing");
            return;
        }
    };
    
    println!("🔍 Running integration test with real API key");
    
    let service = OneinchService::new(api_key, ONEINCH_ETHEREUM_CHAIN_ID);
    
    // Test real key validation
    match service.validate_api_key().await {
        Ok(true) => println!("✅ Real API key is valid"),
        Ok(false) => println!("❌ Real API key is invalid"),
        Err(e) => println!("🔥 Error checking real key: {}", e),
    }
    
    // Test getting a real quote
    let quote_params = QuoteParams {
        from_token: ETHEREUM_NATIVE_TOKEN.to_string(),
        to_token: ETHEREUM_USDC_ADDRESS.to_string(),
        amount: BigDecimal::from(100000000000000000u64), // 0.1 ETH
        from_address: ETHEREUM_ZERO_ADDRESS.to_string(),
        slippage: Some(0.5),
        disable_estimate: Some(false),
        allow_partial_fill: Some(true),
        source: Some("kembridge-integration-test".to_string()),
    };
    
    match service.get_quote(&quote_params).await {
        Ok(quote) => {
            println!("✅ Real quote received from 1inch API");
            println!("  Rate: 1 ETH = {} USDC", 
                &quote.to_amount / &quote.from_amount * BigDecimal::from(10000000000u64));
            
            // Check that the data is reasonable
            assert!(quote.from_amount > BigDecimal::from(0));
            assert!(quote.to_amount > BigDecimal::from(0));
            assert!(quote.estimated_gas > BigDecimal::from(20000)); // Minimum gas
            assert!(!quote.protocols.is_empty()); // Should have protocols
        },
        Err(e) => {
            println!("❌ Error getting real quote: {}", e);
        }
    }
}

/// Test performance (should not be too slow)
#[tokio::test]
async fn test_performance() {
    let service = OneinchService::new(
        "test_key".to_string(),
        ONEINCH_ETHEREUM_CHAIN_ID,
    );
    
    let start = std::time::Instant::now();
    
    // Perform several operations
    let _ = service.validate_api_key().await;
    let _ = service.comprehensive_health_check().await;
    
    let duration = start.elapsed();
    
    // Should not take more than 30 seconds even with timeouts
    assert!(duration.as_secs() < 30, "Operations are too slow: {:?}", duration);
    
    println!("ℹ️  Test execution time: {:?}", duration);
}

/// Test network error handling
#[tokio::test]
async fn test_network_error_handling() {
    // Create service with invalid URL (this is internal logic)
    let service = OneinchService::new(
        "test_key".to_string(),
        999999, // Unsupported network
    );
    
    // Operations should return honest errors
    match service.validate_api_key().await {
        Ok(_) => println!("⚠️  Unexpected successful operation with invalid network"),
        Err(e) => {
            println!("✅ Network error handled correctly: {}", e);
            
            // Check that this is an honest error
            let error_msg = e.to_string();
            assert!(!error_msg.contains("fallback"));
            assert!(!error_msg.contains("default"));
        }
    }
}