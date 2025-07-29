/**
 * Wallet Page Component
 * Main page for wallet connection and management
 */

import React, { useEffect } from "react";
import { useWallet } from "../../hooks/wallet/useWallet";
import { useAuthStatus, useLogout } from "../../hooks/api/useAuth";
import { useUserInfo } from "../../hooks/api/useUser";
import { AuthManager } from "../../components/auth/AuthManager/AuthManager";
import "./WalletPage.scss";

export const WalletPage: React.FC = () => {
  console.log("🏗️ WalletPage: Component rendering");

  // Wallet connection status (MetaMask/NEAR)
  const { state, connect, isConnected } = useWallet();
  const isWalletConnected = isConnected;

  // Backend authentication status
  const { isAuthenticated } = useAuthStatus();
  const { profile, isLoading: isProfileLoading } = useUserInfo();
  const logout = useLogout();


  console.log("📊 WalletPage: State:", {
    isWalletConnected,
    isAuthenticated,
    hasProfile: !!profile,
    walletAddress: state.address,
  });

  // Logout handler
  const handleLogout = async () => {
    try {
      await logout.mutateAsync();
      console.log("✅ WalletPage: Logout successful");
    } catch (error) {
      console.error("❌ WalletPage: Logout failed:", error);
    }
  };

  // If wallet is not connected or not authenticated
  if (!isWalletConnected || !isAuthenticated) {
    return (
      <div className="wallet-page wallet-page--onboarding">
        <div className="wallet-page__container">
          <div className="onboarding">
            <div className="onboarding__icon">🔗</div>
            <h1 className="onboarding__title">Welcome to KEMBridge</h1>
            <p className="onboarding__description">
              Quantum-secured cross-chain bridge for safe asset transfers
            </p>

            {/* Authentication Section */}
            <div className="onboarding__auth">
              <AuthManager
                onAuthSuccess={() => {
                  console.log("🎉 WalletPage: Authentication successful");
                }}
                onAuthError={(error) => {
                  console.error("❌ WalletPage: Authentication failed:", error);
                }}
              />
            </div>

            {/* Connection Status */}
            <div className="onboarding__status">
              <div className="status-indicators">
                <div
                  className={`status-indicator ${
                    isWalletConnected
                      ? "status-indicator--connected"
                      : "status-indicator--disconnected"
                  }`}
                >
                  <div className="status-indicator__dot" />
                  <span className="status-indicator__text">
                    Wallet {isWalletConnected ? "Connected" : "Disconnected"}
                  </span>
                </div>

                <div
                  className={`status-indicator ${
                    isAuthenticated
                      ? "status-indicator--connected"
                      : "status-indicator--disconnected"
                  }`}
                >
                  <div className="status-indicator__dot" />
                  <span className="status-indicator__text">
                    Backend{" "}
                    {isAuthenticated ? "Authenticated" : "Not Authenticated"}
                  </span>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    );
  }

  // Show loading if profile is loading
  if (isProfileLoading) {
    return (
      <div className="wallet-page wallet-page--loading">
        <div className="wallet-page__container">
          <div className="loading-message">
            <div className="loading-spinner">⏳</div>
            <span>Loading user profile...</span>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="wallet-page">
      <div className="wallet-page__container">
        {/* User Profile Header */}
        {profile && (
          <div className="user-profile-header">
            <div className="user-profile-header__main">
              <div className="user-profile-header__info">
                <h2 className="user-profile-header__title">
                  Welcome! 👋
                </h2>
                <div className="user-profile-header__details">
                  <span className={`user-tier user-tier--${profile.tier}`}>
                    {profile.tier.toUpperCase()}
                  </span>
                  {profile.risk_profile && (
                    <span
                      className={`risk-level risk-level--${profile.risk_profile.level}`}
                    >
                      Risk: {profile.risk_profile.level.toUpperCase()}
                    </span>
                  )}
                </div>
              </div>

              <div className="user-profile-header__wallets">
                {profile.wallet_addresses.slice(0, 2).map((address) => (
                  <div key={address} className="wallet-chip">
                    <span className="wallet-chip__icon">
                      {address.startsWith("0x") ? "🦊" : "🔷"}
                    </span>
                    <span className="wallet-chip__address">
                      {address.slice(0, 6)}...{address.slice(-4)}
                    </span>
                  </div>
                ))}
              </div>
            </div>
          </div>
        )}

        {/* Wallet Dashboard */}
        <div className="wallet-dashboard">
          <h2 className="wallet-dashboard__title">Dashboard</h2>
          <p className="wallet-dashboard__description">
            Manage your assets through quantum-secured bridge
          </p>
        </div>

        {/* Quick Actions */}
        <div className="quick-actions">
          <h2 className="quick-actions__title">Quick Actions</h2>
          <div className="quick-actions__grid">
            <button
              className="quick-action-btn"
              onClick={() => {
                // TODO: Navigate to swap page
                console.log("🔄 Navigate to swap page");
              }}
            >
              🔄 Swap
            </button>
            <button
              className="quick-action-btn"
              onClick={() => {
                // TODO: Navigate to history page
                console.log("📋 Navigate to history page");
              }}
            >
              📋 History
            </button>
            <button
              className="quick-action-btn"
              onClick={() => {
                // TODO: Navigate to settings page
                console.log("⚙️ Navigate to settings page");
              }}
            >
              ⚙️ Settings
            </button>
            <button
              className="quick-action-btn"
              onClick={() => {
                // TODO: Show security modal
                console.log("🔐 Show security modal");
              }}
            >
              🔐 Security
            </button>
            <button
              className="quick-action-btn quick-action-btn--danger"
              onClick={handleLogout}
              disabled={logout.isPending}
            >
              {logout.isPending ? "⏳" : "🚪"} Logout
            </button>
          </div>
        </div>

        {/* Security Status */}
        <div className="security-status">
          <h3 className="security-status__title">🛡️ Security Status</h3>
          <div className="security-status__items">
            <div className="security-item">
              <span className="security-item__icon">⚛️</span>
              <span className="security-item__text">
                Quantum Protection Active
              </span>
              <span className="security-item__status security-item__status--active">
                ✓
              </span>
            </div>
            <div className="security-item">
              <span className="security-item__icon">🤖</span>
              <span className="security-item__text">AI Risk Analysis</span>
              <span className="security-item__status security-item__status--active">
                ✓
              </span>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};
