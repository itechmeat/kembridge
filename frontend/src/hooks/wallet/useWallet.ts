import { useState, useEffect, useCallback, useMemo } from "react";
import { useAccount, useConnect, useDisconnect } from "wagmi";
import { useAuthStatus, useEthereumAuth, useNearAuth } from "../api/useAuth";
import { useNearWallet } from "./useNearWallet";
import { STORAGE_KEYS } from "../../services/wallet/utils";

export type WalletType = "metamask" | "walletconnect" | "coinbase" | "near";

export interface WalletState {
  // Connection state
  walletType: WalletType | null;
  isConnected: boolean;
  isConnecting: boolean;
  address: string | null;
  chainId: number | null;
  error: string | null;

  // Account info (for backward compatibility)
  account: WalletAccount | null;
}

export interface WalletAccount {
  address: string;
  chainId?: number;
  type: WalletType;
  network?: {
    name: string;
    type: string;
    chainId: number;
  } | null;
  balances?: Record<string, string>[];
}

export interface UseWalletReturn {
  // State
  state: WalletState;
  isConnected: boolean;
  isConnecting: boolean;
  account: WalletAccount | null;
  error: string | null;

  // Actions
  connect: (walletType: WalletType) => Promise<void>;
  disconnect: () => Promise<void>;
  switchWallet: (walletType: WalletType) => Promise<void>;
  clearError: () => void;
  autoConnect: () => Promise<boolean>;

  // Utility methods
  isWalletAvailable: (walletType: WalletType) => boolean;
  signMessage?: (message: string) => Promise<string>;
  switchNetwork?: (chainId: number) => Promise<void>;
  refreshAccount?: () => Promise<void>;
}

