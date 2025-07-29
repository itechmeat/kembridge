/**
 * TransactionProgress Component
 * Step-by-step progress visualization with real-time updates
 */

import { FC } from "react";
import type { TransactionProgress as TransactionProgressType } from "../../../types/bridge";

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
        className={`transaction-progress transaction-progress--loading ${className}`}
      >
        <div className="transaction-progress__spinner">
          Loading transaction status...
        </div>
      </div>
    );
  }

  if (!transaction) {
    return (
      <div
        className={`transaction-progress transaction-progress--empty ${className}`}
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
    <div className={`transaction-progress ${className}`}>
      <div className="transaction-progress__header">
        <h3>Transaction Progress</h3>
        <div className="transaction-progress__id">
          ID: {transaction.transactionId.substring(0, 8)}...
        </div>
      </div>

      <div className="transaction-progress__overview">
        <div className="transaction-progress__status">
          Status:{" "}
          <span className={`status-${transaction.status}`}>
            {transaction.status}
          </span>
        </div>

        <div className="transaction-progress__bar">
          <div
            className="transaction-progress__fill"
            style={{ width: `${transaction.progress}%` }}
          />
          <span className="transaction-progress__percentage">
            {transaction.progress}%
          </span>
        </div>

        <div className="transaction-progress__current">
          Current Step: {transaction.currentStep}
        </div>

        {transaction.estimatedTimeRemaining && (
          <div className="transaction-progress__eta">
            ETA: {Math.ceil(transaction.estimatedTimeRemaining / 60)} minutes
          </div>
        )}
      </div>

      <div className="transaction-progress__steps">
        {transaction.steps.map((step, index) => (
          <div
            key={index}
            className={`transaction-progress__step transaction-progress__step--${step.status}`}
          >
            <div className="transaction-progress__step-icon">
              {getStepStatusIcon(step.status)}
            </div>

            <div className="transaction-progress__step-content">
              <div className="transaction-progress__step-name">{step.name}</div>
              <div className="transaction-progress__step-description">
                {step.description}
              </div>

              {step.txHash && (
                <div className="transaction-progress__step-tx">
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
                <div className="transaction-progress__step-time">
                  {new Date(step.timestamp).toLocaleTimeString()}
                </div>
              )}
            </div>
          </div>
        ))}
      </div>

      {transaction.errorMessage && (
        <div className="transaction-progress__error">
          <strong>Error:</strong> {transaction.errorMessage}
        </div>
      )}

      <div className="transaction-progress__hashes">
        {transaction.fromTxHash && (
          <div className="transaction-progress__hash">
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
          <div className="transaction-progress__hash">
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
