# KEMBridge Price Oracle System

## Overview

The Price Oracle System is a comprehensive multi-provider price aggregation and validation service integrated into KEMBridge to provide accurate, real-time cryptocurrency pricing data for cross-chain bridge operations. It combines multiple data sources, intelligent aggregation algorithms, and robust caching strategies to ensure reliable price feeds for optimal swap execution.

## Architecture

The Price Oracle System consists of several interconnected components:

### 1. Multi-Provider Architecture (`/backend/src/price_oracle/providers/`)
- **Chainlink Provider**: Primary price feed source (development mock mode)
- **CoinGecko Provider**: Real-time market data integration with API key support
- **Binance Provider**: High-frequency trading data for major trading pairs
- **Provider Interface**: Unified trait-based interface for all price sources

### 2. Price Aggregation Engine (`/backend/src/price_oracle/aggregator.rs`)
- **Multiple Aggregation Methods**: WeightedAverage, MedianPrice, HighestConfidence, MostRecentPrice
- **Anomaly Filtering**: Statistical analysis to remove outlier price data
- **Confidence Scoring**: Quality assessment of aggregated price data
- **Variance Calculation**: Price deviation analysis across providers

### 3. Validation & Quality Control (`/backend/src/price_oracle/validator.rs`)
- **Basic Validation**: Price range checks, staleness detection, confidence thresholds
- **Advanced Validation**: Market-specific validation rules for different trading pairs
- **Anomaly Detection**: Statistical analysis using rolling averages and Z-scores
- **Circuit Breaker**: Automatic failover when validation failure rates exceed thresholds

### 4. Redis Caching Layer (`/backend/src/price_oracle/cache.rs`)
- **Primary Cache**: 60-second TTL for real-time price data
- **Fallback Cache**: 24-hour TTL for emergency price availability
- **Provider-Specific Caching**: Individual cache layers for each price provider
- **Quote Caching**: Short-term caching for swap quotes (30 seconds)

### 5. HTTP API Layer (`/backend/src/handlers/price_oracle.rs`)
- **Price Endpoints**: Individual and batch price retrieval
- **Quote Generation**: Swap quote calculation with price impact analysis
- **System Monitoring**: Provider health checks and cache statistics
- **Price Alerts**: User-configurable price alert system

## How It Works

### Price Aggregation Process

1. **Data Collection**: The system simultaneously queries multiple price providers:
   - **Chainlink**: Mock data for development, real contract calls for production
   - **CoinGecko**: REST API calls with rate limiting and API key support
   - **Binance**: High-frequency ticker data for major pairs (excludes NEAR)

2. **Validation Pipeline**: Each price data point passes through validation:
   - **Range Check**: Ensures prices fall within reasonable bounds per asset
   - **Staleness Check**: Verifies data freshness (max 5 minutes age)
   - **Confidence Check**: Validates provider-specific quality metrics
   - **Anomaly Detection**: Statistical analysis to identify outliers

3. **Aggregation**: Valid price data is combined using configurable methods:
   - **Weighted Average**: Combines prices weighted by confidence scores
   - **Median Price**: Uses middle value to reduce impact of outliers
   - **Highest Confidence**: Selects price from most reliable source
   - **Most Recent**: Uses newest available price data

4. **Caching Strategy**: Processed data is cached with multiple TTL levels:
   - **Primary**: Short-term cache for active trading
   - **Fallback**: Long-term cache for system resilience
   - **Provider**: Individual caches for performance optimization

### Supported Trading Pairs

- **ETH/USD**: Ethereum to US Dollar
- **NEAR/USD**: NEAR Protocol to US Dollar  
- **BTC/USD**: Bitcoin to US Dollar
- **USDT/USD**: Tether to US Dollar
- **USDC/USD**: USD Coin to US Dollar

### Fallback Strategy

The system implements a multi-layered fallback approach:

1. **Chainlink Provider** (Primary)
2. **CoinGecko API** (Secondary)
3. **Binance API** (Tertiary)
4. **Redis Cache** (Fallback)
5. **Static Backup Prices** (Emergency)

## API Endpoints

### Price Retrieval
```
GET /api/v1/price/price?symbol=ETH/USD
GET /api/v1/price/prices?symbols=ETH/USD,NEAR/USD,BTC/USD
GET /api/v1/price/supported
```

### Quote Generation
```
POST /api/v1/price/quote
{
  "from_token": "ETH",
  "to_token": "NEAR", 
  "from_amount": "1.0"
}
```

