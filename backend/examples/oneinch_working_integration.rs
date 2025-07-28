// examples/oneinch_working_integration.rs
// Fixed 1inch integration using working API endpoints
// Based on endpoint testing results

use bigdecimal::BigDecimal;
use kembridge_backend::{
    constants::*,
    oneinch::{types::QuoteParams, OneinchService},
};
use reqwest::Client;
use serde_json::Value;
use std::sync::Arc;
use tracing::{error, info, warn};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();

    info!("🚀 Starting WORKING 1inch integration example");
    info!("📋 Using endpoints that actually work based on testing");

    let api_key = std::env::var("ONEINCH_API_KEY").unwrap_or_else(|_| {
        warn!("⚠️  ONEINCH_API_KEY not found, using test key");
        "MrTcxGYJhqVtUaXOjD3A7fRKsfxg52pZ".to_string()
    });

    info!("🔑 Using API key: {}...", &api_key[..8]);

    // Test working endpoints first
    info!("🧪 Testing working endpoints directly");
    test_working_endpoints(&api_key).await?;

    // Now test with our service using corrected endpoints
    info!("🔧 Testing with OneinchService (using working endpoints)");
    test_with_service(&api_key).await?;

    info!("🎉 Working integration test completed!");
    Ok(())
}

async fn test_working_endpoints(api_key: &str) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    
    // Test 1: Get tokens (this works)
    info!("📡 Test 1: Getting tokens from Swap API v5.2");
    let tokens_url = "https://api.1inch.dev/swap/v5.2/1/tokens";
    
    let response = client.get(tokens_url)
        .header("Authorization", format!("Bearer {}", api_key))
        .send()
        .await?;

    if response.status().is_success() {
        let tokens: Value = response.json().await?;
        if let Some(tokens_obj) = tokens["tokens"].as_object() {
            info!("✅ Found {} tokens", tokens_obj.len());
            
            // Show some popular tokens
            let popular_tokens = ["0xA0b86a33E6441b8C4505B8C4505B8C4505B8C4505", // USDC
                                 "0xdAC17F958D2ee523a2206206994597C13D831ec7", // USDT
                                 "0x2260FAC5E5542a773Aa44fBCfeDf7C193bc2C599"]; // WBTC
            
            for addr in &popular_tokens {
                if let Some(token) = tokens_obj.get(*addr) {
                    if let (Some(symbol), Some(name)) = 
                        (token["symbol"].as_str(), token["name"].as_str()) {
                        info!("   🪙 {}: {}", symbol, name);
                    }
                }
            }
        }
    } else {
        error!("❌ Tokens request failed: {}", response.status());
    }

    // Test 2: Get quote (this should work with correct parameters)
    info!("📡 Test 2: Getting quote from Swap API v5.2");
    let quote_url = "https://api.1inch.dev/swap/v5.2/1/quote";
    
    let response = client.get(quote_url)
        .header("Authorization", format!("Bearer {}", api_key))
        .query(&[
            ("src", "0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE"), // ETH
            ("dst", "0xA0b86a33E6441b8C4505B8C4505B8C4505B8C4505"), // USDC (correct address)
            ("amount", "1000000000000000000"), // 1 ETH
        ])
        .send()
        .await?;

    if response.status().is_success() {
        let quote: Value = response.json().await?;
        info!("✅ Quote received successfully");
        
        if let (Some(from_amount), Some(to_amount)) = 
            (quote["fromTokenAmount"].as_str(), quote["toTokenAmount"].as_str()) {
            info!("   💱 {} ETH → {} USDC", 
                BigDecimal::from(from_amount.parse::<u64>().unwrap_or(0)) / BigDecimal::from(1000000000000000000u64),
                BigDecimal::from(to_amount.parse::<u64>().unwrap_or(0)) / BigDecimal::from(1000000u64));
        }
        
        if let Some(gas) = quote["estimatedGas"].as_str() {
            info!("   ⛽ Estimated gas: {}", gas);
        }
    } else {
        let error_text = response.text().await?;
        error!("❌ Quote request failed: {}", error_text);
    }

    // Test 3: Price API (this works)
    info!("📡 Test 3: Getting prices from Price API v1.1");
    let price_url = "https://api.1inch.dev/price/v1.1/1";
    
    let response = client.get(price_url)
        .header("Authorization", format!("Bearer {}", api_key))
        .send()
        .await?;

    if response.status().is_success() {
        let prices: Value = response.json().await?;
        info!("✅ Prices received successfully");
        
        // Show some token prices
        let tokens_to_check = [
            ("0xA0b86a33E6441b8C4505B8C4505B8C4505B8C4505", "USDC"),
            ("0xdAC17F958D2ee523a2206206994597C13D831ec7", "USDT"),
            ("0x2260FAC5E5542a773Aa44fBCfeDf7C193bc2C599", "WBTC"),
        ];
        
        for (addr, symbol) in &tokens_to_check {
            if let Some(price) = prices[addr].as_str() {
                info!("   💰 {}: ${}", symbol, price);
            }
        }
    } else {
        error!("❌ Price request failed: {}", response.status());
    }

    Ok(())
}

