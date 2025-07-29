/**
 * Balance management hook
 * Handles token balance fetching and updates
 */

import { useState, useEffect, useCallback } from "react";
import { useWallet } from "./useWallet";

export interface TokenBalance {
  symbol: string;
  balance: string;
  decimals: number;
  usdValue?: string;
}

export interface UseBalanceReturn {
  balances: TokenBalance[];
  isLoading: boolean;
  error: string | null;
  refresh: () => Promise<void>;
  getBalance: (symbol: string) => TokenBalance | null;
  getTotalUsdValue: () => number;
}

export const useBalance = (): UseBalanceReturn => {
  const { account, isConnected, state } = useWallet();
  const [balances, setBalances] = useState<TokenBalance[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Update balances when account changes
  useEffect(() => {
    if (account?.balances) {
      // Convert Record<string, string>[] to TokenBalance[]
      const convertedBalances: TokenBalance[] = account.balances.map((balance) => ({
        symbol: balance.symbol || '',
        balance: balance.balance || '0',
        decimals: parseInt(balance.decimals || '18', 10),
        usdValue: balance.usdValue,
      }));
      setBalances(convertedBalances);
      setError(null);
    } else {
      // No wallet integration available - real balances required
      setBalances([]);
      setError(
        "Wallet balance integration not implemented. Please connect wallet to see real balances."
      );
    }
  }, [account, isConnected, state]);

  // Refresh balances
  const refresh = useCallback(async (): Promise<void> => {
    if (!isConnected || !account) {
      return;
    }

    setIsLoading(true);
    setError(null);

    try {
      // TODO: Implement proper balance refresh with new wallet system
      console.log("🔄 Refreshing balances...");

      // Real wallet balance refresh required - no mock data allowed
      throw new Error(
        "Real wallet balance integration not implemented. Cannot refresh balances."
      );
    } catch (err) {
      const errorMessage =
        err instanceof Error ? err.message : "Failed to refresh balances";
      setError(errorMessage);
    } finally {
      setIsLoading(false);
    }
  }, [isConnected, account]);

  // Get specific token balance
  const getBalance = useCallback(
    (symbol: string): TokenBalance | null => {
      return (
        balances.find(
          (balance) => balance.symbol.toLowerCase() === symbol.toLowerCase()
        ) || null
      );
    },
    [balances]
  );

  // Calculate total USD value
  const getTotalUsdValue = useCallback((): number => {
    return balances.reduce((total, balance) => {
      const usdValue = parseFloat(balance.usdValue || "0");
      return total + usdValue;
    }, 0);
  }, [balances]);

  return {
    balances,
    isLoading,
    error,
    refresh,
    getBalance,
    getTotalUsdValue,
  };
};
