pub mod models;
pub mod pool;
pub mod transactions_simple;

pub use models::*;
pub use transactions_simple::{
    TransactionService, RiskStatistics, RiskScoreHistoryEntry, RiskTrendEntry,
    RiskAnalyticsSummary, RiskScoreDistribution, TopRiskyTransaction
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