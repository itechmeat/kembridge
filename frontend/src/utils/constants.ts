// Application constants

import { Network } from "../types";

export const SUPPORTED_NETWORKS: Network[] = [
  {
    id: "ethereum",
    name: "Ethereum",
    chainId: 1,
    rpcUrl: "https://mainnet.infura.io/v3/YOUR_INFURA_KEY",
    blockExplorer: "https://etherscan.io",
    nativeCurrency: {
      name: "Ethereum",
      symbol: "ETH",
      decimals: 18,
    },
  },
  {
    id: "ethereum-sepolia",
    name: "Ethereum Sepolia",
    chainId: 11155111,
    rpcUrl: "https://sepolia.infura.io/v3/YOUR_INFURA_KEY",
    blockExplorer: "https://sepolia.etherscan.io",
    nativeCurrency: {
      name: "Ethereum",
      symbol: "ETH",
      decimals: 18,
    },
  },
  {
    id: "near",
    name: "NEAR Protocol",
    chainId: 0, // NEAR doesn't use chainId
    rpcUrl: "https://rpc.mainnet.near.org",
    blockExplorer: "https://explorer.near.org",
    nativeCurrency: {
      name: "NEAR",
      symbol: "NEAR",
      decimals: 24,
    },
  },
  {
    id: "near-testnet",
    name: "NEAR Testnet",
    chainId: 0,
    rpcUrl: "https://rpc.testnet.near.org",
    blockExplorer: "https://explorer.testnet.near.org",
    nativeCurrency: {
      name: "NEAR",
      symbol: "NEAR",
      decimals: 24,
    },
  },
];

export const DEFAULT_NETWORK = SUPPORTED_NETWORKS[1]; // Ethereum Sepolia for development

export const API_ENDPOINTS = {
  SWAP_QUOTE: "/api/v1/swap/quote",
  SWAP_EXECUTE: "/api/v1/swap/execute",
  SWAP_STATUS: "/api/v1/swap/status",
  PRICE_DATA: "/api/v1/price",
  USER_PROFILE: "/api/v1/user/profile",
  HEALTH: "/api/v1/health",
} as const;

export const WALLET_CONNECT_PROJECT_ID = "YOUR_WALLET_CONNECT_PROJECT_ID";

export const QUERY_KEYS = {
  WALLET_BALANCE: "wallet-balance",
  SWAP_QUOTE: "swap-quote",
  SWAP_STATUS: "swap-status",
  PRICE_DATA: "price-data",
  USER_PROFILE: "user-profile",
  TRANSACTION_HISTORY: "transaction-history",
} as const;

export const STORAGE_KEYS = {
  WALLET_CONNECTION: "kembridge-wallet-connection",
  USER_PREFERENCES: "kembridge-user-preferences",
  THEME: "kembridge-theme",
} as const;

export const THEME_COLORS = {
  PRIMARY: "#0066CC", // Quantum Blue
  SECONDARY: "#00AA44", // Secure Green
  ACCENT: "#FF6600", // Energy Orange
  NEUTRAL: "#F5F7FA", // Modern Gray
  WARNING: "#FF3366", // Alert Red
} as const;

export const BREAKPOINTS = {
  MOBILE: 320,
  TABLET: 768,
  DESKTOP: 1024,
  WIDE: 1440,
} as const;

export const ANIMATION_DURATION = {
  FAST: 150,
  NORMAL: 300,
  SLOW: 500,
} as const;

export const SWAP_SETTINGS = {
  DEFAULT_SLIPPAGE: 0.5, // 0.5%
  MAX_SLIPPAGE: 10, // 10%
  MIN_SLIPPAGE: 0.1, // 0.1%
  DEFAULT_DEADLINE: 20, // 20 minutes
} as const;
