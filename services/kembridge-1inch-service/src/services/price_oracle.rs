use crate::config::ServiceConfig;
use crate::errors::{OneinchServiceError, Result};
use crate::services::cache::CacheService;
use crate::types::{TokenPrice, PriceSource, PriceComparison};
use bigdecimal::BigDecimal;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tracing::{error, info, warn};

pub struct PriceOracleService {
    config: Arc<ServiceConfig>,
    cache: Arc<CacheService>,
}

impl Clone for PriceOracleService {
    fn clone(&self) -> Self {
        Self {
            config: Arc::clone(&self.config),
            cache: Arc::clone(&self.cache),
        }
    }
}

#[async_trait::async_trait]
pub trait PriceSourceTrait {
    async fn get_price(&self, chain_id: u64, token: &str) -> Result<Option<BigDecimal>>;
    fn source_name(&self) -> &str;
    fn confidence_score(&self) -> BigDecimal;
}

impl PriceOracleService {
    pub async fn new(
        config: Arc<ServiceConfig>,
        cache: Arc<CacheService>,
    ) -> Result<Self> {
        let mut price_sources: Vec<Box<dyn PriceSourceTrait + Send + Sync>> = vec![];

        // Add CoinGecko price source (always available)
        price_sources.push(Box::new(CoinGeckoPriceSource::new()));

        // Add 1inch price source if available
        price_sources.push(Box::new(OneinchPriceSource::new()));

        // Add on-chain price source if Ethereum RPC is configured
        #[cfg(feature = "price-oracles")]
        if let Some(rpc_url) = &config.ethereum_rpc_url {
            match OnChainPriceSource::new(rpc_url.clone()).await {
                Ok(source) => {
                    price_sources.push(Box::new(source));
                    info!("On-chain price oracle initialized");
                }
                Err(e) => {
                    warn!("Failed to initialize on-chain price oracle: {}", e);
                }
            }
        }

        Ok(Self {
            config,
            cache,
        })
    }

    pub async fn get_token_price(&self, chain_id: u64, token: &str) -> Result<TokenPrice> {
        let cache_key = self.cache.token_price_key(chain_id, token);
        
        // Try cache first
        if let Some(cached_price) = self.cache.get::<TokenPrice>(&cache_key).await? {
            return Ok(cached_price);
        }

        // Fetch from multiple sources
        let mut price_sources = Vec::new();
        let mut total_confidence = BigDecimal::from(0);
        let mut weighted_price = BigDecimal::from(0);

        for source in &self.price_sources {
            match source.get_price(chain_id, token).await {
                Ok(Some(price)) => {
                    let confidence = source.confidence_score();
                    let weighted_contribution = &price * &confidence;
                    
                    weighted_price += &weighted_contribution;
                    total_confidence += &confidence;
                    
                    price_sources.push(PriceSource {
                        name: source.source_name().to_string(),
                        price,
                        confidence,
                        last_updated: chrono::Utc::now(),
                    });
                }
                Ok(None) => {
                    warn!("No price available from {} for token {} on chain {}", 
                          source.source_name(), token, chain_id);
                }
                Err(e) => {
                    error!("Error fetching price from {}: {}", source.source_name(), e);
                }
            }
        }

        if price_sources.is_empty() {
            return Err(OneinchServiceError::PriceOracleError {
                message: format!("No price sources available for token {} on chain {}", token, chain_id),
            });
        }

        let final_price = if total_confidence > BigDecimal::from(0) {
            weighted_price / total_confidence
        } else {
            // Fallback to simple average
            let sum: BigDecimal = price_sources.iter().map(|s| &s.price).sum();
            sum / BigDecimal::from(price_sources.len())
        };

        let token_price = TokenPrice {
            token: crate::types::TokenInfo {
                address: token.to_string(),
                symbol: "UNKNOWN".to_string(), // Will be filled by token service
                name: "Unknown Token".to_string(),
                decimals: 18,
                chain_id,
                logo_uri: None,
                price_usd: Some(final_price.clone()),
            },
            price_usd: final_price,
            price_change_24h: None, // TODO: Implement 24h tracking
            volume_24h: None,
            market_cap: None,
            last_updated: chrono::Utc::now(),
            sources: price_sources,
        };

        // Cache the result
        let cache_ttl = Duration::from_secs(self.config.price_cache_ttl_seconds);
        if let Err(e) = self.cache.set(&cache_key, &token_price, cache_ttl).await {
            warn!("Failed to cache price for {}: {}", token, e);
        }

        Ok(token_price)
    }

