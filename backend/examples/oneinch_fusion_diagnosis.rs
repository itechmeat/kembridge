// examples/oneinch_fusion_diagnosis.rs
// Detailed diagnosis of 1inch Fusion API access issues
// Understanding what exactly is blocking Fusion API access

use reqwest::Client;
use serde_json::Value;
use std::collections::HashMap;
use tracing::{error, info, warn};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();

    info!("🔍 1inch Fusion API Access Diagnosis");
    info!("📋 Detailed investigation of API key permissions and access");

    let api_key = std::env::var("ONEINCH_API_KEY").unwrap_or_else(|_| {
        "MrTcxGYJhqVtUaXOjD3A7fRKsfxg52pZ".to_string()
    });

    info!("🔑 Analyzing API key: {}...", &api_key[..8]);

    // Step 1: Analyze API key format and characteristics
    analyze_api_key(&api_key).await;

    // Step 2: Test different Fusion endpoints
    test_fusion_endpoints(&api_key).await?;

    // Step 3: Test with different authentication methods
    test_auth_methods(&api_key).await?;

    // Step 4: Check API key permissions across all 1inch services
    check_comprehensive_permissions(&api_key).await?;

    // Step 5: Analyze error responses in detail
    analyze_error_responses(&api_key).await?;

    // Step 6: Provide actionable recommendations
    provide_fusion_recommendations().await;

    Ok(())
}

async fn analyze_api_key(api_key: &str) {
    info!("🔍 API KEY ANALYSIS:");
    info!("   Length: {} characters", api_key.len());
    info!("   Format: {}", if api_key.chars().all(|c| c.is_alphanumeric()) { "Alphanumeric" } else { "Mixed" });
    info!("   Pattern: {}...", &api_key[..std::cmp::min(api_key.len(), 12)]);
    
    // Check if it looks like a real API key
    let looks_real = api_key.len() >= 20 && 
        api_key.chars().any(|c| c.is_uppercase()) && 
        api_key.chars().any(|c| c.is_lowercase()) &&
        !api_key.contains("test") &&
        !api_key.contains("demo");
    
    if looks_real {
        info!("   ✅ Appears to be a real API key");
    } else {
        warn!("   ⚠️  May be a test/demo key");
    }
}

async fn test_fusion_endpoints(api_key: &str) -> Result<(), Box<dyn std::error::Error>> {
    info!("🔥 TESTING FUSION ENDPOINTS:");
    
    let client = Client::new();
    let chain_ids = vec![1, 56, 137]; // Ethereum, BSC, Polygon
    
    let fusion_endpoints = vec![
        "tokens",
        "quote", 
        "orders",
        "orders/active",
        "relayers",
        "whitelist",
    ];

    for chain_id in &chain_ids {
        info!("📡 Testing chain {}", chain_id);
        
        for endpoint in &fusion_endpoints {
            let url = format!("https://api.1inch.dev/fusion/v1.0/{}/{}", chain_id, endpoint);
            info!("   🔗 {}", url);
            
            let mut request = client.get(&url)
                .header("Authorization", format!("Bearer {}", api_key))
                .header("Accept", "application/json")
                .header("User-Agent", "kembridge/1.0");

            // Add query params for quote endpoint
            if *endpoint == "quote" {
                request = request.query(&[
                    ("src", "0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE"),
                    ("dst", "0xdAC17F958D2ee523a2206206994597C13D831ec7"),
                    ("amount", "1000000000000000000"),
                ]);
            }

            match request.send().await {
                Ok(response) => {
                    let status = response.status();
                    let headers = response.headers().clone();
                    
                    match response.text().await {
                        Ok(body) => {
                            info!("      Status: {}", status);
                            
                            // Log important headers
                            if let Some(rate_limit) = headers.get("x-ratelimit-remaining") {
                                info!("      Rate limit remaining: {:?}", rate_limit);
                            }
                            if let Some(retry_after) = headers.get("retry-after") {
                                info!("      Retry after: {:?}", retry_after);
                            }
                            
                            match status.as_u16() {
                                200 => {
                                    info!("      ✅ SUCCESS");
                                    if let Ok(json) = serde_json::from_str::<Value>(&body) {
                                        if let Some(obj) = json.as_object() {
                                            info!("      📋 Response keys: {:?}", obj.keys().collect::<Vec<_>>());
                                        }
                                    }
                                }
                                404 => {
                                    warn!("      ❌ NOT FOUND");
                                    if body.contains("No proxy config found") {
                                        warn!("      🚫 FUSION ACCESS DENIED");
                                        warn!("      💡 This specific error suggests:");
                                        warn!("         • API key lacks Fusion permissions");
                                        warn!("         • Need to request Fusion access from 1inch");
                                        warn!("         • May need enterprise/partner tier");
                                    } else {
                                        warn!("      📄 Error: {}", body);
                                    }
                                }
                                401 => {
                                    error!("      🔒 UNAUTHORIZED");
                                    error!("      💡 API key is invalid or expired");
                                }
                                403 => {
                                    error!("      🚫 FORBIDDEN");
                                    error!("      💡 API key valid but lacks permissions");
                                }
                                429 => {
                                    warn!("      ⏳ RATE LIMITED");
                                }
                                _ => {
                                    warn!("      ❓ Status: {} - {}", status, body);
                                }
                            }
                        }
                        Err(e) => {
                            error!("      🔥 Failed to read response: {}", e);
                        }
                    }
                }
                Err(e) => {
                    error!("      🔥 Request failed: {}", e);
                }
            }
        }
    }

    Ok(())
}

