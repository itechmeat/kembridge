/**
 * Utilities for formatting token balances and currency values
 */

/**
 * Formats a balance for display with appropriate precision
 * Handles both small and large numbers with readable formatting
 */
export const formatBalance = (balance?: string | number): string => {
  if (!balance) return "";

  const num = typeof balance === "string" ? parseFloat(balance) : balance;

  if (isNaN(num) || num === 0) return "0";

  // For very small numbers
  if (num < 0.0001) return "<0.0001";

  // For small numbers, show more precision
  if (num < 1) return num.toFixed(6);

  // For medium numbers, show less precision
  if (num < 1000) return num.toFixed(4);

  // For large numbers, use locale formatting
  if (num < 1000000) {
    return num.toLocaleString(undefined, {
      minimumFractionDigits: 2,
      maximumFractionDigits: 2,
    });
  }

  // For very large numbers, use abbreviated format
  if (num >= 1000000) {
    const millions = num / 1000000;
    if (millions >= 1000) {
      const billions = millions / 1000;
      return `${billions.toFixed(2)}B`;
    }
    return `${millions.toFixed(2)}M`;
  }

  return num.toLocaleString(undefined, { maximumFractionDigits: 2 });
};

/**
 * Formats a USD value for display
 */
export const formatUsdValue = (
  balance?: string | number,
  usdPrice?: string | number
): string => {
  if (!balance || !usdPrice) return "";

  const balanceNum =
    typeof balance === "string" ? parseFloat(balance) : balance;
  const priceNum =
    typeof usdPrice === "string" ? parseFloat(usdPrice) : usdPrice;

  if (isNaN(balanceNum) || isNaN(priceNum)) return "";

  const usdValue = balanceNum * priceNum;

  if (usdValue < 0.01) return "";

  return `$${usdValue.toLocaleString(undefined, {
    minimumFractionDigits: 2,
    maximumFractionDigits: 2,
  })}`;
};

/**
 * Converts Wei to ETH
 */
export const weiToEth = (wei: string | number): string => {
  const weiValue =
    typeof wei === "string" ? BigInt(wei) : BigInt(wei.toString());
  const ethValue = Number(weiValue) / Math.pow(10, 18);
  return ethValue.toString();
};

/**
 * Converts yoctoNEAR to NEAR
 */
export const yoctoToNear = (yocto: string | number): string => {
  const yoctoValue =
    typeof yocto === "string" ? BigInt(yocto) : BigInt(yocto.toString());
  const nearValue = Number(yoctoValue) / Math.pow(10, 24);
  return nearValue.toString();
};

/**
 * Converts raw token amount to human-readable format based on decimals
 */
export const formatTokenAmount = (
  amount: string | number,
  decimals: number = 18
): string => {
  const amountValue =
    typeof amount === "string" ? BigInt(amount) : BigInt(amount.toString());
  const humanValue = Number(amountValue) / Math.pow(10, decimals);
  return humanValue.toString();
};

/**
 * Comprehensive balance formatter that handles raw token amounts
 * Use this as the main formatter for all balance displays
 */
export const formatTokenBalance = (
  balance: string | number,
  symbol: string,
  decimals: number = 18,
  showSymbol: boolean = true
): string => {
  if (!balance) return showSymbol ? `0 ${symbol}` : "0";

  // Convert raw amount to human-readable
  const humanBalance = formatTokenAmount(balance, decimals);
  const formattedBalance = formatBalance(humanBalance);

  return showSymbol ? `${formattedBalance} ${symbol}` : formattedBalance;
};

/**
 * Format balance with USD value
 */
export const formatBalanceWithUsd = (
  balance: string | number,
  symbol: string,
  usdPrice?: string | number,
  decimals: number = 18
): { formatted: string; usd: string } => {
  const humanBalance = formatTokenAmount(balance, decimals);
  const formatted = formatBalance(humanBalance);
  const usd = formatUsdValue(humanBalance, usdPrice);

  return {
    formatted: `${formatted} ${symbol}`,
    usd,
  };
};
