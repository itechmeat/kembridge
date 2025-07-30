/**
 * Auth Manager Component
 * Manages authentication process through Web3 wallets
 */

import React, { useState } from "react";
import { useAccount, useConnect } from "wagmi";
import {
  useEthereumAuth,
  useNearAuth,
  useAuthStatus,
} from "../../../hooks/api/useAuth";
import { useNearWallet } from "../../../hooks/wallet/useNearWallet";
import { useUserProfile } from "../../../hooks/api/useUser";
import { Spinner } from "../../ui/Spinner";
import "./AuthManager.scss";

interface AuthManagerProps {
  onAuthSuccess?: () => void;
  onAuthError?: (error: Error) => void;
}

export const AuthManager: React.FC<AuthManagerProps> = ({
  onAuthSuccess,
  onAuthError,
}) => {
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

  // Ethereum authentication handler
  const handleEthereumAuth = async () => {
    try {
      setError(null);
      setAuthMethod("ethereum");

      // Check if wallet is connected, if not - connect first
      if (!ethereumAuth.isReady) {
        console.log("🔗 AuthManager: Ethereum wallet not connected, initiating connection...");
        
        // Find MetaMask connector
        const metamaskConnector = connectors.find(connector => 
          connector.name.toLowerCase().includes('metamask') || 
          connector.id === 'metaMask'
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
  };

  // NEAR authentication handler
  const handleNearAuth = async () => {
    try {
      setError(null);
      setAuthMethod("near");

      // Check if wallet is connected, if not - connect first
      if (!nearAuth.isReady) {
        console.log("🔗 AuthManager: NEAR wallet not connected, initiating connection...");
        
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
  };

  // If user is already authenticated, show profile
  if (isAuthenticated && userProfile) {
    return (
      <div className="auth-manager auth-manager--authenticated">
        <div className="auth-manager__profile">
          <div className="auth-manager__status">
            <span className="auth-manager__status-icon">✅</span>
            <span className="auth-manager__status-text">Authenticated</span>
          </div>

          <div className="auth-manager__user-info">
            <div className="auth-manager__tier">
              Tier:{" "}
              <span
                className={`auth-manager__tier-badge auth-manager__tier-badge--${userProfile.tier}`}
              >
                {userProfile.tier.toUpperCase()}
              </span>
            </div>

            <div className="auth-manager__wallets">
              {userProfile.wallet_addresses.slice(0, 2).map((address) => (
                <div key={address} className="auth-manager__wallet">
                  <span className="auth-manager__wallet-icon">
                    {address.startsWith("0x") ? "🦊" : "🔷"}
                  </span>
                  <span className="auth-manager__wallet-address">
                    {address.slice(0, 6)}...{address.slice(-4)}
                  </span>
                </div>
              ))}
              {userProfile.wallet_addresses.length > 2 && (
                <span className="auth-manager__wallet-more">
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
      <div className="auth-manager auth-manager--loading">
        <Spinner size="sm" />
        <span>Loading profile...</span>
      </div>
    );
  }

  // If not authenticated, show authentication buttons
  return (
    <div className="auth-manager">
      <h3 className="auth-manager__title">Sign in to KEMBridge</h3>
      <p className="auth-manager__description">
        Connect your wallet to access cross-chain bridge features
      </p>

      {error && (
        <div className="auth-manager__error">
          <span className="auth-manager__error-icon">⚠️</span>
          <span className="auth-manager__error-text">{error}</span>
        </div>
      )}

      <div className="auth-manager__methods">
        {/* Ethereum Authentication */}
        <button
          className="auth-manager__method auth-manager__method--ethereum"
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
              <span className="auth-manager__method-icon">🦊</span>
              <div className="auth-manager__method-content">
                <span className="auth-manager__method-title">
                  Ethereum Wallet
                </span>
                <span className="auth-manager__method-subtitle">
                  {isEthConnected
                    ? "Sign message"
                    : "Connect wallet"}
                </span>
              </div>
            </>
          )}
        </button>

        {/* NEAR Authentication */}
        <button
          className="auth-manager__method auth-manager__method--near"
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
              <span className="auth-manager__method-icon">🔷</span>
              <div className="auth-manager__method-content">
                <span className="auth-manager__method-title">NEAR Wallet</span>
                <span className="auth-manager__method-subtitle">
                  {nearAuth.isReady
                    ? "Sign message"
                    : "Connect wallet"}
                </span>
              </div>
            </>
          )}
        </button>
      </div>

      <div className="auth-manager__security-note">
        <span className="auth-manager__security-icon">🛡️</span>
        <span className="auth-manager__security-text">
          Signing requires no gas and is completely secure
        </span>
      </div>
    </div>
  );
};
