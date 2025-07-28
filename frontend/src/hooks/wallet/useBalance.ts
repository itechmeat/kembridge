/**
 * Balance management hook
 * Handles token balance fetching and updates
 */

import { useState, useEffect, useCallback } from "react";
import { useWallet } from "./useWallet";
import { TokenBalance } from "../../services/wallet/types";

export interface UseBalanceReturn {
  balances: TokenBalance[];
  isLoading: boolean;
  error: string | null;
  refresh: () => Promise<void>;
  getBalance: (symbol: string) => TokenBalance | null;
  getTotalUsdValue: () => number;
}

export const useBalance = (): UseBalanceReturn => {
  const { account, isConnected } = useWallet();
  const [balances, setBalances] = useState<TokenBalance[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Update balances when account changes
  useEffect(() => {
    if (account?.balances) {
      setBalances(account.balances);
      setError(null);
    } else {
      setBalances([]);
    }
  }, [account]);

  // Refresh balances
  const refresh = useCallback(async (): Promise<void> => {
    if (!isConnected || !account) {
      return;
    }

    setIsLoading(true);
    setError(null);

    try {
      // Get wallet manager and refresh account data
      const { getWalletManager } = await import("../../services/wallet");
      const walletManager = getWalletManager();
      await walletManager.refreshAccount();
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
