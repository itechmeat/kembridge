/**
 * Global Test Constants - Centralized Configuration
 * All environment-specific values and test data in one place
 */

/**
 * Environment URLs and endpoints
 */
export const TEST_URLS = {
  // Frontend URLs
  FRONTEND: {
    LOCAL_DEV: "http://localhost:4100",
    STAGING: "https://staging.kembridge.io",
    PRODUCTION: "https://kembridge.io",
  },

  // Backend service URLs
  BACKEND: {
    GATEWAY: "http://localhost:4000",
    ONEINCH: "http://localhost:4001",
    BLOCKCHAIN: "http://localhost:4002",
    CRYPTO: "http://localhost:4003",
    AUTH: "http://localhost:4004",
    AI_ENGINE: "http://localhost:4005",
  },

  // WebSocket endpoints
  WEBSOCKET: {
    GATEWAY: "ws://localhost:4000/ws",
    FRONTEND: "ws://localhost:4100/ws",
    // For testing connection failures
    NON_EXISTENT: "ws://localhost:9999/ws",
  },

  // API endpoints
  API_BASE: "/api/v1",
  AUTH_ENDPOINT: "/api/v1/auth",
  BRIDGE_ENDPOINT: "/api/v1/bridge",
  TOKENS_ENDPOINT: "/api/v1/tokens",

  // External services
  ONEINCH_API: "https://api.1inch.dev",
  NEAR_RPC: "https://rpc.testnet.near.org",
  ETHEREUM_RPC: "https://eth-sepolia.g.alchemy.com",
} as const;

/**
 * Test environment configuration
 */
export const TEST_ENV = {
  // Current environment (can be overridden by env vars)
  CURRENT_BASE_URL: process.env.TEST_BASE_URL || TEST_URLS.FRONTEND.LOCAL_DEV,

  // Timeouts
  TIMEOUTS: {
    SHORT: 2000,
    MEDIUM: 5000,
    LONG: 10000,
    EXTRA_LONG: 30000,
    PAGE_LOAD: 60000,
  },

  // Retry configuration
  RETRY: {
    MAX_ATTEMPTS: 3,
    DELAY: 1000,
    EXPONENTIAL_BACKOFF: true,
  },

  // Browser configuration
  BROWSER: {
    HEADLESS: process.env.CI === "true",
    SLOW_MO: process.env.DEBUG === "true" ? 100 : 0,
    VIEWPORT: {
      width: 1280,
      height: 720,
    },
  },
} as const;

/**
 * Security testing constants
 */
export const SECURITY_TEST_DATA = {
  // JWT tokens for security testing
  JWT_TOKENS: {
    // Valid JWT structure but with invalid signature for testing
    TAMPERED: 'eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.invalidSignature',
    // Completely malformed token
    MALFORMED: 'invalid.token.here',
    // Expired token (for timing attack tests)
    EXPIRED: 'eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyLCJleHAiOjE1MTYyMzkwMjJ9.expiredSignature',
  },

  // XSS attack payloads
  XSS_PAYLOADS: [
    '<script>alert("XSS")</script>',
    '<img src="x" onerror="alert(\'XSS\')">', 
    'javascript:alert("XSS")',
    '<svg onload="alert(\'XSS\')">', 
    '"><script>alert("XSS")</script>',
    "';alert('XSS');//",
    '<iframe src="javascript:alert(\'XSS\')"></iframe>'
  ],

  // SQL injection payloads
  SQL_INJECTION_PAYLOADS: [
    "'; DROP TABLE users; --",
    "' OR '1'='1",
    "' UNION SELECT * FROM users --",
    "'; DELETE FROM transactions; --",
    "' OR 1=1 --",
    "admin'--",
    "' OR 'x'='x"
  ],
} as const;

/**
 * Test data constants
 */
