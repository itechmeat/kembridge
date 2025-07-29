pub mod oneinch_client;
pub mod cache;
pub mod price_oracle;
pub mod quote_manager;
pub mod swap_executor;

use crate::config::ServiceConfig;
use crate::errors::Result;
use deadpool_redis::Pool as RedisPool;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<ServiceConfig>,
    pub redis_pool: RedisPool,
    pub oneinch_client: Arc<oneinch_client::OneinchClient>,
    pub cache: Arc<cache::CacheService>,
    pub price_oracle: Arc<price_oracle::PriceOracleService>,
    pub quote_manager: Arc<quote_manager::QuoteManager>,
    pub swap_executor: Arc<swap_executor::SwapExecutor>,
}

impl AppState {
    pub async fn new(config: ServiceConfig, redis_pool: RedisPool) -> Result<Self> {
        let config = Arc::new(config);
        
        // Initialize services
        let oneinch_client = Arc::new(
            oneinch_client::OneinchClient::new(
                config.oneinch_api_url.clone(),
                config.oneinch_api_key.clone(),
                config.oneinch_timeout_ms,
                config.oneinch_max_retries,
            ).await?
        );

        let cache = Arc::new(
            cache::CacheService::new(redis_pool.clone())
        );

        let price_oracle = Arc::new(
            price_oracle::PriceOracleService::new(
                config.clone(),
                cache.clone(),
            ).await?
        );

        let quote_manager = Arc::new(
            quote_manager::QuoteManager::new(
                oneinch_client.clone(),
                cache.clone(),
                price_oracle.clone(),
            )
        );

        let swap_executor = Arc::new(
            swap_executor::SwapExecutor::new(
                oneinch_client.clone(),
                quote_manager.clone(),
            )
        );

        Ok(Self {
            config,
            redis_pool,
            oneinch_client,
            cache,
            price_oracle,
            quote_manager,
            swap_executor,
        })
    }
}