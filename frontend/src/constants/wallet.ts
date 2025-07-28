/**
 * Wallet-specific constants and metadata
 */

import { WalletType } from "../services/wallet/types";

// Wallet Display Metadata
console.log("📦 Constants: Initializing WALLET_METADATA");
export const WALLET_METADATA = {
  [WalletType.METAMASK]: {
    name: "MetaMask",
    icon: "🦊",
    description: "Connect using MetaMask browser extension",
    bundleUrl: null, // Built-in
    downloadUrl: "https://metamask.io/download/",
    supportedChains: ["ethereum"],
  },
  [WalletType.NEAR]: {
    name: "NEAR Wallet",
    icon: "🔷",
    description: "Connect using NEAR Wallet",
    bundleUrl: () => import("../services/wallet/providers/near"),
    downloadUrl: "https://wallet.near.org/",
    supportedChains: ["near"],
  },
  [WalletType.WALLET_CONNECT]: {
    name: "WalletConnect",
    icon: "📱",
    description: "Connect using mobile wallet app",
    bundleUrl: () => import("../services/wallet/providers/walletconnect"),
    downloadUrl: "https://walletconnect.com/registry",
    supportedChains: ["ethereum", "near"],
  },
  [WalletType.COINBASE]: {
    name: "Coinbase Wallet",
    icon: "🔷",
    description: "Connect using Coinbase Wallet",
    bundleUrl: null, // Will be implemented later
    downloadUrl: "https://www.coinbase.com/wallet",
    supportedChains: ["ethereum"],
  },
} as const;

// Wallet Connection States
export const WALLET_CONNECTION_STATES = {
  IDLE: "idle",
  CONNECTING: "connecting",
  CONNECTED: "connected",
  DISCONNECTING: "disconnecting",
  ERROR: "error",
} as const;

// Wallet Error Types
export const WALLET_ERROR_TYPES = {
  PROVIDER_NOT_FOUND: "provider_not_found",
  USER_REJECTED: "user_rejected",
  NETWORK_MISMATCH: "network_mismatch",
  CONNECTION_FAILED: "connection_failed",
  INSUFFICIENT_FUNDS: "insufficient_funds",
  TRANSACTION_FAILED: "transaction_failed",
} as const;

// Supported Networks
export const SUPPORTED_NETWORKS = {
  ETHEREUM_MAINNET: {
    chainId: 1,
    name: "Ethereum Mainnet",
    symbol: "ETH",
    rpcUrls: ["https://mainnet.infura.io/v3/"],
    blockExplorerUrls: ["https://etherscan.io"],
  },
  ETHEREUM_SEPOLIA: {
    chainId: 11155111,
    name: "Sepolia Testnet",
    symbol: "ETH",
    rpcUrls: ["https://sepolia.infura.io/v3/"],
    blockExplorerUrls: ["https://sepolia.etherscan.io"],
  },
  NEAR_MAINNET: {
    chainId: "mainnet",
    name: "NEAR Mainnet",
    symbol: "NEAR",
    rpcUrls: ["https://rpc.mainnet.near.org"],
    blockExplorerUrls: ["https://explorer.near.org"],
  },
  NEAR_TESTNET: {
    chainId: "testnet",
    name: "NEAR Testnet",
    symbol: "NEAR",
    rpcUrls: ["https://rpc.testnet.near.org"],
    blockExplorerUrls: ["https://explorer.testnet.near.org"],
  },
} as const;
