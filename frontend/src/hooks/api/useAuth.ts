/**
 * Authentication Hooks
 * React hooks for Web3 authentication with TanStack Query
 */

import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useState, useCallback, useEffect } from "react";
import { authService } from "../../services/api/authService";
import { API_CONFIG } from "../../services/api/config";
import { useAccount } from "wagmi";
import { useSignMessage } from "wagmi";
import { useNearWallet } from "../wallet/useNearWallet";

// Query keys for caching
export const AUTH_QUERY_KEYS = {
  NONCE: "auth-nonce",
  VERIFY: "auth-verify",
  PROFILE: "user-profile",
} as const;

/**
 * Hook for getting nonce
 */
export const useGetNonce = () => {
  return useMutation({
    mutationFn: async ({
      walletAddress,
      chainType,
    }: {
      walletAddress: string;
      chainType: "ethereum" | "near";
    }) => {
      return authService.getNonce(walletAddress, chainType);
    },
    onSuccess: () => {
      console.log("✅ useGetNonce: Nonce received successfully");
    },
    onError: (error) => {
      console.error("❌ useGetNonce: Failed to get nonce:", error);
    },
  });
};

/**
 * Hook for verifying wallet signature
 */
export const useVerifyWallet = () => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async ({
      walletAddress,
      signature,
      nonce,
      chainType,
      message,
    }: {
      walletAddress: string;
      signature: string;
      nonce: string;
      chainType: "ethereum" | "near";
      message: string;
    }) => {
      return authService.verifyWallet(
        walletAddress,
        signature,
        nonce,
        chainType,
        message
      );
    },
    onSuccess: () => {
      console.log("✅ useVerifyWallet: Authentication successful");
      // Invalidate user profile cache for updates
      queryClient.invalidateQueries({ queryKey: [AUTH_QUERY_KEYS.PROFILE] });
    },
    onError: (error) => {
      console.error("❌ useVerifyWallet: Authentication failed:", error);
    },
  });
};

/**
 * Hook for Ethereum authentication
 */
export const useEthereumAuth = () => {
  const { address, isConnected } = useAccount();
  const { signMessageAsync } = useSignMessage();
  const getNonce = useGetNonce();
  const verifyWallet = useVerifyWallet();
  const queryClient = useQueryClient();
  const [isAuthenticating, setIsAuthenticating] = useState(false);

  const authenticate = useCallback(async () => {
    if (!address || !isConnected) {
      throw new Error("Ethereum wallet not connected");
    }

    setIsAuthenticating(true);

    try {
      console.log("🔐 Ethereum Auth: Starting authentication for:", address);

      // 1. Get nonce
      const nonceData = await getNonce.mutateAsync({
        walletAddress: address,
        chainType: "ethereum",
      });

      // 2. Use message from backend nonce response
      const message = nonceData.message;
      console.log(
        "📝 Ethereum Auth: Using message from nonce response for signing"
      );

      // 3. Sign message
      console.log(
        "🖊️ Ethereum Auth: Requesting signature for message:",
        message
      );

      let signature: string;
      try {
        signature = await signMessageAsync({ message });
        console.log("✍️ Ethereum Auth: Message signed successfully", {
          signature,
          signatureType: typeof signature,
          signatureLength: signature?.length || 0,
        });

        if (!signature) {
          throw new Error("Signature was not provided by wallet");
        }
      } catch (signError) {
        console.error("❌ Ethereum Auth: Signature failed:", signError);
        throw new Error(`Failed to sign message: ${signError}`);
      }

      // 4. Verify signature on backend
      const authResult = await verifyWallet.mutateAsync({
        walletAddress: address,
        signature,
        nonce: nonceData.nonce,
        chainType: "ethereum",
        message,
      });

      console.log("🎉 Ethereum Auth: Authentication completed successfully");

      // Check if token is actually saved after authentication
      const tokenAfterAuth = authService.getToken();
      const isAuthAfterAuth = authService.isAuthenticated();
      console.log("🔍 Ethereum Auth: Post-auth token check", {
        tokenExists: !!tokenAfterAuth,
        isAuthenticated: isAuthAfterAuth,
        tokenLength: tokenAfterAuth?.length || 0,
      });

      // Invalidate queries for UI updates
      queryClient.invalidateQueries({ queryKey: ["user-profile"] });

      return authResult;
    } catch (error) {
      console.error("❌ Ethereum Auth: Authentication failed:", error);
      throw error;
    } finally {
      setIsAuthenticating(false);
    }
  }, [
    address,
    isConnected,
    getNonce,
    verifyWallet,
    signMessageAsync,
    queryClient,
  ]);

  return {
    authenticate,
    isAuthenticating,
    isReady: isConnected && !!address,
    walletAddress: address,
  };
};

/**
 * Hook for NEAR authentication
 */
