import React, { useEffect } from 'react';
import { RiskAnalysisDisplayProps, RiskScore, RiskFactor } from '../../../types/security';
import './RiskAnalysisDisplay.scss';

export const RiskAnalysisDisplay: React.FC<RiskAnalysisDisplayProps> = ({
  riskData,
  realTime = false,
  showDetails = true,
  onUpdate,
}) => {
  useEffect(() => {
    if (onUpdate && riskData) {
      onUpdate(riskData);
    }
  }, [riskData, onUpdate]);

  if (!riskData) {
    return (
      <div className="risk-analysis-display risk-analysis-display--loading">
        <div className="risk-analysis-display__loading">
          <div className="risk-spinner"></div>
          <span>Analyzing risk...</span>
        </div>
      </div>
    );
  }

  const { riskScore, factors, recommendations, blacklistStatus, analysis } = riskData;

  const getRiskLevelClass = (score: RiskScore) => {
    return `risk-level--${score.level}`;
  };

  const getRiskScoreDisplay = (score: number): string => {
    return `${Math.round(score * 100)}%`;
  };

  const getFactorIcon = (factor: RiskFactor): string => {
    switch (factor.impact) {
      case 'positive':
        return '✅';
      case 'negative':
        return '⚠️';
      default:
        return 'ℹ️';
    }
  };

  const formatConfidence = (confidence: number): string => {
    return `${Math.round(confidence * 100)}%`;
  };

  return (
    <div className={`risk-analysis-display ${getRiskLevelClass(riskScore)}`}>
      {/* Header with Risk Score */}
      <div className="risk-analysis-display__header">
        <div className="risk-analysis-display__score-section">
          <div className="risk-analysis-display__score-main">
            <span className="risk-analysis-display__score-value">
              {getRiskScoreDisplay(riskScore.value)}
            </span>
            <span className="risk-analysis-display__score-label">Risk Score</span>
          </div>
          <div className="risk-analysis-display__score-meta">
            <span className={`risk-analysis-display__level risk-level--${riskScore.level}`}>
              {riskScore.level.toUpperCase()}
            </span>
            <span className="risk-analysis-display__confidence">
              {formatConfidence(riskScore.confidence)} confidence
            </span>
          </div>
        </div>
        
        {realTime && (
          <div className="risk-analysis-display__realtime">
            <div className="realtime-indicator"></div>
            <span>Live</span>
          </div>
        )}
      </div>

      {/* Blacklist Warning */}
      {blacklistStatus.isBlacklisted && (
        <div className="risk-analysis-display__blacklist-warning">
          <div className="blacklist-warning">
            <span className="blacklist-warning__icon">🚨</span>
            <div className="blacklist-warning__content">
              <span className="blacklist-warning__title">Address Blacklisted</span>
              {blacklistStatus.reason && (
                <span className="blacklist-warning__reason">{blacklistStatus.reason}</span>
              )}
            </div>
          </div>
        </div>
      )}

      {/* Risk Factors */}
      {showDetails && factors.length > 0 && (
        <div className="risk-analysis-display__factors">
          <h4 className="risk-analysis-display__section-title">Risk Factors</h4>
          <div className="risk-factors-list">
            {factors.map((factor, index) => (
              <div 
                key={index} 
                className={`risk-factor risk-factor--${factor.impact}`}
              >
                <div className="risk-factor__header">
                  <span className="risk-factor__icon">{getFactorIcon(factor)}</span>
                  <span className="risk-factor__type">{factor.type.replace('_', ' ')}</span>
                  <span className="risk-factor__weight">
                    {Math.round(factor.weight * 100)}%
                  </span>
                </div>
                <p className="risk-factor__description">{factor.description}</p>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Recommendations */}
      {showDetails && recommendations.length > 0 && (
        <div className="risk-analysis-display__recommendations">
          <h4 className="risk-analysis-display__section-title">Recommendations</h4>
          <ul className="recommendations-list">
            {recommendations.map((recommendation, index) => (
              <li key={index} className="recommendation-item">
                <span className="recommendation-item__icon">💡</span>
                <span className="recommendation-item__text">{recommendation}</span>
              </li>
            ))}
          </ul>
        </div>
      )}

      {/* Analysis Details */}
      {showDetails && (
        <div className="risk-analysis-display__analysis">
          <h4 className="risk-analysis-display__section-title">Transaction Analysis</h4>
          <div className="analysis-grid">
            <div className="analysis-item">
              <span className="analysis-item__label">Amount</span>
              <span className="analysis-item__value">${analysis.transactionAmount.toLocaleString()}</span>
            </div>
            <div className="analysis-item">
              <span className="analysis-item__label">User Transactions</span>
              <span className="analysis-item__value">{analysis.userHistory.totalTransactions}</span>
            </div>
            <div className="analysis-item">
              <span className="analysis-item__label">Average Amount</span>
              <span className="analysis-item__value">${Math.round(analysis.userHistory.avgAmount).toLocaleString()}</span>
            </div>
            <div className="analysis-item">
              <span className="analysis-item__label">From Address Risk</span>
              <span className="analysis-item__value">{getRiskScoreDisplay(analysis.addressRisk.from)}</span>
            </div>
            <div className="analysis-item">
              <span className="analysis-item__label">To Address Risk</span>
              <span className="analysis-item__value">{getRiskScoreDisplay(analysis.addressRisk.to)}</span>
            </div>
          </div>
        </div>
      )}

      {/* Timestamp */}
      <div className="risk-analysis-display__footer">
        <span className="risk-analysis-display__timestamp">
          Analyzed at {new Date(riskScore.timestamp).toLocaleTimeString()}
        </span>
      </div>
    </div>
  );
};

export default RiskAnalysisDisplay;