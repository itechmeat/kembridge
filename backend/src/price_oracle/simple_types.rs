// src/price_oracle/simple_types.rs - Simplified types for quick real data integration
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

use crate::constants::*;

/// Simplified price data from a single provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimplePriceData {
    pub symbol: String,
    pub price: f64,
    pub timestamp: DateTime<Utc>,
    pub source: String,
    pub confidence: f64,
    pub volume_24h: Option<f64>,
    pub change_24h: Option<f64>,
}

/// Simplified aggregated price from multiple sources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimpleAggregatedPrice {
    pub symbol: String,
    pub price: f64,
    pub sources: Vec<String>,
    pub confidence: f64,
    pub last_updated: DateTime<Utc>,
    pub price_variance: f64,
    pub volume_24h: Option<f64>,
    pub change_24h: Option<f64>,
}

/// Quick price oracle service for real data
pub struct QuickPriceOracleService {
    pub coingecko_client: reqwest::Client,
    pub binance_client: reqwest::Client,
}

impl QuickPriceOracleService {
    pub fn new() -> Self {
        Self {
            coingecko_client: reqwest::Client::new(),
            binance_client: reqwest::Client::new(),
        }
    }
    
    /// Get real price from CoinGecko
    pub async fn get_real_price(&self, symbol: &str) -> Result<SimpleAggregatedPrice, Box<dyn std::error::Error + Send + Sync>> {
        let coin_id = match symbol.to_uppercase().as_str() {
            "ETH/USD" | "ETH" => "ethereum",
            "NEAR/USD" | "NEAR" => "near",
            "BTC/USD" | "BTC" => "bitcoin",
            "USDT/USD" | "USDT" => "tether",
            "USDC/USD" | "USDC" => "usd-coin",
            _ => return Err("Unsupported symbol".into()),
        };
        
        let url = format!("{}/simple/price?ids={}&vs_currencies=usd&include_24hr_change=true", COINGECKO_API_BASE, coin_id);
        
        let response = self.coingecko_client.get(&url).send().await?;
        let data: serde_json::Value = response.json().await?;
        
        let coin_data = data.get(coin_id).ok_or("Coin not found")?;
        let price = coin_data.get("usd").and_then(|v| v.as_f64()).ok_or("Price not found")?;
        let change_24h = coin_data.get("usd_24h_change").and_then(|v| v.as_f64());
        
        Ok(SimpleAggregatedPrice {
            symbol: symbol.to_string(),
            price,
            sources: vec!["coingecko".to_string()],
            confidence: PRICE_CONFIDENCE_DEFAULT,
            last_updated: Utc::now(),
            price_variance: 0.0,
            volume_24h: None,
            change_24h,
        })
    }
    
    /// Get multiple real prices
    pub async fn get_multiple_real_prices(&self, symbols: &[&str]) -> Result<Vec<SimpleAggregatedPrice>, Box<dyn std::error::Error + Send + Sync>> {
        let mut results = Vec::new();
        
        for &symbol in symbols {
            match self.get_real_price(symbol).await {
                Ok(price) => results.push(price),
                Err(e) => eprintln!("Failed to get price for {}: {}", symbol, e),
            }
        }
        
        Ok(results)
    }
    
    /// Get supported symbols
    pub fn get_supported_symbols(&self) -> Vec<String> {
        vec![
            "ETH/USD".to_string(),
            "NEAR/USD".to_string(),
            "BTC/USD".to_string(),
            "USDT/USD".to_string(),
            "USDC/USD".to_string(),
        ]
    }
}