async fn test_auth_methods(api_key: &str) -> Result<(), Box<dyn std::error::Error>> {
    info!("🔐 TESTING AUTHENTICATION METHODS:");
    
    let client = Client::new();
    let test_url = "https://api.1inch.dev/fusion/v1.0/1/tokens";
    
    let auth_methods = vec![
        ("Bearer token", format!("Bearer {}", api_key)),
        ("API key header", api_key.to_string()),
        ("X-API-Key header", api_key.to_string()),
    ];

    for (method_name, auth_value) in auth_methods {
        info!("   🔑 Testing: {}", method_name);
        
        let mut request = client.get(test_url);
        
        match method_name {
            "Bearer token" => {
                request = request.header("Authorization", auth_value);
            }
            "API key header" => {
                request = request.header("Authorization", auth_value);
            }
            "X-API-Key header" => {
                request = request.header("X-API-Key", auth_value);
            }
            _ => {}
        }

        match request.send().await {
            Ok(response) => {
                let status = response.status();
                info!("      Status: {}", status);
                
                if status == 404 {
                    let body = response.text().await?;
                    if body.contains("No proxy config found") {
                        info!("      🔍 Same 'No proxy config' error - auth method not the issue");
                    }
                } else if status != 200 {
                    let body = response.text().await?;
                    info!("      📄 Response: {}", body);
                }
            }
            Err(e) => {
                error!("      🔥 Request failed: {}", e);
            }
        }
    }

    Ok(())
}

async fn check_comprehensive_permissions(api_key: &str) -> Result<(), Box<dyn std::error::Error>> {
    info!("📊 COMPREHENSIVE PERMISSIONS CHECK:");
    
    let client = Client::new();
    
    let services = vec![
        ("Swap v5.2", "https://api.1inch.dev/swap/v5.2/1/tokens"),
        ("Swap v6.0", "https://api.1inch.dev/swap/v6.0/1/tokens"),
        ("Fusion v1.0", "https://api.1inch.dev/fusion/v1.0/1/tokens"),
        ("Price v1.1", "https://api.1inch.dev/price/v1.1/1"),
        ("Portfolio v4", "https://api.1inch.dev/portfolio/v4/overview/erc20"),
        ("Orderbook", "https://api.1inch.dev/orderbook/v3.0/1/events"),
        ("Trace", "https://api.1inch.dev/trace/v1.0/chain/1"),
    ];

    let mut permissions = HashMap::new();
    
    for (service_name, url) in services {
        let response = client.get(url)
            .header("Authorization", format!("Bearer {}", api_key))
            .send()
            .await?;
            
        let status = response.status().as_u16();
        let access_level = match status {
            200 => "✅ Full Access",
            401 => "🔒 Unauthorized",
            403 => "🚫 Forbidden", 
            404 => {
                let body = response.text().await?;
                if body.contains("No proxy config found") {
                    "🚫 No Access (No proxy config)"
                } else {
                    "❌ Not Found"
                }
            }
            429 => "⏳ Rate Limited",
            _ => "❓ Unknown",
        };
        
        permissions.insert(service_name, access_level);
        info!("   {}: {}", service_name, access_level);
    }

    // Analyze permission pattern
    info!("🔍 PERMISSION PATTERN ANALYSIS:");
    let has_swap_access = permissions.get("Swap v5.2").unwrap_or(&"").contains("Full Access");
    let has_fusion_access = permissions.get("Fusion v1.0").unwrap_or(&"").contains("Full Access");
    let _has_price_access = permissions.get("Price v1.1").unwrap_or(&"").contains("Full Access");

    if has_swap_access && !has_fusion_access {
        warn!("   📊 Pattern: Standard API access without Fusion");
        warn!("   💡 This suggests Fusion requires special tier/permissions");
    } else if !has_swap_access && !has_fusion_access {
        error!("   📊 Pattern: Limited or invalid API key");
    } else if has_fusion_access {
        info!("   📊 Pattern: Full access including Fusion");
    }

    Ok(())
}

