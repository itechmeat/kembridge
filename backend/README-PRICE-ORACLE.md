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

### 🔄 Current Limitations
- Chainlink integration uses development mocks (production requires Web3 integration)
- Price impact calculations are simplified (requires liquidity data)
- Historical price data storage not implemented
- Advanced ML-based anomaly detection not implemented

### 🚀 Next Steps (Phase 6.2 & 6.3)
- 1inch Fusion+ integration for optimal routing
- Dynamic pricing logic for bridge operations
- Real-time price feeds via WebSocket
- Advanced slippage protection mechanisms
- Production Chainlink contract integration

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

*This Price Oracle System provides the foundation for accurate and reliable pricing data in KEMBridge's cross-chain bridge operations, ensuring optimal swap execution and user experience.*