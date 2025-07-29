/**
 * Wallet Button Component
 * Individual wallet connection button with status indicators
 */

import React from "react";
import { Spinner } from "../../ui/Spinner";
import type { WalletConfig, WalletType } from "../WalletConnect/WalletConnect";
import "./WalletButton.scss";

export interface WalletButtonProps {
  wallet: WalletConfig;
  isSelected?: boolean;
  isConnecting?: boolean;
  disabled?: boolean;
  onSelect: (walletType: WalletType) => void;
}

export const WalletButton: React.FC<WalletButtonProps> = ({
  wallet,
  isSelected = false,
  isConnecting = false,
  disabled = false,
  onSelect,
}) => {
  const handleClick = () => {
    if (!disabled && wallet.isAvailable) {
      onSelect(wallet.id);
    }
  };

  const getStatusIndicator = () => {
    if (isConnecting) {
      return <Spinner size="sm" />;
    }

    if (!wallet.isAvailable) {
      return <span className="wallet-button__status wallet-button__status--unavailable">Unavailable</span>;
    }

    if (wallet.id !== "near" && wallet.isInstalled === false) {
      return <span className="wallet-button__status wallet-button__status--install">Install</span>;
    }

    return null;
  };

  const isClickable = wallet.isAvailable && !disabled && !isConnecting;

  return (
    <button
      className={`
        wallet-button 
        ${isSelected ? "wallet-button--selected" : ""}
        ${isConnecting ? "wallet-button--connecting" : ""}
        ${!wallet.isAvailable ? "wallet-button--unavailable" : ""}
        ${!isClickable ? "wallet-button--disabled" : ""}
      `.trim()}
      onClick={handleClick}
      disabled={!isClickable}
      type="button"
    >
      <div className="wallet-button__content">
        <div className="wallet-button__icon">
          <img 
            src={wallet.icon} 
            alt={`${wallet.name} icon`}
            onError={(e) => {
              // Fallback to emoji if icon fails to load
              e.currentTarget.style.display = 'none';
              const fallback = e.currentTarget.nextElementSibling as HTMLElement;
              if (fallback) fallback.style.display = 'block';
            }}
          />
          <span 
            className="wallet-button__icon-fallback"
            style={{ display: 'none' }}
          >
            🔗
          </span>
        </div>

        <div className="wallet-button__info">
          <div className="wallet-button__name">{wallet.name}</div>
          <div className="wallet-button__description">{wallet.description}</div>
        </div>

        <div className="wallet-button__action">
          {getStatusIndicator()}
        </div>
      </div>
    </button>
  );
};