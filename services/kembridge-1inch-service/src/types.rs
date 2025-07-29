use serde::{Deserialize, Serialize};
use bigdecimal::{BigDecimal, Num};
use validator::Validate;

// Quote request types
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct QuoteRequest {
    pub chain_id: u64,
    
    #[validate(length(min = 42, max = 42))]
    pub from_token: String,
    
    #[validate(length(min = 42, max = 42))]
    pub to_token: String,
    
    pub amount: BigDecimal,
    
    pub slippage: Option<BigDecimal>,
    
    pub user_address: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuoteResponse {
    pub from_token: TokenInfo,
    pub to_token: TokenInfo,
    pub from_amount: BigDecimal,
    pub to_amount: BigDecimal,
    pub to_amount_min: BigDecimal,
    pub price_impact: BigDecimal,
    pub gas_estimate: u64,
    pub estimated_gas_fee: BigDecimal,
    pub protocols: Vec<ProtocolInfo>,
    pub quote_id: String,
    pub valid_until: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedQuoteRequest {
    #[serde(flatten)]
    pub base: QuoteRequest,
    
    pub include_gas_comparison: Option<bool>,
    pub include_time_estimates: Option<bool>,
    pub optimization_preference: Option<OptimizationPreference>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OptimizationPreference {
    BestPrice,
    BestGas,
    Fastest,
    Balanced,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedQuoteResponse {
    #[serde(flatten)]
    pub base: QuoteResponse,
    
    pub alternative_routes: Vec<RouteOption>,
    pub gas_comparison: Option<GasComparison>,
    pub time_estimates: Option<TimeEstimates>,
    pub price_rating: PriceRating,
}

// Swap execution types
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct SwapExecutionRequest {
    pub quote_id: String,
    
    #[validate(length(min = 42, max = 42))]
    pub user_address: String,
    
    pub slippage: Option<BigDecimal>,
    
    pub enable_estimate: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapResponse {
    pub transaction_hash: Option<String>,
    pub order_hash: String,
    pub status: SwapStatus,
    pub gas_used: Option<u64>,
    pub actual_gas_fee: Option<BigDecimal>,
    pub execution_time_ms: u64,
    pub final_amounts: Option<FinalAmounts>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SwapStatus {
    Pending,
    Processing,
    Completed,
    Failed,
    Cancelled,
}

// Token and protocol info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenInfo {
    pub address: String,
    pub symbol: String,
    pub name: String,
    pub decimals: u8,
    pub chain_id: u64,
    pub logo_uri: Option<String>,
    pub price_usd: Option<BigDecimal>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolInfo {
    pub name: String,
    pub part: BigDecimal,
    pub from_token_address: String,
    pub to_token_address: String,
}

// Liquidity info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiquidityInfo {
    pub token_pair: TokenPair,
    pub total_liquidity_usd: BigDecimal,
    pub available_protocols: Vec<String>,
    pub current_spread: BigDecimal,
    pub volume_24h: Option<BigDecimal>,
    pub price_impact_levels: Vec<PriceImpactLevel>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenPair {
    pub token_a: TokenInfo,
    pub token_b: TokenInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceImpactLevel {
    pub amount_usd: BigDecimal,
    pub price_impact: BigDecimal,
}

// Enhanced response types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteOption {
    pub route_id: String,
    pub protocols: Vec<ProtocolInfo>,
    pub to_amount: BigDecimal,
    pub gas_estimate: u64,
    pub price_impact: BigDecimal,
    pub confidence_score: BigDecimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GasComparison {
    pub current_gas_price: BigDecimal,
    pub suggested_gas_price: BigDecimal,
    pub estimated_confirmation_time: u64,
    pub gas_savings_opportunity: Option<BigDecimal>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeEstimates {
    pub estimated_confirmation_time: u64,
    pub network_congestion_level: CongestionLevel,
    pub optimal_time_window: Option<TimeWindow>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CongestionLevel {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeWindow {
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub end_time: chrono::DateTime<chrono::Utc>,
    pub potential_savings: BigDecimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceRating {
    pub score: u8,  // 1-10
    pub comparison_to_market: BigDecimal,
    pub confidence_level: ConfidenceLevel,
    pub factors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfidenceLevel {
    VeryLow,
    Low,
    Medium,
    High,
    VeryHigh,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinalAmounts {
    pub input_amount_actual: BigDecimal,
    pub output_amount_actual: BigDecimal,
    pub price_impact_actual: BigDecimal,
}

// Signed swap execution types
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct SignedSwapExecutionRequest {
    #[validate(length(min = 42, max = 42))]
    pub user_address: String,
    pub signature: String,
    pub transaction_data: String,
    pub chain_id: u64,
}

// Fusion+ cross-chain types
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CrossChainQuoteRequest {
    pub source_chain_id: u64,
    pub dest_chain_id: u64,
    pub from_token: String,
    pub to_token: String,
    pub amount: BigDecimal,
    pub recipient: Option<String>,
    pub slippage: Option<BigDecimal>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossChainQuoteResponse {
    pub quote_id: String,
    pub source_chain: ChainInfo,
    pub dest_chain: ChainInfo,
    pub from_token: TokenInfo,
    pub to_token: TokenInfo,
    pub from_amount: BigDecimal,
    pub to_amount: BigDecimal,
    pub bridge_fee: BigDecimal,
    pub gas_fees: CrossChainGasFees,
    pub total_time_estimate: u64,
    pub price_impact: BigDecimal,
    pub exchange_rate: BigDecimal,
    pub valid_until: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainInfo {
    pub chain_id: u64,
    pub name: String,
    pub native_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossChainGasFees {
    pub source_chain_gas: BigDecimal,
    pub dest_chain_gas: BigDecimal,
    pub bridge_gas: BigDecimal,
    pub total_gas_usd: BigDecimal,
}

// Price oracle types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenPrice {
    pub token: TokenInfo,
    pub price_usd: BigDecimal,
    pub price_change_24h: Option<BigDecimal>,
    pub volume_24h: Option<BigDecimal>,
    pub market_cap: Option<BigDecimal>,
    pub last_updated: chrono::DateTime<chrono::Utc>,
    pub sources: Vec<PriceSource>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceSource {
    pub name: String,
    pub price: BigDecimal,
    pub confidence: BigDecimal,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceComparison {
    pub from_token: TokenInfo,
    pub to_token: TokenInfo,
    pub exchange_rate: BigDecimal,
    pub inverse_rate: BigDecimal,
    pub price_sources: Vec<ExchangeRateSource>,
    pub market_depth: Option<MarketDepth>,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeRateSource {
    pub exchange: String,
    pub rate: BigDecimal,
    pub volume: Option<BigDecimal>,
    pub confidence: BigDecimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketDepth {
    pub total_liquidity_usd: BigDecimal,
    pub bid_depth: BigDecimal,
    pub ask_depth: BigDecimal,
    pub spread: BigDecimal,
}