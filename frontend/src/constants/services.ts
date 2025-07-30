/**
 * Service URLs and Configuration Constants
 * Based on microservices architecture
 */

// Service URLs (matching docker-compose and production setup)
export const SERVICE_URLS = {
  // Main backend gateway (from kembridge-common config)
  BACKEND_GATEWAY: import.meta.env.VITE_BACKEND_URL || "http://localhost:4000",
  
  // Microservices (from kembridge-common GatewayConfig defaults)
  AUTH_SERVICE: import.meta.env.VITE_AUTH_SERVICE_URL || "http://localhost:4004",
  CRYPTO_SERVICE: import.meta.env.VITE_CRYPTO_SERVICE_URL || "http://localhost:4003", 
  BLOCKCHAIN_SERVICE: import.meta.env.VITE_BLOCKCHAIN_SERVICE_URL || "http://localhost:4002",
  ONEINCH_SERVICE: import.meta.env.VITE_ONEINCH_SERVICE_URL || "http://localhost:4001",
  
  // AI Engine (from ai-engine/config.py)
  AI_ENGINE: import.meta.env.VITE_AI_ENGINE_URL || "http://localhost:4005",
  
  // Frontend (from CLAUDE.md and CORS config)
  FRONTEND_DEV: "http://localhost:4100",
  FRONTEND_DOCKER: "http://localhost:4010",
} as const;

// Risk Analysis Constants (from ai-engine/config.py)
export const RISK_ANALYSIS = {
  THRESHOLDS: {
    LOW: 0.3,
    MEDIUM: 0.6,
    HIGH: 0.8,
    AUTO_BLOCK: 0.9,
  },
  CONFIDENCE: {
    HIGH: 0.9,
    MEDIUM: 0.7,
    LOW: 0.5,
  },
  LEVELS: {
    LOW: 'low',
    MEDIUM: 'medium', 
    HIGH: 'high',
  }
} as const;

// Request Configuration (from kembridge-common config)
export const REQUEST_CONFIG = {
  TIMEOUT_MS: 30000,
  MAX_RETRIES: 3,
  RETRY_DELAY_MS: 1000,
} as const;

// Default User ID for anonymous risk analysis
export const DEFAULT_USER_ID = "anonymous_user";

// Zero address constant (common in blockchain apps)
export const ZERO_ADDRESS = "0x0000000000000000000000000000000000000000";

// Quantum Security Constants
export const QUANTUM_SECURITY = {
  ALGORITHM: "ML-KEM-1024",
  KEY_ROTATION_INTERVAL_MS: 7 * 24 * 60 * 60 * 1000, // 7 days
  ENCRYPTION_STRENGTH: 1024,
} as const;

// API Versioning
export const API_VERSION = "/api/v1";

// WebSocket Configuration
export const WEBSOCKET_CONFIG = {
  URL: import.meta.env.VITE_WS_URL || "ws://localhost:4000/ws",
  RECONNECT_INTERVAL_MS: 5000,
  MAX_RECONNECT_ATTEMPTS: 10,
  PING_INTERVAL_MS: 30000,
} as const;

// Bridge Configuration
export const BRIDGE_CONFIG = {
  SUPPORTED_CHAINS: ['ethereum', 'near'] as const,
  DEFAULT_FROM_CHAIN: 'ethereum' as const,
  DEFAULT_TO_CHAIN: 'near' as const,
  DEFAULT_SLIPPAGE: 0.5, // 0.5%
  MAX_SLIPPAGE: 5.0, // 5%
  MIN_SLIPPAGE: 0.1, // 0.1%
} as const;

// Error Messages
export const ERROR_MESSAGES = {
  AI_ENGINE_OFFLINE: "AI Risk Engine is temporarily unavailable",
  RISK_ANALYSIS_FAILED: "Risk analysis failed",
  TRANSACTION_BLOCKED: "Transaction blocked by risk analysis",
  NETWORK_ERROR: "Network connection error",
  UNAUTHORIZED: "Unauthorized access",
  INVALID_INPUT: "Invalid input data",
} as const;

// Status Messages
export const STATUS_MESSAGES = {
  ANALYZING: "Analyzing risk...",
  APPROVED: "Transaction approved",
  BLOCKED: "Transaction blocked",
  READY: "Ready for analysis",
} as const;