### System Monitoring
```
GET /api/v1/price/health
GET /api/v1/price/cache/stats
```

### Price Alerts
```
POST /api/v1/price/alerts
GET /api/v1/price/alerts
DELETE /api/v1/price/alerts/{alert_id}
```

### 1inch Integration Endpoints
```
POST /api/v1/swap/quote/enhanced
GET /api/v1/swap/routing
POST /api/v1/bridge/optimized-swap
POST /api/v1/bridge/savings
```
*These endpoints use Price Oracle for 1inch operation comparison and optimization*

## Configuration

### Provider Configuration
- **Chainlink**: Contract addresses and RPC endpoints
- **CoinGecko**: API keys and rate limiting settings
- **Binance**: Symbol mappings and connection parameters

### Validation Rules
- **Price Ranges**: Asset-specific minimum/maximum price bounds
- **Staleness Thresholds**: Maximum age for price data (default: 5 minutes)
- **Confidence Thresholds**: Minimum quality scores per provider
- **Anomaly Detection**: Statistical thresholds for outlier detection

### Cache Settings
- **Primary TTL**: 60 seconds for real-time data
- **Fallback TTL**: 24 hours for emergency availability
- **Provider TTL**: 30 seconds for individual source caching

### Dynamic Pricing Configuration
- **Bridge Base Fee**: 0.15% of transaction amount
- **Protocol Fee**: 0.10% of transaction amount
- **Slippage Protection Fee**: 0.05% of transaction amount
- **Gas Estimation**: 100,000 units at 20 gwei
- **Exchange Rate Weights**: 60% Oracle, 40% 1inch
- **Volatility Thresholds**: ETH/NEAR 15%, ETH/USDT 8%, NEAR/USDT 12%
- **Price Impact Thresholds**: Low <0.5%, Medium 0.5-2%, High >2%
- **Slippage Limits**: Standard 0.5%, Maximum 2.0%

## Error Handling & Resilience

### Circuit Breaker Pattern
- **Failure Threshold**: 5 consecutive failures trigger circuit open
- **Recovery Timeout**: 1 minute before attempting recovery
- **Half-Open State**: Gradual recovery with monitoring

### Graceful Degradation
- **Provider Failures**: Automatic fallback to secondary sources
- **Network Issues**: Redis cache serves as backup data source
- **Data Quality**: Reduced confidence scores for degraded data

### Monitoring & Alerts
- **Provider Health**: Real-time availability monitoring
- **Cache Performance**: Hit rates and response times
- **Price Anomalies**: Unusual price movements or deviations

## Development Status

### ✅ Completed Features
- Multi-provider price aggregation with Chainlink, CoinGecko, and Binance
- Comprehensive validation pipeline with anomaly detection
- Redis caching with primary/fallback strategy
- HTTP API endpoints for all price operations
- Price alerts system for user notifications
- Real-time monitoring and health checks
- Circuit breaker pattern for resilience
- **Dynamic Pricing Logic** with full modular architecture
- **Comprehensive Fee Calculator** with detailed breakdown
- **Hybrid Exchange Rate Calculator** (Oracle + 1inch)
- **Price Impact Analysis** with recommendations
- **Adaptive Slippage Protection** with market analysis
- **Bridge Quote API** with complete pricing information
- **Constants-based Configuration** for all pricing parameters

### 🔄 Current Limitations
- Chainlink integration uses development mocks (production requires Web3 integration)
- Historical price data storage not implemented
- Advanced ML-based anomaly detection not implemented
- Volume-based discount calculation requires historical data
- Real-time WebSocket price feeds not implemented

### 🔗 1inch Fusion+ Integration

The Price Oracle system is now fully integrated with 1inch Fusion+ for swap optimization and price analysis:

#### Price Comparison Service
- **Real-time comparison**: Automatic comparison of 1inch quotes with oracle data
- **Efficiency Analysis**: Calculation of efficiency scores based on price, gas, slippage, and confidence
- **Recommendation Engine**: Intelligent recommendations (HighlyRecommended, Recommended, Neutral, NotRecommended)
- **Risk Assessment**: Transaction execution risk analysis considering market conditions

#### Intelligent Routing Support
- **Oracle-optimized routing**: Using oracle data to select optimal routing paths
- **Multi-criteria optimization**: Combining oracle prices with 1inch routing for maximum efficiency
- **Price deviation analysis**: Monitoring 1inch price deviations from oracle consensus

