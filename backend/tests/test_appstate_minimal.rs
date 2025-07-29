use kembridge_backend::{state::AppState, config::AppConfig};
use std::env;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("🔧 Testing minimal AppState initialization...");

    // Load config
    let config = AppConfig::from_env()?;
    println!("✅ Config loaded");

    // Test database connection only
    let db_pool = kembridge_database::create_pool(&config.database_url).await?;
    println!("✅ Database pool created");

    // Test Redis connection manager only
    let redis_manager = redis::aio::ConnectionManager::new(
        redis::Client::open(config.redis_url.as_str())?
    ).await?;
    println!("✅ Redis connection manager created");

    // Test Redis pool creation only
    let redis_pool = deadpool_redis::Config::from_url(&config.redis_url)
        .create_pool(Some(deadpool_redis::Runtime::Tokio1))?;
    println!("✅ Redis pool created");

    // Now try AppState creation
    println!("🚀 Attempting AppState creation...");
    match AppState::new(db_pool, redis_manager, redis_pool, config).await {
        Ok(_) => println!("✅ AppState created successfully!"),
        Err(e) => {
            println!("❌ AppState creation failed: {}", e);
            return Err(e);
        }
    }

    println!("🎉 All tests passed!");
    Ok(())
}