/**
 * AI Risk Display Component
 * Real-time display of AI-powered risk analysis
 */

import React, { useEffect, useState, useRef, useMemo } from 'react';
import { useAIRiskAnalysis } from '../../../hooks/useAIRiskAnalysis';
import type { AIRiskAnalysisResponse } from '../../../services/ai/aiRiskService';
import { RISK_ANALYSIS, ERROR_MESSAGES, STATUS_MESSAGES } from '../../../constants/services';
import './AIRiskDisplay.scss';

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

export const AIRiskDisplay: React.FC<AIRiskDisplayProps> = ({
  userId,
  transaction,
  autoAnalyze = false,
  onRiskChange,
  onBlock,
  className = '',
}) => {
  const [showDetails, setShowDetails] = useState(false);
  const lastTransactionRef = useRef<string>('');
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
      console.error('AI Risk Analysis Error:', error);
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
    if (!autoAnalyze || !transaction || !isAIEngineHealthy || !transactionFingerprint) {
      return;
    }

    // Skip if transaction hasn't actually changed
    if (lastTransactionRef.current === transactionFingerprint) {
      return;
    }

    // Clear any existing timeout
    if (debounceTimeoutRef.current) {
      clearTimeout(debounceTimeoutRef.current);
    }

    // Debounce the analysis call by 1000ms to prevent spam
    debounceTimeoutRef.current = setTimeout(() => {
      lastTransactionRef.current = transactionFingerprint;
      analyzeTransactionRisk(transaction);
    }, 1000);

    // Cleanup timeout on unmount or dependency change
    return () => {
      if (debounceTimeoutRef.current) {
        clearTimeout(debounceTimeoutRef.current);
      }
    };
  }, [transactionFingerprint, autoAnalyze, isAIEngineHealthy, analyzeTransactionRisk, transaction]);

  // Check if transaction should be blocked
  useEffect(() => {
    if (shouldBlockTransaction() && currentRisk) {
      const reason = `High risk transaction (${currentRisk.risk_level}): ${currentRisk.reasons.join(', ')}`;
      onBlock?.(reason);
    }
  }, [currentRisk, shouldBlockTransaction, onBlock]);

  const riskScore = getRiskScoreWithConfidence();
  const riskFactors = getRiskFactors();

  const getRiskLevelColor = (level: string) => {
    switch (level) {
      case RISK_ANALYSIS.LEVELS.LOW: return '#4CAF50';
      case RISK_ANALYSIS.LEVELS.MEDIUM: return '#FF9800';
      case RISK_ANALYSIS.LEVELS.HIGH: return '#F44336';
      default: return '#9E9E9E';
    }
  };

  const getRiskLevelIcon = (level: string) => {
    switch (level) {
      case RISK_ANALYSIS.LEVELS.LOW: return '🟢';
      case RISK_ANALYSIS.LEVELS.MEDIUM: return '🟡';
      case RISK_ANALYSIS.LEVELS.HIGH: return '🔴';
      default: return '⚪';
    }
  };

  const formatTimestamp = (timestamp: string) => {
    return new Date(timestamp).toLocaleTimeString();
  };

  if (!isAIEngineHealthy) {
    return (
      <div className={`ai-risk-display ai-risk-display--offline ${className}`} data-testid="ai-risk-display-offline">
        <div className="ai-risk-display__header">
          <span className="ai-risk-display__icon">⚠️</span>
          <span className="ai-risk-display__title">AI Risk Engine Offline</span>
        </div>
        <div className="ai-risk-display__message">
          {ERROR_MESSAGES.AI_ENGINE_OFFLINE}
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className={`ai-risk-display ai-risk-display--error ${className}`} data-testid="ai-risk-display-error">
        <div className="ai-risk-display__header">
          <span className="ai-risk-display__icon">❌</span>
          <span className="ai-risk-display__title">Risk Analysis Error</span>
        </div>
        <div className="ai-risk-display__message">
          {error.message}
        </div>
        {transaction && (
          <button
            className="ai-risk-display__retry"
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
      <div className={`ai-risk-display ai-risk-display--loading ${className}`} data-testid="ai-risk-display-loading">
        <div className="ai-risk-display__header">
          <span className="ai-risk-display__icon ai-risk-display__icon--spinning">🤖</span>
          <span className="ai-risk-display__title">{STATUS_MESSAGES.ANALYZING}</span>
        </div>
        <div className="ai-risk-display__progress">
          <div className="ai-risk-display__progress-bar"></div>
        </div>
      </div>
    );
  }

  if (!currentRisk) {
    return (
      <div className={`ai-risk-display ai-risk-display--ready ${className}`} data-testid="ai-risk-display-ready">
        <div className="ai-risk-display__header">
          <span className="ai-risk-display__icon">🤖</span>
          <span className="ai-risk-display__title">AI Risk Engine Ready</span>
        </div>
        {transaction && (
          <button
            className="ai-risk-display__analyze"
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
    <div className={`ai-risk-display ai-risk-display--active ${className}`} data-testid="ai-risk-display">
      <div className="ai-risk-display__header">
        <span className="ai-risk-display__icon">
          {getRiskLevelIcon(currentRisk.risk_level)}
        </span>
        <div className="ai-risk-display__title-group">
          <span className="ai-risk-display__title">AI Risk Analysis</span>
          <span className="ai-risk-display__timestamp">
            {formatTimestamp(currentRisk.analysis_timestamp)}
          </span>
        </div>
        <button
          className="ai-risk-display__toggle"
          onClick={() => setShowDetails(!showDetails)}
          data-testid="ai-risk-toggle-details"
        >
          {showDetails ? '▼' : '▶'}
        </button>
      </div>

      <div className="ai-risk-display__score">
        <div 
          className="ai-risk-display__score-bar"
          style={{ '--risk-score': riskScore?.score || 0 } as React.CSSProperties}
          data-testid="ai-risk-score-bar"
        >
          <div 
            className="ai-risk-display__score-fill"
            style={{ 
              width: `${(riskScore?.score || 0) * 100}%`,
              backgroundColor: getRiskLevelColor(currentRisk.risk_level)
            }}
          />
        </div>
        <div className="ai-risk-display__score-info">
          <span className="ai-risk-display__score-value" data-testid="ai-risk-score-value">
            {Math.round((riskScore?.score || 0) * 100)}%
          </span>
          <span 
            className="ai-risk-display__score-level"
            style={{ color: getRiskLevelColor(currentRisk.risk_level) }}
            data-testid="ai-risk-score-level"
          >
            {currentRisk.risk_level.toUpperCase()}
          </span>
        </div>
      </div>

      <div className="ai-risk-display__status">
        <span 
          className={`ai-risk-display__approval ${currentRisk.approved ? 'approved' : 'blocked'}`}
          data-testid="ai-risk-approval-status"
        >
          {currentRisk.approved ? '✅ Approved' : '❌ Blocked'}
        </span>
        {riskScore?.confidence && (
          <span className="ai-risk-display__confidence" data-testid="ai-risk-confidence">
            {Math.round(riskScore.confidence * 100)}% confidence
          </span>
        )}
      </div>

      {showDetails && (
        <div className="ai-risk-display__details" data-testid="ai-risk-display-details">
          {riskFactors.factors.length > 0 && (
            <div className="ai-risk-display__factors">
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

          {riskFactors.recommendations.length > 0 && (
            <div className="ai-risk-display__recommendations">
              <h4>Recommendations:</h4>
              <ul>
                {riskFactors.recommendations.map((rec, index) => (
                  <li key={index} data-testid={`ai-risk-recommendation-${index}`}>
                    {rec}
                  </li>
                ))}
              </ul>
            </div>
          )}

          {riskFactors.isAnomaly && (
            <div className="ai-risk-display__anomaly" data-testid="ai-risk-anomaly">
              ⚠️ Anomalous transaction pattern detected
            </div>
          )}
        </div>
      )}

      {transaction && (
        <div className="ai-risk-display__actions">
          <button
            className="ai-risk-display__refresh"
            onClick={() => analyzeTransactionRisk(transaction)}
            data-testid="ai-risk-refresh-button"
          >
            🔄 Refresh Analysis
          </button>
        </div>
      )}
    </div>
  );
};

export default AIRiskDisplay;