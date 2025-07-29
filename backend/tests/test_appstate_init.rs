// tests/test_appstate_init.rs - Diagnostic test for AppState initialization
use std::env;
use tokio;
use redis::aio::ConnectionManager;
use redis::AsyncCommands;

// Import required types from the main crate
use kembridge_backend::config::AppConfig;
use kembridge_backend::state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("🔧 KEMBridge AppState Initialization Diagnostic");
    println!("===========================================");
    
    // 1. Test environment variables
    println!("\n1️⃣ Testing Environment Variables...");
    
    let env_vars = [
        "DATABASE_URL",
        "REDIS_URL", 
        "ONEINCH_API_KEY",
        "ETHEREUM_RPC_URL",
        "NEAR_RPC_URL",
        "JWT_SECRET",
        "AI_ENGINE_URL"
    ];
    
    for var in &env_vars {
        match env::var(var) {
            Ok(value) => {
                if var.contains("SECRET") || var.contains("KEY") {
                    println!("   ✅ {}: [REDACTED - {} chars]", var, value.len());
                } else {
                    println!("   ✅ {}: {}", var, value);
                }
            }
            Err(_) => println!("   ❌ {}: NOT SET", var),
        }
    }
    
    // 2. Test configuration loading
    println!("\n2️⃣ Testing Configuration Loading...");
    
    let config = match AppConfig::from_env() {
        Ok(config) => {
            println!("   ✅ Configuration loaded successfully");
            println!("   📋 Database URL: {}", config.database_url);
            println!("   📋 Redis URL: {}", config.redis_url);
            println!("   📋 1inch API Key: {}", 
                if config.oneinch_api_key.is_some() { "✅ Present" } else { "❌ Missing" });
            config
        }
        Err(e) => {
            println!("   ❌ Configuration loading failed: {}", e);
            return Err(e);
        }
    };
    
    // 3. Test database connection
    println!("\n3️⃣ Testing Database Connection...");
    
    let db_pool = match kembridge_database::create_pool(&config.database_url).await {
        Ok(pool) => {
            println!("   ✅ Database pool created successfully");
            pool
        }
        Err(e) => {
            println!("   ❌ Database connection failed: {}", e);
            return Err(e);
        }
    };
    
    // Test database query
    match sqlx::query("SELECT 1 as test").fetch_one(&db_pool).await {
        Ok(_) => println!("   ✅ Database query test successful"),
        Err(e) => {
            println!("   ❌ Database query failed: {}", e);
            return Err(e.into());
        }
    }
    
    // 4. Test Redis connection
    println!("\n4️⃣ Testing Redis Connection...");
    
    let redis_client = match redis::Client::open(config.redis_url.as_str()) {
        Ok(client) => {
            println!("   ✅ Redis client created successfully");
            client
        }
        Err(e) => {
            println!("   ❌ Redis client creation failed: {}", e);
            return Err(e.into());
        }
    };
    
    let redis_manager = match ConnectionManager::new(redis_client).await {
        Ok(manager) => {
            println!("   ✅ Redis connection manager created successfully");
            manager
        }
        Err(e) => {
            println!("   ❌ Redis connection manager failed: {}", e);
            return Err(e.into());
        }
    };
    
    // Test Redis connection
    let mut redis_conn = redis_manager.clone();
    match redis_conn.ping::<String>().await {
        Ok(response) => println!("   ✅ Redis ping successful: {}", response),
        Err(e) => {
            println!("   ❌ Redis ping failed: {}", e);
            return Err(e.into());
        }
    }
    
    // 5. Test Redis pool creation
    println!("\n5️⃣ Testing Redis Pool Creation...");
    
    let redis_pool = match deadpool_redis::Config::from_url(&config.redis_url)
        .create_pool(Some(deadpool_redis::Runtime::Tokio1)) {
        Ok(pool) => {
            println!("   ✅ Redis pool created successfully");
            pool
        }
        Err(e) => {
            println!("   ❌ Redis pool creation failed: {}", e);
            return Err(e.into());
        }
    };
    
    // 6. Test AI Engine connection (optional)
    println!("\n6️⃣ Testing AI Engine Connection...");
    
    let client = reqwest::Client::new();
    match client.get(&format!("{}/health", config.ai_engine_url))
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await {
        Ok(response) => {
            println!("   ✅ AI Engine reachable: status {}", response.status());
        }
        Err(e) => {
            println!("   ⚠️ AI Engine connection failed (non-critical): {}", e);
        }
    }
    
    // 7. Test AppState initialization step by step
    println!("\n7️⃣ Testing AppState Initialization (Step-by-step)...");
    
    println!("   📝 Step 1: Starting AppState::new()...");
    match AppState::new(
        db_pool,
        redis_manager,
        redis_pool,
        config.clone()
    ).await {
        Ok(app_state) => {
            println!("   ✅ AppState initialized successfully!");
            println!("   📋 Auth service: ✅");
            println!("   📋 Quantum service: ✅");
            println!("   📋 AI client: ✅");
            println!("   📋 Bridge service: {}", 
                if app_state.bridge_service.is_some() { "✅" } else { "⚠️ None (acceptable)" });
            println!("   📋 User service: ✅");
            println!("   📋 Risk integration service: ✅");
            println!("   📋 Manual review service: ✅");
            println!("   📋 Transaction service: ✅");
            println!("   📋 WebSocket registry: ✅");
            println!("   📋 Monitoring service: ✅");
            println!("   📋 Price oracle service: ✅");
            println!("   📋 1inch service: ✅");
            println!("   📋 Dynamic pricing service: ✅");
            println!("   📋 Bridge integration service: ✅");
            println!("   📋 Rate limit service: ✅");
        }
        Err(e) => {
            println!("   ❌ AppState initialization failed: {}", e);
            println!("   🔍 Error details: {:?}", e);
            
            // Print the full chain of errors
            let mut source = e.source();
            let mut level = 1;
            while let Some(err) = source {
                println!("   🔍 Caused by (level {}): {}", level, err);
                source = err.source();
                level += 1;
            }
            
            return Err(e);
        }
    }
    
    println!("\n✅ All AppState initialization diagnostic tests PASSED!");
    println!("🎉 The issue is likely NOT in the AppState initialization logic");
    println!("🔍 The problem may be in:");
    println!("   - Docker environment variables not being passed correctly");
    println!("   - Service dependencies that aren't available in Docker"); 
    println!("   - Network connectivity within Docker containers");
    println!("   - Timing issues during container startup");
    
    Ok(())
}