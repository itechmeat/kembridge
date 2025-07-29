import React, { useEffect, useState } from 'react';
import { RiskScoreVisualizationProps } from '../../../types/security';
import './RiskScoreVisualization.scss';

export const RiskScoreVisualization: React.FC<RiskScoreVisualizationProps> = ({
  score,
  animated = true,
  size = 'medium',
  showLabel = true,
  showTrend = false,
  previousScore,
}) => {
  const [displayScore, setDisplayScore] = useState(0);
  const [isAnimating, setIsAnimating] = useState(false);

  // Animate score changes
  useEffect(() => {
    if (animated) {
      setIsAnimating(true);
      const targetScore = Math.max(0, Math.min(1, score));
      const startScore = displayScore;
      const duration = 1000; // 1 second animation
      const steps = 60; // 60fps
      const increment = (targetScore - startScore) / steps;
      
      let currentStep = 0;
      const timer = setInterval(() => {
        currentStep++;
        const newScore = startScore + (increment * currentStep);
        if (currentStep >= steps) {
          setDisplayScore(targetScore);
          setIsAnimating(false);
          clearInterval(timer);
        } else {
          setDisplayScore(newScore);
        }
      }, duration / steps);
      
      return () => clearInterval(timer);
    } else {
      setDisplayScore(score);
    }
  }, [score, animated, displayScore]);

  const getRiskLevel = (riskScore: number): 'low' | 'medium' | 'high' => {
    if (riskScore <= 0.3) return 'low';
    if (riskScore <= 0.7) return 'medium';
    return 'high';
  };

  // Removed unused function - using inline calculation instead

  const getTrendIcon = (): string | null => {
    if (!showTrend || previousScore === undefined) return null;
    
    const diff = score - previousScore;
    if (Math.abs(diff) < 0.01) return '→'; // No significant change
    if (diff > 0) return '↗️'; // Increasing risk
    return '↘️'; // Decreasing risk
  };

  const getTrendClass = (): string => {
    if (!showTrend || previousScore === undefined) return '';
    
    const diff = score - previousScore;
    if (Math.abs(diff) < 0.01) return 'trend--neutral';
    if (diff > 0) return 'trend--up';
    return 'trend--down';
  };

  const riskLevel = getRiskLevel(displayScore);
  const percentage = Math.round(displayScore * 100);
  const circumference = 2 * Math.PI * 45; // radius = 45
  const strokeDashoffset = circumference - (displayScore * circumference);

  return (
    <div className={`risk-score-viz risk-score-viz--${size} risk-score-viz--${riskLevel} ${isAnimating ? 'animating' : ''}`}>
      {/* Circular Progress */}
      <div className="risk-score-viz__circle">
        <svg className="risk-score-viz__svg" viewBox="0 0 100 100">
          {/* Background circle */}
          <circle
            className="risk-score-viz__background"
            cx="50"
            cy="50"
            r="45"
            fill="none"
            stroke="currentColor"
            strokeWidth="8"
          />
          {/* Progress circle */}
          <circle
            className="risk-score-viz__progress"
            cx="50"
            cy="50"
            r="45"
            fill="none"
            stroke="currentColor"
            strokeWidth="8"
            strokeLinecap="round"
            strokeDasharray={circumference}
            strokeDashoffset={strokeDashoffset}
            transform="rotate(-90 50 50)"
          />
        </svg>
        
        {/* Center content */}
        <div className="risk-score-viz__center">
          <span className="risk-score-viz__percentage">{percentage}%</span>
          {showTrend && getTrendIcon() && (
            <span className={`risk-score-viz__trend ${getTrendClass()}`}>
              {getTrendIcon()}
            </span>
          )}
        </div>
      </div>
      
      {/* Label */}
      {showLabel && (
        <div className="risk-score-viz__label">
          <span className="risk-score-viz__level">{riskLevel.toUpperCase()}</span>
          <span className="risk-score-viz__description">Risk Level</span>
        </div>
      )}
    </div>
  );
};

export default RiskScoreVisualization;