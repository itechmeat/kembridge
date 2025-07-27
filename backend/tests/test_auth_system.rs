// Test tool for authentication system
use std::env;
use kembridge_auth::{JwtManager, ChainType};
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    println!("🔐 Testing KEMBridge Authentication System");
    println!("==========================================");
    
    // Test 1: JWT Manager functionality
    println!("\n1. Testing JWT Manager...");
    let jwt_secret = env::var("JWT_SECRET").unwrap_or_else(|_| "dev-secret-change-in-production".to_string());
    let jwt_manager = JwtManager::new(jwt_secret)?;
    
    // Generate a test token
    let test_user_id = Uuid::new_v4();
    let test_wallet = "0x000test_admin_wallet";
    let test_chain = ChainType::Ethereum;
    
    let token = jwt_manager.generate_token(test_user_id, test_wallet, test_chain).await?;
    println!("✅ JWT token generated: {}", &token[..50]);
    
    // Verify the token
    let claims = jwt_manager.verify_token(&token).await?;
    println!("✅ JWT token verified successfully");
    println!("   User ID: {}", claims.user_id);
    println!("   Wallet: {}", claims.wallet_address);
    println!("   Chain: {:?}", claims.chain_type);
    
    // Test 2: User tier determination
    println!("\n2. Testing User Tier Determination...");
    test_user_tier("0x000admin_wallet", "Should be ADMIN");
    test_user_tier("0x123premium_wallet_with_long_address_premium", "Should be PREMIUM");
    test_user_tier("0x456regular", "Should be FREE");
    
    // Test 3: Auth middleware simulation
    println!("\n3. Testing Auth Middleware Logic...");
    
    // Test public endpoints
    test_public_endpoint("/health");
    test_public_endpoint("/api/v1/auth/nonce");
    test_public_endpoint("/api/v1/bridge/quote");
    test_public_endpoint("/api/v1/admin/reviews"); // Should NOT be public
    
    println!("\n✅ Authentication system tests completed!");
    println!("Note: For full integration testing, start the server and use HTTP requests.");
    
    Ok(())
}

fn test_user_tier(wallet_address: &str, expected: &str) {
    // Constants from backend/src/constants.rs
    const ADMIN_WALLET_PREFIX_1: &str = "0x000";
    const ADMIN_WALLET_PREFIX_2: &str = "admin";
    const PREMIUM_WALLET_MIN_LENGTH: usize = 42;
    const PREMIUM_WALLET_SUFFIX: &str = "premium";
    const USER_TIER_ADMIN: &str = "admin";
    const USER_TIER_PREMIUM: &str = "premium";
    const USER_TIER_FREE: &str = "free";
    
    let tier = if wallet_address.starts_with(ADMIN_WALLET_PREFIX_1) || wallet_address.starts_with(ADMIN_WALLET_PREFIX_2) {
        USER_TIER_ADMIN
    } else if wallet_address.len() > PREMIUM_WALLET_MIN_LENGTH || wallet_address.ends_with(PREMIUM_WALLET_SUFFIX) {
        USER_TIER_PREMIUM 
    } else {
        USER_TIER_FREE
    };
    
    println!("   {} -> {} ({})", wallet_address, tier, expected);
}

fn test_public_endpoint(path: &str) {
    let is_public = match path {
        "/health" | "/ready" | "/metrics" => true,
        "/ws" => true,
        "/api/v1/bridge/quote" => true,
        path if path.starts_with("/docs") => true,
        path if path.starts_with("/api/v1/auth") => true,
        path if path.starts_with("/api/v1/bridge/status/") => true,
        path if path.starts_with("/static") => true,
        _ => false,
    };
    
    let status = if is_public { "PUBLIC" } else { "PROTECTED" };
    println!("   {} -> {}", path, status);
}