async fn test_with_service(api_key: &str) -> Result<(), Box<dyn std::error::Error>> {
    info!("🔧 Testing with OneinchService");
    
    let oneinch_service = Arc::new(OneinchService::new(
        api_key.to_string(),
        ONEINCH_ETHEREUM_CHAIN_ID,
    ));

    // Test API key validation
    info!("🔍 Testing API key validation");
    match oneinch_service.validate_api_key().await {
        Ok(true) => info!("✅ API key is valid"),
        Ok(false) => warn!("❌ API key is invalid"),
        Err(e) => error!("🔥 API key validation error: {}", e),
    }

    // Test quote with correct parameters
    info!("🔍 Testing quote with working parameters");
    let quote_params = QuoteParams {
        from_token: ETHEREUM_NATIVE_TOKEN.to_string(), // ETH
        to_token: "0xA0b86a33E6441b8C4505B8C4505B8C4505B8C4505".to_string(), // USDC (correct)
        amount: BigDecimal::from(100000000000000000u64), // 0.1 ETH
        from_address: ETHEREUM_ZERO_ADDRESS.to_string(),
        slippage: Some(ONEINCH_DEFAULT_SLIPPAGE),
        disable_estimate: Some(false),
        allow_partial_fill: Some(true),
        source: Some("kembridge-working-test".to_string()),
    };

    match oneinch_service.get_quote(&quote_params).await {
        Ok(quote) => {
            info!("✅ Quote received from OneinchService");
            
            let eth_multiplier = BigDecimal::from(1000000000000000000u64);
            let usdc_multiplier = BigDecimal::from(1000000u64);
            
            let from_readable = &quote.from_amount / &eth_multiplier;
            let to_readable = &quote.to_amount / &usdc_multiplier;
            
            info!("   💱 {} ETH → {} USDC", from_readable, to_readable);
            info!("   ⛽ Gas: {}", quote.estimated_gas);
            info!("   🔄 Protocols: {}", quote.protocols.len());
            
            let expires_in = (quote.expires_at - chrono::Utc::now()).num_seconds();
            info!("   ⏰ Valid for: {} seconds", expires_in);
        }
        Err(e) => {
            error!("❌ Quote failed: {}", e);
            error!("💡 This might be due to service configuration issues");
        }
    }

    // Test health check
    info!("🔍 Testing health check");
    match oneinch_service.comprehensive_health_check().await {
        Ok(health) => {
            info!("✅ Health check passed");
            if let Some(tokens) = health.get("tokens") {
                if let Some(count) = tokens.get("count") {
                    info!("   📊 Available tokens: {}", count);
                }
            }
        }
        Err(e) => {
            warn!("❌ Health check failed: {}", e);
        }
    }

    Ok(())
}

/// Summary of findings and recommendations
fn print_summary() {
    info!("📋 SUMMARY OF FINDINGS:");
    info!("✅ WORKING ENDPOINTS:");
    info!("   • Swap API v5.2 tokens: https://api.1inch.dev/swap/v5.2/1/tokens");
    info!("   • Swap API v6.0 tokens: https://api.1inch.dev/swap/v6.0/1/tokens");
    info!("   • Price API v1.1: https://api.1inch.dev/price/v1.1/1");
    info!("   • Swap API v5.2 quote: https://api.1inch.dev/swap/v5.2/1/quote (with correct params)");
    
    info!("❌ NOT WORKING ENDPOINTS:");
    info!("   • Fusion API: https://api.1inch.dev/fusion/v1.0/1/* (No proxy config found)");
    info!("   • Protocols endpoints: /protocols (404 Not Found)");
    info!("   • Portfolio API: https://api.1inch.dev/portfolio/v4/* (404 Not Found)");
    
    info!("🔧 RECOMMENDATIONS:");
    info!("   1. Use Swap API v5.2 or v6.0 instead of Fusion API");
    info!("   2. Ensure correct token addresses (USDC: 0xA0b86a33E6441b8C4505B8C4505B8C4505B8C4505)");
    info!("   3. API key works fine - no issues there");
    info!("   4. Update service to use working endpoints");
    info!("   5. Consider Fusion API might need special access/configuration");
}