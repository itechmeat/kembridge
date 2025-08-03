import { FC } from "react";
import cn from "classnames";
import styles from "./QuantumProtectionDisplay.module.scss";

export interface QuantumProtectionDisplayProps {
  isActive: boolean;
  encryptionScheme?: string;
  keyId?: string;
  keyStrength?: number;
  lastRotation?: string;
  nextRotation?: string;
  protectedTransactions?: number;
  encryptionSpeed?: number;
  className?: string;
}

export const QuantumProtectionDisplay: FC<QuantumProtectionDisplayProps> = ({
  isActive,
  encryptionScheme = "ML-KEM-1024",
  keyId,
  keyStrength = 1024,
  lastRotation,
  nextRotation,
  protectedTransactions = 0,
  encryptionSpeed,
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

  const formatNextRotation = (dateString?: string): string => {
    if (!dateString) return "Not scheduled";
    const date = new Date(dateString);
    const now = new Date();
    const diffMs = date.getTime() - now.getTime();
    const diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24));

    if (diffDays < 0) return "Overdue";
    if (diffDays === 0) return "Today";
    if (diffDays === 1) return "Tomorrow";
    if (diffDays < 7) return `In ${diffDays} days`;
    if (diffDays < 30) return `In ${Math.floor(diffDays / 7)} weeks`;
    return `In ${Math.floor(diffDays / 30)} months`;
  };

  const formatTransactionCount = (count: number): string => {
    if (count < 1000) return count.toString();
    if (count < 1000000) return `${(count / 1000).toFixed(1)}K`;
    return `${(count / 1000000).toFixed(1)}M`;
  };

  const formatSpeed = (opsPerSec?: number): string => {
    if (!opsPerSec) return "N/A";
    if (opsPerSec < 1000) return `${opsPerSec} ops/s`;
    return `${Math.round(opsPerSec / 1000)}K ops/s`;
  };

  const getKeyStrengthColor = (strength: number): string => {
    if (strength >= 1024) return "high";
    if (strength >= 512) return "medium";
    return "low";
  };

  const getRotationStatus = (
    nextRotation?: string
  ): "healthy" | "warning" | "overdue" => {
    if (!nextRotation) return "healthy";
    const date = new Date(nextRotation);
    const now = new Date();
    const diffMs = date.getTime() - now.getTime();
    const diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24));

    if (diffDays < 0) return "overdue";
    if (diffDays <= 7) return "warning";
    return "healthy";
  };

  if (!isActive) {
    return (
      <div
        className={cn(
          styles.quantumProtectionDisplay,
          styles.disabled,
          className
        )}
        data-testid="quantum-protection-display"
      >
        <div className={styles.header} data-testid="quantum-header">
          <span className={styles.icon} data-testid="quantum-icon">
            🔓
          </span>
          <span className={styles.title}>Quantum Protection</span>
          <span
            className={cn(styles.status, styles.statusDisabled)}
            data-testid="quantum-status"
          >
            Disabled
          </span>
        </div>
        <div className={styles.message}>
          Post-quantum cryptography is not active. Your transactions may be
          vulnerable to quantum attacks.
        </div>
      </div>
    );
  }

  const rotationStatus = getRotationStatus(nextRotation);

  return (
    <div
      className={cn(styles.quantumProtectionDisplay, styles.active, className)}
      data-testid="quantum-protection-display"
    >
      <div className={styles.header} data-testid="quantum-header">
        <span className={styles.icon} data-testid="quantum-icon">
          🔒
        </span>
        <span className={styles.title}>Quantum Protection</span>
        <span
          className={cn(styles.status, styles.statusActive)}
          data-testid="quantum-status"
        >
          Active
        </span>
      </div>

      <div className={styles.content}>
        <div className={styles.grid}>
          <div className={styles.card} data-testid="quantum-card-encryption">
            <div className={styles.cardHeader}>
              <span className={styles.cardIcon}>⚡</span>
              <span className={styles.cardTitle}>Encryption Scheme</span>
            </div>
            <div className={styles.cardValue} data-testid="encryption-scheme">
              {encryptionScheme}
            </div>
            <div className={styles.cardSubtitle}>
              {keyStrength}-bit security level
            </div>
          </div>

          <div className={styles.card} data-testid="quantum-card-key">
            <div className={styles.cardHeader}>
              <span className={styles.cardIcon}>🔐</span>
              <span className={styles.cardTitle}>Key Information</span>
            </div>
            <div
              className={cn(styles.cardValue, styles.keyId)}
              data-testid="key-information"
            >
              {keyId ? `${keyId.slice(0, 8)}...${keyId.slice(-4)}` : "N/A"}
            </div>
            <div
              className={cn(
                styles.cardSubtitle,
                styles[
                  `strength${
                    getKeyStrengthColor(keyStrength).charAt(0).toUpperCase() +
                    getKeyStrengthColor(keyStrength).slice(1)
                  }`
                ]
              )}
            >
              {keyStrength}-bit strength
            </div>
          </div>

          <div className={styles.card} data-testid="quantum-card-rotation">
            <div className={styles.cardHeader}>
              <span className={styles.cardIcon}>🔄</span>
              <span className={styles.cardTitle}>Key Rotation</span>
            </div>
            <div className={styles.cardValue} data-testid="key-rotation-status">
              {formatDate(lastRotation)}
            </div>
            <div
              className={cn(
                styles.cardSubtitle,
                styles[
                  `rotation${
                    rotationStatus.charAt(0).toUpperCase() +
                    rotationStatus.slice(1)
                  }`
                ]
              )}
              data-testid="next-rotation"
            >
              Next: {formatNextRotation(nextRotation)}
            </div>
          </div>

          <div className={styles.card} data-testid="quantum-card-protected">
            <div className={styles.cardHeader}>
              <span className={styles.cardIcon}>🛡️</span>
              <span className={styles.cardTitle}>Protected</span>
            </div>
            <div
              className={styles.cardValue}
              data-testid="protected-transactions"
            >
              {formatTransactionCount(protectedTransactions)}
            </div>
            <div className={styles.cardSubtitle}>transactions secured</div>
          </div>

          {encryptionSpeed && (
            <div className={styles.card} data-testid="quantum-card-performance">
              <div className={styles.cardHeader}>
                <span className={styles.cardIcon}>⚡</span>
                <span className={styles.cardTitle}>Performance</span>
              </div>
              <div
                className={styles.cardValue}
                data-testid="performance-metrics"
              >
                {formatSpeed(encryptionSpeed)}
              </div>
              <div className={styles.cardSubtitle}>encryption speed</div>
            </div>
          )}
        </div>

        <div className={styles.footer} data-testid="quantum-info-footer">
          <div className={styles.info}>
            <span className={styles.infoIcon}>ℹ️</span>
            <span className={styles.infoText}>
              Your transactions are protected with post-quantum cryptography,
              resistant to both classical and quantum computer attacks.
            </span>
          </div>
        </div>
      </div>
    </div>
  );
};

export default QuantumProtectionDisplay;
