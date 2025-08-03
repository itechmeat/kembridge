import { FC } from "react";
import cn from "classnames";
import { QuantumProtectionDisplayProps } from "../QuantumProtectionDisplay/QuantumProtectionDisplay";
import styles from "./QuantumIndicator.module.scss";

export const QuantumIndicator: FC<QuantumProtectionDisplayProps> = ({
  isActive,
  encryptionScheme = "ML-KEM-1024",
  keyId,
  lastRotation,
  protectedTransactions = 0,
  className = "",
}) => {
  const formatDate = (dateString?: string): string => {
    if (!dateString) return "Never";
    const date = new Date(dateString);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24));

    if (diffDays === 0) return "Today";
    if (diffDays === 1) return "Yesterday";
    if (diffDays < 7) return `${diffDays} days ago`;
    if (diffDays < 30) return `${Math.floor(diffDays / 7)} weeks ago`;
    return `${Math.floor(diffDays / 30)} months ago`;
  };


  const formatTransactionCount = (): string => {
    if (protectedTransactions < 1000) return protectedTransactions.toString();
    if (protectedTransactions < 1000000) {
      return `${(protectedTransactions / 1000).toFixed(1)}K`;
    }
    return `${(protectedTransactions / 1000000).toFixed(1)}M`;
  };


  const getStatusText = (): string => {
    return isActive ? "Active" : "Inactive";
  };

  return (
    <div
      className={cn(
        styles.quantumIndicator,
        isActive ? styles.active : styles.inactive,
        className
      )}
      data-testid="quantum-indicator"
    >
      <div className={styles.content} data-testid="quantum-content">
        <div className={styles.status}>
          <span className={styles.statusText} data-testid="quantum-status">
            {getStatusText()}
          </span>
          <span className={styles.connection} data-testid="quantum-connection">
            {isActive ? "🟢 Online" : "🔴 Offline"}
          </span>
        </div>

        <div className={styles.details} data-testid="quantum-details">
          <div className={styles.detail}>
            <span className={styles.detailLabel}>Quantum Protection:</span>
            <span
              className={cn(styles.detailValue, {
                [styles.enabled]: isActive,
                [styles.disabled]: !isActive,
              })}
              data-testid="encryption-scheme"
            >
              {isActive ? encryptionScheme : "Disabled"}
            </span>
          </div>

          <div className={styles.detail}>
            <span className={styles.detailLabel}>Risk Score:</span>
            <span
              className={cn(styles.detailValue, styles.riskScore)}
              data-testid="risk-score"
            >
              43%
            </span>
          </div>

          {isActive && (
            <>
              <div className={styles.detail}>
                <span className={styles.detailLabel}>Key ID:</span>
                <span
                  className={cn(styles.detailValue, styles.keyId)}
                  data-testid="quantum-key-id"
                >
                  {keyId || "ML-KEM-1024"}
                </span>
              </div>

              <div className={styles.detail}>
                <span className={styles.detailLabel}>Last Rotation:</span>
                <span
                  className={styles.detailValue}
                  data-testid="key-rotation-status"
                >
                  {formatDate(lastRotation)}
                </span>
              </div>

              <div className={styles.detail}>
                <span className={styles.detailLabel}>Protected Txs:</span>
                <span
                  className={styles.detailValue}
                  data-testid="protected-count"
                >
                  {formatTransactionCount()}
                </span>
              </div>
            </>
          )}
        </div>
      </div>
    </div>
  );
};

export default QuantumIndicator;
