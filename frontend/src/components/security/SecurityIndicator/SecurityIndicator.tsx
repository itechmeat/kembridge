import React from 'react';
import { SecurityIndicatorProps, SecurityLevel } from '../../../types';
import './SecurityIndicator.scss';

export const SecurityIndicator: React.FC<SecurityIndicatorProps> = ({
  quantumProtection,
  riskScore,
  isOnline,
  compact = false,
  className = '',
  quantumKeyId,
  encryptionScheme,
  lastKeyRotation,
  transactionCount,
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
        return quantumProtection ? 'Quantum Protected' : 'Protected';
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

  const getQuantumSchemeDisplay = (): string => {
    if (!quantumProtection || !encryptionScheme) return 'Disabled';
    return encryptionScheme;
  };

  const getKeyRotationStatus = (): string => {
    if (!lastKeyRotation) return 'Never';
    const now = new Date();
    const rotation = new Date(lastKeyRotation);
    const diffHours = Math.floor((now.getTime() - rotation.getTime()) / (1000 * 60 * 60));
    
    if (diffHours < 1) return 'Recent';
    if (diffHours < 24) return `${diffHours}h ago`;
    const diffDays = Math.floor(diffHours / 24);
    return `${diffDays}d ago`;
  };

  const formatTransactionCount = (): string => {
    if (!transactionCount) return '0';
    if (transactionCount < 1000) return transactionCount.toString();
    if (transactionCount < 1000000) return `${Math.floor(transactionCount / 1000)}K`;
    return `${Math.floor(transactionCount / 1000000)}M`;
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
      <div className={`security-indicator security-indicator--compact security-indicator--${securityLevel} ${className}`} data-testid="security-indicator">
        <span className="security-indicator__icon" data-testid="security-icon">{getStatusIcon()}</span>
        <span className="security-indicator__score" data-testid="risk-score">{getRiskScoreDisplay()}</span>
      </div>
    );
  }

  const getSecurityClasses = (): string => {
    const baseClasses = `security-indicator security-indicator--${securityLevel}`;
    const quantumClass = quantumProtection ? 'quantum-protected' : '';
    return `${baseClasses} ${quantumClass} ${className}`.trim();
  };

  return (
    <div className={getSecurityClasses()} data-testid="security-indicator">
      <div className="security-indicator__header" data-testid="security-header">
        <span className="security-indicator__icon" data-testid="security-icon">{getStatusIcon()}</span>
        <span className="security-indicator__title">Security Status</span>
      </div>
      
      <div className="security-indicator__content" data-testid="security-content">
        <div className="security-indicator__status">
          <span className="security-indicator__status-text" data-testid="security-level">{getStatusText()}</span>
          <span className="security-indicator__connection" data-testid="connection-status">
            {isOnline ? '🟢 Online' : '🔴 Offline'}
          </span>
        </div>
        
        <div className="security-indicator__details" data-testid="security-details">
          <div className="security-indicator__detail">
            <span className="security-indicator__detail-label">Quantum Protection:</span>
            <span className={`security-indicator__detail-value ${quantumProtection ? 'enabled' : 'disabled'}`} data-testid="quantum-protection-status">
              {getQuantumSchemeDisplay()}
            </span>
          </div>
          
          <div className="security-indicator__detail">
            <span className="security-indicator__detail-label">Risk Score:</span>
            <span className={`security-indicator__detail-value risk-score--${securityLevel}`} data-testid="risk-score">
              {getRiskScoreDisplay()}
            </span>
          </div>

          {quantumProtection && (
            <>
              <div className="security-indicator__detail">
                <span className="security-indicator__detail-label">Key ID:</span>
                <span className="security-indicator__detail-value key-id" data-testid="quantum-key-id">
                  {quantumKeyId ? `${quantumKeyId.slice(0, 8)}...` : 'N/A'}
                </span>
              </div>
              
              <div className="security-indicator__detail">
                <span className="security-indicator__detail-label">Last Rotation:</span>
                <span className="security-indicator__detail-value" data-testid="key-rotation-status">
                  {getKeyRotationStatus()}
                </span>
              </div>
              
              <div className="security-indicator__detail">
                <span className="security-indicator__detail-label">Protected Txs:</span>
                <span className="security-indicator__detail-value" data-testid="protected-count">
                  {formatTransactionCount()}
                </span>
              </div>
            </>
          )}
        </div>
      </div>
    </div>
  );
};

export default SecurityIndicator;