import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useState, useCallback, useEffect, useRef, useMemo } from "react";
import { authService } from "../../services/api/authService";
import { useAccount, useDisconnect } from "wagmi";
import { useSignMessage } from "wagmi";
import { Buffer } from "buffer";
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
      const allTokens = authService.getAllTokens();
      const isAuthAfterAuth = authService.isAuthenticated();
      console.log("🔍 Ethereum Auth: Post-auth token check", {
        tokenExists: !!tokenAfterAuth,
        isAuthenticated: isAuthAfterAuth,
        tokenLength: tokenAfterAuth?.length || 0,
        evmToken: !!allTokens.evm,
        nearToken: !!allTokens.near,
      });

      // Force trigger auth status update
      setTimeout(() => {
        window.dispatchEvent(
          new CustomEvent("auth-token-changed", {
            detail: {
              token: allTokens.evm || tokenAfterAuth,
              authenticated: true,
              source: "ethereum-auth-completed",
              walletType: "evm",
            },
          })
        );
      }, 100);

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
  const { accountId, isConnected, selector } = useNearWallet();
  const getNonce = useGetNonce();
  const verifyWallet = useVerifyWallet();
  const queryClient = useQueryClient();
  const [isAuthenticating, setIsAuthenticating] = useState(false);
  const [error, setError] = useState<Error | null>(null);

  const authenticate = useCallback(async () => {
    if (!accountId || !isConnected || !selector) {
      throw new Error("NEAR wallet not connected or selector not available");
    }

    setIsAuthenticating(true);
    setError(null);

    try {
      console.log("🔐 NEAR Auth: Starting authentication for:", accountId);

      // 1. Get nonce from backend
      const nonceData = await getNonce.mutateAsync({
        walletAddress: accountId,
        chainType: "near",
      });

      console.log("📝 NEAR Auth: Received nonce data:", {
        nonce: nonceData.nonce,
        nonceLength: nonceData.nonce?.length,
        nonceType: typeof nonceData.nonce,
        message: nonceData.message,
        messageLength: nonceData.message?.length,
      });

      // 2. Use message from backend nonce response
      const message = nonceData.message;
      console.log(
        "📝 NEAR Auth: Using message from nonce response for signing"
      );

      // 3. Sign message using NEAR Wallet Selector
      console.log("🖊️ NEAR Auth: Requesting signature for message:", message);

      const wallet = await selector.wallet();

      // NEAR wallet signing requires specific format
      const signRequest = {
        message: message,
        recipient: "kembridge.testnet",
        nonce: Buffer.from(nonceData.nonce, "hex"),
      };

      console.log("🔍 NEAR Auth: Sign request details:", {
        message: signRequest.message,
        recipient: signRequest.recipient,
        nonce: signRequest.nonce,
        nonceBuffer: Array.from(signRequest.nonce), // Show as array for debugging
        originalNonceHex: nonceData.nonce,
      });

      const signedMessage = await (
        wallet.signMessage as (request: typeof signRequest) => Promise<{
          accountId: string;
          publicKey: string;
          signature: string;
          state?: string;
        } | void>
      )(signRequest);

      if (!signedMessage) {
        throw new Error("Failed to sign message with NEAR wallet");
      }

      console.log("✍️ NEAR Auth: Message signed successfully", {
        signature: signedMessage.signature,
        publicKey: signedMessage.publicKey,
        accountId: signedMessage.accountId,
        signatureLength: signedMessage.signature?.length,
        signatureType: typeof signedMessage.signature,
      });

      // The signature is already a string from NEAR Wallet Selector
      // For backend compatibility, add "0x" prefix to NEAR signatures
      const signatureBase64 = signedMessage.signature;
      const formattedSignature = signatureBase64.startsWith("0x")
        ? signatureBase64
        : `0x${signatureBase64}`;

      console.log("🔍 NEAR Auth: Preparing verification request:", {
        walletAddress: accountId,
        signature: formattedSignature,
        originalSignature: signatureBase64,
        signatureLength: formattedSignature?.length,
        nonce: nonceData.nonce,
        nonceLength: nonceData.nonce?.length,
        chainType: "near",
        message: message,
        messageLength: message?.length,
      });

      // 4. Verify signature on backend
      const authResult = await verifyWallet.mutateAsync({
        walletAddress: accountId,
        signature: formattedSignature,
        nonce: nonceData.nonce,
        chainType: "near",
        message,
      });

      console.log("🎉 NEAR Auth: Authentication completed successfully");

      // Check if token is actually saved after authentication
      const tokenAfterAuth = authService.getToken();
      const allTokens = authService.getAllTokens();
      const isAuthAfterAuth = authService.isAuthenticated();
      console.log("🔍 NEAR Auth: Post-auth token check", {
        tokenExists: !!tokenAfterAuth,
        isAuthenticated: isAuthAfterAuth,
        tokenLength: tokenAfterAuth?.length || 0,
        evmToken: !!allTokens.evm,
        nearToken: !!allTokens.near,
      });

      // Force trigger auth status update
      setTimeout(() => {
        window.dispatchEvent(
          new CustomEvent("auth-token-changed", {
            detail: {
              token: allTokens.near || tokenAfterAuth,
              authenticated: true,
              source: "near-auth-completed",
              walletType: "near",
            },
          })
        );
      }, 100);

      // Invalidate queries for UI updates
      queryClient.invalidateQueries({ queryKey: ["user-profile"] });

      return authResult;
    } catch (err) {
      console.error("❌ NEAR Auth: Authentication failed:", err);
      const authError = err instanceof Error ? err : new Error(String(err));
      setError(authError);
      throw authError;
    } finally {
      setIsAuthenticating(false);
    }
  }, [accountId, isConnected, selector, getNonce, verifyWallet, queryClient]);

  return {
    authenticate,
    isAuthenticating,
    isReady: isConnected && !!accountId && !!selector,
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

  // Hooks for wallet disconnection
  const { disconnect: disconnectEVM } = useDisconnect();
  const nearWallet = useNearWallet();

  return useMutation({
    mutationFn: async (walletType?: "evm" | "near" | "all") => {
      if (walletType && walletType !== "all") {
        // Clear specific wallet type
        authService.clearTokenByType(walletType);

        // Disconnect and clear data for specific wallet
        if (walletType === "evm") {
          // Disconnect EVM wallet (MetaMask)
          try {
            await disconnectEVM();
            console.log("🔌 useLogout: Disconnected EVM wallet");
          } catch (error) {
            console.warn("⚠️ useLogout: EVM disconnect failed:", error);
          }

          // Clear EVM-specific localStorage items
          try {
            localStorage.removeItem("wagmi.store");
            localStorage.removeItem("wagmi.wallet");
            localStorage.removeItem("wagmi.cache");
            console.log("🗑️ useLogout: Cleared EVM localStorage data");
          } catch (error) {
            console.warn(
              "⚠️ useLogout: Failed to clear EVM localStorage:",
              error
            );
          }
        } else if (walletType === "near") {
          // Disconnect NEAR wallet
          try {
            await nearWallet.signOut();
            console.log("🔌 useLogout: Disconnected NEAR wallet");
          } catch (error) {
            console.warn("⚠️ useLogout: NEAR disconnect failed:", error);
          }

          // Clear NEAR-specific localStorage items
          try {
            localStorage.removeItem("near-wallet-selector");
            localStorage.removeItem("near_app_wallet_auth_key");
            localStorage.removeItem(
              "nearWalletConnection_keypom-wallet_default"
            );
            console.log("🗑️ useLogout: Cleared NEAR localStorage data");
          } catch (error) {
            console.warn(
              "⚠️ useLogout: Failed to clear NEAR localStorage:",
              error
            );
          }
        }

        console.log(
          `✅ useLogout: Logged out and disconnected ${walletType} wallet`
        );
      } else {
        // Clear all tokens
        await authService.logout();
        authService.clearTokenByType("evm");
        authService.clearTokenByType("near");

        // Disconnect all wallets
        try {
          // Disconnect EVM wallet
          await disconnectEVM();
          console.log("🔌 useLogout: Disconnected EVM wallet");
        } catch (error) {
          console.warn("⚠️ useLogout: EVM disconnect failed:", error);
        }

        try {
          // Disconnect NEAR wallet
          await nearWallet.signOut();
          console.log("🔌 useLogout: Disconnected NEAR wallet");
        } catch (error) {
          console.warn("⚠️ useLogout: NEAR disconnect failed:", error);
        }

        // Clear all wallet-related localStorage items
        try {
          localStorage.removeItem("kembridge_last_connected_wallet");
          localStorage.removeItem("wagmi.store");
          localStorage.removeItem("wagmi.wallet");
          localStorage.removeItem("wagmi.cache");
          localStorage.removeItem("near-wallet-selector");
          localStorage.removeItem("near_app_wallet_auth_key");
          localStorage.removeItem("nearWalletConnection_keypom-wallet_default");
          console.log("🗑️ useLogout: Cleared all wallet localStorage data");
        } catch (error) {
          console.warn("⚠️ useLogout: Failed to clear localStorage:", error);
        }

        console.log("✅ useLogout: Logged out and disconnected all wallets");
      }
    },
    onSuccess: (_, walletType) => {
      console.log("✅ useLogout: Logout successful for:", walletType || "all");

      // Trigger auth status update
      setTimeout(() => {
        window.dispatchEvent(
          new CustomEvent("auth-token-changed", {
            detail: {
              authenticated: false,
              source: "logout-completed",
              walletType: walletType || "all",
            },
          })
        );
      }, 50);

      // Clear relevant cache on logout
      if (!walletType || walletType === "all") {
        queryClient.clear();
      } else {
        queryClient.invalidateQueries({ queryKey: ["user-profile"] });
      }
    },
    onError: (error) => {
      console.error("❌ useLogout: Logout failed:", error);
    },
  });
};

