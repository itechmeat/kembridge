// HTTP integration test for authentication system
use std::env;
use reqwest::Client;
use serde_json::json;
use tokio::time::{sleep, Duration};

const SERVER_URL: &str = "http://localhost:4000";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🌐 Testing KEMBridge Authentication HTTP Integration");
    println!("==================================================");
    
    let client = Client::new();
    
    // Test 1: Check if server is running
    println!("\n1. Testing server connectivity...");
    match client.get(&format!("{}/health", SERVER_URL)).send().await {
        Ok(response) => {
            println!("✅ Server is running, status: {}", response.status());
            if response.status().is_success() {
                let body: serde_json::Value = response.json().await?;
                println!("   Health response: {}", body);
            }
        }
        Err(e) => {
            println!("❌ Server is not running: {}", e);
            println!("   Please start the server first: cargo run --bin kembridge-backend");
            return Ok(());
        }
    }
    
    // Test 2: Test public endpoints (should work without auth)
    println!("\n2. Testing public endpoints...");
    
    // Test health endpoint
    let health_response = client.get(&format!("{}/health", SERVER_URL)).send().await?;
    println!("   /health: {}", health_response.status());
    
    // Test metrics endpoint
    let metrics_response = client.get(&format!("{}/metrics", SERVER_URL)).send().await?;
    println!("   /metrics: {}", metrics_response.status());
    
    // Test bridge quote endpoint
    let quote_response = client.get(&format!("{}/api/v1/bridge/quote?fromChain=ethereum&toChain=near&amount=1000000000000000000&tokenAddress=0x0000000000000000000000000000000000000000", SERVER_URL)).send().await?;
    println!("   /api/v1/bridge/quote: {}", quote_response.status());
    
    // Test 3: Test protected endpoints without auth (should fail)
    println!("\n3. Testing protected endpoints without auth...");
    
    let admin_response = client.get(&format!("{}/api/v1/admin/review/queue", SERVER_URL)).send().await?;
    println!("   /api/v1/admin/review/queue (no auth): {} (should be 401)", admin_response.status());
    
    let manual_review_response = client.post(&format!("{}/api/v1/admin/review/queue", SERVER_URL))
        .json(&json!({
            "transaction_id": "test-tx-123",
            "user_id": "test-user-123",
            "risk_score": 0.75,
            "reason": "High risk transaction"
        }))
        .send().await?;
    println!("   /api/v1/admin/review/queue (no auth): {} (should be 401)", manual_review_response.status());
    
    // Test 4: Test auth flow
    println!("\n4. Testing authentication flow...");
    
    // First, get a nonce for wallet authentication
    let nonce_response = client.get(&format!("{}/api/v1/auth/nonce?wallet_address=0x000test_admin_wallet&chain_type=ethereum", SERVER_URL)).send().await?;
    println!("   GET /api/v1/auth/nonce: {}", nonce_response.status());
    
    if nonce_response.status().is_success() {
        let nonce_data: serde_json::Value = nonce_response.json().await?;
        println!("   Nonce response: {}", nonce_data);
        
        // Note: In a real test, you would need to:
        // 1. Sign the nonce with a private key
        // 2. Send the signature to /api/v1/auth/verify-wallet
        // 3. Use the returned JWT token for authenticated requests
        
        println!("   Note: Full auth flow requires wallet signature (not implemented in this test)");
    }
    
    // Test 5: Test with manually created JWT token
    println!("\n5. Testing with manually created JWT token...");
    
    // Create a JWT token for testing (this simulates what the auth service would do)
    let jwt_secret = env::var("JWT_SECRET").unwrap_or_else(|_| "hackathon-super-secret-key-change-in-production".to_string());
    let jwt_manager = kembridge_auth::JwtManager::new(jwt_secret)?;
    
    let test_user_id = uuid::Uuid::new_v4();
    let admin_wallet = "0x000test_admin_wallet"; // This should be admin tier
    let regular_wallet = "0x123regular_wallet"; // This should be regular tier
    
    // Create admin token
    let admin_token = jwt_manager.generate_token(test_user_id, admin_wallet, kembridge_auth::ChainType::Ethereum).await?;
    println!("   Generated admin token: {}...", &admin_token[..50]);
    
    // Create regular user token
    let regular_token = jwt_manager.generate_token(test_user_id, regular_wallet, kembridge_auth::ChainType::Ethereum).await?;
    println!("   Generated regular token: {}...", &regular_token[..50]);
    
    // Test 6: Test admin endpoints with admin token
    println!("\n6. Testing admin endpoints with admin token...");
    
    let admin_reviews_response = client.get(&format!("{}/api/v1/admin/review/queue", SERVER_URL))
        .header("Authorization", format!("Bearer {}", admin_token))
        .send().await?;
    println!("   /api/v1/admin/review/queue (admin token): {}", admin_reviews_response.status());
    if admin_reviews_response.status().is_success() {
        let body: serde_json::Value = admin_reviews_response.json().await?;
        println!("   Response: {}", body);
    }
    
    // Test manual review creation with admin token
    let manual_review_admin_response = client.post(&format!("{}/api/v1/admin/review/queue", SERVER_URL))
        .header("Authorization", format!("Bearer {}", admin_token))
        .json(&json!({
            "transaction_id": "test-tx-456",
            "user_id": "test-user-456",
            "risk_score": 0.85,
            "reason": "High risk transaction requiring manual review"
        }))
        .send().await?;
    println!("   /api/v1/admin/review/queue (admin token): {}", manual_review_admin_response.status());
    if manual_review_admin_response.status().is_success() {
        let body: serde_json::Value = manual_review_admin_response.json().await?;
        println!("   Response: {}", body);
    }
    
    // Test 7: Test admin endpoints with regular token (should fail)
    println!("\n7. Testing admin endpoints with regular token...");
    
    let admin_reviews_regular_response = client.get(&format!("{}/api/v1/admin/review/queue", SERVER_URL))
        .header("Authorization", format!("Bearer {}", regular_token))
        .send().await?;
    println!("   /api/v1/admin/review/queue (regular token): {} (should be 403)", admin_reviews_regular_response.status());
    
    // Test manual review creation with regular token (should fail)
    let manual_review_regular_response = client.post(&format!("{}/api/v1/admin/review/queue", SERVER_URL))
        .header("Authorization", format!("Bearer {}", regular_token))
        .json(&json!({
            "transaction_id": "test-tx-789",
            "user_id": "test-user-789",
            "risk_score": 0.65,
            "reason": "Medium risk transaction"
        }))
        .send().await?;
    println!("   /api/v1/admin/review/queue (regular token): {} (should be 403)", manual_review_regular_response.status());
    
    println!("\n✅ HTTP Authentication tests completed!");
    println!("Summary:");
    println!("- Public endpoints should return 200");
    println!("- Protected endpoints without auth should return 401");
    println!("- Admin endpoints with admin token should return 200");
    println!("- Admin endpoints with regular token should return 403");
    
    Ok(())
}