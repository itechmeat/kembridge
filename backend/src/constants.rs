// Global constants for KEMBridge backend

// Server configuration
pub const DEFAULT_SERVER_HOST: &str = "0.0.0.0";
pub const DEFAULT_SERVER_PORT: u16 = 4000;
pub const DEFAULT_SERVER_URL: &str = "http://localhost:4000";

// Database defaults
pub const DEFAULT_DB_MAX_CONNECTIONS: u32 = 10;
pub const DEFAULT_DB_MIN_CONNECTIONS: u32 = 1;

// Redis defaults
pub const DEFAULT_REDIS_POOL_SIZE: u32 = 10;

// AI Engine defaults
pub const DEFAULT_AI_ENGINE_TIMEOUT_MS: u64 = 5000;
pub const DEFAULT_AI_ENGINE_MAX_RETRIES: u32 = 3;

// Risk analysis thresholds
pub const DEFAULT_RISK_LOW_THRESHOLD: f64 = 0.3;
pub const DEFAULT_RISK_MEDIUM_THRESHOLD: f64 = 0.6;
pub const DEFAULT_RISK_HIGH_THRESHOLD: f64 = 0.8;
pub const DEFAULT_RISK_AUTO_BLOCK_THRESHOLD: f64 = 0.9;
pub const DEFAULT_RISK_MANUAL_REVIEW_THRESHOLD: f64 = 0.7;

// Monitoring defaults
pub const DEFAULT_TRACING_LEVEL: &str = "debug";
pub const DEFAULT_METRICS_ENDPOINT: &str = "/metrics";

// Blockchain defaults
pub const DEFAULT_NEAR_NETWORK_ID: &str = "testnet";
pub const DEFAULT_ETHEREUM_CHAIN_ID: u64 = 11155111; // Sepolia testnet
pub const DEFAULT_ETHEREUM_GAS_PRICE_MULTIPLIER: f64 = 1.2;
pub const DEFAULT_ETHEREUM_CONFIRMATION_BLOCKS: u64 = 2;

// CORS origins - Frontend and API
pub const CORS_ORIGIN_FRONTEND_DEV: &str = "http://localhost:4001";
pub const CORS_ORIGIN_API_DEV: &str = "http://localhost:4000";

// Database
pub const DEFAULT_DATABASE_URL: &str = "postgresql://kembridge:kembridge@localhost:5432/kembridge";

// Redis
pub const DEFAULT_REDIS_URL: &str = "redis://localhost:6379";

// Blockchain RPC URLs
pub const DEFAULT_ETHEREUM_RPC_URL: &str = "https://sepolia.infura.io/v3/YOUR_PROJECT_ID";
pub const DEFAULT_NEAR_RPC_URL: &str = "https://rpc.testnet.near.org";
pub const DEFAULT_NEAR_HELPER_URL: &str = "https://helper.testnet.near.org";
pub const DEFAULT_NEAR_EXPLORER_URL: &str = "https://explorer.testnet.near.org";
pub const DEFAULT_NEAR_WALLET_URL: &str = "https://wallet.testnet.near.org";

// AI Engine
pub const DEFAULT_AI_ENGINE_URL: &str = "http://localhost:4003";

// JWT
pub const JWT_SECRET_KEY: &str = "dev-secret-change-in-production";
pub const JWT_EXPIRATION_HOURS: u64 = 24;

// API endpoints
pub const API_V1_PREFIX: &str = "/api/v1";

// Error messages
pub const ERROR_INVALID_TOKEN: &str = "Invalid or expired token";
pub const ERROR_UNAUTHORIZED: &str = "Unauthorized access";
pub const ERROR_INTERNAL_SERVER: &str = "Internal server error";
pub const ERROR_BAD_REQUEST: &str = "Bad request";
pub const ERROR_NOT_FOUND: &str = "Resource not found";

// Success messages
pub const SUCCESS_OPERATION_COMPLETED: &str = "Operation completed successfully";

// Cache TTL (in seconds)
pub const CACHE_TTL_SHORT: u64 = 300;   // 5 minutes
pub const CACHE_TTL_MEDIUM: u64 = 1800; // 30 minutes
pub const CACHE_TTL_LONG: u64 = 3600;   // 1 hour

// Pagination
pub const DEFAULT_PAGE_SIZE: usize = 20;
pub const MAX_PAGE_SIZE: usize = 100;