export const TEST_DATA = {
  // Token symbols for testing
  TOKENS: {
    ETHEREUM: {
      NATIVE: "ETH",
      STABLECOINS: ["USDC"], // Only USDC is available in the app
      POPULAR: ["ETH", "USDC"], // Popular tokens that are actually available
      ALL: ["ETH", "USDC"], // Only ETH and USDC are actually available in the app
      // Commented out tokens that are not available in the current app version:
      // UNAVAILABLE: ["USDT", "DAI", "WETH", "LINK", "UNI"]
    },
    NEAR: {
      NATIVE: "NEAR",
      WRAPPED: "wNEAR", // May not be available, need to verify
      STABLECOINS: ["USDC.e", "USDT.e"], // May not be available, need to verify
      POPULAR: ["NEAR"], // Only NEAR confirmed to work
      ALL: ["NEAR"], // Only NEAR confirmed to work in current app version
      // Commented out tokens that may not be available:
      // UNVERIFIED: ["wNEAR", "USDC.e", "USDT.e", "AURORA", "OCT"]
    },
  },

  // Test amounts
  AMOUNTS: {
    MICRO: "0.001",
    SMALL: "0.01",
    MEDIUM: "0.1",
    LARGE: "1.0",
    EXTRA_LARGE: "10.0",
    PRECISION_TEST: "0.123456", // Reduced precision to avoid input issues
    MAX_DECIMALS: "1.123456", // More realistic decimal test
  },

  // Slippage values
  SLIPPAGE: {
    MINIMAL: 0.1,
    LOW: 0.5,
    MEDIUM: 1.0,
    HIGH: 3.0,
    EXTREME: 10.0,
  },

  // Invalid inputs for testing
  INVALID_INPUTS: {
    AMOUNTS: ["abc", "1.2.3", "-1", "", "0", "null", "undefined"],
    SPECIAL_CHARS: ["<script>", "${injection}", "../../etc/passwd"],
  },
} as const;

/**
 * Wallet configuration for testing
 */
export const WALLET_CONFIG = {
  // Mock wallet addresses
  ETHEREUM: {
    ADDRESS: "0x742d35Cc6634C0532925a3b8D4C9db96C4b4d8b6",
    PRIVATE_KEY: "0x" + "0".repeat(64), // Mock private key
  },
  NEAR: {
    ACCOUNT_ID: "test-account.testnet",
    PUBLIC_KEY: "ed25519:" + "A".repeat(44),
  },

  // Wallet connection states
  STATES: {
    DISCONNECTED: "disconnected",
    CONNECTING: "connecting",
    CONNECTED: "connected",
    ERROR: "error",
  },
} as const;

/**
 * API response patterns for validation
 */
export const API_PATTERNS = {
  // Expected API call patterns
  AUTH_CALLS: [
    "/api/v1/auth/nonce",
    "/api/v1/auth/verify",
    "/api/v1/auth/status",
  ],
  BRIDGE_CALLS: [
    "/api/v1/bridge/quote",
    "/api/v1/bridge/execute",
    "/api/v1/bridge/status",
  ],
  TOKEN_CALLS: [
    "/api/v1/tokens/list",
    "/api/v1/tokens/balance",
    "/api/v1/tokens/price",
  ],

  // Response validation patterns
  RESPONSE_SCHEMAS: {
    QUOTE: {
      required: ["fromAmount", "toAmount", "rate", "fees"],
      optional: ["priceImpact", "estimatedGas"],
    },
    TOKEN: {
      required: ["symbol", "name", "decimals", "address"],
      optional: ["logoURI", "balance", "price"],
    },
  },
} as const;

/**
 * Error messages and validation
 */
export const ERROR_MESSAGES = {
  // Expected error messages
  WALLET: {
    NOT_CONNECTED: "Wallet not connected",
    INSUFFICIENT_BALANCE: "Insufficient balance",
    TRANSACTION_REJECTED: "Transaction rejected",
  },
  BRIDGE: {
    INVALID_AMOUNT: "Invalid amount",
    UNSUPPORTED_TOKEN: "Token not supported",
    SLIPPAGE_TOO_HIGH: "Slippage tolerance too high",
  },
  NETWORK: {
    CONNECTION_FAILED: "Network connection failed",
    TIMEOUT: "Request timeout",
    RATE_LIMITED: "Rate limit exceeded",
  },
} as const;

/**
 * Test selectors and UI elements
 */
