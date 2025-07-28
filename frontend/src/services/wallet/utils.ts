/**
 * Wallet service utilities
 * Shared utilities for wallet operations
 */

import {
  NetworkType,
  NetworkInfo,
  WalletType,
  WalletError,
  WalletErrorCode,
} from "./types";

// TODO (util): Add more network configurations as needed
export const SUPPORTED_NETWORKS: Record<string, NetworkInfo> = {
  ethereum_sepolia: {
    chainId: 11155111,
    name: "Sepolia Testnet",
    type: NetworkType.ETHEREUM,
    rpcUrls: ["https://rpc.sepolia.org"],
    blockExplorerUrls: ["https://sepolia.etherscan.io"],
    nativeCurrency: {
      name: "Sepolia Ether",
      symbol: "ETH",
      decimals: 18,
    },
  },
  ethereum_mainnet: {
    chainId: 1,
    name: "Ethereum Mainnet",
    type: NetworkType.ETHEREUM,
    rpcUrls: ["https://eth-mainnet.g.alchemy.com/v2/demo"],
    blockExplorerUrls: ["https://etherscan.io"],
    nativeCurrency: {
      name: "Ether",
      symbol: "ETH",
      decimals: 18,
    },
  },
  near_testnet: {
    chainId: "testnet",
    name: "NEAR Testnet",
    type: NetworkType.NEAR,
    rpcUrls: ["https://rpc.testnet.near.org"],
    blockExplorerUrls: ["https://testnet.nearblocks.io"],
  },
  near_mainnet: {
    chainId: "mainnet",
    name: "NEAR Mainnet",
    type: NetworkType.NEAR,
    rpcUrls: ["https://rpc.mainnet.near.org"],
    blockExplorerUrls: ["https://nearblocks.io"],
  },
};

export const DEFAULT_NETWORK = SUPPORTED_NETWORKS.ethereum_sepolia;

/**
 * Validates Ethereum address format
 */
export const isValidEthereumAddress = (address: string): boolean => {
  return /^0x[a-fA-F0-9]{40}$/.test(address);
};

/**
 * Validates NEAR account ID format
 */
export const isValidNearAccountId = (accountId: string): boolean => {
  // NEAR account ID validation (simplified)
  console.log("🔍 Utils: Validating NEAR account ID:", accountId);

  const isValid =
    /^[a-z0-9_-]+(\.[a-z0-9_-]+)*$/.test(accountId) &&
    accountId.length >= 2 &&
    accountId.length <= 64;

  console.log("✅ Utils: NEAR account ID validation result:", isValid);
  return isValid;
};

/**
 * Formats address for display (truncates middle)
 */
export const formatAddress = (
  address: string,
  startChars = 6,
  endChars = 4
): string => {
  if (!address) return "";
  if (address.length <= startChars + endChars) return address;
  return `${address.slice(0, startChars)}...${address.slice(-endChars)}`;
};

/**
 * Formats balance for display
 */
export const formatBalance = (
  balance: string,
  decimals = 18,
  displayDecimals = 4
): string => {
  const balanceNum = parseFloat(balance);
  if (balanceNum === 0) return "0";

  // Convert from wei/smallest unit to display unit
  const displayBalance = balanceNum / Math.pow(10, decimals);

  if (displayBalance < 0.0001) return "< 0.0001";
  if (displayBalance < 1) return displayBalance.toFixed(displayDecimals);
  if (displayBalance < 1000) return displayBalance.toFixed(2);

  // Format with commas for large numbers
  return new Intl.NumberFormat("en-US", {
    minimumFractionDigits: 0,
    maximumFractionDigits: 2,
  }).format(displayBalance);
};

/**
 * Formats USD value for display
 */
export const formatUsdValue = (usdValue: string | number): string => {
  const value = typeof usdValue === "string" ? parseFloat(usdValue) : usdValue;
  if (value === 0) return "$0.00";
  if (value < 0.01) return "< $0.01";

  return new Intl.NumberFormat("en-US", {
    style: "currency",
    currency: "USD",
    minimumFractionDigits: 2,
    maximumFractionDigits: 2,
  }).format(value);
};

/**
 * Gets network by chain ID
 */
export const getNetworkByChainId = (
  chainId: string | number
): NetworkInfo | null => {
  const chainIdNum =
    typeof chainId === "string" ? parseInt(chainId, 16) : chainId;
  const chainIdStr = chainId.toString();

  return (
    Object.values(SUPPORTED_NETWORKS).find((network) => {
      const networkChainId =
        typeof network.chainId === "string"
          ? network.chainId
          : network.chainId.toString();
      const networkChainIdNum =
        typeof network.chainId === "number"
          ? network.chainId
          : parseInt(network.chainId as string, 10);

      return networkChainIdNum === chainIdNum || networkChainId === chainIdStr;
    }) || null
  );
};