// Validation
pub const MIN_PASSWORD_LENGTH: usize = 8;
pub const MAX_USERNAME_LENGTH: usize = 50;

// Timeouts (in seconds)
pub const HTTP_CLIENT_TIMEOUT: u64 = 30;
pub const DATABASE_TIMEOUT: u64 = 10;

// Price Oracle specific
pub const PRICE_CACHE_TTL: u64 = 60;     // 1 minute
pub const PRICE_STALE_THRESHOLD: i64 = 300; // 5 minutes

// External API URLs
pub const COINGECKO_API_BASE: &str = "https://api.coingecko.com/api/v3";
pub const COINGECKO_PRO_API_BASE: &str = "https://pro-api.coingecko.com/api/v3";
pub const BINANCE_API_BASE: &str = "https://api.binance.com/api/v3";

// WebSocket
pub const WEBSOCKET_PING_INTERVAL: u64 = 30; // seconds
pub const WEBSOCKET_MAX_CONNECTIONS: usize = 1000;

// Price Oracle confidence levels
pub const PRICE_CONFIDENCE_DEFAULT: f64 = 0.85;
pub const PRICE_CONFIDENCE_CHAINLINK: f64 = 0.95;
pub const PRICE_CONFIDENCE_BINANCE: f64 = 0.90;
pub const PRICE_CONFIDENCE_COINGECKO: f64 = 0.85;

// Risk client configuration
pub const RISK_CLIENT_POOL_IDLE_TIMEOUT_SEC: u64 = 30;
pub const RISK_CLIENT_POOL_MAX_IDLE_PER_HOST: usize = 10;
pub const RISK_CLIENT_HEALTH_CHECK_TIMEOUT_SEC: u64 = 5;
pub const RISK_CLIENT_RETRY_BASE_DELAY_MS: u64 = 100;

// Server configuration
pub const REQUEST_BODY_LIMIT_BYTES: usize = 1024 * 1024; // 1MB

// Swagger UI configuration
pub const SWAGGER_UI_VERSION: &str = "5.17.14";
pub const SWAGGER_UI_CDN_BASE: &str = "https://unpkg.com/swagger-ui-dist@";

// Default tracing filter
pub const DEFAULT_TRACING_FILTER: &str = "kembridge_backend=debug,tower_http=debug";

// Authentication constants
pub const BEARER_PREFIX: &str = "Bearer ";
pub const BEARER_PREFIX_LENGTH: usize = 7;

// User tier constants
pub const USER_TIER_ADMIN: &str = "admin";
pub const USER_TIER_PREMIUM: &str = "premium";
pub const USER_TIER_FREE: &str = "free";

// Admin wallet prefixes
pub const ADMIN_WALLET_PREFIX_1: &str = "0x000";
pub const ADMIN_WALLET_PREFIX_2: &str = "admin";

// Premium tier criteria
pub const PREMIUM_WALLET_SUFFIX: &str = "premium";
pub const PREMIUM_WALLET_MIN_LENGTH: usize = 42;

// Price oracle defaults
pub const PRICE_IMPACT_DEFAULT: f64 = 0.1; // 0.1% price impact
pub const PRICE_QUOTE_EXPIRY_SECONDS: i64 = 30;

// Cache statistics defaults (temporary)
pub const CACHE_TOTAL_KEYS_DEFAULT: usize = 15;
pub const CACHE_PRIMARY_PRICES_DEFAULT: usize = 5;
pub const CACHE_FALLBACK_PRICES_DEFAULT: usize = 5;
pub const CACHE_PROVIDER_PRICES_DEFAULT: usize = 5;
pub const CACHE_HIT_RATE_DEFAULT: f64 = 0.85;

// 1inch Fusion+ API configuration
pub const ONEINCH_SWAP_API_BASE: &str = "https://api.1inch.dev/swap/v6.0";
pub const ONEINCH_FUSION_API_BASE: &str = "https://api.1inch.dev/fusion";
pub const ONEINCH_FUSION_API_VERSION: &str = "v1.0";
pub const ONEINCH_FUSION_TIMEOUT_SEC: u64 = 30;
pub const ONEINCH_FUSION_MAX_RETRIES: u32 = 3;
pub const ONEINCH_FUSION_RETRY_DELAY_MS: u64 = 1000;