export const UI_ELEMENTS = {
  // Loading states
  LOADING_INDICATORS: [
    ".spinner",
    ".loading",
    '[data-testid="loading"]',
    ".auth-manager__loading",
  ],

  // Error states
  ERROR_INDICATORS: [
    ".error-message",
    ".auth-manager__error",
    '[data-testid="error"]',
    ".toast-error",
  ],

  // Success states
  SUCCESS_INDICATORS: [
    ".success-message",
    '[data-testid="success"]',
    ".toast-success",
  ],
} as const;

/**
 * Performance benchmarks
 */
export const PERFORMANCE = {
  // Expected load times (in milliseconds)
  PAGE_LOAD: {
    FAST: 1000,
    ACCEPTABLE: 3000,
    SLOW: 5000,
  },

  // API response times
  API_RESPONSE: {
    FAST: 500,
    ACCEPTABLE: 2000,
    SLOW: 5000,
  },

  // Quote generation times
  QUOTE_GENERATION: {
    FAST: 1000,
    ACCEPTABLE: 3000,
    SLOW: 10000,
  },
} as const;

/**
 * Feature flags for conditional testing
 */
export const FEATURE_FLAGS = {
  // Features that might be disabled in test environment
  REAL_WALLET_CONNECTION: process.env.ENABLE_REAL_WALLETS === "true",
  LIVE_PRICE_FEEDS: process.env.ENABLE_LIVE_PRICES === "true",
  WEBSOCKET_TESTING: process.env.ENABLE_WEBSOCKET_TESTS === "true",
  PERFORMANCE_TESTING: process.env.ENABLE_PERFORMANCE_TESTS === "true",

  // Test modes
  MOCK_MODE: process.env.TEST_MODE === "mock",
  INTEGRATION_MODE: process.env.TEST_MODE === "integration",
  E2E_MODE: process.env.TEST_MODE === "e2e",
} as const;

/**
 * Utility functions for test constants
 */
export const TEST_UTILS = {
  /**
   * Get current base URL based on environment
   */
  getBaseUrl: (): string => {
    return TEST_ENV.CURRENT_BASE_URL;
  },

  /**
   * Build full URL from path
   */
  buildUrl: (path: string): string => {
    const baseUrl = TEST_UTILS.getBaseUrl();
    return `${baseUrl}${path.startsWith("/") ? path : "/" + path}`;
  },

  /**
   * Get API endpoint URL
   */
  getApiUrl: (endpoint: string): string => {
    return TEST_UTILS.buildUrl(`${TEST_URLS.API_BASE}${endpoint}`);
  },

  /**
   * Check if feature is enabled
   */
  isFeatureEnabled: (feature: keyof typeof FEATURE_FLAGS): boolean => {
    return FEATURE_FLAGS[feature] === true;
  },

  /**
   * Get timeout based on environment
   */
  getTimeout: (type: keyof typeof TEST_ENV.TIMEOUTS): number => {
    const baseTimeout = TEST_ENV.TIMEOUTS[type];
    // Increase timeouts in CI environment
    return process.env.CI === "true" ? baseTimeout * 2 : baseTimeout;
  },

  /**
   * Get random test amount
   */
  getRandomAmount: (): string => {
    const amounts = Object.values(TEST_DATA.AMOUNTS);
    return amounts[Math.floor(Math.random() * amounts.length)];
  },

  /**
   * Get random token pair
   */
  getRandomTokenPair: (): { from: string; to: string } => {
    const ethTokens = TEST_DATA.TOKENS.ETHEREUM.ALL;
    const nearTokens = TEST_DATA.TOKENS.NEAR.ALL;

    return {
      from: ethTokens[Math.floor(Math.random() * ethTokens.length)],
      to: nearTokens[Math.floor(Math.random() * nearTokens.length)],
    };
  },
} as const;

/**
 * Export all constants as default for easy importing
 */
export default {
  TEST_URLS,
  TEST_ENV,
  TEST_DATA,
  SECURITY_TEST_DATA,
  WALLET_CONFIG,
  API_PATTERNS,
  ERROR_MESSAGES,
  UI_ELEMENTS,
  PERFORMANCE,
  FEATURE_FLAGS,
  TEST_UTILS,
} as const;
