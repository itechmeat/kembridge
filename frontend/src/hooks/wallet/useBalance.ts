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
      setBalances(account.balances);
      setError(null);
    } else {
      // Mock balances for demonstration
      if (isConnected && state.address) {
        const mockBalances: TokenBalance[] = [
          {
            symbol: state.walletType === "near" ? "NEAR" : "ETH",
            balance: "1.234567890123456789",
            decimals: 18,
            usdValue: state.walletType === "near" ? "3.45" : "2456.78",
          },
        ];
        setBalances(mockBalances);
      } else {
        setBalances([]);
      }
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
      
      // Mock refresh for now
      await new Promise(resolve => setTimeout(resolve, 1000));
      
      // Update mock balances based on current wallet
      const mockBalances: TokenBalance[] = [
        {
          symbol: account.type === "near" ? "NEAR" : "ETH",
          balance: "1.234567890123456789",
          decimals: 18,
          usdValue: account.type === "near" ? "3.45" : "2456.78",
        },
      ];
      setBalances(mockBalances);
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
