/**
 * Main wallet hook for React components
 * Provides wallet state and operations
 */

import { useState, useEffect, useCallback, useMemo } from "react";
import { getWalletManager } from "../../services/wallet";
import {
  WalletType,
  WalletState,
  WalletProvider,
  WalletAccount,
  NetworkInfo,
} from "../../services/wallet/types";

export interface UseWalletReturn {
  // State
  state: WalletState;
  isConnecting: boolean;
  isConnected: boolean;
  account: WalletAccount | null;
  error: string | null;

  // Providers
  availableProviders: WalletProvider[];

  // Actions
  connect: (walletType: WalletType) => Promise<WalletAccount>;
  disconnect: () => Promise<void>;
  switchNetwork: (network: NetworkInfo) => Promise<void>;
  signMessage: (message: string) => Promise<string>;
  refreshAccount: () => Promise<void>;
  clearError: () => void;

  // Auto-connect
  autoConnect: () => Promise<boolean>;
}

/**
 * Main wallet hook
 */
export const useWallet = (): UseWalletReturn => {
  console.log("🔄 useWallet: Hook called");
  const walletManager = useMemo(() => {
    console.log("📦 useWallet: Getting wallet manager");
    return getWalletManager();
  }, []);
  const [state, setState] = useState<WalletState>(walletManager.getState());

  // Subscribe to wallet state changes
  useEffect(() => {
    const unsubscribe = walletManager.subscribe(setState);
    return unsubscribe;
  }, [walletManager]);

  // Get all providers (including unavailable ones)
  const availableProviders = useMemo(() => {
    return walletManager.getAllProviders();
  }, [walletManager]);

  // Wallet actions
  const connect = useCallback(
    async (walletType: WalletType): Promise<WalletAccount> => {
      console.log(`🔗 useWallet: Connecting to ${walletType}...`);
      console.log(`📦 useWallet: Calling walletManager.connect(${walletType})`);
      return walletManager.connect(walletType);
    },
    [walletManager]
  );

  const disconnect = useCallback(async (): Promise<void> => {
    return walletManager.disconnect();
  }, [walletManager]);

  const switchNetwork = useCallback(
    async (network: NetworkInfo): Promise<void> => {
      return walletManager.switchNetwork(network);
    },
    [walletManager]
  );

  const signMessage = useCallback(
    async (message: string): Promise<string> => {
      return walletManager.signMessage(message);
    },
    [walletManager]
  );

  const refreshAccount = useCallback(async (): Promise<void> => {
    return walletManager.refreshAccount();
  }, [walletManager]);

  const clearError = useCallback((): void => {
    // Clear error by triggering a state update in wallet manager
    // This is done by getting current state and clearing the error
    if (state.error) {
      // Force a state refresh to clear error
      walletManager.refreshAccount().catch(() => {
        // Ignore refresh errors when clearing
      });
    }
  }, [walletManager, state.error]);

  const autoConnect = useCallback(async (): Promise<boolean> => {
    return walletManager.autoConnect();
  }, [walletManager]);

  // Computed state
  const isConnecting = state.isConnecting;
  const isConnected = state.isConnected;
  const account = state.account;
  const error = state.error?.message || null;

  return {
    state,
    isConnecting,
    isConnected,
    account,
    error,
    availableProviders,
    connect,
    disconnect,
    switchNetwork,
    signMessage,
    refreshAccount,
    clearError,
    autoConnect,
  };
};

/**
 * Hook for wallet-specific operations
 */
export const useWalletActions = () => {
  const {
    connect,
    disconnect,
    switchNetwork,
    signMessage,
    refreshAccount,
    autoConnect,
  } = useWallet();

  return {
    connect,
    disconnect,
    switchNetwork,
    signMessage,
    refreshAccount,
    autoConnect,
  };
};

/**
 * Hook for wallet state only (useful for components that only need to read state)
 */
export const useWalletState = () => {
  const {
    state,
    isConnecting,
    isConnected,
    account,
    error,
    availableProviders,
  } = useWallet();

  return {
    state,
    isConnecting,
    isConnected,
    account,
    error,
    availableProviders,
  };
};
