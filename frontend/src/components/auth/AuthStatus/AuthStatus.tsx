/**
 * Authentication status component
 * Shows current auth state and provides login/logout actions
 */

import React from "react";
import { useAuth } from "../../../hooks/auth/useAuth";
import { useWallet } from "../../../hooks/wallet/useWallet";
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
  const {
    isAuthenticated,
    user,
    isAuthenticating,
    authError,
    isBackendConnected,
    authenticateWithWallet,
    logout,
    clearAuthError,
  } = useAuth();

  const { isConnected, account } = useWallet();

  const handleAuthenticate = async () => {
    if (!isConnected) return;

    clearAuthError();
    await authenticateWithWallet();
  };

  const handleLogout = async () => {
    await logout();
  };

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

    if (isAuthenticated && user) {
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
                ? `Connected (${account?.address.slice(0, 8)}...)`
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
                ? `Authenticated (${user?.id.slice(0, 8)}...)`
                : "Not Authenticated"}
            </span>
          </div>

          {/* User Info */}
          {isAuthenticated && user && (
            <div className="auth-status__user">
              <div className="auth-status__row">
                <span className="auth-status__label">User ID:</span>
                <span className="auth-status__value">{user.id}</span>
              </div>
              <div className="auth-status__row">
                <span className="auth-status__label">Wallet:</span>
                <span className="auth-status__value">
                  {user.wallet_address}
                </span>
              </div>
              <div className="auth-status__row">
                <span className="auth-status__label">Wallets:</span>
                <span className="auth-status__value">
                  {user.wallets.length} connected
                </span>
              </div>
            </div>
          )}

          {/* Error Display */}
          {authError && (
            <div className="auth-status__error">
              <span className="auth-status__error-text">{authError}</span>
              <Button
                variant="ghost"
                size="sm"
                onClick={clearAuthError}
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
