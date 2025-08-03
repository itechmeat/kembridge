import { FC, useState } from "react";
import cn from "classnames";
import { Modal } from "../../ui/Modal/Modal";
import { SecurityIndicator } from "../SecurityIndicator/SecurityIndicator";
import { SecurityLevel } from "../../../types";
import { useWebSocketContext } from "../../../contexts/WebSocketContext";
import styles from "./SecurityStatusBadge.module.scss";

interface SecurityStatusBadgeProps {
  quantumProtection: boolean;
  riskScore: number;
  isOnline: boolean;
  quantumKeyId?: string;
  encryptionScheme?: string;
  lastKeyRotation?: string;
  transactionCount?: number;
  className?: string;
}

export const SecurityStatusBadge: FC<SecurityStatusBadgeProps> = ({
  quantumProtection,
  riskScore,
  isOnline,
  quantumKeyId,
  encryptionScheme,
  lastKeyRotation,
  transactionCount,
  className = "",
}) => {
  const [isModalOpen, setIsModalOpen] = useState(false);

  // WebSocket context for connection status
  const { isConnected: wsConnected, connectionQuality } = useWebSocketContext();

  // Determine security level based on quantum protection and risk score
  const getSecurityLevel = (): SecurityLevel => {
    if (!isOnline) return SecurityLevel.OFFLINE;
    if (!quantumProtection) return SecurityLevel.DANGER;
    if (riskScore > 0.7) return SecurityLevel.DANGER;
    if (riskScore > 0.3) return SecurityLevel.WARNING;
    return SecurityLevel.SECURE;
  };

  const securityLevel = getSecurityLevel();

  const getWebSocketIcon = (): string => {
    if (!wsConnected) return "🔴";
    switch (connectionQuality) {
      case "excellent":
      case "good":
        return "🟢";
      case "poor":
        return "🟡";
      default:
        return "🔴";
    }
  };

  const getRiskScoreText = (): string => {
    const percentage = Math.round(riskScore * 100);
    return `${percentage}%`;
  };

  const getStatusText = (): string => {
    switch (securityLevel) {
      case SecurityLevel.SECURE:
        return "Secure";
      case SecurityLevel.WARNING:
        return "Warning";
      case SecurityLevel.DANGER:
        return "Risk";
      case SecurityLevel.OFFLINE:
        return "Offline";
      default:
        return "Unknown";
    }
  };

  const openModal = () => setIsModalOpen(true);
  const closeModal = () => setIsModalOpen(false);

  return (
    <>
      <button
        className={cn(
          styles.securityBadge,
          styles[securityLevel.toLowerCase()],
          className.trim()
        )}
        onClick={openModal}
        title={`Security Status: ${getStatusText()}`}
        data-testid="security-status-badge"
      >
        <span className={styles.wsIcon}>{getWebSocketIcon()}</span>
        <span className={styles.icon}>{getRiskScoreText()}</span>
        <span className={styles.text}>{getStatusText()}</span>
      </button>

      <Modal
        isOpen={isModalOpen}
        onClose={closeModal}
        title="Security Status"
        className={styles.securityModal}
      >
        <div className={styles.modalContent}>
          <SecurityIndicator
            quantumProtection={quantumProtection}
            riskScore={riskScore}
            isOnline={isOnline}
            quantumKeyId={quantumKeyId}
            encryptionScheme={encryptionScheme}
            lastKeyRotation={lastKeyRotation}
            transactionCount={transactionCount}
            wsConnected={wsConnected}
            connectionQuality={connectionQuality}
            wsErrors={[]}
            compact={false}
            className={styles.securityIndicator}
          />
        </div>
      </Modal>
    </>
  );
};