#### Bridge Integration
- **Cross-chain price optimization**: Using oracle data to optimize cross-chain swap operations
- **Slippage protection**: Dynamic slippage adjustment based on oracle volatility data
- **Cost-benefit analysis**: Comparing bridge operation costs with market rates

### 🚀 Phase 6.3 - Dynamic Pricing Logic (COMPLETED)

The Price Oracle system has been extended with a comprehensive Dynamic Pricing Logic module that provides intelligent, market-aware pricing for bridge operations:

#### Dynamic Pricing Components

**DynamicPricingService** (`/backend/src/dynamic_pricing/mod.rs`)
- Main orchestrator coordinating all pricing components
- Integrates with Price Oracle and 1inch services
- Provides unified interface for bridge quote generation
- Manages service health and graceful degradation

**PricingAlgorithm** (`/backend/src/dynamic_pricing/algorithm.rs`)
- Core pricing algorithm with volatility adjustments
- Market condition analysis and cross-chain rate calculations
- Dynamic fee adjustments based on network conditions
- Comprehensive TODO framework for future ML integration

**FeeCalculator** (`/backend/src/dynamic_pricing/fee_calculator.rs`)
- Detailed fee breakdown calculation
- Components: Base fees, Gas fees, Protocol fees, Slippage protection
- Volume-based discount support
- Real-time gas price integration

**ExchangeRateCalculator** (`/backend/src/dynamic_pricing/exchange_rates.rs`)
- Hybrid exchange rate calculation (Oracle + 1inch)
- Weighted optimization with confidence scoring
- Historical rate support and volatility analysis
- Multi-source rate aggregation

**PriceImpactAnalyzer** (`/backend/src/dynamic_pricing/impact_analyzer.rs`)
- Price impact analysis for large transactions
- Liquidity assessment and market depth analysis
- Recommendation generation based on impact levels
- Risk assessment integration

**SlippageController** (`/backend/src/dynamic_pricing/slippage_control.rs`)
- Adaptive slippage protection mechanisms
- Market volatility analysis and dynamic adjustment
- Protection level recommendations
- Timeout and recovery strategies

#### Bridge Quote API

**GET /api/bridge/quote** - Comprehensive bridge quote generation
```json
{
  "quote_id": "uuid",
  "from_token": "ETH",
  "to_token": "NEAR",
  "from_amount": "1.000000000000000000",
  "to_amount": "45.234567890123456789",
  "fee_breakdown": {
    "base_fee": "0.001500000000000000",
    "gas_fee": "0.002134567890123456",
    "protocol_fee": "0.000750000000000000",
    "slippage_protection_fee": "0.000500000000000000",
    "total_fee_amount": "0.004884567890123456",
    "fee_percentage": 0.488,
    "fee_currency": "ETH"
  },
  "exchange_rate": {
    "rate": "45.234567890123456789",
    "rate_source": "hybrid_oracle_oneinch",
    "confidence_score": 0.85,
    "volatility_indicator": 0.12
  },
  "price_impact": {
    "impact_percentage": 0.15,
    "impact_category": "Low",
    "liquidity_score": 0.92,
    "market_depth_score": 0.88,
    "recommendation": "Proceed"
  },
  "slippage_protection": {
    "recommended_slippage": 0.5,
    "maximum_slippage": 2.0,
    "protection_level": "Standard",
    "timeout_seconds": 300
  }
}
```

### 🎯 Future Enhancements
- Real-time price feeds via WebSocket
- Advanced ML-based price prediction
- Production Chainlink contract integration
- Historical price analysis and trends

## Security Considerations

- **API Key Management**: Secure storage and rotation of provider API keys
- **Rate Limiting**: Compliance with provider rate limits and quotas
- **Data Validation**: Comprehensive input validation and sanitization
- **Cache Security**: Encrypted storage of sensitive pricing data
- **Audit Logging**: Complete audit trail of all price operations

## Performance Metrics

- **Response Time**: < 100ms for cached prices, < 500ms for fresh data
- **Availability**: 99.9% uptime with fallback mechanisms
- **Accuracy**: < 0.1% deviation from market consensus
- **Cache Hit Rate**: > 85% for frequently requested pairs

---

*This comprehensive Price Oracle and Dynamic Pricing system provides the foundation for accurate, intelligent, and market-aware pricing in KEMBridge's cross-chain bridge operations. The system ensures optimal swap execution, fair fee calculation, and superior user experience through advanced price aggregation, dynamic pricing algorithms, and comprehensive risk assessment.*