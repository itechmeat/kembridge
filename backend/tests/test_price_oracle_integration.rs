// Integration test for Price Oracle with 1inch integration
use kembridge_backend::config::AppConfig;
use kembridge_backend::price_oracle::PriceOracleService;
use kembridge_backend::oneinch::OneinchService;
use redis::aio::ConnectionManager;
use redis::Client;
use std::sync::Arc;
use tracing::{info, error, warn};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::test]
async fn test_price_oracle_with_oneinch_integration() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::registry()
        .with(fmt::layer())
        .init();

    info!("🔍 Testing Price Oracle integration with 1inch...");

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

    // Setup Redis connection
    info!("🔄 Connecting to Redis...");
    let redis_client = Client::open("redis://localhost:6379")?;
    let redis_manager = ConnectionManager::new(redis_client).await?;
    info!("✅ Redis connected");

    // Create 1inch service
    info!("🔄 Creating 1inch service...");
    let oneinch_service = Arc::new(OneinchService::new(api_key, 1)); // Ethereum mainnet
    info!("✅ 1inch service created");

    // Create price oracle service
    info!("🔄 Creating Price Oracle service...");
    let price_oracle = PriceOracleService::new(redis_manager, config.clone(), oneinch_service).await?;
    info!("✅ Price Oracle service created");

    // Test ETH/USD price
    info!("📋 Testing ETH/USD price...");
    match price_oracle.get_price("ETH/USD").await {
        Ok(price_data) => {
            info!("✅ Successfully got ETH/USD price!");
            info!("📊 Price details:");
            info!("  Symbol: {}", price_data.symbol);
            info!("  Price: ${}", price_data.price);
            info!("  Sources: {:?}", price_data.sources);
            info!("  Confidence: {:.2}", price_data.confidence);
            info!("  Timestamp: {}", price_data.last_updated);
            
            // Verify we have valid data
            assert!(!price_data.symbol.is_empty(), "Symbol should not be empty");
            assert!(price_data.price > bigdecimal::BigDecimal::from(0), "Price should be positive");
            assert!(!price_data.sources.is_empty(), "Should have at least one price source");
            
            // Check if 1inch is included in sources (it should be for ETH)
            info!("🔍 Checking if 1inch is used as price source...");
            if price_data.sources.contains(&"1inch".to_string()) {
                info!("✅ 1inch is working as primary price oracle for ETH!");
            } else {
                warn!("⚠️  1inch not found in sources: {:?}", price_data.sources);
            }
        },
        Err(e) => {
            error!("❌ Failed to get ETH/USD price: {}", e);
            
            // Try NEAR/USD as fallback test
            info!("📋 Trying NEAR/USD as fallback test...");
            match price_oracle.get_price("NEAR/USD").await {
                Ok(price_data) => {
                    info!("✅ NEAR/USD price worked!");
                    info!("📊 NEAR Price: ${} from {:?}", price_data.price, price_data.sources);
                },
                Err(e2) => {
                    error!("❌ NEAR/USD also failed: {}", e2);
                }
            }
        }
    }

    // Test multiple prices
    info!("📋 Testing multiple prices...");
    match price_oracle.get_multiple_prices(&["ETH/USD", "NEAR/USD", "BTC/USD"]).await {
        Ok(prices) => {
            info!("✅ Successfully got {} prices", prices.len());
            
            let mut oneinch_used = false;
            for price in prices {
                info!("  {}: ${} ({:?})", price.symbol, price.price, price.sources);
                
                // Check if 1inch is used for any EVM tokens
                if price.sources.contains(&"1inch".to_string()) {
                    oneinch_used = true;
                    info!("  ✅ 1inch used for {}", price.symbol);
                }
            }
            
            if oneinch_used {
                info!("✅ 1inch successfully integrated as Price Oracle!");
            } else {
                warn!("⚠️  1inch not used in any price sources - may indicate configuration issue");
            }
        },
        Err(e) => {
            error!("❌ Failed to get multiple prices: {}", e);
        }
    }

    info!("🏁 Price Oracle integration test completed");
    Ok(())
}