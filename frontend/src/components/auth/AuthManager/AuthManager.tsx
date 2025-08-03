import { FC, memo, useState, useCallback } from "react";
import { useAccount, useConnect } from "wagmi";
import {
  useEthereumAuth,
  useNearAuth,
  useAuthStatus,
} from "../../../hooks/api/useAuth";
import { useNearWallet } from "../../../hooks/wallet/useNearWallet";
import { useUserProfile } from "../../../hooks/api/useUser";
import { Spinner } from "../../ui/Spinner/Spinner";
import { WebSocketStatus } from "../../websocket/WebSocketStatus/WebSocketStatus"; // Добавляем импорт
import styles from "./AuthManager.module.scss";
import cn from "classnames";

interface AuthManagerProps {
  onAuthSuccess?: () => void;
  onAuthError?: (error: Error) => void;
}

export const AuthManager: FC<AuthManagerProps> = memo(
  ({ onAuthSuccess, onAuthError }) => {
    const [authMethod, setAuthMethod] = useState<"ethereum" | "near" | null>(
      null
    );
    const [error, setError] = useState<string | null>(null);

    // Authentication state
    const { isAuthenticated } = useAuthStatus();
    const { data: userProfile, isLoading: isProfileLoading } = useUserProfile();

    // Ethereum auth
    const { isConnected: isEthConnected } = useAccount();
    const { connect, connectors } = useConnect();
    const ethereumAuth = useEthereumAuth();

    // NEAR auth
    const nearAuth = useNearAuth();
    const nearWallet = useNearWallet();

    // Memoized Ethereum authentication handler
    const handleEthereumAuth = useCallback(async () => {
      try {
        setError(null);
        setAuthMethod("ethereum");

        // Check if wallet is connected, if not - connect first
        if (!ethereumAuth.isReady) {
          console.log(
            "🔗 AuthManager: Ethereum wallet not connected, initiating connection..."
          );

          // Find MetaMask connector
          const metamaskConnector = connectors.find(
            (connector) =>
              connector.name.toLowerCase().includes("metamask") ||
              connector.id === "metaMask"
          );

          if (metamaskConnector) {
            await connect({ connector: metamaskConnector });
            // Wait for connection to establish
            return;
          } else {
            throw new Error("MetaMask connector not found");
          }
        }

        const result = await ethereumAuth.authenticate();
        console.log(
          "🎉 AuthManager: Ethereum authentication successful:",
          result
        );

        onAuthSuccess?.();
      } catch (error) {
        console.error("❌ AuthManager: Ethereum authentication failed:", error);
        const errorMessage =
          error instanceof Error
            ? error.message
            : "Ethereum authentication failed";
        setError(errorMessage);
        onAuthError?.(error instanceof Error ? error : new Error(errorMessage));
      } finally {
        setAuthMethod(null);
      }
    }, [ethereumAuth, connect, connectors, onAuthSuccess, onAuthError]);

    // Memoized NEAR authentication handler
    const handleNearAuth = useCallback(async () => {
      try {
        setError(null);
        setAuthMethod("near");

        // Check if wallet is connected, if not - connect first
        if (!nearAuth.isReady) {
          console.log(
            "🔗 AuthManager: NEAR wallet not connected, initiating connection..."
          );

          // Trigger wallet connection modal
          nearWallet.signIn();

          // Wait for connection to establish
          // Note: This might require user interaction and page redirect
          return;
        }

        const result = await nearAuth.authenticate();
        console.log("🎉 AuthManager: NEAR authentication successful:", result);

        onAuthSuccess?.();
      } catch (error) {
        console.error("❌ AuthManager: NEAR authentication failed:", error);
        const errorMessage =
          error instanceof Error ? error.message : "NEAR authentication failed";
        setError(errorMessage);
        onAuthError?.(error instanceof Error ? error : new Error(errorMessage));
      } finally {
        setAuthMethod(null);
      }
    }, [nearAuth, nearWallet, onAuthSuccess, onAuthError]);

    // If user is already authenticated, show profile
    if (isAuthenticated && userProfile) {
      return (
        <div className={cn(styles.authManager, styles.authenticated)}>
          <div className={styles.profile} data-testid="user-profile">
            <div className={styles.status} data-testid="authentication-status">
              <span className={styles.statusIcon}>✅</span>
              <span className={styles.statusText}>Authenticated</span>
            </div>

            <div className={styles.userInfo}>
              <div className={styles.tier}>
                Tier:{" "}
                <span
                  className={cn(
                    styles.tierBadge,
                    styles[
                      `tierBadge_${userProfile.tier}` as keyof typeof styles
                    ]
                  )}
                >
                  {userProfile.tier.toUpperCase()}
                </span>
              </div>

              <div className={styles.wallets}>
                {userProfile.wallet_addresses.slice(0, 2).map((address) => (
                  <div key={address} className={styles.wallet}>
                    <span className={styles.walletIcon}>
                      {address.startsWith("0x") ? "🦊" : "🔷"}
                    </span>
                    <span className={styles.walletAddress}>
                      {address.slice(0, 6)}...{address.slice(-4)}
                    </span>
                  </div>
                ))}
                {userProfile.wallet_addresses.length > 2 && (
                  <span className={styles.walletMore}>
                    +{userProfile.wallet_addresses.length - 2} more
                  </span>
                )}
              </div>
            </div>
          </div>
        </div>
      );
    }

    // If loading profile after authentication
    if (isAuthenticated && isProfileLoading) {
      return (
        <div className={cn(styles.authManager, styles.loading)}>
          <Spinner size="sm" />
          <span>Loading profile...</span>
        </div>
      );
    }

    // If not authenticated, show authentication buttons
    return (
      <div className={styles.authManager}>
        <div className={styles.header}>
          <div className={styles.headerLeft}>
            <WebSocketStatus
              compact={true}
              className={styles.websocketStatus}
            />
          </div>
          <div className={styles.headerRight}>
            <h3 className={styles.title}>Sign in to KEMBridge</h3>
          </div>
        </div>

        <p className={styles.description}>
          Connect your wallet to access cross-chain bridge features
        </p>

        {error && (
          <div className={styles.error} data-testid="auth-error">
            <span className={styles.errorIcon}>⚠️</span>
            <span className={styles.errorText}>{error}</span>
          </div>
        )}

        <div className={styles.methods}>
          <button
            className={cn(styles.method, styles.method_ethereum)}
            onClick={handleEthereumAuth}
            disabled={authMethod !== null}
            data-testid="ethereum-wallet-button"
          >
            {authMethod === "ethereum" ? (
              <>
                <Spinner size="sm" />
                <span>Authenticating...</span>
              </>
            ) : (
              <>
                <span className={styles.methodIcon}>🦊</span>
                <div className={styles.methodContent}>
                  <span className={styles.methodTitle}>Ethereum Wallet</span>
                  <span className={styles.methodSubtitle}>
                    {isEthConnected ? "Sign message" : "Connect wallet"}
                  </span>
                </div>
              </>
            )}
          </button>

          <button
            className={cn(styles.method, styles.method_near)}
            onClick={handleNearAuth}
            disabled={authMethod !== null}
            data-testid="near-wallet-button"
          >
            {authMethod === "near" ? (
              <>
                <Spinner size="sm" />
                <span>Authenticating...</span>
              </>
            ) : (
              <>
                <span className={styles.methodIcon}>🔷</span>
                <div className={styles.methodContent}>
                  <span className={styles.methodTitle}>NEAR Wallet</span>
                  <span className={styles.methodSubtitle}>
                    {nearAuth.isReady ? "Sign message" : "Connect wallet"}
                  </span>
                </div>
              </>
            )}
          </button>
        </div>

        <div className={styles.securityNote} data-testid="auth-security-note">
          <span className={styles.securityIcon}>🛡️</span>
          <span className={styles.securityText}>
            Signing requires no gas and is completely secure
          </span>
        </div>
      </div>
    );
  }
);
