// examples/oneinch_fusion_access_analysis.rs
// Comprehensive analysis of 1inch Fusion API access options
// Based on documentation at https://portal.1inch.dev/documentation/apis/swap/fusion-plus/introduction

use reqwest::Client;
// use serde_json::Value;
use std::collections::HashMap;
use tracing::{error, info, warn};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();

    info!("🔍 1inch Fusion API Access Analysis");
    info!("📋 Comprehensive guide to obtaining Fusion API access");

    let api_key = std::env::var("ONEINCH_API_KEY").unwrap_or_else(|_| {
        "MrTcxGYJhqVtUaXOjD3A7fRKsfxg52pZ".to_string()
    });

    // Step 1: Analyze current API key capabilities
    analyze_current_access(&api_key).await?;

    // Step 2: Check developer portal possibilities
    check_portal_options().await;

    // Step 3: Explore alternative approaches
    explore_alternatives().await;

    // Step 4: Provide comprehensive recommendations
    provide_comprehensive_recommendations().await;

    Ok(())
}

async fn analyze_current_access(api_key: &str) -> Result<(), Box<dyn std::error::Error>> {
    info!("🔍 ANALYZING CURRENT API KEY CAPABILITIES:");
    
    let client = Client::new();
    
    // Test different API tiers to understand current access level
    let api_tests = vec![
        ("Basic Swap", "https://api.1inch.dev/swap/v5.2/1/tokens", "Standard tier"),
        ("Advanced Swap", "https://api.1inch.dev/swap/v6.0/1/tokens", "Standard tier"),
        ("Price Oracle", "https://api.1inch.dev/price/v1.1/1", "Standard tier"),
        ("Orderbook", "https://api.1inch.dev/orderbook/v3.0/1/events", "Standard tier"),
        ("Fusion Plus", "https://api.1inch.dev/fusion/v1.0/1/tokens", "Premium tier"),
        ("Trace API", "https://api.1inch.dev/trace/v1.0/chain/1", "Premium tier"),
    ];

    let mut access_summary = HashMap::new();
    
    for (name, url, tier) in api_tests {
        let response = client.get(url)
            .header("Authorization", format!("Bearer {}", api_key))
            .send()
            .await?;

        let status = response.status();
        let has_access = status.is_success();
        
        access_summary.insert(tier, access_summary.get(tier).unwrap_or(&0) + if has_access { 1 } else { 0 });
        
        info!("   {} ({}): {}", 
            name, 
            tier,
            if has_access { "✅ Access" } else { "❌ No Access" }
        );
    }

    info!("📊 ACCESS SUMMARY:");
    for (tier, count) in &access_summary {
        info!("   {}: {}/2 APIs accessible", tier, count);
    }

    // Determine account tier
    let standard_access = access_summary.get("Standard tier").unwrap_or(&0);
    let premium_access = access_summary.get("Premium tier").unwrap_or(&0);

    if *standard_access > 0 && *premium_access == 0 {
        warn!("🎯 ACCOUNT TIER: Standard (Free/Basic)");
        warn!("   💡 You have standard API access but lack premium features");
        warn!("   💡 Fusion API requires premium tier access");
    } else if *premium_access > 0 {
        info!("🎯 ACCOUNT TIER: Premium");
        info!("   ✅ You should have Fusion access - this might be a configuration issue");
    } else {
        error!("🎯 ACCOUNT TIER: Limited/Invalid");
        error!("   ❌ API key has very limited access");
    }

    Ok(())
}

async fn check_portal_options() {
    info!("🔍 DEVELOPER PORTAL ACCESS OPTIONS:");
    info!("");
    info!("📱 1. CHECK YOUR CURRENT PLAN:");
    info!("   🔗 Visit: https://portal.1inch.dev/");
    info!("   📋 Look for:");
    info!("      • Account/Subscription section");
    info!("      • API key management");
    info!("      • Plan details and limits");
    info!("      • Available APIs list");
    info!("");
    info!("💳 2. SUBSCRIPTION TIERS (Typical structure):");
    info!("   🆓 FREE TIER:");
    info!("      • Basic Swap API access");
    info!("      • Limited requests per month");
    info!("      • Standard support");
    info!("      • NO Fusion API access");
    info!("");
    info!("   💼 PROFESSIONAL TIER:");
    info!("      • All Swap APIs");
    info!("      • Higher rate limits");
    info!("      • Priority support");
    info!("      • POSSIBLE Fusion API access");
    info!("");
    info!("   🏢 ENTERPRISE TIER:");
    info!("      • All APIs including Fusion");
    info!("      • Custom rate limits");
    info!("      • Dedicated support");
    info!("      • Custom integrations");
    info!("");
    info!("🔧 3. SELF-SERVICE OPTIONS:");
    info!("   ✅ Things you can do yourself:");
    info!("      • Upgrade subscription tier");
    info!("      • Generate new API keys");
    info!("      • Configure API permissions");
    info!("      • Monitor usage and limits");
    info!("");
    info!("   ❌ Things requiring support:");
    info!("      • Special API access requests");
    info!("      • Custom rate limits");
    info!("      • Beta feature access");
    info!("      • Enterprise features");
}

