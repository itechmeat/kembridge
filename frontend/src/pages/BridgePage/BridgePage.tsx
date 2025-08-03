import { FC, useState, useCallback, useEffect } from "react";
import { SwapForm } from "../../components/bridge/SwapForm/SwapForm";
import { TransactionProgress } from "../../components/bridge/TransactionProgress/TransactionProgress";
import { AIRiskDisplay } from "../../components/features/AIRiskDisplay/AIRiskDisplay";
import { RealTimeNotifications } from "../../components/websocket";
import { useTransactionStatus } from "../../hooks/bridge/useTransactionStatus";
import { useErrorHandling } from "../../hooks/api/useErrorHandling";
import { websocketService } from "../../services/bridge/websocketService";
import { DEFAULT_USER_ID, RISK_ANALYSIS } from "../../constants/services";
import type { AIRiskAnalysisResponse } from "../../services/ai/aiRiskService";
import type {
  SwapFormData,
  TransactionProgress as TransactionProgressType,
} from "../../types/bridge";
import styles from "./BridgePage.module.scss";

export const BridgePage: FC = () => {
  const [activeTransactionId, setActiveTransactionId] = useState<string>("");
  const [currentSwapData, setCurrentSwapData] = useState<SwapFormData | null>(
    null
  );
  const [isRiskBlocked, setIsRiskBlocked] = useState<boolean>(false);
  const [riskBlockReason, setRiskBlockReason] = useState<string>("");

  const { testErrorHandling, showSuccess } = useErrorHandling();

  const { data: activeTransaction, isLoading: transactionLoading } =
    useTransactionStatus(activeTransactionId, {
      enabled: !!activeTransactionId,
    });

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


  // Connect WebSocket on component mount
  useEffect(() => {
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
    <div className={styles.bridgePage}>
      <div className={styles.container}>
        <div className={styles.content}>
          <div className={styles.main}>
            <div className={styles.tabContent}>
              <div data-testid="bridge-form">
                <div className={styles.swapSection}>
                  <div className={styles.swapForm}>
                    <SwapForm
                      onDataChange={(data) =>
                        setCurrentSwapData(data as SwapFormData)
                      }
                      disabled={isRiskBlocked}
                      className={styles.form}
                    />
                    {isRiskBlocked && (
                      <div
                        className={styles.riskWarning}
                        data-testid="risk-warning"
                      >
                        <span className={styles.riskIcon}>⚠️</span>
                        <span className={styles.riskMessage}>
                          {riskBlockReason}
                        </span>
                      </div>
                    )}
                  </div>

                  {currentSwapData && (
                    <div className={styles.aiRisk}>
                      <AIRiskDisplay
                        userId={DEFAULT_USER_ID}
                        transaction={
                          currentSwapData
                            ? {
                                transactionId: activeTransactionId || undefined,
                                amount: parseFloat(currentSwapData.amount) || 0,
                                sourceChain: currentSwapData.fromChain,
                                destinationChain: currentSwapData.toChain,
                                sourceToken: currentSwapData.fromToken.symbol,
                                destinationToken:
                                  currentSwapData.toToken.symbol,
                                userAddress: currentSwapData.recipient,
                              }
                            : undefined
                        }
                        autoAnalyze={!!currentSwapData}
                        onRiskChange={handleRiskChange}
                        onBlock={handleRiskBlock}
                        className={styles.aiRiskDisplay}
                      />
                    </div>
                  )}

                  {activeTransactionId && (
                    <div className={styles.progressSection}>
                      <div className={styles.progressHeader}>
                        <h3>Current Transaction</h3>
                        <button
                          onClick={() => setActiveTransactionId("")}
                          className={styles.closeProgress}
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
                        className={styles.progress}
                      />
                    </div>
                  )}
                </div>
              </div>
            </div>

            {/* Developer Error Testing Panel */}
            {import.meta.env.DEV && !true && (
              <div className={styles.devPanel}>
                <details className={styles.devDetails}>
                  <summary className={styles.devSummary}>
                    🧪 Error Handling Tests
                  </summary>
                  <div className={styles.devContent}>
                    <p className={styles.devDescription}>
                      Test the error handling system with different error types:
                    </p>
                    <div className={styles.devButtons}>
                      <button
                        className={styles.devButton}
                        onClick={() => testErrorHandling("validation")}
                      >
                        Validation Error
                      </button>
                      <button
                        className={styles.devButton}
                        onClick={() => testErrorHandling("auth")}
                      >
                        Auth Error
                      </button>
                      <button
                        className={styles.devButton}
                        onClick={() => testErrorHandling("network")}
                      >
                        Network Error
                      </button>
                      <button
                        className={styles.devButton}
                        onClick={() => testErrorHandling("service")}
                      >
                        Service Error
                      </button>
                      <button
                        className={styles.devButton}
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