/**
 * Checks if network is supported
 */
export const isSupportedNetwork = (chainId: string | number): boolean => {
  return getNetworkByChainId(chainId) !== null;
};

/**
 * Gets wallet type from provider name/type
 */
export const getWalletTypeFromProvider = (
  provider: unknown
): WalletType | null => {
  if (!provider) return null;

  const typedProvider = provider as {
    isMetaMask?: boolean;
    isCoinbaseWallet?: boolean;
    isWalletConnect?: boolean;
  };

  if (typedProvider.isMetaMask) return WalletType.METAMASK;
  if (typedProvider.isCoinbaseWallet) return WalletType.COINBASE;
  if (typedProvider.isWalletConnect) return WalletType.WALLET_CONNECT;

  return null;
};

/**
 * Creates a standardized wallet error
 */
export const createWalletError = (
  code: WalletErrorCode,
  message: string,
  details?: unknown
): WalletError => {
  return { code, message, details };
};

/**
 * Gets user-friendly error message
 */
export const getErrorMessage = (error: WalletError): string => {
  switch (error.code) {
    case WalletErrorCode.WALLET_NOT_FOUND:
      return "Wallet not found. Please install the wallet extension.";
    case WalletErrorCode.USER_REJECTED:
      return "Connection was rejected by user.";
    case WalletErrorCode.NETWORK_MISMATCH:
      return "Please switch to the correct network.";
    case WalletErrorCode.INSUFFICIENT_FUNDS:
      return "Insufficient funds for this transaction.";
    case WalletErrorCode.CONNECTION_FAILED:
      return "Failed to connect to wallet. Please try again.";
    case WalletErrorCode.INVALID_ADDRESS:
      return "Invalid address format.";
    case WalletErrorCode.TRANSACTION_FAILED:
      return "Transaction failed. Please try again.";
    case WalletErrorCode.UNSUPPORTED_CHAIN:
      return "This network is not supported.";
    default:
      return error.message || "An unknown error occurred.";
  }
};

/**
 * Storage keys for persistence
 */
export const STORAGE_KEYS = {
  LAST_CONNECTED_WALLET: "kembridge_last_wallet",
  WALLET_CONNECTION_PREFERENCE: "kembridge_wallet_preference",
  AUTO_CONNECT_ENABLED: "kembridge_auto_connect",
} as const;

/**
 * Saves last connected wallet to localStorage
 */
export const saveLastConnectedWallet = (walletType: WalletType): void => {
  try {
    localStorage.setItem(STORAGE_KEYS.LAST_CONNECTED_WALLET, walletType);
  } catch (error) {
    console.warn("Failed to save last connected wallet:", error);
  }
};

/**
 * Gets last connected wallet from localStorage
 */
export const getLastConnectedWallet = (): WalletType | null => {
  try {
    const saved = localStorage.getItem(STORAGE_KEYS.LAST_CONNECTED_WALLET);
    return (saved as WalletType) || null;
  } catch (error) {
    console.warn("Failed to get last connected wallet:", error);
    return null;
  }
};

/**
 * Clears wallet storage
 */
export const clearWalletStorage = (): void => {
  try {
    Object.values(STORAGE_KEYS).forEach((key) => {
      localStorage.removeItem(key);
    });
  } catch (error) {
    console.warn("Failed to clear wallet storage:", error);
  }
};

/**
 * Detects if running on mobile device
 */
export const isMobile = (): boolean => {
  return /Android|webOS|iPhone|iPad|iPod|BlackBerry|IEMobile|Opera Mini/i.test(
    navigator.userAgent
  );
};

/**
 * Detects if wallet is available in browser
 */
export const isWalletAvailable = (walletType: WalletType): boolean => {
  switch (walletType) {
    case WalletType.METAMASK:
      return typeof window !== "undefined" && !!window.ethereum?.isMetaMask;
    case WalletType.COINBASE:
      return (
        typeof window !== "undefined" && !!window.ethereum?.isCoinbaseWallet
      );
    case WalletType.WALLET_CONNECT:
      return true; // WalletConnect is always available via QR code
    case WalletType.NEAR:
      return true; // NEAR wallet is available via redirect
    default:
      return false;
  }
};
