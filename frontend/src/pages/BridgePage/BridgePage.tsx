/**
 * Bridge Page
 * Main bridge interface page with full functionality
 */

import React, { useState, useCallback } from "react";
import { SwapForm } from "../../components/bridge/SwapForm/SwapForm";
import { TransactionProgress } from "../../components/bridge/TransactionProgress/TransactionProgress";
import { TransactionHistory } from "../../components/bridge/TransactionHistory/TransactionHistory";
import { QuantumProtectionDisplay } from "../../components/security/QuantumProtectionDisplay/QuantumProtectionDisplay";
import { SecurityIndicator } from "../../components/security/SecurityIndicator/SecurityIndicator";
import { useTransactionStatus } from "../../hooks/bridge/useTransactionStatus";
import { useBridgeHistory } from "../../hooks/bridge/useBridgeHistory";
import { websocketService } from "../../services/bridge/websocketService";
import type {
  SwapFormData,
  TransactionProgress as TransactionProgressType,
  ChainType,
} from "../../types/bridge";

export const BridgePage: React.FC = () => {
  const [activeTransactionId, setActiveTransactionId] = useState<string>("");
  const [activeTab, setActiveTab] = useState<"swap" | "history">("swap");

  // Get active transaction status
  const { data: activeTransaction, isLoading: transactionLoading } =
    useTransactionStatus(activeTransactionId, {
      enabled: !!activeTransactionId,
    });

  // Get transaction history
  const {
    data: historyData,
    isLoading: historyLoading,
    error: historyError,
  } = useBridgeHistory(1, 10);

  // Handle swap execution
  const handleSwapExecute = useCallback(async (data: SwapFormData) => {
    console.log("Bridge: Swap executed:", data);

    try {
      // Import bridge service
      const { bridgeService } = await import("../../services/api/bridgeService");
      
      // Execute real swap through bridge service
      const result = await bridgeService.executeSwap({
        fromToken: { 
          symbol: data.fromToken.symbol, 
          decimals: data.fromToken.decimals 
        },
        toToken: { 
          symbol: data.toToken.symbol, 
          decimals: data.toToken.decimals 
        },
        fromChain: data.fromChain,
        toChain: data.toChain,
        amount: data.amount,
        recipient: data.recipient,
        slippage: data.slippage / 100, // Convert percentage to decimal
      });

      console.log("✅ Bridge: Swap executed successfully:", result);

      // Set active transaction for monitoring
      setActiveTransactionId(result.transaction_id);
      
      // Switch to history tab to show progress
      // setActiveTab("history");
      
      return result;
    } catch (error) {
      console.error("❌ Bridge: Swap execution failed:", error);
      throw error;
    }
  }, []);

  // Connect WebSocket on component mount
  React.useEffect(() => {
    if (!websocketService.isConnected()) {
      websocketService.connect().catch(console.error);
    }

    return () => {
      if (activeTransactionId) {
        websocketService.unsubscribeFromTransaction(activeTransactionId);
      }
    };
  }, [activeTransactionId]);

  return (
    <div className="bridge-page">
      <div className="bridge-page__container">
        <header className="bridge-page__header">
          <div className="bridge-page__title-section">
            <h1 className="bridge-page__title">KEMBridge</h1>
            <p className="bridge-page__subtitle">
              Secure cross-chain transfers between Ethereum and NEAR
            </p>
            <div className="bridge-page__features">
              <span className="bridge-page__feature">🔒 Quantum Protected</span>
              <span className="bridge-page__feature">⚡ Fast & Secure</span>
              <span className="bridge-page__feature">💰 Low Fees</span>
            </div>
          </div>
        </header>

        <div className="bridge-page__content">
          {/* Mobile-First Main Content */}
          <div className="bridge-page__main">
            {/* Tab Navigation */}
            <div className="bridge-page__tabs">
              <button
                className={`bridge-page__tab ${
                  activeTab === "swap" ? "bridge-page__tab--active" : ""
                }`}
                onClick={() => setActiveTab("swap")}
              >
                <span className="bridge-page__tab-icon">⚡</span>
                Bridge
              </button>
              <button
                className={`bridge-page__tab ${
                  activeTab === "history" ? "bridge-page__tab--active" : ""
                }`}
                onClick={() => setActiveTab("history")}
              >
                <span className="bridge-page__tab-icon">📋</span>
                History
              </button>
            </div>

            {/* Tab Content */}
            <div className="bridge-page__tab-content">
              {activeTab === "swap" && (
                <div className="bridge-page__swap-section">
                  {/* Bridge Stats - Mobile Quick View */}
                  <div className="bridge-page__quick-stats">
                    <div className="bridge-page__quick-stat">
                      <span className="bridge-page__quick-stat-value">
                        $2.4M
                      </span>
                      <span className="bridge-page__quick-stat-label">
                        Volume
                      </span>
                    </div>
                    <div className="bridge-page__quick-stat">
                      <span className="bridge-page__quick-stat-value">
                        1,247
                      </span>
                      <span className="bridge-page__quick-stat-label">
                        Swaps
                      </span>
                    </div>
                    <div className="bridge-page__quick-stat">
                      <span className="bridge-page__quick-stat-value">
                        99.8%
                      </span>
                      <span className="bridge-page__quick-stat-label">
                        Success
                      </span>
                    </div>
                    <div className="bridge-page__quick-stat">
                      <span className="bridge-page__quick-stat-value">12m</span>
                      <span className="bridge-page__quick-stat-label">
                        Avg Time
                      </span>
                    </div>
                  </div>

                  <div className="bridge-page__swap-form">
                    <SwapForm
                      onSwapExecute={handleSwapExecute}
                      className="bridge-page__form"
                    />
                  </div>

                  {/* Active Transaction Progress */}
                  {activeTransactionId && (
                    <div className="bridge-page__progress-section">
                      <div className="bridge-page__progress-header">
                        <h3>Current Transaction</h3>
                        <button
                          onClick={() => setActiveTransactionId("")}
                          className="bridge-page__close-progress"
                        >
                          ×
                        </button>
                      </div>

                      <TransactionProgress
                        transaction={
                          activeTransaction
                            ? {
                                transactionId: activeTransaction.id,
                                status: activeTransaction.status,
                                progress: getProgressFromStatus(
                                  activeTransaction.status
                                ),
                                currentStep: getCurrentStep(
                                  activeTransaction.status
                                ),
                                steps: getTransactionSteps(
                                  activeTransaction.status
                                ),
                                fromTxHash:
                                  activeTransaction.from_transaction_hash,
                                toTxHash: activeTransaction.to_transaction_hash,
                                errorMessage: undefined, // TODO: Add error handling
                              }
                            : undefined
                        }
                        loading={transactionLoading}
                        className="bridge-page__progress"
                      />
                    </div>
                  )}

                  {/* Quantum Security Display */}
                  <div className="bridge-page__quantum-security">
                    <QuantumProtectionDisplay
                      isActive={true}
                      encryptionScheme="ML-KEM-1024"
                      keyId="12345678-1234-1234-1234-123456789abc"
                      keyStrength={1024}
                      lastRotation={new Date(Date.now() - 24 * 60 * 60 * 1000).toISOString()}
                      nextRotation={new Date(Date.now() + 7 * 24 * 60 * 60 * 1000).toISOString()}
                      protectedTransactions={1247}
                      encryptionSpeed={15000}
                      className="bridge-page__quantum-display"
                    />
                  </div>

                  {/* Security Indicator */}
                  <div className="bridge-page__security-indicator">
                    <SecurityIndicator
                      quantumProtection={true}
                      riskScore={0.2}
                      isOnline={true}
                      quantumKeyId="12345678-1234-1234-1234-123456789abc"
                      encryptionScheme="ML-KEM-1024"
                      lastKeyRotation={new Date(Date.now() - 24 * 60 * 60 * 1000).toISOString()}
                      transactionCount={1247}
                      className="bridge-page__security"
                    />
                  </div>
                </div>
              )}

              {activeTab === "history" && (
                <div className="bridge-page__history-section">
                  <TransactionHistory
                    transactions={
                      historyData?.transactions.map((tx) => ({
                        id: tx.id,
                        fromChain: tx.from_chain as ChainType,
                        toChain: tx.to_chain as ChainType,
                        fromToken: tx.from_token,
                        toToken: tx.to_token,
                        fromAmount: tx.from_amount,
                        toAmount: tx.to_amount,
                        status: tx.status,
                        createdAt: tx.created_at,
                        completedAt: tx.actual_completion_at,
                        fromTxHash: tx.from_transaction_hash,
                        toTxHash: tx.to_transaction_hash,
                        usdValue: undefined, // TODO: Calculate USD value
                      })) || []
                    }
                    loading={historyLoading}
                    error={historyError?.message}
                    onLoadMore={() => {
                      // TODO: Implement pagination
                    }}
                    hasMore={false}
                    className="bridge-page__history"
                  />
                </div>
              )}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};

// Helper functions for transaction status mapping
function getProgressFromStatus(status: string): number {
  const progressMap: Record<string, number> = {
    pending: 20,
    confirmed: 60,
    completed: 100,
    failed: 0,
    expired: 0,
  };
  return progressMap[status] || 0;
}

function getCurrentStep(status: string): string {
  const stepMap: Record<string, string> = {
    pending: "Validating transaction",
    confirmed: "Processing cross-chain transfer",
    completed: "Transaction completed",
    failed: "Transaction failed",
    expired: "Transaction expired",
  };
  return stepMap[status] || "Unknown status";
}

function getTransactionSteps(status: string): TransactionProgressType["steps"] {
  const baseSteps = [
    {
      name: "Validate",
      status: "completed" as const,
      description: "Transaction validated",
    },
    { name: "Lock", status: "pending" as const, description: "Locking tokens" },
    {
      name: "Transfer",
      status: "pending" as const,
      description: "Cross-chain transfer",
    },
    {
      name: "Complete",
      status: "pending" as const,
      description: "Finalizing transaction",
    },
  ];

  // Update step statuses based on transaction status
  if (status === "completed") {
    return baseSteps.map((step) => ({ ...step, status: "completed" as const }));
  } else if (status === "confirmed") {
    return baseSteps.map((step, index) => ({
      ...step,
      status:
        index < 2
          ? ("completed" as const)
          : index === 2
          ? ("in_progress" as const)
          : ("pending" as const),
    }));
  } else if (status === "failed" || status === "expired") {
    return baseSteps.map((step, index) => ({
      ...step,
      status: index === 0 ? ("failed" as const) : ("pending" as const),
    }));
  }

  return baseSteps;
}