async fn explore_alternatives() {
    info!("🔍 ALTERNATIVE APPROACHES:");
    info!("");
    info!("🎯 OPTION 1: UPGRADE SUBSCRIPTION");
    info!("   ✅ Pros:");
    info!("      • Immediate access once upgraded");
    info!("      • Official support");
    info!("      • All premium features");
    info!("      • Reliable service");
    info!("   ❌ Cons:");
    info!("      • Monthly/yearly cost");
    info!("      • May be overkill for testing");
    info!("   💰 Estimated cost: $50-500/month (varies by provider)");
    info!("");
    info!("🎯 OPTION 2: REQUEST TRIAL ACCESS");
    info!("   ✅ Pros:");
    info!("      • Free temporary access");
    info!("      • Full feature testing");
    info!("      • No long-term commitment");
    info!("   ❌ Cons:");
    info!("      • Limited time period");
    info!("      • Requires approval");
    info!("      • May have usage limits");
    info!("   ⏰ Timeline: 1-5 business days");
    info!("");
    info!("🎯 OPTION 3: DEVELOPER PROGRAM");
    info!("   ✅ Pros:");
    info!("      • Free access for qualified projects");
    info!("      • Long-term access");
    info!("      • Developer support");
    info!("   ❌ Cons:");
    info!("      • Strict qualification criteria");
    info!("      • Application process");
    info!("      • Usage restrictions");
    info!("   📋 Requirements: Open source project, educational use, or significant volume");
    info!("");
    info!("🎯 OPTION 4: PARTNERSHIP PROGRAM");
    info!("   ✅ Pros:");
    info!("      • Full API access");
    info!("      • Revenue sharing opportunities");
    info!("      • Marketing support");
    info!("   ❌ Cons:");
    info!("      • High barriers to entry");
    info!("      • Revenue/volume requirements");
    info!("      • Long approval process");
    info!("   📊 Requirements: Significant trading volume or user base");
    info!("");
    info!("🎯 OPTION 5: HYBRID APPROACH (RECOMMENDED)");
    info!("   ✅ Strategy:");
    info!("      • Use Swap API for immediate development");
    info!("      • Implement Fusion-ready architecture");
    info!("      • Add Fusion when access is available");
    info!("      • Provide feature toggle for users");
    info!("   ✅ Benefits:");
    info!("      • No development delays");
    info!("      • Future-proof architecture");
    info!("      • Gradual feature rollout");
    info!("      • Cost-effective development");
}

async fn provide_comprehensive_recommendations() {
    info!("🎯 COMPREHENSIVE RECOMMENDATIONS:");
    info!("");
    info!("📋 IMMEDIATE ACTIONS (Next 24 hours):");
    info!("   1. 🔍 Portal Investigation:");
    info!("      • Log into https://portal.1inch.dev/");
    info!("      • Check current subscription tier");
    info!("      • Look for upgrade options");
    info!("      • Review API key permissions");
    info!("");
    info!("   2. 📧 Support Contact:");
    info!("      • Email: support@1inch.io");
    info!("      • Subject: 'Fusion API Access Request - Bridge Development'");
    info!("      • Include: API key, project description, expected volume");
    info!("      • Request: Trial access or upgrade guidance");
    info!("");
    info!("   3. 🔧 Technical Preparation:");
    info!("      • Implement dual API architecture");
    info!("      • Create Fusion API client (ready to activate)");
    info!("      • Add configuration switches");
    info!("      • Test with Swap API as fallback");
    info!("");
    info!("📋 SHORT-TERM ACTIONS (1-2 weeks):");
    info!("   1. 💳 Subscription Decision:");
    info!("      • Evaluate upgrade costs vs. benefits");
    info!("      • Consider trial period if available");
    info!("      • Compare with alternative DEX aggregators");
    info!("");
    info!("   2. 🏗️ Architecture Implementation:");
    info!("      • Build configurable API layer");
    info!("      • Implement feature flags");
    info!("      • Create monitoring and fallback logic");
    info!("      • Add performance comparison tools");
    info!("");
    info!("📋 LONG-TERM STRATEGY (1-3 months):");
    info!("   1. 📊 Volume Analysis:");
    info!("      • Track API usage patterns");
    info!("      • Measure cost vs. benefit");
    info!("      • Evaluate user demand for Fusion features");
    info!("");
    info!("   2. 🤝 Partnership Exploration:");
    info!("      • Build user base and volume");
    info!("      • Document integration value");
    info!("      • Apply for partnership programs");
    info!("");
    info!("💡 DECISION MATRIX:");
    info!("");
    info!("   🚀 If you need Fusion IMMEDIATELY:");
    info!("      → Upgrade subscription (fastest path)");
    info!("");
    info!("   💰 If budget is a concern:");
    info!("      → Request trial access + hybrid approach");
    info!("");
    info!("   🔬 If this is for testing/development:");
    info!("      → Use Swap API + prepare Fusion integration");
    info!("");
    info!("   🏢 If building enterprise solution:");
    info!("      → Contact sales for enterprise tier");
    info!("");
    info!("🎯 RECOMMENDED NEXT STEP:");
    info!("   1. Check portal for self-service upgrade options");
    info!("   2. If no upgrade available, contact support immediately");
    info!("   3. Meanwhile, implement hybrid architecture");
    info!("   4. This ensures no development delays while pursuing access");
}

