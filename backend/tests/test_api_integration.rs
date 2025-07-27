use reqwest::Client;
use serde_json::{json, Value};
use std::env;
use base64::prelude::*;

/// KEMBridge API Integration Test
/// 
/// Tests both Ethereum RPC (MetaMask/Infura) and 1inch API connectivity
/// to ensure all external services are properly configured.
/// 
/// Usage: cargo run --bin test_api_integration

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 KEMBridge API Integration Test");
    println!("==================================\n");
    
    // Load environment variables from parent directory
    std::env::set_current_dir("..").ok();
    dotenvy::dotenv().ok();
    
    let client = Client::new();
    
    // Test 1: Ethereum RPC
    println!("🔗 Testing Ethereum RPC Connection...");
    test_ethereum_rpc(&client).await?;
    
    // Test 2: 1inch API
    println!("\n📱 Testing 1inch API Connection...");
    test_oneinch_api(&client).await?;
    
    println!("\n✅ All API integrations tested successfully!");
    println!("Ready for bridge operations! 🌉");
    
    Ok(())
}

async fn test_ethereum_rpc(client: &Client) -> Result<(), Box<dyn std::error::Error>> {
    let rpc_url = env::var("ETHEREUM_RPC_URL")
        .map_err(|_| "ETHEREUM_RPC_URL not found in .env file")?;
    
    let infura_secret = env::var("INFURA_API_SECRET")
        .map_err(|_| "INFURA_API_SECRET not found in .env file")?;
    
    println!("   🔍 RPC URL: {}", rpc_url);
    
    // Test basic connectivity with eth_blockNumber
    let payload = json!({
        "jsonrpc": "2.0",
        "method": "eth_blockNumber",
        "params": [],
        "id": 1
    });
    
    let response = client
        .post(&rpc_url)
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Basic {}", 
            BASE64_STANDARD.encode(format!(":{}", infura_secret))))
        .json(&payload)
        .send()
        .await?;
    
    if response.status().is_success() {
        let body: Value = response.json().await?;
        if let Some(result) = body.get("result") {
            let block_hex = result.as_str().unwrap_or("0x0");
            let block_number = u64::from_str_radix(&block_hex[2..], 16).unwrap_or(0);
            println!("   ✅ Connected! Latest block: {} ({})", block_hex, block_number);
            
            // Verify it's Sepolia testnet
            let net_payload = json!({
                "jsonrpc": "2.0",
                "method": "net_version",
                "params": [],
                "id": 2
            });
            
            let net_response = client
                .post(&rpc_url)
                .header("Content-Type", "application/json")
                .header("Authorization", format!("Basic {}", 
                    BASE64_STANDARD.encode(format!(":{}", infura_secret))))
                .json(&net_payload)
                .send()
                .await?;
            
            if let Ok(net_body) = net_response.json::<Value>().await {
                if let Some(net_result) = net_body.get("result") {
                    let network_id = net_result.as_str().unwrap_or("unknown");
                    if network_id == "11155111" {
                        println!("   ✅ Network: Sepolia testnet ✓");
                    } else {
                        println!("   ⚠️  Network: {} (expected Sepolia: 11155111)", network_id);
                    }
                }
            }
        } else {
            return Err("Invalid RPC response format".into());
        }
    } else {
        let error_text = response.text().await?;
        return Err(format!("RPC request failed: {}", error_text).into());
    }
    
    Ok(())
}

async fn test_oneinch_api(client: &Client) -> Result<(), Box<dyn std::error::Error>> {
    let api_key = env::var("ONEINCH_API_KEY")
        .map_err(|_| "ONEINCH_API_KEY not found in .env file")?;
    
    println!("   🔑 API Key: {}...", &api_key[..8]);
    
    // Test 1: Get supported protocols on Ethereum mainnet
    let protocols_url = "https://api.1inch.dev/swap/v6.0/1/liquidity-sources";
    
    let response = client
        .get(protocols_url)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("accept", "application/json")
        .send()
        .await?;
    
    if response.status().is_success() {
        let body: Value = response.json().await?;
        if let Some(protocols) = body.get("protocols") {
            if let Some(protocols_array) = protocols.as_array() {
                println!("   ✅ Found {} liquidity sources on Ethereum", protocols_array.len());
                
                // Show a few examples
                let examples: Vec<&str> = protocols_array
                    .iter()
                    .take(3)
                    .filter_map(|p| p.get("title")?.as_str())
                    .collect();
                
                if !examples.is_empty() {
                    println!("   📋 Examples: {}", examples.join(", "));
                }
            }
        }
    } else if response.status().as_u16() == 401 {
        return Err("1inch API authentication failed - check API key".into());
    } else if response.status().as_u16() == 403 {
        return Err("1inch API access forbidden - check API key permissions".into());
    } else {
        let status = response.status();
        let error_text = response.text().await?;
        return Err(format!("1inch API request failed: {} - {}", status, error_text).into());
    }
    
    // Test 2: Verify API key with a simple endpoint
    let token_url = "https://api.1inch.dev/token/v1.2/1/custom/";
    
    let token_response = client
        .get(token_url)
        .header("Authorization", format!("Bearer {}", api_key))
        .send()
        .await?;
    
    if token_response.status().is_success() {
        println!("   ✅ API key validation successful");
    } else if token_response.status().as_u16() == 400 {
        // This is expected for the custom tokens endpoint without parameters
        println!("   ✅ API key is valid (400 response is expected for this endpoint)");
    } else {
        println!("   ⚠️  API key validation returned: {}", token_response.status());
    }
    
    Ok(())
}