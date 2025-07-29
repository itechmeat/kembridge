use sqlx::{PgPool, Row};
use std::env;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("🔍 Testing simple database connection...");
    
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://postgres:dev_password@postgres:5432/kembridge_dev".to_string());
    
    println!("📊 Database URL: {}", database_url);
    
    // Test basic connection
    let pool = PgPool::connect(&database_url).await?;
    println!("✅ Database connection successful");
    
    // Test simple query
    let row = sqlx::query("SELECT 1 as test_col")
        .fetch_one(&pool)
        .await?;
    
    let result: i32 = row.get("test_col");
    println!("✅ Simple query result: {}", result);
    
    // Test table count
    let result: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM _sqlx_migrations")
        .fetch_one(&pool)
        .await?;
    
    println!("✅ Migration table has {} rows", result.0);
    
    pool.close().await;
    println!("🎉 Database test completed successfully");
    
    Ok(())
}