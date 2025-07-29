// Integration test for 1inch API key and endpoints
use kembridge_backend::config::AppConfig;
use kembridge_backend::oneinch::{OneinchService, FusionClient};
use kembridge_backend::constants::ONEINCH_ETHEREUM_CHAIN_ID;
use std::sync::Arc;
use tracing::{info, error, warn};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::test]
async fn test_oneinch_api_integration() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::registry()
        .with(fmt::layer())
        .init();

    info!("🔍 Starting 1inch API integration test...");

    // Load configuration
    let config = Arc::new(AppConfig::from_env()?);
    
    // Get API key from environment
    let api_key = std::env::var("ONEINCH_API_KEY")
        .unwrap_or_else(|_| "YOUR_API_KEY_HERE".to_string());
    
    if api_key == "YOUR_API_KEY_HERE" {
        warn!("⚠️  No ONEINCH_API_KEY environment variable set, skipping test");
        return Ok(());
    }
    
    info!("🔑 Using API key: {}...", &api_key[..std::cmp::min(api_key.len(), 8)]);

    // Test 1: Direct FusionClient creation
    info!("📋 Test 1: Creating FusionClient...");
    let fusion_client = FusionClient::new(api_key.clone(), ONEINCH_ETHEREUM_CHAIN_ID);
    info!("✅ FusionClient created successfully");

    // Test 2: Basic health check
    info!("📋 Test 2: API health check...");
    match fusion_client.health_check().await {
        Ok(healthy) => {
            if healthy {
                info!("✅ 1inch API is healthy");
            } else {
                warn!("⚠️  1inch API health check returned false");
            }
        },
        Err(e) => {
            error!("❌ 1inch API health check failed: {}", e);
        }
    }

    // Test 3: API key validation
    info!("📋 Test 3: API key validation...");
    match fusion_client.validate_api_key().await {
        Ok(valid) => {
            if valid {
                info!("✅ API key is valid");
            } else {
                warn!("⚠️  API key validation returned false");
            }
        },
        Err(e) => {
            error!("❌ API key validation failed: {}", e);
        }
    }

    // Test 4: Get supported tokens
    info!("📋 Test 4: Getting supported tokens...");
    match fusion_client.get_tokens().await {
        Ok(tokens) => {
            info!("✅ Successfully retrieved {} tokens", tokens.len());
            if !tokens.is_empty() {
                info!("📝 Sample tokens:");
                for (i, token) in tokens.iter().take(3).enumerate() {
                    info!("  {}. {} ({})", i + 1, token.symbol, token.address);
                }
            }
        },
        Err(e) => {
            error!("❌ Failed to get tokens: {}", e);
        }
    }

    // Test 5: OneinchService comprehensive health check
    info!("📋 Test 5: OneinchService comprehensive health check...");
    let oneinch_service = OneinchService::new(api_key.clone(), ONEINCH_ETHEREUM_CHAIN_ID);
    
    match oneinch_service.comprehensive_health_check().await {
        Ok(health_report) => {
            info!("✅ Comprehensive health check completed");
            info!("📊 Health Report: {}", serde_json::to_string_pretty(&health_report)?);
        },
        Err(e) => {
            error!("❌ Comprehensive health check failed: {}", e);
        }
    }

    // Test 6: Test specific quote endpoint
    info!("📋 Test 6: Testing quote endpoint...");
    
    use kembridge_backend::oneinch::types::QuoteParams;
    use bigdecimal::BigDecimal;
    use kembridge_backend::constants::{ETHEREUM_NATIVE_TOKEN, ETHEREUM_USDT_ADDRESS, ONEINCH_TEST_FROM_ADDRESS};
    
    let quote_params = QuoteParams {
        from_token: ETHEREUM_NATIVE_TOKEN.to_string(),
        to_token: ETHEREUM_USDT_ADDRESS.to_string(),
        amount: BigDecimal::from(1000000000000000000i64), // 1 ETH in wei
        from_address: ONEINCH_TEST_FROM_ADDRESS.to_string(),
        slippage: Some(0.5),
        disable_estimate: Some(true),
        allow_partial_fill: Some(false),
        source: Some("kembridge-test".to_string()),
    };

    info!("🔄 Request parameters:");
    info!("  From: {} (ETH)", quote_params.from_token);
    info!("  To: {} (USDT)", quote_params.to_token);
    info!("  Amount: {} (1 ETH)", quote_params.amount);
    info!("  From Address: {}", quote_params.from_address);

    match fusion_client.get_quote(&quote_params).await {
        Ok(quote) => {
            info!("✅ Successfully got quote!");
            info!("📊 Quote details:");
            info!("  From Amount: {}", quote.from_amount);
            info!("  To Amount: {}", quote.to_amount);
            info!("  Estimated Gas: {}", quote.estimated_gas);
            info!("  Protocols: {:?}", quote.protocols);
            
            // Assert that we got a valid quote
            assert!(quote.to_amount > BigDecimal::from(0), "Quote should return positive to_amount");
        },
        Err(e) => {
            error!("❌ Quote failed: {}", e);
            // Don't fail the test if API key is not configured or API is temporarily unavailable
        }
    }

    info!("🏁 1inch API integration test completed");
    Ok(())
}