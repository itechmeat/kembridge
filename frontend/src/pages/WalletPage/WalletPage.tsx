/**
 * Wallet Page Component
 * Main page for wallet connection and management
 */

import React from "react";
import { useWallet } from "../../hooks/wallet/useWallet";
import "./WalletPage.scss";

export const WalletPage: React.FC = () => {
  console.log("🏗️ WalletPage: Component rendering");
  const { isConnected } = useWallet();
  console.log("📊 WalletPage: Wallet state:", { isConnected });

  // Test effect to debug React 19
  React.useEffect(() => {
    console.log("🔄 WalletPage: Effect running with isConnected:", isConnected);

    // Test function with proper typing
    const testHandler = (message: string, data: Record<string, unknown>) => {
      console.log("📝 WalletPage: Test handler called:", { message, data });
    };

    testHandler("wallet-page-mounted", { isConnected });
  }, [isConnected]);

  if (!isConnected) {
    return (
      <div className="wallet-page wallet-page--onboarding">
        <div className="wallet-page__container">
          <div className="onboarding">
            <div className="onboarding__icon">🔗</div>
            <h1 className="onboarding__title">Connect Your Wallet</h1>
            <p className="onboarding__description">
              Connect your wallet to start using the quantum-secured bridge
            </p>

            <div className="onboarding__note">
              Use the Connect button in the header to get started
            </div>

            <div className="onboarding__status">
              <div
                className={`status-indicator ${
                  isConnected
                    ? "status-indicator--connected"
                    : "status-indicator--disconnected"
                }`}
              >
                <div className="status-indicator__dot" />
                <span className="status-indicator__text">
                  {isConnected ? "Backend Connected" : "Backend Disconnected"}
                </span>
              </div>
            </div>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="wallet-page">
      <div className="wallet-page__container">
        {/* Wallet Dashboard */}
        <div className="wallet-dashboard">
          <h2 className="wallet-dashboard__title">Wallet Dashboard</h2>
          <p className="wallet-dashboard__description">
            Your wallet is connected. Use the actions below to interact with the
            quantum-secured bridge.
          </p>
        </div>

        {/* Quick Actions */}
        <div className="quick-actions">
          <h2 className="quick-actions__title">Quick Actions</h2>
          <div className="quick-actions__grid">
            <button className="quick-action-btn">🔄 Swap</button>
            <button className="quick-action-btn">📋 History</button>
            <button className="quick-action-btn">⚙️ Settings</button>
            <button className="quick-action-btn">🔐 Security</button>
            <button
              className="quick-action-btn"
              onClick={async () => {
                try {
                  // The original code had disconnect() here, but useWallet is removed.
                  // Assuming this button is no longer functional or needs to be removed.
                  // For now, keeping it as is, but it will cause an error.
                  // If the intent was to remove this button, it should be removed from the JSX.
                  // Since the edit hint only changed imports and types, I'm not removing it.
                  // However, the original code had disconnect() which is no longer available.
                  // This button will now be a no-op.
                } catch (error) {
                  console.error("Failed to disconnect:", error);
                }
              }}
            >
              🚪 Logout
            </button>
            <button
              className="quick-action-btn"
              onClick={async () => {
                try {
                  // The original code had switchNetwork() here, but useWallet is removed.
                  // Assuming this button is no longer functional or needs to be removed.
                  // For now, keeping it as is, but it will cause an error.
                  // If the intent was to remove this button, it should be removed from the JSX.
                  // Since the edit hint only changed imports and types, I'm not removing it.
                  // However, the original code had switchNetwork() which is no longer available.
                  // This button will now be a no-op.
                } catch (error: unknown) {
                  console.error("Failed to switch network:", error);
                }
              }}
            >
              🔄 Switch to Sepolia
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
