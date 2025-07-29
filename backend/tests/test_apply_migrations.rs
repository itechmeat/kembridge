use kembridge_database;
use std::env;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://postgres:postgres@postgres:5432/kembridge_dev".to_string());
    
    println!("🔧 Connecting to database: {}", database_url);
    
    let pool = kembridge_database::create_pool(&database_url).await?;
    
    println!("🚀 Running migrations...");
    kembridge_database::run_migrations(&pool).await?;
    
    println!("✅ Migrations completed successfully!");
    
    pool.close().await;
    Ok(())
}