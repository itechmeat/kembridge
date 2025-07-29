// Simple migration runner without compilation checks
use sqlx::PgPool;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://postgres:dev_password@postgres:5432/kembridge_dev".to_string());
    
    println!("🔧 Connecting to database: {}", database_url);
    
    let pool = PgPool::connect(&database_url).await?;
    
    println!("🚀 Running migrations...");
    
    // Run migrations from the folder where they exist
    sqlx::migrate!("./migrations").run(&pool).await?;
    
    println!("✅ Migrations completed successfully!");
    
    pool.close().await;
    Ok(())
}