export const useWallet = (): UseWalletReturn => {
  const [walletType, setWalletType] = useState<WalletType | null>(null);
  const [isConnecting, setIsConnecting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [autoAuthAttempted, setAutoAuthAttempted] = useState(false);

  // Wagmi hooks for Ethereum wallets
  const { address, isConnected: isEthConnected, chainId } = useAccount();
  const { connect: connectWagmi, connectors } = useConnect();
  const { disconnect: disconnectWagmi } = useDisconnect();

  // NEAR wallet hook
  const nearWallet = useNearWallet();

  // Auth hooks
  const { isAuthenticated } = useAuthStatus();
  const ethereumAuth = useEthereumAuth();
  const nearAuth = useNearAuth();

  // Determine connection state (memoized)
  const isConnected = useMemo(() => {
    const connected =
      walletType === "near" ? nearWallet.isConnected : isEthConnected;

    // Log wallet state changes for debugging
    console.log("🔍 useWallet: Connection state check:", {
      walletType,
      nearWalletConnected: nearWallet.isConnected,
      nearAccountId: nearWallet.accountId,
      isEthConnected,
      finalConnected: connected,
    });

    return connected;
  }, [
    walletType,
    nearWallet.isConnected,
    nearWallet.accountId,
    isEthConnected,
  ]);

  const currentAddress = useMemo(() => {
    return walletType === "near" ? nearWallet.accountId : address;
  }, [walletType, nearWallet.accountId, address]);

  // Create account object for backward compatibility (memoized)
  const account: WalletAccount | null = useMemo(() => {
    if (!isConnected || !currentAddress || !walletType) {
      return null;
    }

    return {
      address: currentAddress,
      chainId: chainId,
      type: walletType,
      network: chainId
        ? {
            name:
              chainId === 11155111
                ? "Sepolia Testnet"
                : chainId === 1
                ? "Ethereum Mainnet"
                : chainId === 137
                ? "Polygon"
                : `Chain ${chainId}`,
            type: walletType === "near" ? "near" : "ethereum",
            chainId: chainId || 0,
          }
        : null,
    };
  }, [isConnected, currentAddress, walletType, chainId]);

  // Create state object (memoized)
  const state: WalletState = useMemo(
    () => ({
      walletType,
      isConnected,
      isConnecting,
      address: currentAddress || null,
      chainId: chainId || null,
      error,
      account,
    }),
    [
      walletType,
      isConnected,
      isConnecting,
      currentAddress,
      chainId,
      error,
      account,
    ]
  );

  // Load saved wallet type from storage
  useEffect(() => {
    const savedWallet = localStorage.getItem(
      STORAGE_KEYS.LAST_CONNECTED_WALLET
    );
    if (savedWallet) {
      try {
        setWalletType(savedWallet as WalletType);
      } catch (error) {
        console.error("Failed to restore wallet type:", error);
        localStorage.removeItem(STORAGE_KEYS.LAST_CONNECTED_WALLET);
      }
    }
  }, []);

  // Auto-authenticate when wallet becomes connected but not authenticated
  useEffect(() => {
    if (
      isConnected &&
      !isAuthenticated &&
      walletType &&
      !isConnecting &&
      !autoAuthAttempted &&
      !ethereumAuth.isPending &&
      !nearAuth.isPending
    ) {
      setAutoAuthAttempted(true);

      const autoAuthenticate = async () => {
        try {
          console.log(
            `🔐 useWallet: Auto-authenticating connected wallet (${walletType})`
          );

          if (walletType === "near") {
            await nearAuth.authenticate();
          } else {
            await ethereumAuth.authenticate();
          }

          console.log(`✅ useWallet: Auto-authentication successful`);
        } catch (authError) {
          console.log(
            `ℹ️ useWallet: Auto-authentication failed (manual auth still works):`,
            authError
          );
          // Reset flag to allow manual retry
          setAutoAuthAttempted(false);
        }
      };

      // Small delay to ensure wallet connection is fully established
      const timeoutId = setTimeout(autoAuthenticate, 1000);
      return () => clearTimeout(timeoutId);
    }
  }, [
    isConnected,
    isAuthenticated,
    walletType,
    isConnecting,
    autoAuthAttempted,
    ethereumAuth.isPending,
    ethereumAuth,
    nearAuth,
    nearAuth.isAuthenticating,
  ]);

  // Reset auto-auth flag when authentication becomes successful
  useEffect(() => {
    if (isAuthenticated && autoAuthAttempted) {
      setAutoAuthAttempted(false);
      console.log(
        "✅ useWallet: Authentication successful, resetting auto-auth flag"
      );
    }
  }, [isAuthenticated, autoAuthAttempted]);

  // Auto-connect is handled by Wagmi automatically for Ethereum wallets
  // NEAR auto-connect needs to be triggered manually when needed

  // Connect wallet with automatic authentication
  const connect = useCallback(
    async (type: WalletType): Promise<void> => {
      if (isConnecting) return;

      setIsConnecting(true);
      setError(null);

      try {
        if (type === "near") {
          if (!nearWallet.selector) {
            throw new Error("NEAR wallet selector not ready");
          }
          console.log("🔗 useWallet: Initiating NEAR sign in...");
          await nearWallet.signIn();

          // Wait for NEAR wallet to actually connect
          console.log("⏳ useWallet: Waiting for NEAR connection...");
          let attempts = 0;
          while (!nearWallet.isConnected && attempts < 10) {
            await new Promise((resolve) => setTimeout(resolve, 500));
            attempts++;
          }

          if (!nearWallet.isConnected) {
            throw new Error("NEAR wallet failed to connect within timeout");
          }

          console.log("✅ useWallet: NEAR wallet connected successfully");
        } else {
          // Ethereum wallets
          const connector = connectors.find((c) => {
            const id = c.id.toLowerCase();
            return (
              (type === "metamask" && id.includes("metamask")) ||
              (type === "walletconnect" && id.includes("walletconnect")) ||
              (type === "coinbase" && id.includes("coinbase"))
            );
          });

          if (!connector) {
            throw new Error(`${type} connector not found`);
          }

          await connectWagmi({ connector });
        }

        // Set wallet type AFTER successful connection
        setWalletType(type);
        localStorage.setItem(STORAGE_KEYS.LAST_CONNECTED_WALLET, type);

        console.log(`✅ useWallet: Connected to ${type}, wallet type set`);

        // Auto-authenticate after successful connection
        try {
          console.log(`🔐 useWallet: Starting auto-authentication for ${type}`);

          if (type === "near") {
            await nearAuth.authenticate();
          } else {
            await ethereumAuth.authenticate();
          }

          console.log(
            `✅ useWallet: Auto-authentication successful for ${type}`
          );
        } catch (authError) {
          // Don't throw auth errors - wallet connection was successful
          console.log(
            `ℹ️ useWallet: Auto-authentication failed for ${type} (manual auth still works):`,
            authError
          );
          // User can still manually authenticate later if needed
        }
      } catch (err) {
        const errorMessage =
          err instanceof Error ? err.message : `Failed to connect to ${type}`;
        setError(errorMessage);
        console.error(`❌ useWallet: ${errorMessage}`, err);
        throw err;
      } finally {
        setIsConnecting(false);
      }
    },
    [isConnecting, connectors, connectWagmi, nearWallet, ethereumAuth, nearAuth]
  );

  // Disconnect wallet
  const disconnect = useCallback(async (): Promise<void> => {
    try {
      // Clear auth token BEFORE disconnecting wallet to prevent race conditions
      console.log("🔐 useWallet: Clearing auth token before disconnect");

      // Import authService dynamically to avoid circular dependencies
      const authModule = await import("../../services/api/authService");
      await authModule.authService.logout();

      if (walletType === "near") {
        await nearWallet.signOut();
      } else {
        await disconnectWagmi();
      }

      setWalletType(null);
      setError(null);
      setAutoAuthAttempted(false); // Reset auto-auth flag
      localStorage.removeItem(STORAGE_KEYS.LAST_CONNECTED_WALLET);

      console.log("✅ useWallet: Disconnected");
    } catch (err) {
      const errorMessage =
        err instanceof Error ? err.message : "Failed to disconnect";
      setError(errorMessage);
      console.error("❌ useWallet: Disconnect failed", err);
      throw err;
    }
  }, [walletType, nearWallet, disconnectWagmi]);

  // Switch wallet
  const switchWallet = useCallback(
    async (type: WalletType): Promise<void> => {
      console.log(`🔄 useWallet: Switching from ${walletType} to ${type}`);

      if (isConnected) {
        console.log("🔌 useWallet: Disconnecting current wallet before switch");
        await disconnect();

        // Wait a bit to ensure disconnect is fully processed
        await new Promise((resolve) => setTimeout(resolve, 500));
      }

      console.log("🔌 useWallet: Connecting new wallet");
      await connect(type);
    },
    [isConnected, disconnect, connect, walletType]
  );

  // Auto-connect
  const autoConnect = useCallback(async (): Promise<boolean> => {
    if (!walletType || isConnected) {
      return isConnected;
    }

    try {
      await connect(walletType);
      return true;
    } catch (error) {
      console.log(
        "ℹ️ useWallet: Auto-connect failed (expected if no previous connection)",
        error
      );
      return false;
    }
  }, [walletType, isConnected, connect]);

  // Clear error
  const clearError = useCallback(() => {
    setError(null);
  }, []);

  // Check if wallet is available
  const isWalletAvailable = useCallback(
    (type: WalletType): boolean => {
      if (type === "near") {
        return !!nearWallet.selector;
      }

      return connectors.some((c) => {
        const id = c.id.toLowerCase();
        return (
          (type === "metamask" && id.includes("metamask")) ||
          (type === "walletconnect" && id.includes("walletconnect")) ||
          (type === "coinbase" && id.includes("coinbase"))
        );
      });
    },
    [connectors, nearWallet.selector]
  );

  return {
    state,
    isConnected,
    isConnecting,
    account,
    error,
    connect,
    disconnect,
    switchWallet,
    clearError,
    autoConnect,
    isWalletAvailable,
    // Optional methods for backward compatibility
    signMessage: undefined, // TODO: Implement if needed
    switchNetwork: undefined, // TODO: Implement if needed
    refreshAccount: undefined, // TODO: Implement if needed
  };
};
