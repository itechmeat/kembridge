/**
 * Authentication status component
 * Shows current auth state and provides login/logout actions
 * Enhanced with improved wallet connection handling
 */

import React, { useState } from "react";
import {
  useAuthStatus,
  useEthereumAuth,
  useNearAuth,
  useLogout,
} from "../../../hooks/api/useAuth";
import { useWallet } from "../../../hooks/wallet/useWallet";
import { useUserInfo } from "../../../hooks/api/useUser";
import { Button } from "../../ui/Button";
import { Spinner } from "../../ui/Spinner";
import "./AuthStatus.scss";

interface AuthStatusProps {
  showFullStatus?: boolean;
  className?: string;
}

export const AuthStatus: React.FC<AuthStatusProps> = ({
  showFullStatus = false,
  className = "",
}) => {
  const { isAuthenticated } = useAuthStatus();
  const { profile } = useUserInfo();
  const { state, isConnected, account } = useWallet();
  const ethereumAuth = useEthereumAuth();
  const nearAuth = useNearAuth();
  const logout = useLogout();
  const [authError, setAuthError] = useState<string | null>(null);

  const handleAuthenticate = async () => {
    if (!isConnected) return;

    setAuthError(null);

    try {
      if (account?.type === "near") {
        await nearAuth.authenticate();
      } else {
        await ethereumAuth.authenticate();
      }
      console.log("✅ Authentication successful");
    } catch (error) {
      const errorMessage =
        error instanceof Error ? error.message : "Authentication failed";
      console.error("❌ Authentication failed:", errorMessage);
      setAuthError(errorMessage);
    }
  };

  const handleLogout = async () => {
    try {
      await logout.mutateAsync();
    } catch (error) {
      console.error("Logout failed:", error);
    }
  };

  const clearError = () => {
    setAuthError(null);
  };

  const isAuthenticating = ethereumAuth.isPending || nearAuth.isPending;
  const hookAuthError = ethereumAuth.error?.message || nearAuth.error?.message;
  const displayError = authError || hookAuthError;
  const isBackendConnected = true; // Backend is always connected for this demo

  const getStatusIndicator = () => {
    if (!isBackendConnected) {
      return (
        <div className="auth-status__indicator auth-status__indicator--error">
          <span className="auth-status__dot"></span>
          Backend Offline
        </div>
      );
    }

    if (!isConnected) {
      return (
        <div className="auth-status__indicator auth-status__indicator--warning">
          <span className="auth-status__dot"></span>
          No Wallet
        </div>
      );
    }

    if (isAuthenticating) {
      return (
        <div className="auth-status__indicator auth-status__indicator--loading">
          <Spinner size="sm" />
          Authenticating...
        </div>
      );
    }

    if (isConnected && !isAuthenticated) {
      return (
        <div className="auth-status__indicator auth-status__indicator--info">
          <span className="auth-status__dot"></span>
          Wallet Connected
        </div>
      );
    }

    if (isAuthenticated && profile) {
      return (
        <div className="auth-status__indicator auth-status__indicator--success">
          <span className="auth-status__dot"></span>
          Authenticated
        </div>
      );
    }

    return (
      <div className="auth-status__indicator auth-status__indicator--warning">
        <span className="auth-status__dot"></span>
        Not Authenticated
      </div>
    );
  };

  return (
    <div
      className={`auth-status ${
        showFullStatus ? "auth-status--full" : ""
      } ${className}`}
    >
      {/* Status Indicator */}
      {getStatusIndicator()}

      {/* Full Status View */}
      {showFullStatus && (
        <div className="auth-status__details">
          {/* Backend Status */}
          <div className="auth-status__row">
            <span className="auth-status__label">Backend:</span>
            <span
              className={`auth-status__value ${
                isBackendConnected
                  ? "auth-status__value--success"
                  : "auth-status__value--error"
              }`}
            >
              {isBackendConnected ? "Connected" : "Offline"}
            </span>
          </div>

          {/* Wallet Status */}
          <div className="auth-status__row">
            <span className="auth-status__label">Wallet:</span>
            <span
              className={`auth-status__value ${
                isConnected
                  ? "auth-status__value--success"
                  : "auth-status__value--error"
              }`}
            >
              {isConnected
                ? `Connected (${account?.address?.slice(0, 8)}...)`
                : "Not Connected"}
            </span>
          </div>

          {/* Authentication Status */}
          <div className="auth-status__row">
            <span className="auth-status__label">Auth:</span>
            <span
              className={`auth-status__value ${
                isAuthenticated
                  ? "auth-status__value--success"
                  : "auth-status__value--error"
              }`}
            >
              {isAuthenticated
                ? `Authenticated (${profile?.id?.slice(0, 8)}...)`
                : "Not Authenticated"}
            </span>
          </div>

          {/* Wallet Details */}
          {isConnected && account && (
            <>
              <div className="auth-status__row">
                <span className="auth-status__label">Type:</span>
                <span className="auth-status__value">{account.type}</span>
              </div>

              {account.address && (
                <div className="auth-status__row">
                  <span className="auth-status__label">Address:</span>
                  <span
                    className="auth-status__value"
                    style={{ fontSize: "0.8em" }}
                  >
                    {account.address}
                  </span>
                </div>
              )}

              {account.chainId && (
                <div className="auth-status__row">
                  <span className="auth-status__label">Chain ID:</span>
                  <span className="auth-status__value">{account.chainId}</span>
                </div>
              )}
            </>
          )}

          {/* User Info */}
          {isAuthenticated && profile && (
            <div className="auth-status__user">
              <div className="auth-status__row">
                <span className="auth-status__label">User ID:</span>
                <span className="auth-status__value">{profile.id}</span>
              </div>
              <div className="auth-status__row">
                <span className="auth-status__label">Tier:</span>
                <span className="auth-status__value">{profile.tier}</span>
              </div>
              <div className="auth-status__row">
                <span className="auth-status__label">Wallets:</span>
                <span className="auth-status__value">
                  {profile.wallet_addresses.length} connected
                </span>
              </div>
            </div>
          )}

          {/* Error Display */}
          {(state.error || displayError) && (
            <div className="auth-status__error">
              <span className="auth-status__error-text">
                {displayError || state.error}
              </span>
              <Button
                variant="ghost"
                size="sm"
                onClick={() => {
                  clearError();
                  ethereumAuth.reset();
                  nearAuth.reset();
                }}
                className="auth-status__error-clear"
              >
                ✕
              </Button>
            </div>
          )}

          {/* Actions */}
          <div className="auth-status__actions">
            {isConnected && !isAuthenticated && !isAuthenticating && (
              <Button
                variant="primary"
                size="sm"
                onClick={handleAuthenticate}
                disabled={!isBackendConnected}
              >
                Sign In with Wallet
              </Button>
            )}

            {isAuthenticated && (
              <Button variant="secondary" size="sm" onClick={handleLogout}>
                Sign Out
              </Button>
            )}
          </div>
        </div>
      )}
    </div>
  );
};
