import { FC, useState } from "react";
import cn from "classnames";
import { Modal } from "../../ui/Modal/Modal";
import { QuantumIndicator } from "../QuantumIndicator/QuantumIndicator";
import { QuantumProtectionDisplayProps } from "../QuantumProtectionDisplay/QuantumProtectionDisplay";
import styles from "./QuantumProtectionBadge.module.scss";

interface QuantumProtectionBadgeProps extends QuantumProtectionDisplayProps {
  className?: string;
}

export const QuantumProtectionBadge: FC<QuantumProtectionBadgeProps> = ({
  isActive,
  encryptionScheme,
  keyId,
  keyStrength,
  lastRotation,
  nextRotation,
  protectedTransactions,
  encryptionSpeed,
  className = "",
}) => {
  const [isModalOpen, setIsModalOpen] = useState(false);

  const getStatusText = (): string => {
    if (!isActive) return "Inactive";
    return "Active";
  };

  const getStatusIcon = (): string => {
    if (!isActive) return "🔓";
    return "⚛️";
  };

  const openModal = () => setIsModalOpen(true);
  const closeModal = () => setIsModalOpen(false);

  return (
    <>
      <button
        className={cn(
          styles.quantumBadge,
          isActive ? styles.active : styles.inactive,
          className.trim()
        )}
        onClick={openModal}
        title={`Quantum Protection: ${getStatusText()}`}
        data-testid="quantum-protection-badge"
      >
        <span className={styles.icon}>{getStatusIcon()}</span>
        <span className={styles.text}>{getStatusText()}</span>
      </button>

      <Modal
        isOpen={isModalOpen}
        onClose={closeModal}
        title="Quantum Protection"
        className={styles.quantumModal}
      >
        <div className={styles.modalContent}>
          <QuantumIndicator
            isActive={isActive}
            encryptionScheme={encryptionScheme}
            keyId={keyId}
            keyStrength={keyStrength}
            lastRotation={lastRotation}
            nextRotation={nextRotation}
            protectedTransactions={protectedTransactions}
            encryptionSpeed={encryptionSpeed}
            className={styles.quantumDisplay}
          />
        </div>
      </Modal>
    </>
  );
};
