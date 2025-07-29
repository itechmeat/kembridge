/**
 * TransactionHistory Component
 * History with basic functionality (virtualization for Phase 8.2)
 */

import { FC } from "react";
import type { TransactionHistoryItem } from "../../../types/bridge";

export interface TransactionHistoryProps {
  transactions: TransactionHistoryItem[];
  loading?: boolean;
  error?: string;
  onLoadMore?: () => void;
  hasMore?: boolean;
  className?: string;
}

export const TransactionHistory: FC<TransactionHistoryProps> = ({
  transactions,
  loading = false,
  error,
  onLoadMore,
  hasMore = false,
  className = "",
}) => {
  const formatDate = (dateString: string) => {
    return new Date(dateString).toLocaleDateString(undefined, {
      year: "numeric",
      month: "short",
      day: "numeric",
      hour: "2-digit",
      minute: "2-digit",
    });
  };

  const getStatusColor = (status: string) => {
    const colorMap: Record<string, string> = {
      completed: "#10b981",
      pending: "#f59e0b",
      failed: "#ef4444",
      cancelled: "#6b7280",
      expired: "#6b7280",
    };
    return colorMap[status] || "#6b7280";
  };

  if (error) {
    return (
      <div
        className={`transaction-history transaction-history--error ${className}`}
      >
        <div className="transaction-history__error">{error}</div>
      </div>
    );
  }

  return (
    <div className={`transaction-history ${className}`}>
      <div className="transaction-history__header">
        <h3>Transaction History</h3>
        <span className="transaction-history__count">
          {transactions.length} transactions
        </span>
      </div>

      {loading && transactions.length === 0 ? (
        <div className="transaction-history__loading">
          Loading transactions...
        </div>
      ) : (
        <div className="transaction-history__list">
          {transactions.length === 0 ? (
            <div className="transaction-history__empty">
              No transactions found
            </div>
          ) : (
            transactions.map((transaction) => (
              <div key={transaction.id} className="transaction-history__item">
                <div className="transaction-history__main">
                  <div className="transaction-history__route">
                    <span className="transaction-history__chain">
                      {transaction.fromChain}
                    </span>
                    <span className="transaction-history__arrow">→</span>
                    <span className="transaction-history__chain">
                      {transaction.toChain}
                    </span>
                  </div>

                  <div className="transaction-history__amounts">
                    <div className="transaction-history__from">
                      {transaction.fromAmount} {transaction.fromToken}
                    </div>
                    <div className="transaction-history__to">
                      {transaction.toAmount} {transaction.toToken}
                    </div>
                  </div>

                  <div
                    className="transaction-history__status"
                    style={{ color: getStatusColor(transaction.status) }}
                  >
                    {transaction.status}
                  </div>
                </div>

                <div className="transaction-history__details">
                  <div className="transaction-history__date">
                    {formatDate(transaction.createdAt)}
                  </div>

                  {transaction.usdValue && (
                    <div className="transaction-history__usd">
                      ≈ ${transaction.usdValue}
                    </div>
                  )}

                  <div className="transaction-history__hashes">
                    {transaction.fromTxHash && (
                      <a
                        href={`#/tx/${transaction.fromTxHash}`}
                        target="_blank"
                        rel="noopener noreferrer"
                        className="transaction-history__hash"
                      >
                        Source TX
                      </a>
                    )}

                    {transaction.toTxHash && (
                      <a
                        href={`#/tx/${transaction.toTxHash}`}
                        target="_blank"
                        rel="noopener noreferrer"
                        className="transaction-history__hash"
                      >
                        Destination TX
                      </a>
                    )}
                  </div>
                </div>
              </div>
            ))
          )}

          {hasMore && (
            <div className="transaction-history__load-more">
              <button
                onClick={onLoadMore}
                disabled={loading}
                className="transaction-history__load-button"
              >
                {loading ? "Loading..." : "Load More"}
              </button>
            </div>
          )}
        </div>
      )}
    </div>
  );
};