    pub async fn compare_prices(
        &self,
        chain_id: u64,
        from_token: &str,
        to_token: &str,
    ) -> Result<PriceComparison> {
        let from_price_future = self.get_token_price(chain_id, from_token);
        let to_price_future = self.get_token_price(chain_id, to_token);

        let (from_price, to_price) = tokio::try_join!(from_price_future, to_price_future)?;

        let exchange_rate = &from_price.price_usd / &to_price.price_usd;
        let inverse_rate = &to_price.price_usd / &from_price.price_usd;

        // Combine sources from both tokens
        let mut exchange_rate_sources = Vec::new();
        
        for from_source in &from_price.sources {
            for to_source in &to_price.sources {
                let rate = &from_source.price / &to_source.price;
                let confidence = (&from_source.confidence + &to_source.confidence) / BigDecimal::from(2);
                
                exchange_rate_sources.push(crate::types::ExchangeRateSource {
                    exchange: format!("{}-{}", from_source.name, to_source.name),
                    rate,
                    volume: None,
                    confidence,
                });
            }
        }

        Ok(PriceComparison {
            from_token: from_price.token,
            to_token: to_price.token,
            exchange_rate,
            inverse_rate,
            price_sources: exchange_rate_sources,
            market_depth: None, // TODO: Implement market depth
            last_updated: chrono::Utc::now(),
        })
    }

    pub async fn health_check(&self) -> Result<()> {
        let mut healthy_sources = 0;
        
        // Test each price source with a common token (WETH)
        let test_token = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2"; // WETH
        let test_chain = 1; // Ethereum mainnet

        for source in &self.price_sources {
            match source.get_price(test_chain, test_token).await {
                Ok(Some(_)) => {
                    healthy_sources += 1;
                    info!("Price source {} is healthy", source.source_name());
                }
                Ok(None) => {
                    warn!("Price source {} returned no data", source.source_name());
                }
                Err(e) => {
                    error!("Price source {} is unhealthy: {}", source.source_name(), e);
                }
            }
        }

        if healthy_sources == 0 {
            return Err(OneinchServiceError::PriceOracleError {
                message: "No healthy price sources available".to_string(),
            });
        }

        info!("Price oracle health check: {}/{} sources healthy", 
              healthy_sources, self.price_sources.len());
        Ok(())
    }
}

// CoinGecko price source implementation
struct CoinGeckoPriceSource {
    client: reqwest::Client,
}

impl CoinGeckoPriceSource {
    fn new() -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .user_agent("KEMBridge/1.0")
            .build()
            .expect("Failed to create CoinGecko client");

        Self { client }
    }
}

#[async_trait::async_trait]
impl PriceSourceTrait for CoinGeckoPriceSource {
    async fn get_price(&self, _chain_id: u64, token: &str) -> Result<Option<BigDecimal>> {
        // For demo purposes, return mock data
        // In production, this would call CoinGecko API
        match token.to_lowercase().as_str() {
            "0xa0b86a33e6ba30b22cdcaa4f1ee79f4b14b6a67e" => Ok(Some(BigDecimal::from(2000))), // ETH
            "0xdac17f958d2ee523a2206206994597c13d831ec7" => Ok(Some(BigDecimal::from(1))),    // USDT
            _ => Ok(None),
        }
    }

    fn source_name(&self) -> &str {
        "CoinGecko"
    }

    fn confidence_score(&self) -> BigDecimal {
        BigDecimal::from(80) // High confidence
    }
}

// 1inch price source implementation
struct OneinchPriceSource {
    client: reqwest::Client,
}

impl OneinchPriceSource {
    fn new() -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .user_agent("KEMBridge/1.0")
            .build()
            .expect("Failed to create 1inch client");

        Self { client }
    }
}

#[async_trait::async_trait]
impl PriceSourceTrait for OneinchPriceSource {
    async fn get_price(&self, _chain_id: u64, token: &str) -> Result<Option<BigDecimal>> {
        // For demo purposes, return mock data
        // In production, this would derive prices from 1inch quotes
        match token.to_lowercase().as_str() {
            "0xa0b86a33e6ba30b22cdcaa4f1ee79f4b14b6a67e" => Ok(Some(BigDecimal::from(2001))), // ETH (slightly different)
            "0xdac17f958d2ee523a2206206994597c13d831ec7" => Ok(Some(BigDecimal::from(1))),    // USDT
            _ => Ok(None),
        }
    }

    fn source_name(&self) -> &str {
        "1inch"
    }

    fn confidence_score(&self) -> BigDecimal {
        BigDecimal::from(90) // Very high confidence for DEX prices
    }
}

// On-chain price source (optional, with ethers dependency)
#[cfg(feature = "price-oracles")]
struct OnChainPriceSource {
    // In production, this would contain ethers Provider
    rpc_url: String,
}

#[cfg(feature = "price-oracles")]
impl OnChainPriceSource {
    async fn new(rpc_url: String) -> Result<Self> {
        // In production, this would initialize ethers Provider
        // and verify connection
        Ok(Self { rpc_url })
    }
}

#[cfg(feature = "price-oracles")]
#[async_trait::async_trait]
impl PriceSourceTrait for OnChainPriceSource {
    async fn get_price(&self, _chain_id: u64, _token: &str) -> Result<Option<BigDecimal>> {
        // In production, this would:
        // 1. Query Chainlink price feeds
        // 2. Query Uniswap V3 pools
        // 3. Query other on-chain sources
        Ok(None) // Not implemented in demo
    }

    fn source_name(&self) -> &str {
        "OnChain"
    }

    fn confidence_score(&self) -> BigDecimal {
        BigDecimal::from(95) // Highest confidence for on-chain data
    }
}