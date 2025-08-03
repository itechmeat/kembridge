import { FC } from "react";
import { Spinner } from "../../ui/Spinner/Spinner";
import type { WalletConfig, WalletType } from "../WalletConnect/WalletConnect";
import styles from "./WalletButton.module.scss";
import cn from "classnames";

export interface WalletButtonProps {
  wallet: WalletConfig;
  isSelected?: boolean;
  isConnecting?: boolean;
  disabled?: boolean;
  onSelect: (walletType: WalletType) => void;
}

export const WalletButton: FC<WalletButtonProps> = ({
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
      return (
        <span className={cn(styles.status, styles.status_unavailable)}>
          Unavailable
        </span>
      );
    }

    if (wallet.id !== "near" && wallet.isInstalled === false) {
      return (
        <span className={cn(styles.status, styles.status_install)}>
          Install
        </span>
      );
    }

    return null;
  };

  const isClickable = wallet.isAvailable && !disabled && !isConnecting;

  return (
    <button
      className={cn(styles.walletButton, {
        [styles.selected]: isSelected,
        [styles.connecting]: isConnecting,
        [styles.unavailable]: !wallet.isAvailable,
        [styles.disabled]: !isClickable,
      })}
      onClick={handleClick}
      disabled={!isClickable}
      type="button"
    >
      <div className={styles.content}>
        <div className={styles.icon}>
          <img
            src={wallet.icon}
            alt={`${wallet.name} icon`}
            onError={(e) => {
              e.currentTarget.style.display = "none";
              const fallback = e.currentTarget
                .nextElementSibling as HTMLElement;
              if (fallback) fallback.style.display = "block";
            }}
          />
          <span className={styles.iconFallback} style={{ display: "none" }}>
            🔗
          </span>
        </div>

        <div className={styles.info}>
          <div className={styles.name}>{wallet.name}</div>
          <div className={styles.description}>{wallet.description}</div>
        </div>

        <div className={styles.action}>{getStatusIndicator()}</div>
      </div>
    </button>
  );
};