// 1inch Fusion+ swap parameters
pub const ONEINCH_DEFAULT_SLIPPAGE: f64 = 0.5; // 0.5%
pub const ONEINCH_MAX_SLIPPAGE: f64 = 5.0; // 5%
pub const ONEINCH_MIN_SLIPPAGE: f64 = 0.1; // 0.1%
pub const ONEINCH_ORDER_TIMEOUT_SEC: u64 = 300; // 5 minutes

// 1inch supported networks
pub const ONEINCH_ETHEREUM_CHAIN_ID: u64 = 1;
pub const ONEINCH_BSC_CHAIN_ID: u64 = 56;
pub const ONEINCH_POLYGON_CHAIN_ID: u64 = 137;
pub const ONEINCH_AVALANCHE_CHAIN_ID: u64 = 43114;
pub const ONEINCH_ARBITRUM_CHAIN_ID: u64 = 42161;
pub const ONEINCH_OPTIMISM_CHAIN_ID: u64 = 10;

// 1inch testnet networks
pub const ONEINCH_SEPOLIA_CHAIN_ID: u64 = 11155111;
pub const ONEINCH_BSC_TESTNET_CHAIN_ID: u64 = 97;
pub const ONEINCH_POLYGON_MUMBAI_CHAIN_ID: u64 = 80001;

// 1inch order status
pub const ONEINCH_ORDER_STATUS_PENDING: &str = "pending";
pub const ONEINCH_ORDER_STATUS_FILLED: &str = "filled";
pub const ONEINCH_ORDER_STATUS_EXPIRED: &str = "expired";
pub const ONEINCH_ORDER_STATUS_CANCELLED: &str = "cancelled";
pub const ONEINCH_ORDER_STATUS_FAILED: &str = "failed";

// 1inch gas estimation
pub const ONEINCH_GAS_LIMIT_BUFFER: f64 = 1.2; // 20% buffer
pub const ONEINCH_DEFAULT_GAS_PRICE_GWEI: u64 = 20;

// Native token addresses for bridge integration
pub const ETHEREUM_NATIVE_TOKEN: &str = "0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE";
pub const BSC_NATIVE_TOKEN: &str = "0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE";
pub const POLYGON_NATIVE_TOKEN: &str = "0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE";
pub const AVALANCHE_NATIVE_TOKEN: &str = "0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE";
pub const ARBITRUM_NATIVE_TOKEN: &str = "0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE";
pub const OPTIMISM_NATIVE_TOKEN: &str = "0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE";
pub const NEAR_NATIVE_TOKEN: &str = "near";

// Bridge integration defaults
pub const BRIDGE_DEFAULT_OPTIMIZATION_STRATEGY: &str = "balanced";
pub const BRIDGE_MIN_OPTIMIZATION_THRESHOLD: f64 = 0.01; // 1% minimum improvement
pub const BRIDGE_MAX_SLIPPAGE_BRIDGE: f64 = 2.0; // 2% max slippage for bridge operations

// 1inch source identifiers
pub const ONEINCH_DEFAULT_SOURCE: &str = "kembridge";
pub const ONEINCH_BRIDGE_INTEGRATION_SOURCE: &str = "kembridge-bridge-integration";
pub const ONEINCH_ROUTING_SOURCE: &str = "kembridge-routing";

// 1inch execution probability base
pub const ONEINCH_EXECUTION_PROBABILITY_BASE: f64 = 0.9;

// ETH price fallback for gas calculations
pub const ONEINCH_ETH_PRICE_FALLBACK_USD: f64 = 2000.0;

// Ethereum zero address constant
pub const ETHEREUM_ZERO_ADDRESS: &str = "0x0000000000000000000000000000000000000000";

// Ethereum transaction constants
pub const ETHEREUM_BASE_GAS: u64 = 21000;
pub const ETHEREUM_WEI_MULTIPLIER: u64 = 1_000_000_000_000_000_000;

// API routes
pub const API_ROUTE_HEALTH: &str = "/health";
pub const API_ROUTE_READY: &str = "/ready";
pub const API_ROUTE_METRICS: &str = "/metrics";

// Dynamic Pricing Constants
pub const BRIDGE_ESTIMATED_EXECUTION_TIME_MINUTES: i64 = 15;
pub const BRIDGE_QUOTE_VALIDITY_MINUTES: i64 = 5;

// Bridge volatility adjustments
pub const BRIDGE_ATOMIC_VOLATILITY_ADJUSTMENT: f64 = 1.02; // 2% for atomic swaps
pub const BRIDGE_OPTIMISTIC_VOLATILITY_ADJUSTMENT: f64 = 1.01; // 1% for optimistic
pub const BRIDGE_CANONICAL_VOLATILITY_ADJUSTMENT: f64 = 1.005; // 0.5% for canonical

