import React from 'react';
import { SecurityIndicatorProps, SecurityLevel } from '../../../types';
import './SecurityIndicator.scss';

export const SecurityIndicator: React.FC<SecurityIndicatorProps> = ({
  quantumProtection,
  riskScore,
  isOnline,
  compact = false,
  className = '',
}) => {
  // Determine security level based on quantum protection and risk score
  const getSecurityLevel = (): SecurityLevel => {
    if (!isOnline) return SecurityLevel.OFFLINE;
    if (!quantumProtection) return SecurityLevel.DANGER;
    if (riskScore > 0.7) return SecurityLevel.DANGER;
    if (riskScore > 0.3) return SecurityLevel.WARNING;
    return SecurityLevel.SECURE;
  };

  const securityLevel = getSecurityLevel();

  const getStatusText = (): string => {
    switch (securityLevel) {
      case SecurityLevel.SECURE:
        return 'Quantum Protected';
      case SecurityLevel.WARNING:
        return 'Medium Risk';
      case SecurityLevel.DANGER:
        return quantumProtection ? 'High Risk' : 'Quantum Offline';
      case SecurityLevel.OFFLINE:
        return 'System Offline';
      default:
        return 'Unknown';
    }
  };

  const getStatusIcon = (): string => {
    switch (securityLevel) {
      case SecurityLevel.SECURE:
        return '🔒';
      case SecurityLevel.WARNING:
        return '⚠️';
      case SecurityLevel.DANGER:
        return '🚨';
      case SecurityLevel.OFFLINE:
        return '📴';
      default:
        return '❓';
    }
  };

  const getRiskScoreDisplay = (): string => {
    return `${Math.round(riskScore * 100)}%`;
  };

  if (compact) {
    return (
      <div className={`security-indicator security-indicator--compact security-indicator--${securityLevel} ${className}`}>
        <span className="security-indicator__icon">{getStatusIcon()}</span>
        <span className="security-indicator__score">{getRiskScoreDisplay()}</span>
      </div>
    );
  }

  return (
    <div className={`security-indicator security-indicator--${securityLevel} ${className}`}>
      <div className="security-indicator__header">
        <span className="security-indicator__icon">{getStatusIcon()}</span>
        <span className="security-indicator__title">Security Status</span>
      </div>
      
      <div className="security-indicator__content">
        <div className="security-indicator__status">
          <span className="security-indicator__status-text">{getStatusText()}</span>
          <span className="security-indicator__connection">
            {isOnline ? '🟢 Online' : '🔴 Offline'}
          </span>
        </div>
        
        <div className="security-indicator__details">
          <div className="security-indicator__detail">
            <span className="security-indicator__detail-label">Quantum Protection:</span>
            <span className={`security-indicator__detail-value ${quantumProtection ? 'enabled' : 'disabled'}`}>
              {quantumProtection ? 'ML-KEM-1024' : 'Disabled'}
            </span>
          </div>
          
          <div className="security-indicator__detail">
            <span className="security-indicator__detail-label">Risk Score:</span>
            <span className={`security-indicator__detail-value risk-score--${securityLevel}`}>
              {getRiskScoreDisplay()}
            </span>
          </div>
        </div>
      </div>
    </div>
  );
};

export default SecurityIndicator;