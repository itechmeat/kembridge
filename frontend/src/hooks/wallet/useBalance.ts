import { useState, useEffect, useCallback } from "react";
import { useWalletAuthStatus } from "../api/useAuth";
import { weiToEth, yoctoToNear } from "../../utils/formatBalance";
import type { BridgeToken } from "../../types/bridge";

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
  enrichTokensWithBalances: (tokens: { symbol: string; name: string; address: string; decimals: number; chain: string; logo_url?: string }[]) => BridgeToken[];
}

export const useBalance = (): UseBalanceReturn => {
  const {
    isEvmAuthenticated,
    isNearAuthenticated,
    evmAddress: authEvmAddress,
    nearAddress: authNearAddress,
  } = useWalletAuthStatus();

  const [balances, setBalances] = useState<TokenBalance[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Helper function to get ETH balance
  const getEthBalance = useCallback(
    async (address: string): Promise<TokenBalance | null> => {
      try {
        if (typeof window !== "undefined" && window.ethereum) {
          const balance = await window.ethereum.request({
            method: "eth_getBalance",
            params: [address, "latest"],
          });

          if (balance && balance !== "0x") {
            const balanceWei = BigInt(balance).toString();
            // Convert Wei to ETH using centralized utility
            const ethAmount = weiToEth(balanceWei);
            const usdValue = (parseFloat(ethAmount) * 2500).toFixed(2); // Approximate ETH price

            return {
              symbol: "ETH",
              balance: ethAmount, // Уже в ETH формате
              decimals: 18,
              usdValue: usdValue,
            };
          }
        }
        return null;
      } catch (error) {
        console.warn("Failed to get ETH balance:", error);
        return null;
      }
    },
    []
  );

  // Helper function to get NEAR balance
  const getNearBalance = useCallback(
    async (accountId: string): Promise<TokenBalance | null> => {
      try {
        const response = await fetch("https://rpc.testnet.near.org", {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({
            jsonrpc: "2.0",
            id: "dontcare",
            method: "query",
            params: {
              request_type: "view_account",
              finality: "final",
              account_id: accountId,
            },
          }),
        });

        const data = await response.json();
        if (data.result) {
          const balanceYocto = data.result.amount;
          // Convert yoctoNEAR to NEAR using centralized utility
          const nearAmount = yoctoToNear(balanceYocto);
          const usdValue = (parseFloat(nearAmount) * 3.5).toFixed(2); // Approximate NEAR price

          return {
            symbol: "NEAR",
            balance: nearAmount, // Уже в NEAR формате
            decimals: 24,
            usdValue: usdValue,
          };
        }
        return null;
      } catch (error) {
        console.warn("Failed to get NEAR balance:", error);
        return null;
      }
    },
    []
  );

  // Update balances when accounts change
  useEffect(() => {
    const fetchBalances = async () => {
      setIsLoading(true);
      setError(null);
      const newBalances: TokenBalance[] = [];

      try {
        // Get ETH balance if EVM wallet is connected and authenticated
        if (isEvmAuthenticated && authEvmAddress) {
          const ethBalance = await getEthBalance(authEvmAddress);
          if (ethBalance) {
            newBalances.push(ethBalance);
          }
        }

        // Get NEAR balance if NEAR wallet is connected and authenticated
        if (isNearAuthenticated && authNearAddress) {
          const nearBalance = await getNearBalance(authNearAddress);
          if (nearBalance) {
            newBalances.push(nearBalance);
          }
        }

        setBalances(newBalances);
      } catch (err) {
        setError(
          err instanceof Error ? err.message : "Failed to fetch balances"
        );
      } finally {
        setIsLoading(false);
      }
    };

    if (isEvmAuthenticated || isNearAuthenticated) {
      fetchBalances();
    } else {
      setBalances([]);
      setError(null);
    }
  }, [
    isEvmAuthenticated,
    isNearAuthenticated,
    authEvmAddress,
    authNearAddress,
    getEthBalance,
    getNearBalance,
  ]);

  // Refresh balances
  const refresh = useCallback(async (): Promise<void> => {
    if (!isEvmAuthenticated && !isNearAuthenticated) {
      return;
    }

    setIsLoading(true);
    setError(null);
    const newBalances: TokenBalance[] = [];

    try {
      console.log("🔄 Refreshing balances...");

      // Get ETH balance if EVM wallet is connected and authenticated
      if (isEvmAuthenticated && authEvmAddress) {
        const ethBalance = await getEthBalance(authEvmAddress);
        if (ethBalance) {
          newBalances.push(ethBalance);
        }
      }

      // Get NEAR balance if NEAR wallet is connected and authenticated
      if (isNearAuthenticated && authNearAddress) {
        const nearBalance = await getNearBalance(authNearAddress);
        if (nearBalance) {
          newBalances.push(nearBalance);
        }
      }

      setBalances(newBalances);
    } catch (err) {
      const errorMessage =
        err instanceof Error ? err.message : "Failed to refresh balances";
      setError(errorMessage);
    } finally {
      setIsLoading(false);
    }
  }, [
    isEvmAuthenticated,
    isNearAuthenticated,
    authEvmAddress,
    authNearAddress,
    getEthBalance,
    getNearBalance,
  ]);

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

  // Centralized function to enrich tokens with balance data
  const enrichTokensWithBalances = useCallback((tokens: { symbol: string; name: string; address: string; decimals: number; chain: string; logo_url?: string }[]): BridgeToken[] => {
    return tokens.map((token) => {
      const balance = getBalance(token.symbol);
      return {
        symbol: token.symbol,
        name: token.name,
        address: token.address,
        decimals: token.decimals,
        chain: token.chain,
        logoUrl: token.logo_url,
        balance: balance?.balance,
        usdValue: balance?.usdValue,
      } as BridgeToken;
    });
  }, [getBalance]);

  return {
    balances,
    isLoading,
    error,
    refresh,
    getBalance,
    getTotalUsdValue,
    enrichTokensWithBalances,
  };
};
