import { FC } from "react";
import { Button } from "../../ui/Button/Button";
import { Spinner } from "../../ui/Spinner/Spinner";
import { WalletConnectButton } from "../../wallet/WalletConnectButton/WalletConnectButton";
import { WalletInfo } from "../../wallet/WalletInfo/WalletInfo";
import { AuthStatus } from "../../auth/AuthStatus/AuthStatus";
import { useWallet } from "../../../hooks/wallet/useWallet";
import { APP_TEXT, UI_CONFIG } from "../../../constants";
import styles from "./Hero.module.scss";

export const Hero: FC = () => {
  const { isConnected } = useWallet();

  return (
    <header className={styles.hero}>
      <div className={styles.container}>
        <div className={styles.branding}>
          <h1 className={styles.title}>
            <span className={styles.titleMain}>{APP_TEXT.TITLE}</span>
            <span className={styles.titleQuantum}>⚛️</span>
          </h1>
          <p className={styles.subtitle}>{APP_TEXT.SUBTITLE}</p>
        </div>

        <div className={styles.wallet}>
          {isConnected ? (
            <WalletInfo showBalance={true} showNetwork={true} />
          ) : (
            <WalletConnectButton />
          )}
        </div>

        <div className={styles.auth}>
          <AuthStatus showFullStatus={true} />
        </div>

        <div className={styles.description}>
          <p className={styles.text}>{APP_TEXT.DESCRIPTION}</p>
        </div>

        <div className={styles.actions}>
          <Button
            variant={UI_CONFIG.BUTTON_VARIANTS.PRIMARY}
            size={UI_CONFIG.BUTTON_SIZES.LG}
            className={styles.cta}
          >
            {APP_TEXT.BUTTONS.LAUNCH_BRIDGE}
          </Button>
          <Button
            variant={UI_CONFIG.BUTTON_VARIANTS.SECONDARY}
            size={UI_CONFIG.BUTTON_SIZES.LG}
            className={styles.demo}
          >
            {APP_TEXT.BUTTONS.VIEW_DEMO}
          </Button>
        </div>

        <div className={styles.status}>
          <div className={styles.statusIndicator}>
            <Spinner size={UI_CONFIG.SPINNER_SIZES.SM} color="primary" />
            <span className={styles.statusText}>
              {APP_TEXT.STATUS.QUANTUM_PROTECTION_ACTIVE}
            </span>
          </div>
        </div>
      </div>
    </header>
  );
};
