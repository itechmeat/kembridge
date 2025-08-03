import { FC, useEffect } from "react";
import cn from "classnames";
import {
  RiskAnalysisDisplayProps,
  RiskScore,
  RiskFactor,
} from "../../../types/security";
import styles from "./RiskAnalysisDisplay.module.scss";

export const RiskAnalysisDisplay: FC<RiskAnalysisDisplayProps> = ({
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
      <div className={cn(styles.riskAnalysisDisplay, styles.loading)}>
        <div className={styles.loadingContent}>
          <div className={styles.spinner}></div>
          <span>Analyzing risk...</span>
        </div>
      </div>
    );
  }

  const { riskScore, factors, recommendations, blacklistStatus, analysis } =
    riskData;

  const getRiskLevelClass = (score: RiskScore) => {
    return styles[
      `riskLevel${score.level.charAt(0).toUpperCase() + score.level.slice(1)}`
    ];
  };

  const getRiskScoreDisplay = (score: number): string => {
    return `${Math.round(score * 100)}%`;
  };

  const getFactorIcon = (factor: RiskFactor): string => {
    switch (factor.impact) {
      case "positive":
        return "✅";
      case "negative":
        return "⚠️";
      default:
        return "ℹ️";
    }
  };

  const formatConfidence = (confidence: number): string => {
    return `${Math.round(confidence * 100)}%`;
  };

  return (
    <div
      className={cn(styles.riskAnalysisDisplay, getRiskLevelClass(riskScore))}
      data-testid="risk-analysis-display"
    >
      <div className={styles.header}>
        <div className={styles.scoreSection}>
          <div className={styles.scoreMain}>
            <span className={styles.scoreValue}>
              {getRiskScoreDisplay(riskScore.value)}
            </span>
            <span className={styles.scoreLabel}>Risk Score</span>
          </div>
          <div className={styles.scoreMeta}>
            <span
              className={cn(
                styles.level,
                styles[
                  `riskLevel${
                    riskScore.level.charAt(0).toUpperCase() +
                    riskScore.level.slice(1)
                  }`
                ]
              )}
            >
              {riskScore.level.toUpperCase()}
            </span>
            <span className={styles.confidence}>
              {formatConfidence(riskScore.confidence)} confidence
            </span>
          </div>
        </div>

        {realTime && (
          <div className={styles.realtime}>
            <div className={styles.realtimeIndicator}></div>
            <span>Live</span>
          </div>
        )}
      </div>

      {blacklistStatus.isBlacklisted && (
        <div className={styles.blacklistWarning}>
          <div className={styles.blacklistContent}>
            <span className={styles.blacklistIcon}>🚨</span>
            <div className={styles.blacklistInfo}>
              <span className={styles.blacklistTitle}>Address Blacklisted</span>
              {blacklistStatus.reason && (
                <span className={styles.blacklistReason}>
                  {blacklistStatus.reason}
                </span>
              )}
            </div>
          </div>
        </div>
      )}

      {showDetails && factors.length > 0 && (
        <div className={styles.factors}>
          <h4 className={styles.sectionTitle}>Risk Factors</h4>
          <div className={styles.factorsList}>
            {factors.map((factor, index) => (
              <div
                key={index}
                className={cn(
                  styles.factor,
                  styles[
                    `factor${
                      factor.impact.charAt(0).toUpperCase() +
                      factor.impact.slice(1)
                    }`
                  ]
                )}
              >
                <div className={styles.factorHeader}>
                  <span className={styles.factorIcon}>
                    {getFactorIcon(factor)}
                  </span>
                  <span className={styles.factorType}>
                    {factor.type.replace("_", " ")}
                  </span>
                  <span className={styles.factorWeight}>
                    {Math.round(factor.weight * 100)}%
                  </span>
                </div>
                <p className={styles.factorDescription}>{factor.description}</p>
              </div>
            ))}
          </div>
        </div>
      )}

      {showDetails && recommendations.length > 0 && (
        <div className={styles.recommendations}>
          <h4 className={styles.sectionTitle}>Recommendations</h4>
          <ul className={styles.recommendationsList}>
            {recommendations.map((recommendation, index) => (
              <li key={index} className={styles.recommendationItem}>
                <span className={styles.recommendationIcon}>💡</span>
                <span className={styles.recommendationText}>
                  {recommendation}
                </span>
              </li>
            ))}
          </ul>
        </div>
      )}

      {showDetails && (
        <div className={styles.analysis}>
          <h4 className={styles.sectionTitle}>Transaction Analysis</h4>
          <div className={styles.analysisGrid}>
            <div className={styles.analysisItem}>
              <span className={styles.analysisLabel}>Amount</span>
              <span className={styles.analysisValue}>
                ${analysis.transactionAmount.toLocaleString()}
              </span>
            </div>
            <div className={styles.analysisItem}>
              <span className={styles.analysisLabel}>User Transactions</span>
              <span className={styles.analysisValue}>
                {analysis.userHistory.totalTransactions}
              </span>
            </div>
            <div className={styles.analysisItem}>
              <span className={styles.analysisLabel}>Average Amount</span>
              <span className={styles.analysisValue}>
                ${Math.round(analysis.userHistory.avgAmount).toLocaleString()}
              </span>
            </div>
            <div className={styles.analysisItem}>
              <span className={styles.analysisLabel}>From Address Risk</span>
              <span className={styles.analysisValue}>
                {getRiskScoreDisplay(analysis.addressRisk.from)}
              </span>
            </div>
            <div className={styles.analysisItem}>
              <span className={styles.analysisLabel}>To Address Risk</span>
              <span className={styles.analysisValue}>
                {getRiskScoreDisplay(analysis.addressRisk.to)}
              </span>
            </div>
          </div>
        </div>
      )}

      <div className={styles.footer}>
        <span className={styles.timestamp}>
          Analyzed at {new Date(riskScore.timestamp).toLocaleTimeString()}
        </span>
      </div>
    </div>
  );
};

export default RiskAnalysisDisplay;
