import { FC, useState } from "react";
import {
  useAuthStatus,
  useEthereumAuth,
  useNearAuth,
  useLogout,
} from "../../../hooks/api/useAuth";
import { useWallet } from "../../../hooks/wallet/useWallet";
import { useUserInfo } from "../../../hooks/api/useUser";
import { Button } from "../../ui/Button/Button";
import { Spinner } from "../../ui/Spinner/Spinner";
import styles from "./AuthStatus.module.scss";
import cn from "classnames";

interface AuthStatusProps {
  showFullStatus?: boolean;
  className?: string;
}

export const AuthStatus: FC<AuthStatusProps> = ({
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
      await logout.mutateAsync("all");
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
        <div className={cn(styles.indicator, styles.indicator_error)}>
          <span className={styles.dot}></span>
          Backend Offline
        </div>
      );
    }

    if (!isConnected) {
      return (
        <div className={cn(styles.indicator, styles.indicator_warning)}>
          <span className={styles.dot}></span>
          No Wallet
        </div>
      );
    }

    if (isAuthenticating) {
      return (
        <div className={cn(styles.indicator, styles.indicator_loading)}>
          <Spinner size="sm" />
          Authenticating...
        </div>
      );
    }

    if (isConnected && !isAuthenticated) {
      return (
        <div className={cn(styles.indicator, styles.indicator_info)}>
          <span className={styles.dot}></span>
          Wallet Connected
        </div>
      );
    }

    if (isAuthenticated && profile) {
      return (
        <div className={cn(styles.indicator, styles.indicator_success)}>
          <span className={styles.dot}></span>
          Authenticated
        </div>
      );
    }

    return (
      <div className={cn(styles.indicator, styles.indicator_warning)}>
        <span className={styles.dot}></span>
        Not Authenticated
      </div>
    );
  };

  return (
    <div
      className={cn(styles.root, { [styles.full]: showFullStatus }, className)}
    >
      {getStatusIndicator()}

      {showFullStatus && (
        <div className={styles.details}>
          <div className={styles.row}>
            <span className={styles.label}>Backend:</span>
            <span
              className={cn(
                styles.value,
                isBackendConnected ? styles.value_success : styles.value_error
              )}
            >
              {isBackendConnected ? "Connected" : "Offline"}
            </span>
          </div>

          <div className={styles.row}>
            <span className={styles.label}>Wallet:</span>
            <span
              className={cn(
                styles.value,
                isConnected ? styles.value_success : styles.value_error
              )}
            >
              {isConnected
                ? `Connected (${account?.address?.slice(0, 8)}...)`
                : "Not Connected"}
            </span>
          </div>

          <div className={styles.row}>
            <span className={styles.label}>Auth:</span>
            <span
              className={cn(
                styles.value,
                isAuthenticated ? styles.value_success : styles.value_error
              )}
            >
              {isAuthenticated
                ? `Authenticated (${profile?.id?.slice(0, 8)}...)`
                : "Not Authenticated"}
            </span>
          </div>

          {isConnected && account && (
            <>
              <div className={styles.row}>
                <span className={styles.label}>Type:</span>
                <span className={styles.value}>{account.type}</span>
              </div>

              {account.address && (
                <div className={styles.row}>
                  <span className={styles.label}>Address:</span>
                  <span className={styles.value} style={{ fontSize: "0.8em" }}>
                    {account.address}
                  </span>
                </div>
              )}

              {account.chainId && (
                <div className={styles.row}>
                  <span className={styles.label}>Chain ID:</span>
                  <span className={styles.value}>{account.chainId}</span>
                </div>
              )}
            </>
          )}

          {isAuthenticated && profile && (
            <div className={styles.user}>
              <div className={styles.row}>
                <span className={styles.label}>User ID:</span>
                <span className={styles.value}>{profile.id}</span>
              </div>
              <div className={styles.row}>
                <span className={styles.label}>Tier:</span>
                <span className={styles.value}>{profile.tier}</span>
              </div>
              <div className={styles.row}>
                <span className={styles.label}>Wallets:</span>
                <span className={styles.value}>
                  {profile.wallet_addresses.length} connected
                </span>
              </div>
            </div>
          )}

          {(state.error || displayError) && (
            <div className={styles.error}>
              <span className={styles.errorText}>
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
                className={styles.errorClear}
              >
                ✕
              </Button>
            </div>
          )}

          <div className={styles.actions}>
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
