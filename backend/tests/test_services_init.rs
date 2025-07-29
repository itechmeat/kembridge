use kembridge_backend::{config::AppConfig, services::*};
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("🔧 Testing individual service initialization...");
    
    let config = AppConfig::from_env()?;
    println!("✅ Config loaded");
    
    let db_pool = kembridge_database::create_pool(&config.database_url).await?;
    println!("✅ Database pool created");
    
    let redis_manager = redis::aio::ConnectionManager::new(
        redis::Client::open(config.redis_url.as_str())?
    ).await?;
    println!("✅ Redis connection manager created");
    
    // Test each service with timeout protection
    println!("🚀 Testing AiClient (simple HTTP client)...");
    let _ai_client = AiClient::new(&config.ai_engine_url)?;
    println!("✅ AiClient initialized");
    
    println!("🚀 Testing TransactionService (simple struct)...");
    let _transaction_service = kembridge_database::TransactionService::new(db_pool.clone());
    println!("✅ TransactionService initialized");
    
    println!("🚀 Testing AuthService (may hang here)...");
    tokio::time::timeout(std::time::Duration::from_secs(10), async {
        let _auth_service = AuthService::new(db_pool.clone(), redis_manager.clone(), config.jwt_secret.clone()).await?;
        Ok::<(), anyhow::Error>(())
    }).await??;
    println!("✅ AuthService initialized");
    
    println!("🚀 Testing QuantumService (may hang here)...");
    tokio::time::timeout(std::time::Duration::from_secs(10), async {
        let _quantum_service = QuantumService::new(db_pool.clone(), &config).await?;
        Ok::<(), anyhow::Error>(())
    }).await??;
    println!("✅ QuantumService initialized");
    
    println!("🎉 All basic services initialized successfully!");
    Ok(())
}