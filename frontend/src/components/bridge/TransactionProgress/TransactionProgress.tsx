import { FC } from "react";
import cn from "classnames";
import type { TransactionProgress as TransactionProgressType } from "../../../types/bridge";
import styles from "./TransactionProgress.module.scss";

export interface TransactionProgressProps {
  transaction?: TransactionProgressType;
  loading?: boolean;
  className?: string;
}

export const TransactionProgress: FC<TransactionProgressProps> = ({
  transaction,
  loading = false,
  className = "",
}) => {
  if (loading) {
    return (
      <div
        className={cn(
          styles.transactionProgress,
          styles.loading,
          className.trim()
        )}
      >
        <div className={styles.spinner} data-testid="loading-spinner">
          Loading transaction status...
        </div>
      </div>
    );
  }

  if (!transaction) {
    return (
      <div
        className={cn(
          styles.transactionProgress,
          styles.empty,
          className.trim()
        )}
      >
        <p>No active transaction</p>
      </div>
    );
  }

  const getStepStatusIcon = (status: string) => {
    switch (status) {
      case "completed":
        return "✅";
      case "in_progress":
        return "⏳";
      case "failed":
        return "❌";
      default:
        return "⚪";
    }
  };

  return (
    <div className={cn(styles.transactionProgress, className.trim())}>
      <div className={styles.header}>
        <h3>Transaction Progress</h3>
        <div className={styles.id}>
          ID: {transaction.transactionId.substring(0, 8)}...
        </div>
      </div>

      <div className={styles.overview}>
        <div className={styles.status} data-testid="transaction-status">
          Status:{" "}
          <span
            className={cn({
              [styles.statusCompleted]: transaction.status === "completed",
              [styles.statusInProgress]: transaction.status === "in_progress",
              [styles.statusFailed]: transaction.status === "failed",
              [styles.statusPending]: transaction.status === "pending",
            })}
          >
            {transaction.status}
          </span>
        </div>

        <div className={styles.bar}>
          <div
            className={styles.fill}
            style={{ width: `${transaction.progress}%` }}
          />
          <span className={styles.percentage}>{transaction.progress}%</span>
        </div>

        <div className={styles.current}>
          Current Step: {transaction.currentStep}
        </div>

        {transaction.estimatedTimeRemaining && (
          <div className={styles.eta}>
            ETA: {Math.ceil(transaction.estimatedTimeRemaining / 60)} minutes
          </div>
        )}
      </div>

      <div className={styles.steps}>
        {transaction.steps.map((step, index) => (
          <div
            key={index}
            className={cn(styles.step, {
              [styles.stepPending]: step.status === "pending",
              [styles.stepInProgress]: step.status === "in_progress",
              [styles.stepCompleted]: step.status === "completed",
              [styles.stepFailed]: step.status === "failed",
            })}
          >
            <div className={styles.stepIcon}>
              {getStepStatusIcon(step.status)}
            </div>

            <div className={styles.stepContent}>
              <div className={styles.stepName}>{step.name}</div>
              <div className={styles.stepDescription}>{step.description}</div>

              {step.txHash && (
                <div className={styles.stepTx}>
                  <a
                    href={`#/tx/${step.txHash}`}
                    target="_blank"
                    rel="noopener noreferrer"
                  >
                    View Transaction: {step.txHash.substring(0, 10)}...
                  </a>
                </div>
              )}

              {step.timestamp && (
                <div className={styles.stepTime}>
                  {new Date(step.timestamp).toLocaleTimeString()}
                </div>
              )}
            </div>
          </div>
        ))}
      </div>

      {transaction.errorMessage && (
        <div className={styles.error}>
          <strong>Error:</strong> {transaction.errorMessage}
        </div>
      )}

      <div className={styles.hashes}>
        {transaction.fromTxHash && (
          <div className={styles.hash}>
            <span>Source TX:</span>
            <a
              href={`#/tx/${transaction.fromTxHash}`}
              target="_blank"
              rel="noopener noreferrer"
            >
              {transaction.fromTxHash.substring(0, 10)}...
            </a>
          </div>
        )}

        {transaction.toTxHash && (
          <div className={styles.hash}>
            <span>Destination TX:</span>
            <a
              href={`#/tx/${transaction.toTxHash}`}
              target="_blank"
              rel="noopener noreferrer"
            >
              {transaction.toTxHash.substring(0, 10)}...
            </a>
          </div>
        )}
      </div>
    </div>
  );
};
