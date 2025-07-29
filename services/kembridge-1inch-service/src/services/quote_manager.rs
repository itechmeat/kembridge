use crate::errors::{OneinchServiceError, Result};
use crate::services::{cache::CacheService, oneinch_client::OneinchClient, price_oracle::PriceOracleService};
use crate::types::{QuoteRequest, QuoteResponse, EnhancedQuoteRequest, EnhancedQuoteResponse};
use bigdecimal::BigDecimal;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use tracing::{error, info, warn};
use uuid::Uuid;

#[derive(Clone)]
pub struct QuoteManager {
    oneinch_client: Arc<OneinchClient>,
    cache: Arc<CacheService>,
    price_oracle: Arc<PriceOracleService>,
}

impl QuoteManager {
    pub fn new(
        oneinch_client: Arc<OneinchClient>,
        cache: Arc<CacheService>,
        price_oracle: Arc<PriceOracleService>,
    ) -> Self {
        Self {
            oneinch_client,
            cache,
            price_oracle,
        }
    }

    pub async fn get_quote(&self, request: &QuoteRequest) -> Result<QuoteResponse> {
        // Validate request
        self.validate_quote_request(request)?;

        // Check cache first
        let cache_key = self.cache.quote_key(
            request.chain_id,
            &request.from_token,
            &request.to_token,
            &request.amount.to_string(),
        );

        if let Some(cached_quote) = self.cache.get::<QuoteResponse>(&cache_key).await? {
            if cached_quote.valid_until > chrono::Utc::now() {
                info!("Returning cached quote for {}->{}", request.from_token, request.to_token);
                return Ok(cached_quote);
            }
        }

        // Fetch fresh quote from 1inch
        let oneinch_request = self.build_oneinch_quote_request(request)?;
        let oneinch_response = self.oneinch_client.get_quote(&oneinch_request).await?;

        // Convert to our format
        let quote_response = self.convert_oneinch_quote(request, oneinch_response).await?;

        // Cache the result
        let cache_ttl = Duration::from_secs(30); // 30 seconds for quotes
        if let Err(e) = self.cache.set(&cache_key, &quote_response, cache_ttl).await {
            warn!("Failed to cache quote: {}", e);
        }

        Ok(quote_response)
    }

    pub async fn get_enhanced_quote(&self, request: &EnhancedQuoteRequest) -> Result<EnhancedQuoteResponse> {
        // Get base quote
        let base_quote = self.get_quote(&request.base).await?;

        // Add enhanced features
        let mut enhanced = EnhancedQuoteResponse {
            base: base_quote,
            alternative_routes: Vec::new(),
            gas_comparison: None,
            time_estimates: None,
            price_rating: crate::types::PriceRating {
                score: 8, // Default good score
                comparison_to_market: BigDecimal::from(0),
                confidence_level: crate::types::ConfidenceLevel::High,
                factors: vec!["1inch aggregation".to_string()],
            },
        };

        // Add alternative routes if requested
        if request.include_gas_comparison.unwrap_or(false) {
            enhanced.alternative_routes = self.get_alternative_routes(&request.base).await?;
        }

        // Add gas comparison
        if request.include_gas_comparison.unwrap_or(false) {
            enhanced.gas_comparison = Some(self.get_gas_comparison(request.base.chain_id).await?);
        }

        // Add time estimates
        if request.include_time_estimates.unwrap_or(false) {
            enhanced.time_estimates = Some(self.get_time_estimates(request.base.chain_id).await?);
        }

        // Calculate price rating
        enhanced.price_rating = self.calculate_price_rating(&request.base, &enhanced.base).await?;

        Ok(enhanced)
    }

    fn validate_quote_request(&self, request: &QuoteRequest) -> Result<()> {
        // Validate chain ID
        if !self.is_supported_chain(request.chain_id) {
            return Err(OneinchServiceError::ChainNotSupported {
                chain_id: request.chain_id,
            });
        }

        // Validate token addresses
        if !self.is_valid_token_address(&request.from_token) {
            return Err(OneinchServiceError::TokenNotSupported {
                token: request.from_token.clone(),
                chain_id: request.chain_id,
            });
        }

        if !self.is_valid_token_address(&request.to_token) {
            return Err(OneinchServiceError::TokenNotSupported {
                token: request.to_token.clone(),
                chain_id: request.chain_id,
            });
        }

        // Validate amount
        if request.amount <= BigDecimal::from(0) {
            return Err(OneinchServiceError::InvalidQuote {
                reason: "Amount must be greater than 0".to_string(),
            });
        }

        // Validate slippage
        if let Some(slippage) = &request.slippage {
            if *slippage < BigDecimal::from_str("0.1").unwrap() || 
               *slippage > BigDecimal::from(50) {
                return Err(OneinchServiceError::InvalidSlippage {
                    slippage: slippage.to_string(),
                });
            }
        }

        Ok(())
    }

    fn build_oneinch_quote_request(&self, request: &QuoteRequest) -> Result<crate::services::oneinch_client::OneinchQuoteRequest> {
        Ok(crate::services::oneinch_client::OneinchQuoteRequest {
            chain_id: request.chain_id,
            src: request.from_token.clone(),
            dst: request.to_token.clone(),
            amount: request.amount.to_string(),
            slippage: request.slippage.as_ref().map(|s| s.to_string()),
            from: request.user_address.clone(),
        })
    }

