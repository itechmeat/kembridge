/**
 * Hero Section Component
 * Landing page hero with branding and main actions
 */

import { FC } from "react";
import { Button } from "../../ui/Button";
import { Spinner } from "../../ui/Spinner";
import { WalletConnectButton } from "../../wallet/WalletConnectButton/WalletConnectButton";
import { WalletInfo } from "../../wallet/WalletInfo/WalletInfo";
import { AuthStatus } from "../../auth/AuthStatus/AuthStatus";
import { useWallet } from "../../../hooks/wallet/useWallet";
import { APP_TEXT, UI_CONFIG } from "../../../constants";
import "./Hero.scss";

export const Hero: FC = () => {
  const { isConnected } = useWallet();

  return (
    <header className="hero">
      <div className="hero__container">
        <div className="hero__branding">
          <h1 className="hero__title">
            <span className="hero__title-main">{APP_TEXT.TITLE}</span>
            <span className="hero__title-quantum">⚛️</span>
          </h1>
          <p className="hero__subtitle">{APP_TEXT.SUBTITLE}</p>
        </div>

        {/* Wallet Section */}
        <div className="hero__wallet">
          {isConnected ? (
            <WalletInfo showBalance={true} showNetwork={true} />
          ) : (
            <WalletConnectButton />
          )}
        </div>

        {/* Authentication Status */}
        <div className="hero__auth">
          <AuthStatus showFullStatus={true} />
        </div>

        <div className="hero__description">
          <p className="hero__text">{APP_TEXT.DESCRIPTION}</p>
        </div>

        <div className="hero__actions">
          <Button
            variant={UI_CONFIG.BUTTON_VARIANTS.PRIMARY}
            size={UI_CONFIG.BUTTON_SIZES.LG}
            className="hero__cta"
          >
            {APP_TEXT.BUTTONS.LAUNCH_BRIDGE}
          </Button>
          <Button
            variant={UI_CONFIG.BUTTON_VARIANTS.SECONDARY}
            size={UI_CONFIG.BUTTON_SIZES.LG}
            className="hero__demo"
          >
            {APP_TEXT.BUTTONS.VIEW_DEMO}
          </Button>
        </div>

        <div className="hero__status">
          <div className="status-indicator">
            <Spinner size={UI_CONFIG.SPINNER_SIZES.SM} color="primary" />
            <span className="status-text">
              {APP_TEXT.STATUS.QUANTUM_PROTECTION_ACTIVE}
            </span>
          </div>
        </div>
      </div>
    </header>
  );
};
