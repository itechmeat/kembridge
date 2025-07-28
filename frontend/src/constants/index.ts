/**
 * Application constants and configuration
 */

// Environment and API configuration
export const CONFIG = {
  NODE_ENV: import.meta.env.MODE || "development",
  API_BASE_URL: import.meta.env.VITE_API_BASE_URL || "http://localhost:8080",
  IS_DEVELOPMENT: import.meta.env.DEV,
  IS_PRODUCTION: import.meta.env.PROD,
} as const;

// UI Constants
export const UI_CONFIG = {
  MODAL_SIZES: {
    SM: "sm",
    MD: "md",
    LG: "lg",
    XL: "xl",
  },
  BUTTON_VARIANTS: {
    PRIMARY: "primary",
    SECONDARY: "secondary",
    DANGER: "danger",
    SUCCESS: "success",
  },
  BUTTON_SIZES: {
    SM: "sm",
    MD: "md",
    LG: "lg",
  },
  SPINNER_SIZES: {
    SM: "sm",
    MD: "md",
    LG: "lg",
  },
} as const;

// Wallet Configuration
export const WALLET_CONFIG = {
  AUTO_CONNECT_DELAY: 500,
  CONNECTION_TIMEOUT: 30000,
  RECONNECT_ATTEMPTS: 3,
} as const;

// Error Messages
export const ERROR_MESSAGES = {
  WALLET_NOT_CONNECTED: "No wallet connected",
  WALLET_CONNECTION_FAILED: "Failed to connect wallet",
  AUTHENTICATION_FAILED: "Authentication failed",
  BACKEND_UNAVAILABLE: "Backend service unavailable",
  NETWORK_ERROR: "Network connection error",
  INSUFFICIENT_BALANCE: "Insufficient balance",
  TRANSACTION_FAILED: "Transaction failed",
  USER_REJECTED: "User rejected the request",
  UNKNOWN_ERROR: "An unknown error occurred",
} as const;

// Success Messages
export const SUCCESS_MESSAGES = {
  WALLET_CONNECTED: "Wallet connected successfully",
  AUTHENTICATION_SUCCESS: "Authentication successful",
  TRANSACTION_SUCCESS: "Transaction completed successfully",
  PROFILE_UPDATED: "Profile updated successfully",
} as const;

// Application Text
export const APP_TEXT = {
  TITLE: "KEMBridge",
  SUBTITLE: "Cross-Chain Intelligence Meets Quantum Security",
  DESCRIPTION:
    "The world's first quantum-secured cross-chain bridge powered by ML-KEM-1024 post-quantum cryptography and AI risk analysis.",
  BUTTONS: {
    CONNECT_WALLET: "Connect Wallet",
    DISCONNECT_WALLET: "Disconnect",
    LAUNCH_BRIDGE: "Launch Bridge",
    VIEW_DEMO: "View Demo",
    CONNECTING: "Connecting...",
    AUTHENTICATING: "Authenticating...",
  },
  STATUS: {
    QUANTUM_PROTECTION_ACTIVE: "Quantum Protection Active",
    BACKEND_CONNECTED: "Backend: Connected",
    BACKEND_DISCONNECTED: "Backend: Disconnected",
    WALLET_CONNECTED: "Wallet: Connected",
    WALLET_DISCONNECTED: "Wallet: Not Connected",
    AUTHENTICATED: "Auth: Authenticated",
    NOT_AUTHENTICATED: "Auth: Not Authenticated",
  },
} as const;

// Network Configuration
export const NETWORKS = {
  ETHEREUM_MAINNET: {
    chainId: 1,
    name: "Ethereum Mainnet",
    type: "ethereum" as const,
    rpcUrls: ["https://mainnet.infura.io/v3"],
    blockExplorerUrls: ["https://etherscan.io"],
    nativeCurrency: {
      name: "Ethereum",
      symbol: "ETH",
      decimals: 18,
    },
  },
  ETHEREUM_SEPOLIA: {
    chainId: 11155111,
    name: "Sepolia Testnet",
    type: "ethereum" as const,
    rpcUrls: ["https://sepolia.infura.io/v3"],
    blockExplorerUrls: ["https://sepolia.etherscan.io"],
    nativeCurrency: {
      name: "Ethereum",
      symbol: "ETH",
      decimals: 18,
    },
  },
  NEAR_MAINNET: {
    chainId: "mainnet",
    name: "NEAR Mainnet",
    type: "near" as const,
    rpcUrls: ["https://rpc.mainnet.near.org"],
    blockExplorerUrls: ["https://explorer.near.org"],
    nativeCurrency: {
      name: "NEAR",
      symbol: "NEAR",
      decimals: 24,
    },
  },
  NEAR_TESTNET: {
    chainId: "testnet",
    name: "NEAR Testnet",
    type: "near" as const,
    rpcUrls: ["https://rpc.testnet.near.org"],
    blockExplorerUrls: ["https://explorer.testnet.near.org"],
    nativeCurrency: {
      name: "NEAR",
      symbol: "NEAR",
      decimals: 24,
    },
  },
} as const;

// Features Data
export const FEATURES = [
  {
    id: "quantum-safe",
    icon: "🔐",
    title: "Quantum-Safe",
    description:
      "ML-KEM-1024 post-quantum cryptography protects against future quantum attacks",
  },
  {
    id: "ai-risk-engine",
    icon: "🤖",
    title: "AI Risk Engine",
    description:
      "Real-time transaction analysis with machine learning threat detection",
  },
  {
    id: "near-1click",
    icon: "⚡",
    title: "NEAR 1Click",
    description:
      "Simplified cross-chain swaps with atomic transaction guarantees",
  },
] as const;

// Statistics Data
export const STATS = [
  {
    id: "quantum-security",
    value: "256-bit",
    label: "Quantum Security",
  },
  {
    id: "bridge-time",
    value: "<2s",
    label: "Bridge Time",
  },
  {
    id: "uptime",
    value: "99.9%",
    label: "Uptime",
  },
  {
    id: "supported-chains",
    value: "2",
    label: "Supported Chains",
  },
] as const;

// Footer Links
export const FOOTER_LINKS = {
  protocol: [
    { title: "Documentation", url: "#docs" },
    { title: "Security", url: "#security" },
    { title: "Audits", url: "#audits" },
  ],
  community: [
    { title: "Discord", url: "#discord" },
    { title: "Twitter", url: "#twitter" },
    { title: "GitHub", url: "#github" },
  ],
} as const;

// Local Storage Keys
export const STORAGE_KEYS = {
  WALLET_TYPE: "kembridge_wallet_type",
  AUTH_TOKEN: "kembridge_auth_token",
  USER_PREFERENCES: "kembridge_user_preferences",
} as const;

// Animation Durations (in milliseconds)
export const ANIMATION_DURATION = {
  FAST: 150,
  NORMAL: 300,
  SLOW: 500,
} as const;
