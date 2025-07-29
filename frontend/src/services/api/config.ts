/**
 * API Configuration
 * Centralized configuration for backend API communication
 */

// Use constants from global project settings
export const API_CONFIG = {
  // Backend URL - from constants.rs DEFAULT_SERVER_URL
  BASE_URL: import.meta.env.VITE_API_BASE_URL || "http://localhost:4000",

  // Frontend URL - for CORS
  FRONTEND_URL: "http://localhost:4100",

  // API version - from constants.rs API_V1_PREFIX
  VERSION: "/api/v1",

  // Timeouts - from constants.rs HTTP_CLIENT_TIMEOUT
  TIMEOUT: 30000, // 30 seconds

  // Retry configuration
  MAX_RETRIES: 3,
  RETRY_DELAY: 1000, // 1 second

  // Auth token storage
  TOKEN_STORAGE_KEY: "kembridge_auth_token",

  // WebSocket URL for real-time monitoring
  WS_URL: import.meta.env.VITE_WS_URL || "ws://localhost:4000/ws",
} as const;
// API endpoints - following the backend routes structure
export const API_ENDPOINTS = {
  // Health endpoints
  HEALTH: "/health",
  READY: "/ready",

  // Authentication endpoints - from auth.rs routes
  AUTH: {
    NONCE: "/auth/nonce",
    VERIFY_WALLET: "/auth/verify-wallet",
    REFRESH: "/auth/refresh",
  },

  // User endpoints - from user.rs routes
  USER: {
    PROFILE: "/user/profile",
    UPDATE_PROFILE: "/user/profile",
  },

  // Bridge endpoints - from bridge.rs routes
  BRIDGE: {
    INIT_SWAP: "/bridge/swap",
    STATUS: "/bridge/status",
    QUOTE: "/bridge/quote",
    HISTORY: "/bridge/history",
    SUPPORTED_TOKENS: "/bridge/tokens",
  },

  // Quantum crypto endpoints - from quantum.rs routes
  QUANTUM: {
    GENERATE_KEYS: "/crypto/generate-keys",
    ENCAPSULATE: "/crypto/encapsulate",
    DECAPSULATE: "/crypto/decapsulate",
  },

  // Risk analysis endpoints - from risk.rs routes
  RISK: {
    ANALYZE: "/risk/analyze",
    PROFILE: "/risk/profile",
    BLACKLIST_CHECK: "/risk/blacklist/check",
    THRESHOLDS: "/risk/thresholds",
  },

  // Price oracle endpoints - from price_oracle.rs routes
  PRICE: {
    QUOTE: "/price/quote",
    SUPPORTED_TOKENS: "/price/supported-tokens",
    HEALTH: "/price/health",
  },

  // WebSocket endpoints
  WEBSOCKET: {
    CONNECT: "/ws",
    MONITORING: "/ws/monitoring",
  },
} as const;

// Error codes - from constants.rs
export const API_ERROR_CODES = {
  INVALID_TOKEN: "Invalid or expired token",
  UNAUTHORIZED: "Unauthorized access",
  INTERNAL_SERVER: "Internal server error",
  BAD_REQUEST: "Bad request",
  NOT_FOUND: "Resource not found",
} as const;

// HTTP methods
export const HTTP_METHODS = {
  GET: "GET",
  POST: "POST",
  PUT: "PUT",
  DELETE: "DELETE",
  PATCH: "PATCH",
} as const;
