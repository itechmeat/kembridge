import { useEffect, useState, useRef, useMemo, FC, CSSProperties } from "react";
import cn from "classnames";
import { useAIRiskAnalysis } from "../../../hooks/security/useAIRiskAnalysis";
import type { AIRiskAnalysisResponse } from "../../../services/ai/aiRiskService";
import {
  RISK_ANALYSIS,
  ERROR_MESSAGES,
  STATUS_MESSAGES,
} from "../../../constants/services";
import styles from "./AIRiskDisplay.module.scss";

export interface AIRiskDisplayProps {
  userId: string;
  transaction?: {
    transactionId?: string;
    amount: number;
    sourceChain: string;
    destinationChain: string;
    sourceToken: string;
    destinationToken: string;
    userAddress?: string;
  };
  autoAnalyze?: boolean;
  onRiskChange?: (risk: AIRiskAnalysisResponse) => void;
  onBlock?: (reason: string) => void;
  className?: string;
}

export const AIRiskDisplay: FC<AIRiskDisplayProps> = ({
  userId,
  transaction,
  autoAnalyze = false,
  onRiskChange,
  onBlock,
  className = "",
}) => {
  const [showDetails, setShowDetails] = useState(false);
  const lastTransactionRef = useRef<string>("");
  const debounceTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const {
    isAnalyzing,
    currentRisk,
    error,
    isAIEngineHealthy,
    analyzeTransactionRisk,
    shouldBlockTransaction,
    getRiskScoreWithConfidence,
    getRiskFactors,
  } = useAIRiskAnalysis({
    userId,
    onRiskChange,
    onError: (error) => {
      console.error("AI Risk Analysis Error:", error);
    },
  });

  // Memoize transaction fingerprint to prevent unnecessary re-renders
  const transactionFingerprint = useMemo(() => {
    if (!transaction) return null;
    return JSON.stringify({
      amount: transaction.amount,
      sourceChain: transaction.sourceChain,
      destinationChain: transaction.destinationChain,
      sourceToken: transaction.sourceToken,
      destinationToken: transaction.destinationToken,
      userAddress: transaction.userAddress,
    });
  }, [transaction]);

  // Auto-analyze when transaction fingerprint changes (with debouncing)
  useEffect(() => {
    if (
      !autoAnalyze ||
      !transaction ||
      !isAIEngineHealthy ||
      !transactionFingerprint
    ) {
      return;
    }

    if (lastTransactionRef.current === transactionFingerprint) {
      return;
    }

    if (debounceTimeoutRef.current) {
      clearTimeout(debounceTimeoutRef.current);
    }

    debounceTimeoutRef.current = setTimeout(() => {
      lastTransactionRef.current = transactionFingerprint;
      analyzeTransactionRisk(transaction);
    }, 1000);

    return () => {
      if (debounceTimeoutRef.current) {
        clearTimeout(debounceTimeoutRef.current);
      }
    };
  }, [
    transactionFingerprint,
    autoAnalyze,
    isAIEngineHealthy,
    analyzeTransactionRisk,
    transaction,
  ]);

  useEffect(() => {
    if (shouldBlockTransaction() && currentRisk) {
      const reason = `High risk transaction (${
        currentRisk.risk_level
      }): ${currentRisk.reasons.join(", ")}`;
      onBlock?.(reason);
    }
  }, [currentRisk, shouldBlockTransaction, onBlock]);

  const riskScore = getRiskScoreWithConfidence();
  const riskFactors = getRiskFactors();

  const getRiskLevelColor = (level: string) => {
    switch (level) {
      case RISK_ANALYSIS.LEVELS.LOW:
        return "#4CAF50";
      case RISK_ANALYSIS.LEVELS.MEDIUM:
        return "#FF9800";
      case RISK_ANALYSIS.LEVELS.HIGH:
        return "#F44336";
      default:
        return "#9E9E9E";
    }
  };

  const getRiskLevelIcon = (level: string) => {
    switch (level) {
      case RISK_ANALYSIS.LEVELS.LOW:
        return "🟢";
      case RISK_ANALYSIS.LEVELS.MEDIUM:
        return "🟡";
      case RISK_ANALYSIS.LEVELS.HIGH:
        return "🔴";
      default:
        return "⚪";
    }
  };

  const formatTimestamp = (timestamp: string) => {
    return new Date(timestamp).toLocaleTimeString();
  };

  if (!isAIEngineHealthy) {
    return (
      <div
        className={cn(styles.aiRiskDisplay, styles.offline, className.trim())}
        data-testid="ai-risk-display-offline"
      >
        <div className={styles.header}>
          <span className={styles.icon}>⚠️</span>
          <span className={styles.title}>AI Risk Engine Offline</span>
        </div>
        <div className={styles.message}>{ERROR_MESSAGES.AI_ENGINE_OFFLINE}</div>
      </div>
    );
  }

  if (error) {
    return (
      <div
        className={cn(styles.aiRiskDisplay, styles.error, className.trim())}
        data-testid="ai-risk-display-error"
      >
        <div className={styles.header}>
          <span className={styles.icon}>❌</span>
          <span className={styles.title}>Risk Analysis Error</span>
        </div>
        <div className={styles.message}>{error.message}</div>
        {transaction && (
          <button
            className={styles.retry}
            onClick={() => analyzeTransactionRisk(transaction)}
            data-testid="ai-risk-retry-button"
          >
            Retry Analysis
          </button>
        )}
      </div>
    );
  }

  if (isAnalyzing) {
    return (
      <div
        className={cn(styles.aiRiskDisplay, styles.loading, className.trim())}
        data-testid="ai-risk-display-loading"
      >
        <div className={styles.header}>
          <span className={cn(styles.icon, styles.spinning)}>🤖</span>
          <span className={styles.title}>{STATUS_MESSAGES.ANALYZING}</span>
        </div>
        <div className={styles.progress}>
          <div className={styles.progressBar}></div>
        </div>
      </div>
    );
  }

  if (!currentRisk) {
    return (
      <div
        className={cn(styles.aiRiskDisplay, styles.ready, className.trim())}
        data-testid="ai-risk-display-ready"
      >
        <div className={styles.header}>
          <span className={styles.icon}>🤖</span>
          <span className={styles.title}>AI Risk Engine Ready</span>
        </div>
        {transaction && (
          <button
            className={styles.analyze}
            onClick={() => analyzeTransactionRisk(transaction)}
            data-testid="ai-risk-analyze-button"
          >
            Analyze Transaction Risk
          </button>
        )}
      </div>
    );
  }

  return (
    <div
      className={cn(styles.aiRiskDisplay, styles.active, className.trim())}
      data-testid="ai-risk-display"
    >
      <div className={styles.header}>
        <span className={styles.icon}>
          {getRiskLevelIcon(currentRisk.risk_level)}
        </span>
        <div className={styles.titleGroup}>
          <span className={styles.title}>AI Risk Analysis</span>
          <span className={styles.timestamp}>
            {formatTimestamp(currentRisk.analysis_timestamp)}
          </span>
        </div>
        <button
          className={styles.toggle}
          onClick={() => setShowDetails(!showDetails)}
          data-testid="ai-risk-toggle-details"
        >
          {showDetails ? "▼" : "▶"}
        </button>
      </div>

      <div className={styles.score}>
        <div
          className={styles.scoreBar}
          style={{ "--risk-score": riskScore?.score || 0 } as CSSProperties}
          data-testid="ai-risk-score-bar"
        >
          <div
            className={styles.scoreFill}
            style={{
              width: `${(riskScore?.score || 0) * 100}%`,
              backgroundColor: getRiskLevelColor(currentRisk.risk_level),
            }}
          />
        </div>
        <div className={styles.scoreInfo}>
          <span className={styles.scoreValue} data-testid="ai-risk-score-value">
            {Math.round((riskScore?.score || 0) * 100)}%
          </span>
          <span
            className={styles.scoreLevel}
            style={{ color: getRiskLevelColor(currentRisk.risk_level) }}
            data-testid="ai-risk-score-level"
          >
            {currentRisk.risk_level.toUpperCase()}
          </span>
        </div>
      </div>

      <div className={styles.status}>
        <span
          className={cn(styles.approval, {
            [styles.approved]: currentRisk.approved,
            [styles.blocked]: !currentRisk.approved,
          })}
          data-testid={
            currentRisk.approved ? "ai-risk-approval-status" : "risk-warning"
          }
        >
          {currentRisk.approved ? "✅ Approved" : "❌ Blocked"}
        </span>
        {riskScore?.confidence && (
          <span className={styles.confidence} data-testid="ai-risk-confidence">
            {Math.round(riskScore.confidence * 100)}% confidence
          </span>
        )}
      </div>

      {showDetails && (
        <div className={styles.details} data-testid="ai-risk-display-details">
          {riskFactors.factors.length > 0 && (
            <div className={styles.factors}>
              <h4>Risk Factors:</h4>
              <ul>
                {riskFactors.factors.map((factor, index) => (
                  <li key={index} data-testid={`ai-risk-factor-${index}`}>
                    {factor}
                  </li>
                ))}
              </ul>
            </div>
          )}

          {riskFactors.recommendations.length > 0 && !true && (
            <div className={styles.recommendations}>
              <h4>Recommendations:</h4>
              <ul>
                {riskFactors.recommendations.map((rec, index) => (
                  <li
                    key={index}
                    data-testid={`ai-risk-recommendation-${index}`}
                  >
                    {rec}
                  </li>
                ))}
              </ul>
            </div>
          )}

          {riskFactors.isAnomaly && (
            <div className={styles.anomaly} data-testid="ai-risk-anomaly">
              ⚠️ Anomalous transaction pattern detected
            </div>
          )}
        </div>
      )}

      {transaction && (
        <div className={styles.actions}>
          <button
            className={styles.refresh}
            onClick={() => analyzeTransactionRisk(transaction)}
            data-testid="ai-risk-refresh-button"
          >
            Refresh Analysis
          </button>
        </div>
      )}
    </div>
  );
};

export default AIRiskDisplay;