/**
 * Hook for checking authentication status for specific wallet types
 */
export const useWalletAuthStatus = () => {
  const [evmToken, setEvmToken] = useState<string | null>(null);
  const [nearToken, setNearToken] = useState<string | null>(null);

  // Check wallet connection states
  const { isConnected: isWagmiConnected, address: evmAddress } = useAccount();
  const { isConnected: isNearConnected, accountId: nearAccountId } =
    useNearWallet();
  const [isCustomWalletConnected, setIsCustomWalletConnected] = useState(false);

  // Add debounce ref to prevent rapid updates
  const debounceTimeoutRef = useRef<number | null>(null);

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
    const interval = setInterval(checkCustomWallet, 30000); // Увеличили с 10сек до 30сек
    return () => clearInterval(interval);
  }, []);

  // Subscribe to token and wallet connection changes
  useEffect(() => {
    const checkAuth = () => {
      // Get tokens by wallet type from localStorage
      const allTokens = authService.getAllTokens();
      const currentEvmToken = allTokens.evm || null;
      const currentNearToken = allTokens.near || null;

      // Check wallet connections
      const hasEvmConnection = isWagmiConnected && !!evmAddress;
      const hasNearConnection = isNearConnected && !!nearAccountId;
      const hasCustomConnection = isCustomWalletConnected;

      // EVM authentication: token + (wagmi connected OR custom wallet)
      const isEvmAuthenticated =
        !!currentEvmToken && (hasEvmConnection || hasCustomConnection);

      // NEAR authentication: token + near connected
      const isNearAuthenticated = !!currentNearToken && hasNearConnection;

      // Only log auth state changes to reduce console spam
      if (import.meta.env.DEV && 
          (currentEvmToken !== evmToken || currentNearToken !== nearToken)) {
        console.log("🔐 WalletAuthStatus: Multi-wallet auth state changed:", {
          evmTokenChanged: currentEvmToken !== evmToken,
          nearTokenChanged: currentNearToken !== nearToken,
          isEvmAuthenticated,
          isNearAuthenticated,
        });
      }

      // Update states only if changed
      if (currentEvmToken !== evmToken) {
        setEvmToken(currentEvmToken);
      }
      if (currentNearToken !== nearToken) {
        setNearToken(currentNearToken);
      }
    };

    // Debounced version of checkAuth to prevent rapid updates
    const debouncedCheckAuth = () => {
      if (debounceTimeoutRef.current) {
        clearTimeout(debounceTimeoutRef.current);
      }

      debounceTimeoutRef.current = window.setTimeout(() => {
        checkAuth();
      }, 500); // Увеличили с 100ms до 500ms для снижения частоты вызовов
    };

    // Check immediately on mount
    checkAuth();

    // Listen for localStorage changes (when auth token is saved)
    const handleStorageChange = (e: StorageEvent) => {
      if (e.key?.includes("kembridge")) {
        console.log("🔄 WalletAuthStatus: Storage change detected:", {
          key: e.key,
          hasNewValue: !!e.newValue,
        });
        debouncedCheckAuth();
      }
    };

    // Listen for custom auth events
    const handleAuthEvent = (e: Event) => {
      const detail = (e as CustomEvent).detail;
      // Only log auth events in development and when there are actual changes
      if (import.meta.env.DEV && detail?.source) {
        console.log("🔄 WalletAuthStatus: Auth event:", detail.source);
      }
      debouncedCheckAuth();
    };

    // Add event listeners
    window.addEventListener("storage", handleStorageChange);
    window.addEventListener("auth-token-changed", handleAuthEvent);

    // Set interval for periodic checking (reduced frequency as fallback)
    const interval = setInterval(checkAuth, 60000); // Reduced from 30s to 60s to prevent spam

    return () => {
      if (debounceTimeoutRef.current) {
        clearTimeout(debounceTimeoutRef.current);
      }
      clearInterval(interval);
      window.removeEventListener("storage", handleStorageChange);
      window.removeEventListener("auth-token-changed", handleAuthEvent);
    };
  }, [evmToken, nearToken, isWagmiConnected, evmAddress, isNearConnected, nearAccountId, isCustomWalletConnected]); // Include all used dependencies

  // Memoized computed values to prevent unnecessary re-renders
  const authStatus = useMemo(() => ({
    // EVM wallet status
    isEvmAuthenticated:
      !!evmToken &&
      ((isWagmiConnected && !!evmAddress) || isCustomWalletConnected),
    evmAddress: evmAddress || null,
    evmToken,
    isEvmConnected:
      (isWagmiConnected && !!evmAddress) || isCustomWalletConnected,

    // NEAR wallet status
    isNearAuthenticated: !!nearToken && isNearConnected && !!nearAccountId,
    nearAddress: nearAccountId || null,
    nearToken,
    isNearConnected: isNearConnected && !!nearAccountId,

    // Overall status
    hasAnyAuth: !!evmToken || !!nearToken,
    hasBothAuth: !!evmToken && !!nearToken,
  }), [
    evmToken,
    nearToken,
    isWagmiConnected,
    evmAddress,
    isNearConnected,
    nearAccountId,
    isCustomWalletConnected,
  ]);

  return authStatus;
};

/**
 * Legacy hook for backwards compatibility
 * @deprecated Use useWalletAuthStatus instead
 */
export const useAuthStatus = () => {
  const { isEvmAuthenticated, isNearAuthenticated, evmToken, nearToken } =
    useWalletAuthStatus();

  return {
    isAuthenticated: isEvmAuthenticated || isNearAuthenticated,
    token: evmToken || nearToken || null, // EVM token has priority
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