// Bridge cross-chain adjustments
pub const BRIDGE_ETH_TO_NEAR_ADJUSTMENT: f64 = 1.015; // 1.5% adjustment
pub const BRIDGE_NEAR_TO_ETH_ADJUSTMENT: f64 = 1.012; // 1.2% adjustment

// Bridge fee percentages
pub const BRIDGE_BASE_FEE_PERCENTAGE: f64 = 0.15; // 0.15% base fee
pub const BRIDGE_PROTOCOL_FEE_PERCENTAGE: f64 = 0.05; // 0.05% protocol fee
pub const BRIDGE_SLIPPAGE_PROTECTION_FEE_PERCENTAGE: f64 = 0.03; // 0.03% slippage protection

// Bridge gas estimation
pub const BRIDGE_ESTIMATED_GAS_UNITS: u64 = 150000;
pub const BRIDGE_DEFAULT_GAS_PRICE_GWEI: u64 = 25;

// Exchange rate constants
pub const EXCHANGE_RATE_DEFAULT_SLIPPAGE: f64 = 0.5; // 0.5% default slippage
pub const EXCHANGE_RATE_ORACLE_WEIGHT: f64 = 0.6; // 60% oracle weight
pub const EXCHANGE_RATE_ONEINCH_WEIGHT: f64 = 0.4; // 40% 1inch weight
pub const EXCHANGE_RATE_BASE_CONFIDENCE: f64 = 0.8; // 80% base confidence

// Exchange rate volatility indicators
pub const EXCHANGE_RATE_ETH_NEAR_VOLATILITY: f64 = 0.25; // 25% volatility
pub const EXCHANGE_RATE_ETH_USDT_VOLATILITY: f64 = 0.15; // 15% volatility  
pub const EXCHANGE_RATE_NEAR_USDT_VOLATILITY: f64 = 0.20; // 20% volatility
pub const EXCHANGE_RATE_DEFAULT_VOLATILITY: f64 = 0.30; // 30% default volatility

// Price impact thresholds
pub const PRICE_IMPACT_LOW_THRESHOLD: f64 = 0.5; // 0.5% low impact
pub const PRICE_IMPACT_MEDIUM_THRESHOLD: f64 = 2.0; // 2% medium impact
pub const PRICE_IMPACT_HIGH_THRESHOLD: f64 = 5.0; // 5% high impact
pub const PRICE_IMPACT_MAX_PERCENTAGE: f64 = 20.0; // 20% max impact

// Price impact liquidity estimates (in USD)
pub const PRICE_IMPACT_ETH_NEAR_LIQUIDITY: f64 = 500000.0; // $500k liquidity
pub const PRICE_IMPACT_ETH_USDT_LIQUIDITY: f64 = 2000000.0; // $2M liquidity
pub const PRICE_IMPACT_NEAR_USDT_LIQUIDITY: f64 = 300000.0; // $300k liquidity
pub const PRICE_IMPACT_DEFAULT_LIQUIDITY: f64 = 100000.0; // $100k default

// Price impact market depth defaults
pub const PRICE_IMPACT_DEFAULT_BID_DEPTH: f64 = 100000.0; // $100k bid depth
pub const PRICE_IMPACT_DEFAULT_ASK_DEPTH: f64 = 100000.0; // $100k ask depth
pub const PRICE_IMPACT_DEFAULT_SPREAD_PERCENTAGE: f64 = 0.1; // 0.1% spread

// Price impact fragmentation risk
pub const PRICE_IMPACT_ETH_NEAR_FRAGMENTATION: f64 = 0.3; // 30% fragmentation
pub const PRICE_IMPACT_ETH_USDT_FRAGMENTATION: f64 = 0.1; // 10% fragmentation
pub const PRICE_IMPACT_NEAR_USDT_FRAGMENTATION: f64 = 0.4; // 40% fragmentation
pub const PRICE_IMPACT_DEFAULT_FRAGMENTATION: f64 = 0.5; // 50% default
pub const PRICE_IMPACT_HIGH_FRAGMENTATION_THRESHOLD: f64 = 0.35; // 35% high fragmentation