/// Helper function to test specific API endpoints with detailed analysis
async fn test_api_endpoint_details(
    client: &Client,
    api_key: &str,
    _name: &str,
    url: &str,
) -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
    let mut details = HashMap::new();
    
    let response = client.get(url)
        .header("Authorization", format!("Bearer {}", api_key))
        .send()
        .await?;

    let status = response.status();
    let headers = response.headers().clone();
    let body = response.text().await?;

    details.insert("status".to_string(), status.to_string());
    details.insert("body".to_string(), body.clone());

    // Extract useful headers
    if let Some(rate_limit) = headers.get("x-ratelimit-remaining") {
        details.insert("rate_limit_remaining".to_string(), 
            rate_limit.to_str().unwrap_or("unknown").to_string());
    }

    if let Some(rate_limit_reset) = headers.get("x-ratelimit-reset") {
        details.insert("rate_limit_reset".to_string(), 
            rate_limit_reset.to_str().unwrap_or("unknown").to_string());
    }

    // Analyze error messages for insights
    if !status.is_success() {
        if body.contains("No proxy config found") {
            details.insert("error_type".to_string(), "access_denied".to_string());
            details.insert("solution".to_string(), "upgrade_required".to_string());
        } else if body.contains("Unauthorized") {
            details.insert("error_type".to_string(), "auth_failed".to_string());
            details.insert("solution".to_string(), "check_api_key".to_string());
        } else if body.contains("rate limit") {
            details.insert("error_type".to_string(), "rate_limited".to_string());
            details.insert("solution".to_string(), "wait_or_upgrade".to_string());
        }
    }

    Ok(details)
}

/// Function to generate support email template
fn generate_support_email_template(api_key: &str) -> String {
    format!(r#"
Subject: Fusion API Access Request - Bridge/DEX Aggregation Service

Dear 1inch Support Team,

I am developing a cross-chain bridge service with DEX aggregation capabilities and would like to request access to the Fusion API.

Project Details:
- Service Type: Cross-chain bridge with DEX aggregation
- Use Case: Providing users with optimal swap routes across multiple chains
- Expected Volume: Starting with 100-1000 swaps/month, scaling to 10k+/month
- Integration: Honest API integration without fallbacks or mocks

Current Status:
- API Key: {}... (first 8 characters)
- Current Access: Swap API v5.2, v6.0, Price API (working)
- Missing Access: Fusion API (returns "No proxy config found")

Request:
I would like to request either:
1. Trial access to Fusion API for development and testing
2. Information about upgrading to a tier that includes Fusion API
3. Guidance on the application process for Fusion API access

Technical Implementation:
- Ready to implement Fusion API integration
- Have fallback to Swap API for reliability
- Committed to following API best practices and rate limits

Please let me know what information or steps are needed to proceed.

Thank you for your time and consideration.

Best regards,
[Your Name]
[Your Company/Project]
[Contact Information]
"#, &api_key[..8])
}

/// Function to create a checklist for portal investigation
fn create_portal_checklist() -> Vec<String> {
    vec![
        "✅ Log into https://portal.1inch.dev/".to_string(),
        "✅ Navigate to Account/Dashboard section".to_string(),
        "✅ Check current subscription/plan details".to_string(),
        "✅ Look for API key management section".to_string(),
        "✅ Review available APIs and permissions".to_string(),
        "✅ Check for upgrade/subscription options".to_string(),
        "✅ Look for Fusion API in available services".to_string(),
        "✅ Check rate limits and usage statistics".to_string(),
        "✅ Look for trial or developer program options".to_string(),
        "✅ Check billing/payment section for upgrade costs".to_string(),
        "✅ Look for support/contact options".to_string(),
        "✅ Check documentation for access requirements".to_string(),
    ]
}