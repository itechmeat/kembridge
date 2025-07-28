/**
 * Wallet utility functions
 * Centralized wallet metadata and helper functions
 */

import { WalletType } from "../services/wallet/types";
import { WALLET_METADATA } from "../constants/wallet";

/**
 * Get wallet display name
 */
export const getWalletName = (walletType: WalletType): string => {
  console.log("🔍 Utils: Getting name for wallet type:", walletType);
  const name = WALLET_METADATA[walletType]?.name || "Unknown Wallet";
  console.log("📝 Utils: Name for wallet type:", walletType, "=", name);
  return name;
};

/**
 * Get wallet icon
 */
export const getWalletIcon = (walletType: WalletType): string => {
  console.log("🔍 Utils: Getting icon for wallet type:", walletType);
  const icon = WALLET_METADATA[walletType]?.icon || "💼";
  console.log("🎨 Utils: Icon for wallet type:", walletType, "=", icon);
  return icon;
};

/**
 * Get wallet description
 */
export const getWalletDescription = (walletType: WalletType): string => {
  console.log("🔍 Utils: Getting description for wallet type:", walletType);
  const description =
    WALLET_METADATA[walletType]?.description || "Connect to this wallet";
  console.log(
    "📝 Utils: Description for wallet type:",
    walletType,
    "=",
    description
  );
  return description;
};

/**
 * Get wallet download URL
 */
export const getWalletDownloadUrl = (walletType: WalletType): string => {
  return WALLET_METADATA[walletType]?.downloadUrl || "#";
};

/**
 * Check if wallet supports chain
 */
export const isWalletSupportedOnChain = (
  walletType: WalletType,
  chainType: string
): boolean => {
  const metadata = WALLET_METADATA[walletType];
  if (!metadata || !metadata.supportedChains) return false;
  return (metadata.supportedChains as readonly string[]).includes(chainType);
};

/**
 * Get supported wallets for chain
 */
export const getSupportedWalletsForChain = (
  chainType: string
): WalletType[] => {
  return Object.entries(WALLET_METADATA)
    .filter(([, metadata]) =>
      (metadata.supportedChains as readonly string[])?.includes(chainType)
    )
    .map(([walletType]) => walletType as WalletType);
};

/**
 * Format wallet address for display
 */
export const formatWalletAddress = (
  address: string,
  startChars: number = 6,
  endChars: number = 4
): string => {
  if (!address || address.length <= startChars + endChars) {
    return address;
  }

  return `${address.slice(0, startChars)}...${address.slice(-endChars)}`;
};

/**
 * Validate wallet address format
 */
export const isValidWalletAddress = (
  address: string,
  walletType: WalletType
): boolean => {
  if (!address) return false;

  switch (walletType) {
    case WalletType.METAMASK:
    case WalletType.WALLET_CONNECT:
    case WalletType.COINBASE:
      // Ethereum address validation (0x + 40 hex chars)
      return /^0x[a-fA-F0-9]{40}$/.test(address);

    case WalletType.NEAR:
      // NEAR address validation (account.near or hex)
      return (
        /^[a-z0-9_-]+\.(near|testnet)$/.test(address) ||
        /^[a-f0-9]{64}$/.test(address)
      );

    default:
      return false;
  }
};

/**
 * Get explorer URL for address
 */
export const getExplorerUrl = (
  address: string,
  chainId: string | number,
  type: "address" | "tx" = "address"
): string => {
  // This would be expanded based on supported networks
  const baseUrls: Record<string | number, string> = {
    1: "https://etherscan.io",
    11155111: "https://sepolia.etherscan.io",
    mainnet: "https://explorer.near.org",
    testnet: "https://explorer.testnet.near.org",
  };

  const baseUrl = baseUrls[chainId];
  if (!baseUrl) return "#";

  return `${baseUrl}/${type}/${address}`;
};

/**
 * Load wallet provider dynamically
 */
export const loadWalletProvider = async (walletType: WalletType) => {
  const metadata = WALLET_METADATA[walletType];

  if (!metadata?.bundleUrl) {
    // Provider is already available (e.g., MetaMask)
    return null;
  }

  try {
    const module = await metadata.bundleUrl();
    return (module as { default?: unknown }).default || module;
  } catch (error) {
    console.error(`Failed to load wallet provider ${walletType}:`, error);
    throw new Error(`Failed to load ${getWalletName(walletType)} provider`);
  }
};

/**
 * Convert wei to ETH
 */
export const weiToEth = (weiAmount: string): string => {
  try {
    const wei = BigInt(weiAmount);
    const eth = Number(wei) / Math.pow(10, 18);
    return eth.toString();
  } catch (error) {
    console.warn("Failed to convert wei to ETH:", error);
    return "0";
  }
};

/**
 * Convert ETH to wei
 */
export const ethToWei = (ethAmount: string): string => {
  try {
    const eth = parseFloat(ethAmount);
    const wei = BigInt(Math.floor(eth * Math.pow(10, 18)));
    return wei.toString();
  } catch (error) {
    console.warn("Failed to convert ETH to wei:", error);
    return "0";
  }
};

/**
 * Format token balance for display
 */
export const formatTokenBalance = (
  balance: string,
  decimals: number = 18,
  displayDecimals: number = 4
): string => {
  try {
    const balanceBigInt = BigInt(balance);
    const divisor = BigInt(Math.pow(10, decimals));
    const integerPart = balanceBigInt / divisor;
    const fractionalPart = balanceBigInt % divisor;

    // Convert to decimal
    const fractionalDecimal = Number(fractionalPart) / Math.pow(10, decimals);
    const fullAmount = Number(integerPart) + fractionalDecimal;

    // Format with specified decimal places
    return fullAmount.toFixed(displayDecimals).replace(/\.?0+$/, "");
  } catch (error) {
    console.warn("Failed to format token balance:", error);
    return "0";
  }
};

/**
 * Format balance for display (auto-selects appropriate format)
 */
export const formatDisplayBalance = (
  balance: string,
  symbol: string,
  decimals: number = 18
): string => {
  const formatted = formatTokenBalance(balance, decimals, 4);
  const numValue = parseFloat(formatted);

  // Show more decimal places for small amounts
  if (numValue < 0.001 && numValue > 0) {
    return `< 0.001 ${symbol}`;
  }

  // Show appropriate decimal places
  if (numValue < 1) {
    return `${formatTokenBalance(balance, decimals, 6)} ${symbol}`;
  } else if (numValue < 100) {
    return `${formatTokenBalance(balance, decimals, 4)} ${symbol}`;
  } else {
    return `${formatTokenBalance(balance, decimals, 2)} ${symbol}`;
  }
};