// Price impact stability scores
pub const PRICE_IMPACT_HIGH_STABILITY: f64 = 0.9; // 90% stability
pub const PRICE_IMPACT_MEDIUM_STABILITY: f64 = 0.7; // 70% stability
pub const PRICE_IMPACT_LOW_STABILITY: f64 = 0.5; // 50% stability
pub const PRICE_IMPACT_DEFAULT_STABILITY: f64 = 0.6; // 60% default stability

// Slippage control constants
pub const SLIPPAGE_BASE_PERCENTAGE: f64 = 0.5; // 0.5% base slippage
pub const SLIPPAGE_MIN_PERCENTAGE: f64 = 0.1; // 0.1% minimum slippage
pub const SLIPPAGE_MAX_PERCENTAGE: f64 = 5.0; // 5% maximum slippage
pub const SLIPPAGE_WARNING_THRESHOLD: f64 = 3.0; // 3% warning threshold

// Slippage volatility factors
pub const SLIPPAGE_ETH_NEAR_VOLATILITY: f64 = 0.15; // 15% volatility
pub const SLIPPAGE_ETH_USDT_VOLATILITY: f64 = 0.05; // 5% volatility
pub const SLIPPAGE_NEAR_USDT_VOLATILITY: f64 = 0.10; // 10% volatility
pub const SLIPPAGE_DEFAULT_VOLATILITY: f64 = 0.20; // 20% default volatility

pub const SLIPPAGE_VOLATILITY_MULTIPLIER: f64 = 1.2; // 20% volatility multiplier
pub const SLIPPAGE_VOLATILITY_FACTOR: f64 = 2.0; // 2x volatility factor
pub const SLIPPAGE_DYNAMIC_THRESHOLD: f64 = 0.15; // 15% dynamic threshold
pub const SLIPPAGE_HIGH_VOLATILITY_THRESHOLD: f64 = 0.25; // 25% high volatility
pub const SLIPPAGE_HIGH_VOLATILITY_TIMEOUT_FACTOR: f64 = 0.5; // 50% timeout reduction

// Slippage trade size adjustments
pub const SLIPPAGE_LARGE_TRADE_THRESHOLD: f64 = 10000.0; // $10k large trade
pub const SLIPPAGE_MEDIUM_TRADE_THRESHOLD: f64 = 1000.0; // $1k medium trade
pub const SLIPPAGE_LARGE_TRADE_ADJUSTMENT: f64 = 0.5; // 0.5% large trade adjustment
pub const SLIPPAGE_MEDIUM_TRADE_ADJUSTMENT: f64 = 0.2; // 0.2% medium trade adjustment
pub const SLIPPAGE_SMALL_TRADE_ADJUSTMENT: f64 = 0.0; // 0% small trade adjustment

// Slippage protection level thresholds
pub const SLIPPAGE_BASIC_THRESHOLD: f64 = 0.5; // 0.5% basic threshold
pub const SLIPPAGE_STANDARD_THRESHOLD: f64 = 1.0; // 1% standard threshold
pub const SLIPPAGE_ADVANCED_THRESHOLD: f64 = 2.0; // 2% advanced threshold

// Slippage timeout settings (in minutes)
pub const SLIPPAGE_BASIC_TIMEOUT_MINUTES: i64 = 5; // 5 minutes basic
pub const SLIPPAGE_STANDARD_TIMEOUT_MINUTES: i64 = 10; // 10 minutes standard
pub const SLIPPAGE_ADVANCED_TIMEOUT_MINUTES: i64 = 15; // 15 minutes advanced
pub const SLIPPAGE_MAXIMUM_TIMEOUT_MINUTES: i64 = 30; // 30 minutes maximum
pub const SLIPPAGE_MIN_TIMEOUT_MINUTES: i64 = 2; // 2 minutes minimum

pub fn get_bridge_contract_address() -> Result<String, std::env::VarError> {
    std::env::var("BRIDGE_CONTRACT_ADDRESS")
}

pub fn get_bridge_contract_deployed_block() -> Result<u64, Box<dyn std::error::Error>> {
    let block_str = std::env::var("BRIDGE_CONTRACT_DEPLOYED_BLOCK")?;
    let block_num = block_str.parse::<u64>()?;
    Ok(block_num)
}
pub const BRIDGE_MIN_LOCK_AMOUNT_WEI: u64 = 1_000_000_000_000_000; // 0.001 ETH
pub const BRIDGE_MAX_LOCK_AMOUNT_WEI: u64 = 10_000_000_000_000_000_000; // 10 ETH

