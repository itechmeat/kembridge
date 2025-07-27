pub mod models;
pub mod pool;
pub mod transaction_service;

pub use models::*;
pub use transaction_service::{
    TransactionService, RiskStatistics, RiskScoreHistoryEntry, RiskTrendEntry,
    RiskAnalyticsSummary, RiskScoreDistribution, TopRiskyTransaction, TransactionDetails
};

use sqlx::PgPool;

pub async fn create_pool(database_url: &str) -> anyhow::Result<PgPool> {
    let pool = PgPool::connect(database_url).await?;
    Ok(pool)
}

pub async fn run_migrations(pool: &PgPool) -> anyhow::Result<()> {
    sqlx::migrate!("../../migrations").run(pool).await?;
    Ok(())
}