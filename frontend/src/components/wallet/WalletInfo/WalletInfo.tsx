import { useState, FC } from "react";
import cn from "classnames";
import { useWallet } from "../../../hooks/wallet/useWallet";
import { useBalance } from "../../../hooks/wallet/useBalance";
import {
  useWalletAuthStatus,
  useLogout,
  useEthereumAuth,
  useNearAuth,
} from "../../../hooks/api/useAuth";
import { formatAddress } from "../../../services/wallet/utils";
import { formatBalance } from "../../../utils/formatBalance";
import { Button } from "../../ui/Button/Button";
import { Spinner } from "../../ui/Spinner/Spinner";
import { WalletIcon } from "./WalletIcon";
import styles from "./WalletInfo.module.scss";

interface WalletInfoProps {
  showBalance?: boolean;
  showNetwork?: boolean;
  showTechnicalInfo?: boolean; // Show technical debugging info
  compact?: boolean;
  className?: string;
}

export const WalletInfo: FC<WalletInfoProps> = ({
  showBalance = true,
  compact = false,
  className = "",
}) => {
  const { connect } = useWallet();
  const {
    isEvmAuthenticated,
    isNearAuthenticated,
    evmAddress,
    nearAddress,
    isEvmConnected,
    isNearConnected,
    hasAnyAuth,
  } = useWalletAuthStatus();

  // Auth hooks
  const ethereumAuth = useEthereumAuth();
  const nearAuth = useNearAuth();
  const logout = useLogout();

  const {
    balances,
    isLoading: balanceLoading,
    refresh: refreshBalance,
  } = useBalance();
  const [isDropdownOpen, setIsDropdownOpen] = useState(false);

  const handleEvmConnect = async () => {
    try {
      await connect("metamask");
      setIsDropdownOpen(false);
    } catch (error) {
      console.error("Failed to connect EVM wallet:", error);
    }
  };

  const handleNearConnect = async () => {
    try {
      await connect("near");
      setIsDropdownOpen(false);
    } catch (error) {
      console.error("Failed to connect NEAR wallet:", error);
    }
  };

  const handleEvmAuth = async () => {
    try {
      await ethereumAuth.authenticate();
      setIsDropdownOpen(false);
    } catch (error) {
      console.error("Failed to authenticate EVM wallet:", error);
    }
  };

  const handleNearAuth = async () => {
    try {
      await nearAuth.authenticate();
      setIsDropdownOpen(false);
    } catch (error) {
      console.error("Failed to authenticate NEAR wallet:", error);
    }
  };

  const handleEvmLogout = async () => {
    try {
      await logout.mutateAsync("evm");
    } catch (error) {
      console.error("Failed to logout from EVM:", error);
    }
  };

  const handleNearLogout = async () => {
    try {
      await logout.mutateAsync("near");
    } catch (error) {
      console.error("Failed to logout from NEAR:", error);
    }
  };

  const handleRefreshBalance = async () => {
    try {
      await refreshBalance();
    } catch (error) {
      console.error("Failed to refresh balance:", error);
    }
  };

  const toggleDropdown = () => {
    setIsDropdownOpen(!isDropdownOpen);
  };

  // Determine what to show in main button
  const getPrimaryDisplay = () => {
    // If both wallets are authenticated, show both icons
    if (isEvmAuthenticated && isNearAuthenticated) {
      return {
        type: "both",
        authenticated: true,
      };
    }

    // If only EVM is authenticated (prioritize authenticated over connected)
    if (isEvmAuthenticated && !isNearAuthenticated) {
      return {
        address: evmAddress,
        type: "evm",
        authenticated: true,
      };
    }

    // If only NEAR is authenticated (prioritize authenticated over connected)
    if (isNearAuthenticated && !isEvmAuthenticated) {
      return {
        address: nearAddress,
        type: "near",
        authenticated: true,
      };
    }

    // If only EVM is connected but not authenticated
    if (isEvmConnected && !isNearConnected) {
      return {
        address: evmAddress,
        type: "evm",
        authenticated: false,
      };
    }

    // If only NEAR is connected but not authenticated
    if (isNearConnected && !isEvmConnected) {
      return {
        address: nearAddress,
        type: "near",
        authenticated: false,
      };
    }

    // If both are connected but neither authenticated, prioritize EVM
    if (isEvmConnected && isNearConnected) {
      return {
        address: evmAddress,
        type: "evm",
        authenticated: false,
      };
    }

    return null;
  };

  const primaryDisplay = getPrimaryDisplay();

  return (
    <div
      className={cn(
        styles.walletInfo,
        {
          [styles.compact]: compact,
        },
        className.trim()
      )}
    >
      <div className={styles.main} onClick={toggleDropdown}>
        {primaryDisplay ? (
          <div className={styles.account}>
            {primaryDisplay.type === "both" ? (
              // Both wallets authenticated - show both icons with + between them
              <div className={styles.bothWallets}>
                <WalletIcon type="metamask" size="sm" />
                <span className={styles.plus}>+</span>
                <WalletIcon type="near" size="sm" />
              </div>
            ) : (
              // Single wallet - show icon and address
              <div className={styles.singleWallet}>
                <WalletIcon
                  type={primaryDisplay.type === "evm" ? "metamask" : "near"}
                  size="sm"
                />
                <div className={styles.walletInfo}>
                  <div className={styles.address}>
                    {formatAddress(primaryDisplay.address || "")}
                  </div>
                </div>
              </div>
            )}
          </div>
        ) : (
          <div className={styles.connectButton}>
            <span>Connect Wallet</span>
          </div>
        )}

        <div className={styles.dropdownArrow}>{isDropdownOpen ? "▲" : "▼"}</div>
      </div>

      {isDropdownOpen && (
        <div className={styles.dropdown}>
          <div className={styles.dropdownContent}>
            {/* EVM Section */}
            <div className={styles.networkSection}>
              <div className={styles.networkHeader}>
                <div className={styles.networkTitle}>
                  <WalletIcon type="metamask" size="sm" />
                  Ethereum
                </div>
                {isEvmAuthenticated && (
                  <span className={styles.statusBadge}>✅ Authenticated</span>
                )}
              </div>

              {isEvmConnected ? (
                <div className={styles.networkContent}>
                  <div className={styles.walletAddress}>
                    {formatAddress(evmAddress || "", 10, 10)}
                  </div>

                  {/* EVM Balance */}
                  {showBalance &&
                    (() => {
                      const ethBalance = balances.find(
                        (b) => b.symbol === "ETH"
                      );
                      return ethBalance ? (
                        <div className={styles.networkBalance}>
                          <div className={styles.balanceDisplay}>
                            <div className={styles.balanceAmount}>
                              {formatBalance(ethBalance.balance)} ETH
                              {ethBalance.usdValue && (
                                <span className={styles.balanceUsd}>
                                  ≈ ${ethBalance.usdValue}
                                </span>
                              )}
                            </div>
                            <Button
                              variant="ghost"
                              size="sm"
                              onClick={handleRefreshBalance}
                              disabled={balanceLoading}
                              className={styles.refreshBtn}
                            >
                              {balanceLoading ? <Spinner size="sm" /> : "🔄"}
                            </Button>
                          </div>
                        </div>
                      ) : null;
                    })()}

                  <div className={styles.walletActions}>
                    {isEvmAuthenticated ? (
                      <Button
                        variant="secondary"
                        size="sm"
                        onClick={handleEvmLogout}
                        disabled={logout.isPending}
                      >
                        {logout.isPending ? <Spinner size="sm" /> : "Logout"}
                      </Button>
                    ) : (
                      <Button
                        variant="primary"
                        size="sm"
                        onClick={handleEvmAuth}
                        disabled={ethereumAuth.isAuthenticating}
                      >
                        {ethereumAuth.isAuthenticating ? (
                          <Spinner size="sm" />
                        ) : (
                          "Authenticate"
                        )}
                      </Button>
                    )}
                  </div>
                </div>
              ) : (
                <div className={styles.networkContent}>
                  <Button
                    variant="secondary"
                    size="sm"
                    onClick={handleEvmConnect}
                    className={styles.connectBtn}
                  >
                    Connect MetaMask
                  </Button>
                </div>
              )}
            </div>

            {/* Divider between networks */}
            <div className={styles.sectionDivider}></div>

            {/* NEAR Section */}
            <div className={styles.networkSection}>
              <div className={styles.networkHeader}>
                <div className={styles.networkTitle}>
                  <WalletIcon type="near" size="sm" />
                  NEAR Network
                </div>
                {isNearAuthenticated && (
                  <span className={styles.statusBadge}>✅ Authenticated</span>
                )}
              </div>

              {isNearConnected ? (
                <div className={styles.networkContent}>
                  <div className={styles.walletAddress}>
                    {formatAddress(nearAddress || "", 10, 10)}
                  </div>

                  {/* NEAR Balance */}
                  {showBalance &&
                    (() => {
                      const nearBalance = balances.find(
                        (b) => b.symbol === "NEAR"
                      );
                      return nearBalance ? (
                        <div className={styles.networkBalance}>
                          <div className={styles.balanceDisplay}>
                            <div className={styles.balanceAmount}>
                              {formatBalance(nearBalance.balance)} NEAR
                              {nearBalance.usdValue && (
                                <span className={styles.balanceUsd}>
                                  ≈ ${nearBalance.usdValue}
                                </span>
                              )}
                            </div>
                            <Button
                              variant="ghost"
                              size="sm"
                              onClick={handleRefreshBalance}
                              disabled={balanceLoading}
                              className={styles.refreshBtn}
                            >
                              {balanceLoading ? <Spinner size="sm" /> : "🔄"}
                            </Button>
                          </div>
                        </div>
                      ) : null;
                    })()}

                  <div className={styles.walletActions}>
                    {isNearAuthenticated ? (
                      <Button
                        variant="secondary"
                        size="sm"
                        onClick={handleNearLogout}
                        disabled={logout.isPending}
                      >
                        {logout.isPending ? <Spinner size="sm" /> : "Logout"}
                      </Button>
                    ) : (
                      <Button
                        variant="primary"
                        size="sm"
                        onClick={handleNearAuth}
                        disabled={nearAuth.isAuthenticating}
                      >
                        {nearAuth.isAuthenticating ? (
                          <Spinner size="sm" />
                        ) : (
                          "Authenticate"
                        )}
                      </Button>
                    )}
                  </div>
                </div>
              ) : (
                <div className={styles.networkContent}>
                  <Button
                    variant="secondary"
                    size="sm"
                    onClick={handleNearConnect}
                    className={styles.connectBtn}
                  >
                    Connect NEAR
                  </Button>
                </div>
              )}
            </div>

            {/* Global Actions */}
            {hasAnyAuth && (
              <div className={styles.dropdownActions}>
                <Button
                  variant="secondary"
                  size="sm"
                  onClick={() => logout.mutateAsync("all")}
                  disabled={logout.isPending}
                  className={styles.disconnectBtn}
                >
                  {logout.isPending ? <Spinner size="sm" /> : "Logout All"}
                </Button>
              </div>
            )}
          </div>
        </div>
      )}
    </div>
  );
};
