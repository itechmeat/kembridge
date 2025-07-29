/**
 * Wallet information display component
 * Shows connected wallet details and balance
 */

import React, { useState } from "react";
import { useWallet } from "../../../hooks/wallet/useWallet";
import { useBalance } from "../../../hooks/wallet/useBalance";
import { useAuthStatus } from "../../../hooks/api/useAuth";
import { formatAddress, formatUsdValue } from "../../../services/wallet/utils";
import {
  formatDisplayBalance,
  formatTokenBalance,
} from "../../../utils/wallet";
import { Button } from "../../ui/Button";
import { Spinner } from "../../ui/Spinner";
import "./WalletInfo.scss";

interface WalletInfoProps {
  showBalance?: boolean;
  showNetwork?: boolean;
  showTechnicalInfo?: boolean; // Show technical debugging info
  compact?: boolean;
  className?: string;
}

export const WalletInfo: React.FC<WalletInfoProps> = ({
  showBalance = true,
  showNetwork = true,
  showTechnicalInfo = true, // Show by default for now
  compact = false,
  className = "",
}) => {
  const { account, disconnect, isConnected, state } = useWallet();
  const { isAuthenticated } = useAuthStatus();
  const {
    balances,
    isLoading: balanceLoading,
    refresh: refreshBalance,
  } = useBalance();
  const [isDropdownOpen, setIsDropdownOpen] = useState(false);

  if (!isConnected || !account) {
    return null;
  }

  const handleDisconnect = async () => {
    try {
      await disconnect();
      setIsDropdownOpen(false);
    } catch (error) {
      console.error("Failed to disconnect wallet:", error);
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

  const totalUsdValue = balances.reduce((total, balance) => {
    const usdValue = parseFloat(balance.usdValue || "0");
    return total + usdValue;
  }, 0);

  const primaryBalance = balances[0]; // First balance is usually the native token

  return (
    <div
      className={`wallet-info ${
        compact ? "wallet-info--compact" : ""
      } ${className}`}
    >
      <div className="wallet-info__main" onClick={toggleDropdown}>
        <div className="wallet-info__account">
          <div className="wallet-info__avatar">
            {/* TODO (feat): Add wallet-specific avatars */}
            💼
          </div>

          <div className="wallet-info__details">
            <div className="wallet-info__address">
              {formatAddress(account.address)}
            </div>

            {showNetwork && account.network && (
              <div className="wallet-info__network">{account.network.name}</div>
            )}
          </div>
        </div>

        {showBalance && !compact && (
          <div className="wallet-info__balance">
            {balanceLoading ? (
              <Spinner size="sm" />
            ) : primaryBalance ? (
              <div className="wallet-info__balance-details">
                <div className="wallet-info__balance-amount">
                  {formatDisplayBalance(
                    primaryBalance.balance,
                    primaryBalance.symbol,
                    primaryBalance.decimals
                  )}
                </div>
                {totalUsdValue > 0 && (
                  <div className="wallet-info__balance-usd">
                    {formatUsdValue(totalUsdValue)}
                  </div>
                )}
              </div>
            ) : (
              <div className="wallet-info__balance-empty">No balance</div>
            )}
          </div>
        )}

        <div className="wallet-info__dropdown-arrow">
          {isDropdownOpen ? "▲" : "▼"}
        </div>
      </div>

      {isDropdownOpen && (
        <div className="wallet-info__dropdown">
          <div className="wallet-info__dropdown-content">
            {/* Full address */}
            <div className="wallet-info__dropdown-section">
              <div className="wallet-info__dropdown-label">Address</div>
              <div className="wallet-info__dropdown-value">
                {account.address}
              </div>
            </div>

            {/* Network info */}
            {account.network && (
              <div className="wallet-info__dropdown-section">
                <div className="wallet-info__dropdown-label">Network</div>
                <div className="wallet-info__dropdown-value">
                  {account.network.name} ({account.network.type})
                </div>
              </div>
            )}

            {/* Balances */}
            {showBalance && balances.length > 0 && (
              <div className="wallet-info__dropdown-section">
                <div className="wallet-info__dropdown-label">
                  Balances
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={handleRefreshBalance}
                    disabled={balanceLoading}
                    className="wallet-info__refresh-btn"
                  >
                    {balanceLoading ? <Spinner size="sm" /> : "🔄"}
                  </Button>
                </div>
                <div className="wallet-info__balances">
                  {balances.map((balance, index) => (
                    <div key={index} className="wallet-info__balance-item">
                      <div className="wallet-info__balance-token">
                        <span className="wallet-info__balance-symbol">
                          {balance.symbol}
                        </span>
                        <span className="wallet-info__balance-amount">
                          {formatTokenBalance(
                            balance.balance,
                            balance.decimals,
                            4
                          )}
                        </span>
                      </div>
                      {balance.usdValue && parseFloat(balance.usdValue) > 0 && (
                        <div className="wallet-info__balance-usd">
                          {formatUsdValue(balance.usdValue)}
                        </div>
                      )}
                    </div>
                  ))}
                </div>
              </div>
            )}

            {/* Technical Info */}
            {showTechnicalInfo && (
              <div className="wallet-info__dropdown-section">
                <div className="wallet-info__dropdown-label">
                  Technical Info
                </div>
                <div className="wallet-info__technical-info">
                  <div className="wallet-info__tech-item">
                    <span className="wallet-info__tech-label">Connected:</span>
                    <span className="wallet-info__tech-value">
                      {isConnected ? "✅ Yes" : "❌ No"}
                    </span>
                  </div>
                  <div className="wallet-info__tech-item">
                    <span className="wallet-info__tech-label">
                      Authenticated:
                    </span>
                    <span className="wallet-info__tech-value">
                      {isAuthenticated ? "✅ Yes" : "❌ No"}
                    </span>
                  </div>
                  <div className="wallet-info__tech-item">
                    <span className="wallet-info__tech-label">
                      Wallet Type:
                    </span>
                    <span className="wallet-info__tech-value">
                      {account?.type || "None"}
                    </span>
                  </div>
                  {account?.chainId && (
                    <div className="wallet-info__tech-item">
                      <span className="wallet-info__tech-label">Chain ID:</span>
                      <span className="wallet-info__tech-value">
                        {account.chainId}
                      </span>
                    </div>
                  )}
                  <div className="wallet-info__tech-item">
                    <span className="wallet-info__tech-label">Connecting:</span>
                    <span className="wallet-info__tech-value">
                      {state.isConnecting ? "⏳ Yes" : "❌ No"}
                    </span>
                  </div>
                  {state.error && (
                    <div className="wallet-info__tech-item">
                      <span className="wallet-info__tech-label">Error:</span>
                      <span className="wallet-info__tech-value wallet-info__tech-error">
                        ⚠️ {state.error}
                      </span>
                    </div>
                  )}
                </div>
              </div>
            )}

            {/* Actions */}
            <div className="wallet-info__dropdown-actions">
              <Button
                variant="secondary"
                size="sm"
                onClick={handleDisconnect}
                className="wallet-info__disconnect-btn"
              >
                Disconnect
              </Button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};
