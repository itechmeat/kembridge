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

