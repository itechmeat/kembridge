import ethIcon from "../assets/coins/eth.webp";
import usdcIcon from "../assets/coins/usdc.webp";
import usdtIcon from "../assets/coins/usdt.webp";
import nearIcon from "../assets/coins/near.webp";

/**
 * Mapping of coin symbols to their icon imports
 */
const COIN_ICONS: Record<string, string> = {
  ETH: ethIcon,
  ETHEREUM: ethIcon,
  USDC: usdcIcon,
  USDT: usdtIcon,
  NEAR: nearIcon,
  // Add aliases for consistency
  eth: ethIcon,
  usdc: usdcIcon,
  usdt: usdtIcon,
  near: nearIcon,
};

/**
 * Get the appropriate coin icon for a token symbol
 */
export const getCoinIcon = (symbol: string): string | undefined => {
  if (!symbol) return undefined;
  return COIN_ICONS[symbol.toUpperCase()] || COIN_ICONS[symbol.toLowerCase()];
};

/**
 * Check if we have a local icon for this coin
 */
export const hasCoinIcon = (symbol: string): boolean => {
  return !!getCoinIcon(symbol);
};

/**
 * Get fallback letter for coins without icons
 */
export const getCoinFallback = (symbol: string): string => {
  return symbol?.charAt(0)?.toUpperCase() || "?";
};

/**
 * Get display props for a coin (icon or fallback)
 */
export const getCoinDisplayProps = (symbol: string) => {
  const icon = getCoinIcon(symbol);

  if (icon) {
    return {
      type: "icon" as const,
      src: icon,
      alt: symbol,
    };
  }

  return {
    type: "fallback" as const,
    letter: getCoinFallback(symbol),
  };
};
