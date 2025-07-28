/**
 * Wallet Provider Card Component
 * Displays individual wallet provider with connection functionality
 */

import { FC } from "react";
import { WalletProvider } from "../../../services/wallet/types";
import { Spinner } from "../../ui/Spinner";
import {
  getWalletName,
  getWalletIcon,
  getWalletDescription,
} from "../../../utils/wallet";
import { UI_CONFIG } from "../../../constants";
import "./WalletProviderCard.scss";

export interface WalletProviderCardProps {
  provider: WalletProvider;
  isLoading?: boolean;
  isSelected?: boolean;
  onClick: () => void;
  disabled?: boolean;
}

export const WalletProviderCard: FC<WalletProviderCardProps> = ({
  provider,
  isLoading = false,
  isSelected = false,
  onClick,
  disabled = false,
}) => {
  const handleClick = () => {
    if (!disabled && provider.isAvailable) {
      onClick();
    }
  };

  return (
    <button
      className={`wallet-provider-card ${
        isSelected ? "wallet-provider-card--loading" : ""
      } ${!provider.isAvailable ? "wallet-provider-card--unavailable" : ""}`}
      onClick={handleClick}
      disabled={disabled || !provider.isAvailable}
      type="button"
    >
      <div className="wallet-provider-card__icon">
        {isLoading && isSelected ? (
          <Spinner size={UI_CONFIG.SPINNER_SIZES.SM} />
        ) : (
          getWalletIcon(provider.type)
        )}
      </div>

      <div className="wallet-provider-card__content">
        <h3 className="wallet-provider-card__name">
          {getWalletName(provider.type)}
        </h3>
        <p className="wallet-provider-card__description">
          {getWalletDescription(provider.type)}
        </p>
      </div>

      {!provider.isAvailable && (
        <div className="wallet-provider-card__status">Not Available</div>
      )}
    </button>
  );
};
