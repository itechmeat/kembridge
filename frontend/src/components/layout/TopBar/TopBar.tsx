import { FC } from "react";
import { useWallet } from "../../../hooks/wallet/useWallet";
import { useSecurityStatus } from "../../../hooks/security/useSecurityStatus";
import { useUserInfo } from "../../../hooks/api/useUser";
import { WalletConnectButton } from "../../wallet/WalletConnectButton/WalletConnectButton";
import { WalletInfo } from "../../wallet/WalletInfo/WalletInfo";
import { SecurityStatusBadge } from "../../security/SecurityStatusBadge/SecurityStatusBadge";
import styles from "./TopBar.module.scss";

export const TopBar: FC = () => {
  const { isConnected } = useWallet();
  const { securityStatus, isOnline, quantumProtection } = useSecurityStatus();
  const { riskScore, transactionCount } = useUserInfo();

  return (
    <header className={styles.topBar}>
      <div className={styles.container}>
        <div className={styles.brand}>
          <h1 className={styles.logo}>
            <span className={styles.logoStart}>KEM</span>
            <span className={styles.logoEnd}>Bridge</span>
          </h1>
        </div>

        <div className={styles.actions}>
          <SecurityStatusBadge
            quantumProtection={quantumProtection}
            riskScore={riskScore}
            isOnline={isOnline}
            quantumKeyId={securityStatus?.quantumProtection?.algorithm}
            encryptionScheme={
              securityStatus?.quantumProtection?.algorithm ?? "N/A"
            }
            lastKeyRotation={securityStatus?.quantumProtection?.keyRotationDate}
            transactionCount={transactionCount || 0}
          />

          {isConnected ? (
            <WalletInfo compact={true} showBalance={true} />
          ) : (
            <WalletConnectButton compact={true} />
          )}
        </div>
      </div>
    </header>
  );
};
