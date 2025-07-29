/**
 * Authentication Hooks
 * React hooks for Web3 authentication with TanStack Query
 */

import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useState, useCallback, useEffect } from "react";
import { authService } from "../../services/api/authService";
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
  const [error, setError] = useState<Error | null>(null);

  const authenticate = useCallback(async () => {
    if (!address || !isConnected) {
      throw new Error("Ethereum wallet not connected");
    }

    setIsAuthenticating(true);
    setError(null);

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
    } catch (err) {
      console.error("❌ Ethereum Auth: Authentication failed:", err);
      const authError = err instanceof Error ? err : new Error(String(err));
      setError(authError);
      throw authError;
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
    isPending: isAuthenticating,
    error,
    reset: () => {
      setError(null);
      setIsAuthenticating(false);
    },
  };
};

/**
 * Hook for NEAR authentication
 */
export const useNearAuth = () => {
  const { accountId, isConnected } = useNearWallet();
  // TODO: Implement NEAR authentication
  // const getNonce = useGetNonce();
  // const verifyWallet = useVerifyWallet();
  const [isAuthenticating, setIsAuthenticating] = useState(false);
  const [error, setError] = useState<Error | null>(null);

  const authenticate = useCallback(async () => {
    if (!accountId || !isConnected) {
      throw new Error("NEAR wallet not connected");
    }

    setIsAuthenticating(true);
    setError(null);

    try {
      console.log("🔐 NEAR Auth: Starting authentication for:", accountId);

      console.log(
        "📝 NEAR Auth: Using message from nonce response for signing"
      );

      // Real NEAR message signing required - no mock implementation
      throw new Error(
        "NEAR wallet message signing not implemented - real signature required"
      );
    } catch (err) {
      console.error("❌ NEAR Auth: Authentication failed:", err);
      const authError = err instanceof Error ? err : new Error(String(err));
      setError(authError);
      throw authError;
    } finally {
      setIsAuthenticating(false);
    }
  }, [accountId, isConnected]);

  return {
    authenticate,
    isAuthenticating,
    isReady: isConnected && !!accountId,
    walletAddress: accountId,
    isPending: isAuthenticating,
    error,
    reset: () => {
      setError(null);
      setIsAuthenticating(false);
    },
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

// Global state to prevent duplicate AuthStatus logs
let lastLoggedAuthState: string | null = null;

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
    const interval = setInterval(checkCustomWallet, 10000);
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

      // Only log significant changes and avoid duplicate logs across all hook instances
      if (currentAuth !== isAuthenticated) {
        const logMessage = `${currentAuth}-${hasToken}-${hasWalletConnection}`;
        if (logMessage !== lastLoggedAuthState) {
          console.log(
            `🔐 AuthStatus: Authentication changed to: ${currentAuth}`
          );
          lastLoggedAuthState = logMessage;
        }
        setIsAuthenticated(currentAuth);
      }

      if (currentToken !== token) {
        setToken(currentToken);
      }
    };

    // Check immediately on mount
    checkAuth();

    // Set interval for periodic checking (reduced frequency)
    const interval = setInterval(checkAuth, 10000);

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
