// examples/oneinch_api_endpoints_test.rs
// Test different 1inch API endpoints to find working ones
// This helps diagnose API access issues and find correct endpoints

use reqwest::Client;
use serde_json::Value;
use std::collections::HashMap;
use tracing::{error, info, warn};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();

    info!("🔍 Testing different 1inch API endpoints");

    let api_key = std::env::var("ONEINCH_API_KEY").unwrap_or_else(|_| {
        warn!("⚠️  ONEINCH_API_KEY not found, using test key");
        "MrTcxGYJhqVtUaXOjD3A7fRKsfxg52pZ".to_string()
    });

    let client = Client::new();
    let chain_id = 1; // Ethereum mainnet

    // Test different API endpoints
    let endpoints_to_test = vec![
        // Swap API v5.2 (older, more stable)
        ("Swap API v5.2 - Tokens", format!("https://api.1inch.dev/swap/v5.2/{}/tokens", chain_id)),
        ("Swap API v5.2 - Protocols", format!("https://api.1inch.dev/swap/v5.2/{}/protocols", chain_id)),
        ("Swap API v5.2 - Quote", format!("https://api.1inch.dev/swap/v5.2/{}/quote", chain_id)),
        
        // Swap API v6.0 (newer)
        ("Swap API v6.0 - Tokens", format!("https://api.1inch.dev/swap/v6.0/{}/tokens", chain_id)),
        ("Swap API v6.0 - Protocols", format!("https://api.1inch.dev/swap/v6.0/{}/protocols", chain_id)),
        
        // Fusion API (the one that was failing)
        ("Fusion API - Tokens", format!("https://api.1inch.dev/fusion/v1.0/{}/tokens", chain_id)),
        ("Fusion API - Quote", format!("https://api.1inch.dev/fusion/v1.0/{}/quote", chain_id)),
        
        // Portfolio API
        ("Portfolio API - Overview", format!("https://api.1inch.dev/portfolio/v4/overview/erc20")),
        
        // Price API
        ("Price API - Current", format!("https://api.1inch.dev/price/v1.1/{}", chain_id)),
    ];

    info!("🧪 Testing {} endpoints with API key: {}...", 
        endpoints_to_test.len(), 
        &api_key[..8]);

    for (name, url) in endpoints_to_test {
        info!("\n📡 Testing: {}", name);
        info!("🔗 URL: {}", url);

        let mut request = client.get(&url)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Accept", "application/json");

        // Add query parameters for quote endpoints
        if url.contains("/quote") {
            request = request
                .query(&[
                    ("src", "0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE"), // ETH
                    ("dst", "0xA0b86a33E6441b8C4505B8C4505B8C4505B8C4505"), // USDC
                    ("amount", "1000000000000000000"), // 1 ETH
                ]);
        }

        match request.send().await {
            Ok(response) => {
                let status = response.status();
                info!("📊 Status: {}", status);

                match response.text().await {
                    Ok(body) => {
                        if status.is_success() {
                            info!("✅ SUCCESS");
                            
                            // Try to parse as JSON and show some info
                            if let Ok(json) = serde_json::from_str::<Value>(&body) {
                                if let Some(obj) = json.as_object() {
                                    info!("📋 Response keys: {:?}", obj.keys().collect::<Vec<_>>());
                                    
                                    // Show specific info based on endpoint type
                                    if name.contains("Tokens") {
                                        if let Some(tokens) = json.as_object() {
                                            info!("🪙 Found {} tokens", tokens.len());
                                            // Show first few tokens
                                            for (addr, token) in tokens.iter().take(3) {
                                                if let (Some(symbol), Some(name)) = 
                                                    (token["symbol"].as_str(), token["name"].as_str()) {
                                                    info!("   • {} ({}): {}", symbol, name, &addr[..10]);
                                                }
                                            }
                                        }
                                    } else if name.contains("Protocols") {
                                        if let Some(protocols) = json["protocols"].as_array() {
                                            info!("🔄 Found {} protocols", protocols.len());
                                            for protocol in protocols.iter().take(5) {
                                                if let Some(name) = protocol["title"].as_str() {
                                                    info!("   • {}", name);
                                                }
                                            }
                                        }
                                    } else if name.contains("Quote") {
                                        if let Some(to_amount) = json["toAmount"].as_str() {
                                            info!("💱 Quote amount: {}", to_amount);
                                        }
                                        if let Some(gas) = json["estimatedGas"].as_str() {
                                            info!("⛽ Estimated gas: {}", gas);
                                        }
                                    }
                                }
                            } else {
                                info!("📄 Response (first 200 chars): {}", 
                                    body.chars().take(200).collect::<String>());
                            }
                        } else {
                            warn!("❌ FAILED");
                            error!("📄 Error response: {}", body);
                            
                            // Parse error details if possible
                            if let Ok(json) = serde_json::from_str::<Value>(&body) {
                                if let Some(error) = json["error"].as_str() {
                                    error!("🔥 Error: {}", error);
                                }
                                if let Some(description) = json["description"].as_str() {
                                    error!("📝 Description: {}", description);
                                }
                                if let Some(status_code) = json["statusCode"].as_u64() {
                                    error!("🔢 Status Code: {}", status_code);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        error!("❌ Failed to read response body: {}", e);
                    }
                }
            }
            Err(e) => {
                error!("❌ Request failed: {}", e);
            }
        }
    }

    // Test API key validation specifically
    info!("\n🔑 Testing API key validation");
    test_api_key_validation(&client, &api_key).await?;

    // Test rate limiting
    info!("\n⏱️  Testing rate limiting");
    test_rate_limiting(&client, &api_key).await?;

    info!("\n🎉 API endpoint testing completed!");
    info!("💡 Use the working endpoints in your integration");

    Ok(())
}

async fn test_api_key_validation(
    client: &Client, 
    api_key: &str
) -> Result<(), Box<dyn std::error::Error>> {
    info!("🔍 Testing API key validation methods");

    // Method 1: Try a simple tokens endpoint
    let url = "https://api.1inch.dev/swap/v5.2/1/tokens";
    
    info!("📡 Testing with API key...");
    let response = client.get(url)
        .header("Authorization", format!("Bearer {}", api_key))
        .send()
        .await?;

    if response.status().is_success() {
        info!("✅ API key works with Swap API v5.2");
    } else {
        warn!("❌ API key failed with Swap API v5.2: {}", response.status());
    }

    // Method 2: Try without API key to see difference
    info!("📡 Testing without API key...");
    let response_no_key = client.get(url).send().await?;
    
    if response_no_key.status().is_success() {
        info!("ℹ️  Endpoint works without API key (public endpoint)");
    } else {
        info!("🔒 Endpoint requires API key: {}", response_no_key.status());
    }

    Ok(())
}

async fn test_rate_limiting(
    client: &Client, 
    api_key: &str
) -> Result<(), Box<dyn std::error::Error>> {
    info!("🔍 Testing rate limiting behavior");

    let url = "https://api.1inch.dev/swap/v5.2/1/tokens";
    let mut success_count = 0;
    let mut rate_limited_count = 0;

    // Make several rapid requests
    for i in 1..=10 {
        let start = std::time::Instant::now();
        
        let response = client.get(url)
            .header("Authorization", format!("Bearer {}", api_key))
            .send()
            .await?;

        let duration = start.elapsed();
        
        match response.status().as_u16() {
            200 => {
                success_count += 1;
                info!("✅ Request {}: Success ({}ms)", i, duration.as_millis());
            }
            429 => {
                rate_limited_count += 1;
                warn!("⏳ Request {}: Rate limited ({}ms)", i, duration.as_millis());
                
                // Check rate limit headers
                if let Some(retry_after) = response.headers().get("retry-after") {
                    if let Ok(retry_str) = retry_after.to_str() {
                        warn!("   Retry after: {} seconds", retry_str);
                    }
                }
            }
            status => {
                warn!("❓ Request {}: Status {} ({}ms)", i, status, duration.as_millis());
            }
        }

        // Small delay between requests
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }

    info!("📊 Rate limiting test results:");
    info!("   ✅ Successful requests: {}", success_count);
    info!("   ⏳ Rate limited requests: {}", rate_limited_count);
    
    if rate_limited_count > 0 {
        warn!("💡 Consider implementing request throttling");
    } else {
        info!("✅ No rate limiting encountered");
    }

    Ok(())
}