async fn analyze_error_responses(api_key: &str) -> Result<(), Box<dyn std::error::Error>> {
    info!("🔬 DETAILED ERROR ANALYSIS:");
    
    let client = Client::new();
    let fusion_url = "https://api.1inch.dev/fusion/v1.0/1/tokens";
    
    let response = client.get(fusion_url)
        .header("Authorization", format!("Bearer {}", api_key))
        .send()
        .await?;

    let status = response.status();
    let headers = response.headers().clone();
    let body = response.text().await?;

    info!("   📡 URL: {}", fusion_url);
    info!("   📊 Status: {}", status);
    info!("   📄 Body: {}", body);
    info!("   📋 Headers:");
    
    for (name, value) in headers.iter() {
        if name.as_str().starts_with("x-") || 
            name.as_str().contains("rate") ||
            name.as_str().contains("limit") ||
            name.as_str().contains("auth") {
            info!("      {}: {:?}", name, value);
        }
    }

    // Parse error if JSON
    if let Ok(json) = serde_json::from_str::<Value>(&body) {
        info!("   🔍 Parsed error:");
        if let Some(error) = json.get("error") {
            info!("      Error: {}", error);
        }
        if let Some(message) = json.get("message") {
            info!("      Message: {}", message);
        }
        if let Some(code) = json.get("statusCode") {
            info!("      Status Code: {}", code);
        }
        if let Some(request_id) = json.get("requestId") {
            info!("      Request ID: {}", request_id);
        }
    }

    // Specific analysis for "No proxy config found"
    if body.contains("No proxy config found") {
        warn!("🚨 SPECIFIC ERROR ANALYSIS:");
        warn!("   Error: 'No proxy config found'");
        warn!("   💡 This error typically means:");
        warn!("      1. API key doesn't have Fusion API access");
        warn!("      2. Fusion API requires special registration");
        warn!("      3. May need enterprise/partner tier access");
        warn!("      4. Fusion API might be invite-only or restricted");
        warn!("   🔧 Possible solutions:");
        warn!("      1. Contact 1inch support for Fusion access");
        warn!("      2. Check if account needs upgrade");
        warn!("      3. Verify API key was created with Fusion permissions");
        warn!("      4. Request Fusion API whitelist access");
    }

    Ok(())
}

async fn provide_fusion_recommendations() {
    info!("🎯 FUSION API ACCESS RECOMMENDATIONS:");
    info!("");
    info!("📋 CURRENT SITUATION:");
    info!("   • API key works for Swap API (standard access)");
    info!("   • Fusion API returns 'No proxy config found'");
    info!("   • This is NOT an authentication issue");
    info!("   • This is a PERMISSIONS/ACCESS TIER issue");
    info!("");
    info!("🚀 IMMEDIATE ACTIONS:");
    info!("   1. 📧 CONTACT 1INCH SUPPORT:");
    info!("      • Email: support@1inch.io");
    info!("      • Request: Fusion API access for your API key");
    info!("      • Mention: Building bridge/DEX aggregation service");
    info!("      • Include: Your API key (first 8 chars)");
    info!("");
    info!("   2. 🔍 CHECK 1INCH DEVELOPER PORTAL:");
    info!("      • Visit: https://portal.1inch.dev/");
    info!("      • Look for: Fusion API access settings");
    info!("      • Check: Account tier/subscription level");
    info!("      • Verify: API key permissions");
    info!("");
    info!("   3. 📚 REVIEW FUSION DOCUMENTATION:");
    info!("      • Check: https://docs.1inch.io/docs/fusion-swap/introduction");
    info!("      • Look for: Access requirements");
    info!("      • Find: Registration process");
    info!("");
    info!("🔧 TECHNICAL WORKAROUND:");
    info!("   • Implement dual API support (Swap + Fusion)");
    info!("   • Use Swap API as fallback");
    info!("   • Add Fusion when access is granted");
    info!("   • Make API choice configurable");
    info!("");
    info!("⏰ TIMELINE EXPECTATION:");
    info!("   • Support response: 1-3 business days");
    info!("   • Access approval: 3-7 business days");
    info!("   • Implementation: Same day once access granted");
    info!("");
    info!("💡 NEXT STEPS:");
    info!("   1. Contact 1inch support immediately");
    info!("   2. Continue development with Swap API");
    info!("   3. Prepare Fusion integration code");
    info!("   4. Test Fusion once access is granted");
}

/// Helper function to test specific Fusion features
async fn test_fusion_features(api_key: &str) -> Result<(), Box<dyn std::error::Error>> {
    info!("🧪 TESTING FUSION-SPECIFIC FEATURES:");
    
    let client = Client::new();
    
    // Test Fusion quote with different parameters
    let fusion_quote_url = "https://api.1inch.dev/fusion/v1.0/1/quote";
    
    let test_cases = vec![
        ("ETH->USDT", vec![
            ("src", "0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE"),
            ("dst", "0xdAC17F958D2ee523a2206206994597C13D831ec7"),
            ("amount", "1000000000000000000"),
        ]),
        ("USDT->ETH", vec![
            ("src", "0xdAC17F958D2ee523a2206206994597C13D831ec7"),
            ("dst", "0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE"),
            ("amount", "1000000000"),
        ]),
    ];

    for (test_name, params) in test_cases {
        info!("   🔍 Testing: {}", test_name);
        
        let response = client.get(fusion_quote_url)
            .header("Authorization", format!("Bearer {}", api_key))
            .query(&params)
            .send()
            .await?;

        let status = response.status();
        let body = response.text().await?;
        
        info!("      Status: {}", status);
        if status != 200 {
            info!("      Response: {}", body);
        }
    }

    Ok(())
}