export const useNearAuth = () => {
  const { accountId, isConnected } = useNearWallet();
  const getNonce = useGetNonce();
  const verifyWallet = useVerifyWallet();
  const [isAuthenticating, setIsAuthenticating] = useState(false);

  const authenticate = useCallback(async () => {
    if (!accountId || !isConnected) {
      throw new Error("NEAR wallet not connected");
    }

    setIsAuthenticating(true);

    try {
      console.log("🔐 NEAR Auth: Starting authentication for:", accountId);

      // 1. Get nonce
      const nonceData = await getNonce.mutateAsync({
        walletAddress: accountId,
        chainType: "near",
      });

      // 2. Use message from backend nonce response
      const message = nonceData.message;
      console.log(
        "📝 NEAR Auth: Using message from nonce response for signing"
      );

      // TODO: Implement message signing through NEAR wallet
      // For now use mock signature for testing
      const signature = `near_signature_${Date.now()}`;
      console.log("✍️ NEAR Auth: Message signed (mock implementation)");

      // 3. Verify signature on backend
      const authResult = await verifyWallet.mutateAsync({
        walletAddress: accountId,
        signature,
        nonce: nonceData.nonce,
        chainType: "near",
        message,
      });

      console.log("🎉 NEAR Auth: Authentication completed successfully");
      return authResult;
    } catch (error) {
      console.error("❌ NEAR Auth: Authentication failed:", error);
      throw error;
    } finally {
      setIsAuthenticating(false);
    }
  }, [accountId, isConnected, getNonce, verifyWallet]);

  return {
    authenticate,
    isAuthenticating,
    isReady: isConnected && !!accountId,
    walletAddress: accountId,
  };
};

/**
 * Hook for logout
 */
export const useLogout = () => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async () => {
      await authService.logout();
    },
    onSuccess: () => {
      console.log("✅ useLogout: Logged out successfully");
      // Clear all cache on logout
      queryClient.clear();
    },
    onError: (error) => {
      console.error("❌ useLogout: Logout failed:", error);
    },
  });
};

/**
 * Hook for checking authentication status
 */
export const useAuthStatus = () => {
  const [isAuthenticated, setIsAuthenticated] = useState(false);
  const [token, setToken] = useState(authService.getToken());

  // Check both Wagmi and custom wallet connection states
  const { isConnected: isWagmiConnected } = useAccount();
  const [isCustomWalletConnected, setIsCustomWalletConnected] = useState(false);

  // Check custom wallet connection state
  useEffect(() => {
    const checkCustomWallet = async () => {
      try {
        const { getWalletManager } = await import("../../services/wallet");
        const walletManager = getWalletManager();
        const state = walletManager.getState();
        setIsCustomWalletConnected(state.isConnected && !!state.account);
      } catch {
        setIsCustomWalletConnected(false);
      }
    };

    checkCustomWallet();
    const interval = setInterval(checkCustomWallet, 2000);
    return () => clearInterval(interval);
  }, []);

  // Subscribe to token and wallet connection changes
  useEffect(() => {
    const checkAuth = () => {
      const hasToken = authService.isAuthenticated();
      const currentToken = authService.getToken();
      const hasWalletConnection = isWagmiConnected || isCustomWalletConnected;

      // User is authenticated only if they have BOTH token AND wallet connection
      const currentAuth = hasToken && hasWalletConnection;

      // Only log when there's a state change to avoid spam
      if (currentAuth !== isAuthenticated) {
        console.log(`🔍 AuthStatus Debug: Auth state changing`, {
          hasToken,
          tokenExists: !!currentToken,
          tokenPreview: currentToken
            ? currentToken.substring(0, 20) + "..."
            : "none",
          isWagmiConnected,
          isCustomWalletConnected,
          hasWalletConnection,
          from: isAuthenticated,
          to: currentAuth,
          localStorage: localStorage.getItem(API_CONFIG.TOKEN_STORAGE_KEY)
            ? "exists"
            : "missing",
        });
      }

      if (currentAuth !== isAuthenticated) {
        console.log(
          `🔐 AuthStatus: Authentication state changed: ${currentAuth}`,
          {
            hasToken,
            isWagmiConnected,
            isCustomWalletConnected,
            hasWalletConnection,
          }
        );
        setIsAuthenticated(currentAuth);
      }

      if (currentToken !== token) {
        setToken(currentToken);
      }
    };

    // Check immediately on mount
    checkAuth();

    // Set interval for periodic checking
    const interval = setInterval(checkAuth, 1000);

    return () => clearInterval(interval);
  }, [isAuthenticated, token, isWagmiConnected, isCustomWalletConnected]);

  return {
    isAuthenticated,
    token,
  };
};

/**
 * Hook for automatic authentication initialization
 * Checks the stored token on application load
 */
export const useAuthInit = () => {
  const queryClient = useQueryClient();
  const [isInitialized, setIsInitialized] = useState(false);

  useEffect(() => {
    const initAuth = async () => {
      console.log("🔄 Auth Init: Checking stored authentication...");

      const token = authService.getToken();
      if (token) {
        console.log("✅ Auth Init: Found stored token, user is authenticated");
        // Invalidate all auth-related queries for UI updates
        queryClient.invalidateQueries({ queryKey: ["user-profile"] });
        queryClient.invalidateQueries({ queryKey: ["auth"] });
      } else {
        console.log("📝 Auth Init: No stored token found");
      }

      setIsInitialized(true);
    };

    initAuth();
  }, [queryClient]);

  return {
    isInitialized,
    isAuthenticated: authService.isAuthenticated(),
  };
};
