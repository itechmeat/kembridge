/**
 * Mobile Top Bar Component
 * App header with wallet status and notifications
 */

import { FC } from "react";
import { useWallet } from "../../../hooks/wallet/useWallet";
import { useAuth } from "../../../hooks/auth/useAuth";
import { WalletConnect } from "../../wallet/WalletConnect/WalletConnect";
import { WalletInfo } from "../../wallet/WalletInfo/WalletInfo";
import { APP_TEXT } from "../../../constants";
import "./TopBar.scss";

export const TopBar: FC = () => {
  const { isConnected } = useWallet();
  const { isBackendConnected } = useAuth();

  return (
    <header className="top-bar">
      <div className="top-bar__container">
        <div className="top-bar__brand">
          <h1 className="top-bar__title">
            {APP_TEXT.TITLE}
            <span className="top-bar__quantum">⚛️</span>
          </h1>
        </div>

        <div className="top-bar__actions">
          <div
            className={`status-dot ${
              isBackendConnected
                ? "status-dot--connected"
                : "status-dot--disconnected"
            }`}
          />

          {isConnected ? (
            <WalletInfo compact={true} showBalance={true} />
          ) : (
            <WalletConnect compact={true} />
          )}
        </div>
      </div>
    </header>
  );
};
