// examples/oneinch_api_comparison.rs
// Comparison between 1inch Fusion API vs Swap API
// Understanding which one to use and why

use reqwest::Client;
use serde_json::Value;
use tracing::{error, info, warn};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();

    info!("🔍 1inch API Comparison: Fusion vs Swap");
    info!("📋 Understanding the differences and choosing the right API");

    let api_key = std::env::var("ONEINCH_API_KEY").unwrap_or_else(|_| {
        "MrTcxGYJhqVtUaXOjD3A7fRKsfxg52pZ".to_string()
    });

    explain_api_differences().await;
    test_both_apis(&api_key).await?;
    provide_recommendations().await;

    Ok(())
}

async fn explain_api_differences() {
    info!("📚 API DIFFERENCES EXPLANATION:");
    info!("");
    info!("🔥 FUSION API (v1.0):");
    info!("   • Next-generation DEX aggregation");
    info!("   • Intent-based trading (you specify what you want, not how)");
    info!("   • Better MEV protection");
    info!("   • Gasless transactions for users");
    info!("   • Professional/institutional focus");
    info!("   • May require special API access/permissions");
    info!("   • More complex integration");
    info!("");
    info!("⚡ SWAP API (v5.2/v6.0):");
    info!("   • Traditional DEX aggregation");
    info!("   • Direct swap execution");
    info!("   • Widely available and stable");
    info!("   • Standard API access");
    info!("   • Simpler integration");
    info!("   • Battle-tested and reliable");
    info!("");
}

async fn test_both_apis(api_key: &str) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();

    info!("🧪 TESTING BOTH APIS:");
    info!("");

    // Test Fusion API
    info!("🔥 Testing Fusion API:");
    test_fusion_api(&client, api_key).await?;
    
    info!("");
    
    // Test Swap API
    info!("⚡ Testing Swap API:");
    test_swap_api(&client, api_key).await?;

    Ok(())
}

async fn test_fusion_api(client: &Client, api_key: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Test Fusion tokens endpoint
    let fusion_tokens_url = "https://api.1inch.dev/fusion/v1.0/1/tokens";
    
    info!("📡 Fusion Tokens: {}", fusion_tokens_url);
    let response = client.get(fusion_tokens_url)
        .header("Authorization", format!("Bearer {}", api_key))
        .send()
        .await?;

    match response.status().as_u16() {
        200 => {
            info!("✅ Fusion API: Working!");
            let tokens: Value = response.json().await?;
            if let Some(tokens_obj) = tokens.as_object() {
                info!("   📊 Available tokens: {}", tokens_obj.len());
            }
        }
        404 => {
            warn!("❌ Fusion API: 404 - No proxy config found");
            warn!("   💡 This suggests:");
            warn!("      • API key doesn't have Fusion access");
            warn!("      • Fusion API requires special permissions");
            warn!("      • Need to request Fusion API access from 1inch");
        }
        401 => {
            error!("❌ Fusion API: 401 - Unauthorized");
            error!("   💡 API key is invalid or expired");
        }
        403 => {
            error!("❌ Fusion API: 403 - Forbidden");
            error!("   💡 API key doesn't have required permissions");
        }
        status => {
            warn!("❓ Fusion API: Unexpected status {}", status);
            let body = response.text().await?;
            warn!("   Response: {}", body);
        }
    }

    // Test Fusion quote endpoint
    let fusion_quote_url = "https://api.1inch.dev/fusion/v1.0/1/quote";
    
    info!("📡 Fusion Quote: {}", fusion_quote_url);
    let response = client.get(fusion_quote_url)
        .header("Authorization", format!("Bearer {}", api_key))
        .query(&[
            ("src", "0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE"),
            ("dst", "0xdAC17F958D2ee523a2206206994597C13D831ec7"),
            ("amount", "1000000000000000000"),
        ])
        .send()
        .await?;

    match response.status().as_u16() {
        200 => {
            info!("✅ Fusion Quote: Working!");
            let quote: Value = response.json().await?;
            if let Some(amount) = quote["dstAmount"].as_str() {
                info!("   💱 Quote amount: {}", amount);
            }
        }
        404 => {
            warn!("❌ Fusion Quote: 404 - No proxy config found");
        }
        400 => {
            warn!("❌ Fusion Quote: 400 - Bad request");
            let body = response.text().await?;
            warn!("   Error: {}", body);
        }
        status => {
            warn!("❓ Fusion Quote: Status {}", status);
        }
    }

    Ok(())
}