    async fn convert_oneinch_quote(
        &self,
        request: &QuoteRequest,
        oneinch_response: crate::services::oneinch_client::OneinchQuoteResponse,
    ) -> Result<QuoteResponse> {
        let quote_id = Uuid::new_v4().to_string();
        let valid_until = chrono::Utc::now() + chrono::Duration::seconds(30);

        // Convert tokens
        let from_token = self.convert_token_info(&oneinch_response.src_token, request.chain_id);
        let to_token = self.convert_token_info(&oneinch_response.dst_token, request.chain_id);

        // Parse amounts
        let from_amount = request.amount.clone();
        let to_amount = BigDecimal::parse_bytes(oneinch_response.dst_amount.as_bytes(), 10)
            .ok_or_else(|| OneinchServiceError::InvalidQuote {
                reason: "Invalid destination amount format".to_string(),
            })?;

        // Calculate minimum amount with slippage
        let slippage = request.slippage.as_ref().unwrap_or(&BigDecimal::from(1)); // 1% default
        let slippage_multiplier = (BigDecimal::from(100) - slippage) / BigDecimal::from(100);
        let to_amount_min = &to_amount * slippage_multiplier;

        // Calculate price impact (simplified)
        let price_impact = self.calculate_price_impact(&from_token, &to_token, &from_amount, &to_amount).await?;

        // Convert protocols
        let protocols = self.convert_protocols(&oneinch_response.protocols);

        Ok(QuoteResponse {
            from_token,
            to_token,
            from_amount,
            to_amount,
            to_amount_min,
            price_impact,
            gas_estimate: oneinch_response.estimated_gas.unwrap_or(150000),
            estimated_gas_fee: BigDecimal::from(0), // TODO: Calculate from gas estimate
            protocols,
            quote_id,
            valid_until,
        })
    }

    fn convert_token_info(&self, oneinch_token: &crate::services::oneinch_client::OneinchToken, chain_id: u64) -> crate::types::TokenInfo {
        crate::types::TokenInfo {
            address: oneinch_token.address.clone(),
            symbol: oneinch_token.symbol.clone(),
            name: oneinch_token.name.clone(),
            decimals: oneinch_token.decimals,
            chain_id,
            logo_uri: oneinch_token.logo_uri.clone(),
            price_usd: None, // Will be filled later if needed
        }
    }

    fn convert_protocols(&self, oneinch_protocols: &[Vec<crate::services::oneinch_client::OneinchProtocol>]) -> Vec<crate::types::ProtocolInfo> {
        let mut protocols = Vec::new();
        
        for protocol_route in oneinch_protocols {
            for protocol in protocol_route {
                protocols.push(crate::types::ProtocolInfo {
                    name: protocol.name.clone(),
                    part: BigDecimal::try_from(protocol.part).unwrap_or(BigDecimal::from(0)),
                    from_token_address: protocol.from_token_address.clone(),
                    to_token_address: protocol.to_token_address.clone(),
                });
            }
        }

        protocols
    }

    async fn calculate_price_impact(
        &self,
        from_token: &crate::types::TokenInfo,
        to_token: &crate::types::TokenInfo,
        from_amount: &BigDecimal,
        to_amount: &BigDecimal,
    ) -> Result<BigDecimal> {
        // Try to get oracle prices for comparison
        match self.price_oracle.compare_prices(from_token.chain_id, &from_token.address, &to_token.address).await {
            Ok(comparison) => {
                let expected_amount = from_amount * &comparison.exchange_rate;
                let impact = ((&expected_amount - to_amount) / &expected_amount) * BigDecimal::from(100);
                Ok(impact.abs())
            }
            Err(_) => {
                // Fallback to simple calculation
                Ok(BigDecimal::from(0)) // No price impact data available
            }
        }
    }

    async fn get_alternative_routes(&self, _request: &QuoteRequest) -> Result<Vec<crate::types::RouteOption>> {
        // In production, this would fetch multiple route options
        Ok(vec![])
    }

    async fn get_gas_comparison(&self, _chain_id: u64) -> Result<crate::types::GasComparison> {
        // In production, this would fetch current gas prices
        Ok(crate::types::GasComparison {
            current_gas_price: BigDecimal::from(20000000000u64), // 20 gwei
            suggested_gas_price: BigDecimal::from(22000000000u64), // 22 gwei
            estimated_confirmation_time: 60, // seconds
            gas_savings_opportunity: Some(BigDecimal::from(5)), // 5% savings
        })
    }

    async fn get_time_estimates(&self, _chain_id: u64) -> Result<crate::types::TimeEstimates> {
        // In production, this would analyze network congestion
        Ok(crate::types::TimeEstimates {
            estimated_confirmation_time: 60,
            network_congestion_level: crate::types::CongestionLevel::Medium,
            optimal_time_window: None,
        })
    }

    async fn calculate_price_rating(&self, _request: &QuoteRequest, _quote: &QuoteResponse) -> Result<crate::types::PriceRating> {
        // In production, this would compare with multiple sources
        Ok(crate::types::PriceRating {
            score: 8, // Good score
            comparison_to_market: BigDecimal::from(0), // At market rate
            confidence_level: crate::types::ConfidenceLevel::High,
            factors: vec![
                "1inch aggregation".to_string(),
                "Multiple DEX sources".to_string(),
            ],
        })
    }

    fn is_supported_chain(&self, chain_id: u64) -> bool {
        // Major chains supported by 1inch
        matches!(chain_id, 1 | 56 | 137 | 10 | 42161 | 43114 | 250)
    }

    fn is_valid_token_address(&self, address: &str) -> bool {
        // Basic Ethereum address validation
        address.starts_with("0x") && address.len() == 42 && 
        address.chars().skip(2).all(|c| c.is_ascii_hexdigit())
    }
}