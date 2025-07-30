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
import { AIRiskDisplay } from "../../components/features/security/AIRiskDisplay";
import {
  WebSocketStatus,
  RealTimeNotifications,
} from "../../components/websocket";
import { useTransactionStatus } from "../../hooks/bridge/useTransactionStatus";
import { useBridgeHistory } from "../../hooks/bridge/useBridgeHistory";
import { useErrorHandling } from "../../hooks/useErrorHandling";
import { ErrorContext } from "../../services/errorHandlingService";
import { websocketService } from "../../services/bridge/websocketService";
import { DEFAULT_USER_ID, RISK_ANALYSIS } from "../../constants/services";
import type { AIRiskAnalysisResponse } from "../../services/ai/aiRiskService";
import type {
  SwapFormData,
  TransactionProgress as TransactionProgressType,
  ChainType,
} from "../../types/bridge";

export const BridgePage: React.FC = () => {
  const [activeTransactionId, setActiveTransactionId] = useState<string>("");
  const [activeTab, setActiveTab] = useState<"swap" | "history">("swap");
  const [currentSwapData, setCurrentSwapData] = useState<SwapFormData | null>(
    null
  );
  const [isRiskBlocked, setIsRiskBlocked] = useState<boolean>(false);
  const [riskBlockReason, setRiskBlockReason] = useState<string>("");

  // Error handling hook
  const { handleError, testErrorHandling, showSuccess } = useErrorHandling();

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

  // Handle risk analysis results
  const handleRiskChange = useCallback((risk: AIRiskAnalysisResponse) => {
    console.log("Bridge: Risk analysis updated:", risk);
    if (!risk.approved || risk.risk_score > RISK_ANALYSIS.THRESHOLDS.HIGH) {
      setIsRiskBlocked(true);
      setRiskBlockReason(
        `High risk transaction (${risk.risk_level}): ${risk.reasons.join(", ")}`
      );
    } else {
      setIsRiskBlocked(false);
      setRiskBlockReason("");
    }
  }, []);

  const handleRiskBlock = useCallback((reason: string) => {
    setIsRiskBlocked(true);
    setRiskBlockReason(reason);
    console.warn("Bridge: Transaction blocked by AI risk analysis:", reason);
  }, []);

  // Handle swap execution
  const handleSwapExecute = useCallback(
    async (data: SwapFormData) => {
      console.log("Bridge: Swap executed:", data);

      // Check if transaction is blocked by risk analysis
      if (isRiskBlocked) {
        throw new Error(`Transaction blocked: ${riskBlockReason}`);
      }

      // Store current swap data for risk analysis
      setCurrentSwapData(data);

      try {
        // Import bridge service
        const { bridgeService } = await import(
          "../../services/api/bridgeService"
        );

        // Execute real swap through bridge service
        const result = await bridgeService.executeSwap({
          fromToken: {
            symbol: data.fromToken.symbol,
            decimals: data.fromToken.decimals,
          },
          toToken: {
            symbol: data.toToken.symbol,
            decimals: data.toToken.decimals,
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

        showSuccess("Transaction initiated successfully!");
        return result;
      } catch (error) {
        console.error("❌ Bridge: Swap execution failed:", error);
        handleError(error, {
          operation: "bridge_swap",
          transactionData: data as unknown as Record<string, unknown>,
        } as ErrorContext);
        throw error;
      }
    },
    [isRiskBlocked, riskBlockReason, handleError, showSuccess]
  );

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

          {/* WebSocket Status */}
          <div className="bridge-page__websocket-status">
            <WebSocketStatus showDetails={false} />
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
                      onDataChange={(data) =>
                        setCurrentSwapData(data as SwapFormData)
                      }
                      disabled={isRiskBlocked}
                      className="bridge-page__form"
                    />
                    {isRiskBlocked && (
                      <div
                        className="bridge-page__risk-warning"
                        data-testid="risk-warning"
                      >
                        <span className="bridge-page__risk-icon">⚠️</span>
                        <span className="bridge-page__risk-message">
                          {riskBlockReason}
                        </span>
                      </div>
                    )}
                  </div>

                  {/* AI Risk Analysis Display */}
                  <div className="bridge-page__ai-risk">
                    <AIRiskDisplay
                      userId={DEFAULT_USER_ID} // Using default user ID from constants
                      transaction={
                        currentSwapData
                          ? {
                              transactionId: activeTransactionId || undefined,
                              amount: parseFloat(currentSwapData.amount) || 0,
                              sourceChain: currentSwapData.fromChain,
                              destinationChain: currentSwapData.toChain,
                              sourceToken: currentSwapData.fromToken.symbol,
                              destinationToken: currentSwapData.toToken.symbol,
                              userAddress: currentSwapData.recipient,
                            }
                          : undefined
                      }
                      autoAnalyze={!!currentSwapData}
                      onRiskChange={handleRiskChange}
                      onBlock={handleRiskBlock}
                      className="bridge-page__ai-risk-display"
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
                      lastRotation={new Date(
                        Date.now() - 24 * 60 * 60 * 1000
                      ).toISOString()}
                      nextRotation={new Date(
                        Date.now() + 7 * 24 * 60 * 60 * 1000
                      ).toISOString()}
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
                      lastKeyRotation={new Date(
                        Date.now() - 24 * 60 * 60 * 1000
                      ).toISOString()}
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

            {/* Developer Error Testing Panel */}
            {process.env.NODE_ENV === "development" && (
              <div className="bridge-page__dev-panel">
                <details className="bridge-page__dev-details">
                  <summary className="bridge-page__dev-summary">
                    🧪 Error Handling Tests
                  </summary>
                  <div className="bridge-page__dev-content">
                    <p className="bridge-page__dev-description">
                      Test the error handling system with different error types:
                    </p>
                    <div className="bridge-page__dev-buttons">
                      <button
                        className="bridge-page__dev-button"
                        onClick={() => testErrorHandling("validation")}
                      >
                        Validation Error
                      </button>
                      <button
                        className="bridge-page__dev-button"
                        onClick={() => testErrorHandling("auth")}
                      >
                        Auth Error
                      </button>
                      <button
                        className="bridge-page__dev-button"
                        onClick={() => testErrorHandling("network")}
                      >
                        Network Error
                      </button>
                      <button
                        className="bridge-page__dev-button"
                        onClick={() => testErrorHandling("service")}
                      >
                        Service Error
                      </button>
                      <button
                        className="bridge-page__dev-button"
                        onClick={() =>
                          showSuccess("Test success notification!")
                        }
                      >
                        Success Test
                      </button>
                    </div>
                  </div>
                </details>
              </div>
            )}
          </div>
        </div>
      </div>

      {/* Real-time Notifications */}
      <RealTimeNotifications
        maxNotifications={3}
        autoHide={true}
        autoHideDelay={8000}
      />
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