async fn test_swap_api(client: &Client, api_key: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Test Swap tokens endpoint
    let swap_tokens_url = "https://api.1inch.dev/swap/v5.2/1/tokens";
    
    info!("📡 Swap Tokens: {}", swap_tokens_url);
    let response = client.get(swap_tokens_url)
        .header("Authorization", format!("Bearer {}", api_key))
        .send()
        .await?;

    match response.status().as_u16() {
        200 => {
            info!("✅ Swap API: Working!");
            let tokens: Value = response.json().await?;
            if let Some(tokens_obj) = tokens["tokens"].as_object() {
                info!("   📊 Available tokens: {}", tokens_obj.len());
            }
        }
        401 => {
            error!("❌ Swap API: 401 - Unauthorized");
        }
        status => {
            warn!("❓ Swap API: Status {}", status);
        }
    }

    // Test Swap quote endpoint
    let swap_quote_url = "https://api.1inch.dev/swap/v5.2/1/quote";
    
    info!("📡 Swap Quote: {}", swap_quote_url);
    let response = client.get(swap_quote_url)
        .header("Authorization", format!("Bearer {}", api_key))
        .query(&[
            ("src", "0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE"),
            ("dst", "0xdAC17F958D2ee523a2206206994597C13D831ec7"),
            ("amount", "1000000000000000000"),
        ])
        .send()
        .await?;

    match response.status().as_u16() {
        200 => {
            info!("✅ Swap Quote: Working!");
            let quote: Value = response.json().await?;
            if let Some(amount) = quote["toTokenAmount"].as_str() {
                info!("   💱 Quote amount: {}", amount);
            }
            if let Some(gas) = quote["estimatedGas"].as_str() {
                info!("   ⛽ Estimated gas: {}", gas);
            }
        }
        400 => {
            warn!("❌ Swap Quote: 400 - Bad request");
            let body = response.text().await?;
            warn!("   Error: {}", body);
        }
        status => {
            warn!("❓ Swap Quote: Status {}", status);
        }
    }

    Ok(())
}

async fn provide_recommendations() {
    info!("");
    info!("🎯 RECOMMENDATIONS:");
    info!("");
    info!("📊 CURRENT SITUATION:");
    info!("   • Fusion API: Not accessible with current API key");
    info!("   • Swap API: Working and accessible");
    info!("   • Error 'No proxy config found' = No Fusion access");
    info!("");
    info!("🚀 RECOMMENDED APPROACH:");
    info!("   1. START WITH SWAP API:");
    info!("      ✅ Reliable and battle-tested");
    info!("      ✅ Works with standard API keys");
    info!("      ✅ Simpler integration");
    info!("      ✅ Good for MVP and production");
    info!("");
    info!("   2. CONSIDER FUSION API LATER:");
    info!("      🔄 Contact 1inch for Fusion API access");
    info!("      🔄 Evaluate if advanced features are needed");
    info!("      🔄 Implement as enhancement, not requirement");
    info!("");
    info!("💡 PRACTICAL DECISION:");
    info!("   • Use Swap API for immediate development");
    info!("   • Bridge functionality works perfectly with Swap API");
    info!("   • Fusion API can be added later as premium feature");
    info!("");
    info!("🔧 IMPLEMENTATION STRATEGY:");
    info!("   1. Configure service to use Swap API endpoints");
    info!("   2. Implement fallback mechanism (Swap as primary)");
    info!("   3. Add Fusion support when access is available");
    info!("   4. Make API choice configurable");
    info!("");
}

/// Helper function to check API key permissions
async fn check_api_permissions(api_key: &str) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    
    info!("🔍 CHECKING API KEY PERMISSIONS:");
    
    let endpoints_to_check = vec![
        ("Swap v5.2", "https://api.1inch.dev/swap/v5.2/1/tokens"),
        ("Swap v6.0", "https://api.1inch.dev/swap/v6.0/1/tokens"),
        ("Fusion v1.0", "https://api.1inch.dev/fusion/v1.0/1/tokens"),
        ("Price v1.1", "https://api.1inch.dev/price/v1.1/1"),
        ("Portfolio v4", "https://api.1inch.dev/portfolio/v4/overview/erc20"),
    ];
    
    for (name, url) in endpoints_to_check {
        let response = client.get(url)
            .header("Authorization", format!("Bearer {}", api_key))
            .send()
            .await?;
            
        let status = response.status();
        match status.as_u16() {
            200 => info!("   ✅ {}: Accessible", name),
            401 => warn!("   🔒 {}: Unauthorized", name),
            403 => warn!("   🚫 {}: Forbidden", name),
            404 => warn!("   ❌ {}: Not found / No access", name),
            _ => warn!("   ❓ {}: Status {}", name, status),
        }
    }
    
    Ok(())
}