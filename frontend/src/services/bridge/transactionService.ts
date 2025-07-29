/**
 * Transaction Service
 * Transaction tracking and status management
 */

import { SwapTransaction } from "../api/bridgeService";

export interface TransactionStatusUpdate {
  id: string;
  status: SwapTransaction["status"];
  progress: number;
  currentStep: string;
  fromTxHash?: string;
  toTxHash?: string;
  errorMessage?: string;
}

class TransactionService {
  private listeners: Map<string, (update: TransactionStatusUpdate) => void> =
    new Map();

  /**
   * Subscribe to transaction status updates
   */
  subscribeToTransaction(
    transactionId: string,
    callback: (update: TransactionStatusUpdate) => void
  ): () => void {
    this.listeners.set(transactionId, callback);

    // Return unsubscribe function
    return () => {
      this.listeners.delete(transactionId);
    };
  }

  /**
   * Notify listeners of transaction update
   */
  notifyTransactionUpdate(update: TransactionStatusUpdate): void {
    const listener = this.listeners.get(update.id);
    if (listener) {
      listener(update);
    }
  }

  /**
   * Maps backend transaction status to display progress
   */
  getProgressFromStatus(status: SwapTransaction["status"]): number {
    const progressMap: Record<SwapTransaction["status"], number> = {
      pending: 10,
      confirmed: 40,
      completed: 100,
      failed: 0,
      expired: 0,
    };
    return progressMap[status] || 0;
  }

  /**
   * Gets human-readable status description
   */
  getStatusDescription(status: SwapTransaction["status"]): string {
    const descriptions: Record<SwapTransaction["status"], string> = {
      pending: "Waiting for confirmation",
      confirmed: "Processing cross-chain transfer",
      completed: "Transaction completed successfully",
      failed: "Transaction failed",
      expired: "Transaction expired",
    };
    return descriptions[status] || status;
  }

  /**
   * Gets estimated completion steps
   */
  getTransactionSteps(transaction: SwapTransaction): Array<{
    name: string;
    status: "pending" | "in_progress" | "completed" | "failed";
    description: string;
  }> {
    const steps = [
      {
        name: "Initialize",
        description: "Transaction created and validated",
        status: "pending" as "pending" | "in_progress" | "completed" | "failed",
      },
      {
        name: "Lock Tokens",
        description: `Locking tokens on ${transaction.from_chain}`,
        status: "pending" as "pending" | "in_progress" | "completed" | "failed",
      },
      {
        name: "Cross-Chain Transfer",
        description: "Processing cross-chain bridge operation",
        status: "pending" as "pending" | "in_progress" | "completed" | "failed",
      },
      {
        name: "Mint/Unlock",
        description: `Minting/unlocking tokens on ${transaction.to_chain}`,
        status: "pending" as "pending" | "in_progress" | "completed" | "failed",
      },
      {
        name: "Complete",
        description: "Transaction completed successfully",
        status: "pending" as "pending" | "in_progress" | "completed" | "failed",
      },
    ];

    // Update step statuses based on transaction status
    steps.forEach((step, index) => {
      if (transaction.status === "failed" || transaction.status === "expired") {
        step.status = index === 0 ? "failed" : "pending";
      } else if (transaction.status === "completed") {
        step.status = "completed";
      } else if (transaction.status === "confirmed") {
        step.status =
          index <= 2 ? "completed" : index === 3 ? "in_progress" : "pending";
      } else if (transaction.status === "pending") {
        step.status =
          index === 0 ? "completed" : index === 1 ? "in_progress" : "pending";
      } else {
        step.status = "pending";
      }
    });

    return steps;
  }

  /**
   * Checks if transaction needs user action
   */
  requiresUserAction(): {
    required: boolean;
    action?: string;
    description?: string;
  } {
    // TODO: Implement based on actual bridge logic
    // For now, no user actions required after initial submission
    return { required: false };
  }

  /**
   * Gets explorer URL for transaction hash
   */
  getExplorerUrl(txHash: string, chain: string): string {
    const explorers: Record<string, string> = {
      ethereum: "https://sepolia.etherscan.io/tx",
      near: "https://testnet.nearblocks.io/txns",
    };

    const baseUrl = explorers[chain];
    return baseUrl ? `${baseUrl}/${txHash}` : "#";
  }

  /**
   * Formats transaction for display
   */
  formatTransaction(transaction: SwapTransaction): {
    displayId: string;
    displayRoute: string;
    displayAmount: string;
    displayStatus: string;
    displayTime: string;
  } {
    return {
      displayId: `${transaction.id.substring(0, 8)}...`,
      displayRoute: `${transaction.from_chain} → ${transaction.to_chain}`,
      displayAmount: `${transaction.from_amount} ${transaction.from_token}`,
      displayStatus: this.getStatusDescription(transaction.status),
      displayTime: new Date(transaction.created_at).toLocaleString(),
    };
  }

  /**
   * Estimates time remaining for transaction
   */
  estimateTimeRemaining(transaction: SwapTransaction): number | null {
    if (
      transaction.status === "completed" ||
      transaction.status === "failed" ||
      transaction.status === "expired"
    ) {
      return null;
    }

    // TODO: Implement based on actual transaction timing data
    // For now, return basic estimates
    const baseEstimate = 15 * 60 * 1000; // 15 minutes in milliseconds
    const elapsed = Date.now() - new Date(transaction.created_at).getTime();

    return Math.max(0, baseEstimate - elapsed);
  }
}

export const transactionService = new TransactionService();
export default transactionService;
