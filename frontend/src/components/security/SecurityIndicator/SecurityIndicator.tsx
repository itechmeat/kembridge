import { FC } from "react";
import cn from "classnames";
import { SecurityIndicatorProps, SecurityLevel } from "../../../types";
import styles from "./SecurityIndicator.module.scss";

export const SecurityIndicator: FC<SecurityIndicatorProps> = ({
  quantumProtection,
  riskScore,
  isOnline,
  compact = false,
  className = "",
  quantumKeyId,
  encryptionScheme,
  lastKeyRotation,
  transactionCount,
  wsConnected = false,
  connectionQuality = "unknown",
  wsErrors = [],
}) => {
  // Determine security level based on quantum protection and risk score
  const getSecurityLevel = (): SecurityLevel => {
    if (!isOnline) return SecurityLevel.OFFLINE;
    if (!quantumProtection) return SecurityLevel.DANGER;
    if (riskScore > 0.7) return SecurityLevel.DANGER;
    if (riskScore > 0.3) return SecurityLevel.WARNING;
    return SecurityLevel.SECURE;
  };

  const securityLevel = getSecurityLevel();

  const getStatusText = (): string => {
    switch (securityLevel) {
      case SecurityLevel.SECURE:
        return quantumProtection ? "Quantum Protected" : "Protected";
      case SecurityLevel.WARNING:
        return "Medium Risk";
      case SecurityLevel.DANGER:
        return quantumProtection ? "High Risk" : "Quantum Offline";
      case SecurityLevel.OFFLINE:
        return "System Offline";
      default:
        return "Unknown";
    }
  };

  const getQuantumSchemeDisplay = (): string => {
    if (!quantumProtection || !encryptionScheme) return "Disabled";
    return encryptionScheme;
  };

  const getKeyRotationStatus = (): string => {
    if (!lastKeyRotation) return "Never";
    const now = new Date();
    const rotation = new Date(lastKeyRotation);
    const diffHours = Math.floor(
      (now.getTime() - rotation.getTime()) / (1000 * 60 * 60)
    );

    if (diffHours < 1) return "Recent";
    if (diffHours < 24) return `${diffHours}h ago`;
    const diffDays = Math.floor(diffHours / 24);
    return `${diffDays}d ago`;
  };

  const formatTransactionCount = (): string => {
    if (!transactionCount) return "0";
    if (transactionCount < 1000) return transactionCount.toString();
    if (transactionCount < 1000000)
      return `${Math.floor(transactionCount / 1000)}K`;
    return `${Math.floor(transactionCount / 1000000)}M`;
  };

  const getQuantumKeyStrength = (): string => {
    // Получаем силу ключа из encryptionScheme или используем дефолт
    const keyStrength = encryptionScheme?.includes("1024")
      ? 1024
      : encryptionScheme?.includes("512")
      ? 512
      : 256;

    // Конвертируем в процент надежности
    const strengthPercent = Math.min(
      Math.round((keyStrength / 1024) * 100),
      100
    );
    return `${strengthPercent}%`;
  };

  const getSystemEfficiency = (): string => {
    // Комбинируем факторы: онлайн статус, квантовая защита, низкий риск
    let efficiency = 0;
    if (isOnline) efficiency += 40;
    if (quantumProtection) efficiency += 40;
    if (riskScore < 0.3) efficiency += 20;

    return `${Math.round(efficiency)}%`;
  };

  const getWebSocketStatus = (): string => {
    if (!wsConnected) return "Disconnected";
    switch (connectionQuality) {
      case "excellent":
        return "Stable";
      case "good":
        return "Connected";
      case "poor":
        return "Unstable";
      default:
        return "Unknown";
    }
  };

  const getWebSocketHealthColor = (): string => {
    if (!wsConnected) return "disabled";
    switch (connectionQuality) {
      case "excellent":
      case "good":
        return "enabled";
      case "poor":
        return "warning";
      default:
        return "disabled";
    }
  };

  const getStatusIcon = (): string => {
    switch (securityLevel) {
      case SecurityLevel.SECURE:
        return "🔒";
      case SecurityLevel.WARNING:
        return "⚠️";
      case SecurityLevel.DANGER:
        return "🚨";
      case SecurityLevel.OFFLINE:
        return "📴";
      default:
        return "❓";
    }
  };

  const getRiskScoreDisplay = (): string => {
    return `${Math.round(riskScore * 100)}%`;
  };

  if (compact) {
    return (
      <div
        className={cn(
          styles.securityIndicator,
          styles.compact,
          styles[securityLevel],
          className
        )}
        data-testid="security-indicator"
      >
        <span className={styles.icon} data-testid="security-icon">
          {getStatusIcon()}
        </span>
        <span className={styles.score} data-testid="risk-score">
          {getRiskScoreDisplay()}
        </span>
      </div>
    );
  }

  return (
    <div
      className={cn(
        styles.securityIndicator,
        styles[securityLevel],
        { [styles.quantumProtected]: quantumProtection },
        className
      )}
      data-testid="security-indicator"
    >
      <div className={styles.content} data-testid="security-content">
        <div className={styles.status}>
          <span className={styles.statusText} data-testid="security-level">
            {getStatusText()}
          </span>
          <span className={styles.connection} data-testid="connection-status">
            {isOnline ? "🟢 Online" : "🔴 Offline"}
          </span>
        </div>

        <div className={styles.details} data-testid="security-details">
          <div className={styles.detail}>
            <span className={styles.detailLabel}>
              AI Risk Score (from AI Engine):
            </span>
            <span
              className={cn(
                styles.detailValue,
                styles[
                  `riskScore${
                    securityLevel.charAt(0).toUpperCase() +
                    securityLevel.slice(1)
                  }`
                ]
              )}
              data-testid="risk-score"
            >
              {getRiskScoreDisplay()}
            </span>
          </div>

          <div className={styles.detail}>
            <span className={styles.detailLabel}>Quantum Key Strength:</span>
            <span
              className={cn(styles.detailValue, {
                [styles.enabled]: quantumProtection,
                [styles.disabled]: !quantumProtection,
              })}
              data-testid="quantum-key-strength"
            >
              {quantumProtection ? getQuantumKeyStrength() : "N/A"}
            </span>
          </div>

          <div className={styles.detail}>
            <span className={styles.detailLabel}>System Efficiency:</span>
            <span
              className={cn(styles.detailValue, {
                [styles.enabled]: isOnline && quantumProtection,
                [styles.disabled]: !isOnline || !quantumProtection,
              })}
              data-testid="system-efficiency"
            >
              {getSystemEfficiency()}
            </span>
          </div>

          <div className={styles.detail}>
            <span className={styles.detailLabel}>
              Quantum Protection (Backend):
            </span>
            <span
              className={cn(styles.detailValue, {
                [styles.enabled]: quantumProtection,
                [styles.disabled]: !quantumProtection,
              })}
              data-testid="quantum-protection-status"
            >
              {getQuantumSchemeDisplay()}
            </span>
          </div>

          {quantumProtection && (
            <>
              <div className={styles.detail}>
                <span className={styles.detailLabel}>
                  Encryption Algorithm:
                </span>
                <span
                  className={cn(styles.detailValue, styles.keyId)}
                  data-testid="quantum-key-id"
                >
                  {quantumKeyId || "N/A"}
                </span>
              </div>

              <div className={styles.detail}>
                <span className={styles.detailLabel}>
                  Key Rotation (Security Service):
                </span>
                <span
                  className={styles.detailValue}
                  data-testid="key-rotation-status"
                >
                  {getKeyRotationStatus()}
                </span>
              </div>

              <div className={styles.detail}>
                <span className={styles.detailLabel}>
                  User Transactions (API):
                </span>
                <span
                  className={styles.detailValue}
                  data-testid="protected-count"
                >
                  {formatTransactionCount()}
                </span>
              </div>

              <div className={styles.detail}>
                <span className={styles.detailLabel}>
                  WebSocket Connection:
                </span>
                <span
                  className={cn(
                    styles.detailValue,
                    styles[getWebSocketHealthColor()]
                  )}
                  data-testid="websocket-status"
                >
                  {getWebSocketStatus()}
                </span>
              </div>

              <div className={styles.detail}>
                <span className={styles.detailLabel}>System Health:</span>
                <span
                  className={cn(styles.detailValue, {
                    [styles.enabled]: isOnline,
                    [styles.disabled]: !isOnline,
                  })}
                  data-testid="system-health"
                >
                  {isOnline ? "All Systems Operational" : "System Offline"}
                </span>
              </div>

              {wsErrors.length > 0 && (
                <div className={styles.detail}>
                  <span className={styles.detailLabel}>Connection Issues:</span>
                  <span
                    className={cn(styles.detailValue, styles.disabled)}
                    data-testid="websocket-errors"
                  >
                    {wsErrors.length} error{wsErrors.length > 1 ? "s" : ""}
                  </span>
                </div>
              )}
            </>
          )}
        </div>
      </div>
    </div>
  );
};

export default SecurityIndicator;
