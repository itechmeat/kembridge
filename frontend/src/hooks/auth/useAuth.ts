/**
 * Authentication hook
 * Provides authentication state and operations
 */

import { useState, useEffect, useCallback } from "react";
import { authService, type AuthenticationResult } from "../../services/auth";
import { useWallet } from "../wallet/useWallet";
import { WalletType } from "../../services/wallet/types";
import type { UserProfile } from "../../services/api/client";

export interface UseAuthReturn {
  // Authentication state
  isAuthenticated: boolean;
  user: UserProfile | null;
  isAuthenticating: boolean;
  authError: string | null;

  // Backend connection
  isBackendConnected: boolean;

  // Actions
  authenticateWithWallet: () => Promise<AuthenticationResult>;
  logout: () => Promise<void>;
  clearAuthError: () => void;
  checkBackendHealth: () => Promise<boolean>;

  // User profile management
  updateProfile: (updates: Partial<UserProfile>) => Promise<UserProfile | null>;
  refreshUser: () => Promise<void>;
}

export const useAuth = (): UseAuthReturn => {
  const { isConnected, account, signMessage } = useWallet();

  const [isAuthenticated, setIsAuthenticated] = useState(false);
  const [user, setUser] = useState<UserProfile | null>(null);
  const [isAuthenticating, setIsAuthenticating] = useState(false);
  const [authError, setAuthError] = useState<string | null>(null);
  const [isBackendConnected, setIsBackendConnected] = useState(false);

  // Check authentication status on mount and wallet changes
  useEffect(() => {
    const checkAuth = async () => {
      const authenticated = authService.isAuthenticated();
      setIsAuthenticated(authenticated);

      if (authenticated) {
        try {
          const currentUser = await authService.getCurrentUser();
          setUser(currentUser);
        } catch (error) {
          console.warn("Failed to get current user:", error);
          setIsAuthenticated(false);
        }
      }
    };

    checkAuth();
  }, []);

  // Check backend health on mount
  useEffect(() => {
    const checkHealth = async () => {
      const healthy = await authService.checkBackendHealth();
      setIsBackendConnected(healthy);
    };

    checkHealth();
  }, []);

  const authenticateWithWallet =
    useCallback(async (): Promise<AuthenticationResult> => {
      if (!isConnected || !account) {
        const error = "No wallet connected";
        setAuthError(error);
        return { success: false, error };
      }

      setIsAuthenticating(true);
      setAuthError(null);

      try {
        // Map network type to wallet type
        const getWalletType = () => {
          if (account.network.type === "ethereum") {
            return account.network.chainId === 1 ||
              account.network.chainId === 11155111
              ? WalletType.METAMASK
              : WalletType.METAMASK; // Default to MetaMask for Ethereum networks
          }
          return WalletType.NEAR;
        };

        const result = await authService.authenticateWallet({
          walletAddress: account.address,
          walletType: getWalletType(),
          signMessage,
        });

        if (result.success && result.user) {
          setIsAuthenticated(true);
          setUser(result.user);
          setAuthError(null);
        } else {
          setAuthError(result.error || "Authentication failed");
        }

        return result;
      } catch (error) {
        const errorMessage =
          error instanceof Error ? error.message : "Authentication failed";
        setAuthError(errorMessage);
        return { success: false, error: errorMessage };
      } finally {
        setIsAuthenticating(false);
      }
    }, [isConnected, account, signMessage]);

  const logout = useCallback(async (): Promise<void> => {
    try {
      await authService.logout();
    } catch (error) {
      console.warn("Logout error:", error);
    } finally {
      setIsAuthenticated(false);
      setUser(null);
      setAuthError(null);
    }
  }, []);

  // Auto-logout when wallet disconnects
  useEffect(() => {
    if (!isConnected && isAuthenticated) {
      logout();
    }
  }, [isConnected, isAuthenticated, logout]);

  const clearAuthError = useCallback((): void => {
    setAuthError(null);
  }, []);

  const checkBackendHealth = useCallback(async (): Promise<boolean> => {
    const healthy = await authService.checkBackendHealth();
    setIsBackendConnected(healthy);
    return healthy;
  }, []);

  const updateProfile = useCallback(
    async (updates: Partial<UserProfile>): Promise<UserProfile | null> => {
      try {
        const updatedUser = await authService.updateProfile(updates);
        if (updatedUser) {
          setUser(updatedUser);
        }
        return updatedUser;
      } catch (error) {
        console.error("Failed to update profile:", error);
        return null;
      }
    },
    []
  );

  const refreshUser = useCallback(async (): Promise<void> => {
    if (!isAuthenticated) return;

    try {
      const currentUser = await authService.getCurrentUser();
      setUser(currentUser);
    } catch (error) {
      console.warn("Failed to refresh user:", error);
    }
  }, [isAuthenticated]);

  return {
    isAuthenticated,
    user,
    isAuthenticating,
    authError,
    isBackendConnected,
    authenticateWithWallet,
    logout,
    clearAuthError,
    checkBackendHealth,
    updateProfile,
    refreshUser,
  };
};
