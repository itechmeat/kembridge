import { FC } from "react";
import cn from "classnames";
import type { TransactionHistoryItem } from "../../../types/bridge";
import styles from "./TransactionHistory.module.scss";

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
        className={cn(
          styles.transactionHistory,
          styles.error,
          className.trim()
        )}
      >
        <div className={styles.errorMessage}>{error}</div>
      </div>
    );
  }

  return (
    <div className={cn(styles.transactionHistory, className.trim())}>
      <div className={styles.header}>
        <h3>Transaction History</h3>
        <span className={styles.count}>{transactions.length} transactions</span>
      </div>

      {loading && transactions.length === 0 ? (
        <div className={styles.loading}>Loading transactions...</div>
      ) : (
        <div className={styles.list}>
          {transactions.length === 0 ? (
            <div className={styles.empty}>No transactions found</div>
          ) : (
            transactions.map((transaction) => (
              <div key={transaction.id} className={styles.item}>
                <div className={styles.main}>
                  <div className={styles.route}>
                    <span className={styles.chain}>
                      {transaction.fromChain}
                    </span>
                    <span className={styles.arrow}>→</span>
                    <span className={styles.chain}>{transaction.toChain}</span>
                  </div>

                  <div className={styles.amounts}>
                    <div className={styles.from}>
                      {transaction.fromAmount} {transaction.fromToken}
                    </div>
                    <div className={styles.to}>
                      {transaction.toAmount} {transaction.toToken}
                    </div>
                  </div>

                  <div
                    className={styles.status}
                    style={{ color: getStatusColor(transaction.status) }}
                  >
                    {transaction.status}
                  </div>
                </div>

                <div className={styles.details}>
                  <div className={styles.date}>
                    {formatDate(transaction.createdAt)}
                  </div>

                  {transaction.usdValue && (
                    <div className={styles.usd}>≈ ${transaction.usdValue}</div>
                  )}

                  <div className={styles.hashes}>
                    {transaction.fromTxHash && (
                      <a
                        href={`#/tx/${transaction.fromTxHash}`}
                        target="_blank"
                        rel="noopener noreferrer"
                        className={styles.hash}
                      >
                        Source TX
                      </a>
                    )}

                    {transaction.toTxHash && (
                      <a
                        href={`#/tx/${transaction.toTxHash}`}
                        target="_blank"
                        rel="noopener noreferrer"
                        className={styles.hash}
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
            <div className={styles.loadMore}>
              <button
                onClick={onLoadMore}
                disabled={loading}
                className={styles.loadButton}
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
