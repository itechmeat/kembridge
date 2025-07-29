// Temporary migration runner
use std::env;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://postgres:postgres@postgres:5432/kembridge_dev".to_string());
    
    println!("Connecting to database: {}", database_url);
    
    let pool = sqlx::PgPool::connect(&database_url).await?;
    
    println!("Running migrations...");
    sqlx::migrate!("./migrations").run(&pool).await?;
    
    println!("Migrations completed successfully!");
    
    pool.close().await;
    Ok(())
}