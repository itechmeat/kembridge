import { FC, useCallback, memo } from "react";
import { useWallet } from "../../hooks/wallet/useWallet";
import { useAuthStatus, useLogout } from "../../hooks/api/useAuth";
import { useUserInfo } from "../../hooks/api/useUser";
import { AuthManager } from "../../components/auth/AuthManager/AuthManager";
import classNames from "classnames";
import styles from "./WalletPage.module.scss";

export const WalletPage: FC = memo(() => {
  console.log("🏗️ WalletPage: Component rendering");

  // Wallet connection status (MetaMask/NEAR)
  const { state, isConnected } = useWallet();
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

  // Memoized logout handler to prevent re-renders
  const handleLogout = useCallback(async () => {
    try {
      await logout.mutateAsync("all");
      console.log("✅ WalletPage: Logout successful");
    } catch (error) {
      console.error("❌ WalletPage: Logout failed:", error);
    }
  }, [logout]);

  // Memoized auth success handler
  const handleAuthSuccess = useCallback(() => {
    console.log("🎉 WalletPage: Authentication successful");
  }, []);

  // Memoized auth error handler
  const handleAuthError = useCallback((error: Error) => {
    console.error("❌ WalletPage: Authentication failed:", error);
  }, []);

  // If wallet is not connected or not authenticated
  if (!isWalletConnected || !isAuthenticated) {
    return (
      <div className={classNames(styles.walletPage, styles.onboarding)}>
        <div className={styles.container}>
          <div className={styles.onboardingContent}>
            <div className={styles.onboardingIcon}>🔗</div>
            <h1 className={styles.onboardingTitle}>Welcome to KEMBridge</h1>
            <p className={styles.onboardingDescription}>
              Quantum-secured cross-chain bridge for safe asset transfers
            </p>

            {/* Authentication Section */}
            <div className={styles.onboardingAuth}>
              <AuthManager
                onAuthSuccess={handleAuthSuccess}
                onAuthError={handleAuthError}
              />
            </div>

            {/* Connection Status */}
            <div className={styles.onboardingStatus}>
              <div className={styles.statusIndicators}>
                <div
                  className={classNames(styles.statusIndicator, {
                    [styles.connected]: isWalletConnected,
                    [styles.disconnected]: !isWalletConnected,
                  })}
                >
                  <div className={styles.statusDot} />
                  <span className={styles.statusText}>
                    Wallet {isWalletConnected ? "Connected" : "Disconnected"}
                  </span>
                </div>

                <div
                  className={classNames(styles.statusIndicator, {
                    [styles.connected]: isAuthenticated,
                    [styles.disconnected]: !isAuthenticated,
                  })}
                >
                  <div className={styles.statusDot} />
                  <span className={styles.statusText}>
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
      <div className={classNames(styles.walletPage, styles.loading)}>
        <div className={styles.container}>
          <div className={styles.loadingMessage}>
            <div className={styles.loadingSpinner}>⏳</div>
            <span>Loading user profile...</span>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className={styles.walletPage}>
      <div className={styles.container}>
        {/* User Profile Header */}
        {profile && (
          <div className={styles.userProfileHeader}>
            <div className={styles.userProfileMain}>
              <div className={styles.userProfileInfo}>
                <h2 className={styles.userProfileTitle}>Welcome! 👋</h2>
                <div className={styles.userProfileDetails}>
                  <span
                    className={classNames(
                      styles.userTier,
                      styles[profile.tier]
                    )}
                  >
                    {profile.tier.toUpperCase()}
                  </span>
                  {profile.risk_profile && (
                    <span
                      className={classNames(
                        styles.riskLevel,
                        styles[profile.risk_profile.level]
                      )}
                    >
                      Risk: {profile.risk_profile.level.toUpperCase()}
                    </span>
                  )}
                </div>
              </div>

              <div className={styles.userProfileWallets}>
                {profile.wallet_addresses.slice(0, 2).map((address) => (
                  <div key={address} className={styles.walletChip}>
                    <span className={styles.walletChipIcon}>
                      {address.startsWith("0x") ? "🦊" : "🔷"}
                    </span>
                    <span className={styles.walletChipAddress}>
                      {address.slice(0, 6)}...{address.slice(-4)}
                    </span>
                  </div>
                ))}
              </div>
            </div>
          </div>
        )}

        {/* Wallet Dashboard */}
        <div className={styles.walletDashboard}>
          <h2 className={styles.walletDashboardTitle}>Dashboard</h2>
          <p className={styles.walletDashboardDescription}>
            Manage your assets through quantum-secured bridge
          </p>
        </div>

        {/* Quick Actions */}
        <div className={styles.quickActions}>
          <h2 className={styles.quickActionsTitle}>Quick Actions</h2>
          <div className={styles.quickActionsGrid}>
            <button
              className={styles.quickActionBtn}
              onClick={() => {
                // TODO: Navigate to swap page
                console.log("🔄 Navigate to swap page");
              }}
              data-testid="quick-action-swap"
            >
              🔄 Swap
            </button>
            <button
              className={styles.quickActionBtn}
              onClick={() => {
                // TODO: Navigate to history page
                console.log("📋 Navigate to history page");
              }}
              data-testid="bottom-nav-history"
            >
              📋 History
            </button>
            <button
              className={styles.quickActionBtn}
              onClick={() => {
                // TODO: Navigate to settings page
                console.log("⚙️ Navigate to settings page");
              }}
              data-testid="bottom-nav-settings"
            >
              ⚙️ Settings
            </button>
            <button
              className={styles.quickActionBtn}
              onClick={() => {
                // TODO: Show security modal
                console.log("🔐 Show security modal");
              }}
              data-testid="bottom-nav-wallet"
            >
              🔐 Security
            </button>
            <button
              className={classNames(styles.quickActionBtn, styles.danger)}
              onClick={handleLogout}
              disabled={logout.isPending}
              data-testid="disconnect-button"
            >
              {logout.isPending ? "⏳" : "🚪"} Logout
            </button>
          </div>
        </div>

        {/* Security Status */}
        <div className={styles.securityStatus}>
          <h3 className={styles.securityStatusTitle}>🛡️ Security Status</h3>
          <div className={styles.securityStatusItems}>
            <div className={styles.securityItem}>
              <span className={styles.securityItemIcon}>⚛️</span>
              <span className={styles.securityItemText}>
                Quantum Protection Active
              </span>
              <span
                className={classNames(styles.securityItemStatus, styles.active)}
              >
                ✓
              </span>
            </div>
            <div className={styles.securityItem}>
              <span className={styles.securityItemIcon}>🤖</span>
              <span className={styles.securityItemText}>AI Risk Analysis</span>
              <span
                className={classNames(styles.securityItemStatus, styles.active)}
              >
                ✓
              </span>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
});
