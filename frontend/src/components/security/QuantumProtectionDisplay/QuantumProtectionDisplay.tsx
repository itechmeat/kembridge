import React from 'react';
import './QuantumProtectionDisplay.scss';

export interface QuantumProtectionDisplayProps {
  isActive: boolean;
  encryptionScheme?: string;
  keyId?: string;
  keyStrength?: number;
  lastRotation?: string;
  nextRotation?: string;
  protectedTransactions?: number;
  encryptionSpeed?: number; // operations per second
  className?: string;
}

export const QuantumProtectionDisplay: React.FC<QuantumProtectionDisplayProps> = ({
  isActive,
  encryptionScheme = 'ML-KEM-1024',
  keyId,
  keyStrength = 1024,
  lastRotation,
  nextRotation,
  protectedTransactions = 0,
  encryptionSpeed,
  className = '',
}) => {
  const formatDate = (dateString?: string): string => {
    if (!dateString) return 'Never';
    const date = new Date(dateString);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24));
    
    if (diffDays === 0) return 'Today';
    if (diffDays === 1) return 'Yesterday';
    if (diffDays < 7) return `${diffDays} days ago`;
    if (diffDays < 30) return `${Math.floor(diffDays / 7)} weeks ago`;
    return `${Math.floor(diffDays / 30)} months ago`;
  };

  const formatNextRotation = (dateString?: string): string => {
    if (!dateString) return 'Not scheduled';
    const date = new Date(dateString);
    const now = new Date();
    const diffMs = date.getTime() - now.getTime();
    const diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24));
    
    if (diffDays < 0) return 'Overdue';
    if (diffDays === 0) return 'Today';
    if (diffDays === 1) return 'Tomorrow';
    if (diffDays < 7) return `In ${diffDays} days`;
    if (diffDays < 30) return `In ${Math.floor(diffDays / 7)} weeks`;
    return `In ${Math.floor(diffDays / 30)} months`;
  };

  const formatTransactionCount = (count: number): string => {
    if (count < 1000) return count.toString();
    if (count < 1000000) return `${(count / 1000).toFixed(1)}K`;
    return `${(count / 1000000).toFixed(1)}M`;
  };

  const formatSpeed = (opsPerSec?: number): string => {
    if (!opsPerSec) return 'N/A';
    if (opsPerSec < 1000) return `${opsPerSec} ops/s`;
    return `${Math.round(opsPerSec / 1000)}K ops/s`;
  };

  const getKeyStrengthColor = (strength: number): string => {
    if (strength >= 1024) return 'high';
    if (strength >= 512) return 'medium';
    return 'low';
  };

  const getRotationStatus = (nextRotation?: string): 'healthy' | 'warning' | 'overdue' => {
    if (!nextRotation) return 'healthy';
    const date = new Date(nextRotation);
    const now = new Date();
    const diffMs = date.getTime() - now.getTime();
    const diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24));
    
    if (diffDays < 0) return 'overdue';
    if (diffDays <= 7) return 'warning';
    return 'healthy';
  };

  if (!isActive) {
    return (
      <div className={`quantum-protection quantum-protection--disabled ${className}`} data-testid="quantum-protection-display">
        <div className="quantum-protection__header" data-testid="quantum-header">
          <span className="quantum-protection__icon" data-testid="quantum-icon">🔓</span>
          <span className="quantum-protection__title">Quantum Protection</span>
          <span className="quantum-protection__status quantum-protection__status--disabled" data-testid="quantum-status">
            Disabled
          </span>
        </div>
        <div className="quantum-protection__message">
          Post-quantum cryptography is not active. Your transactions may be vulnerable to quantum attacks.
        </div>
      </div>
    );
  }

  const rotationStatus = getRotationStatus(nextRotation);

  return (
    <div className={`quantum-protection quantum-protection--active ${className}`} data-testid="quantum-protection-display">
      <div className="quantum-protection__header" data-testid="quantum-header">
        <span className="quantum-protection__icon" data-testid="quantum-icon">🔒</span>
        <span className="quantum-protection__title">Quantum Protection</span>
        <span className="quantum-protection__status quantum-protection__status--active" data-testid="quantum-status">
          Active
        </span>
      </div>

      <div className="quantum-protection__content">
        <div className="quantum-protection__grid">
          <div className="quantum-protection__card" data-testid="quantum-card-encryption">
            <div className="quantum-protection__card-header">
              <span className="quantum-protection__card-icon">⚡</span>
              <span className="quantum-protection__card-title">Encryption Scheme</span>
            </div>
            <div className="quantum-protection__card-value" data-testid="encryption-scheme">
              {encryptionScheme}
            </div>
            <div className="quantum-protection__card-subtitle">
              {keyStrength}-bit security level
            </div>
          </div>

          <div className="quantum-protection__card" data-testid="quantum-card-key">
            <div className="quantum-protection__card-header">
              <span className="quantum-protection__card-icon">🔐</span>
              <span className="quantum-protection__card-title">Key Information</span>
            </div>
            <div className="quantum-protection__card-value key-id" data-testid="key-information">
              {keyId ? `${keyId.slice(0, 8)}...${keyId.slice(-4)}` : 'N/A'}
            </div>
            <div className={`quantum-protection__card-subtitle strength--${getKeyStrengthColor(keyStrength)}`}>
              {keyStrength}-bit strength
            </div>
          </div>

          <div className="quantum-protection__card" data-testid="quantum-card-rotation">
            <div className="quantum-protection__card-header">
              <span className="quantum-protection__card-icon">🔄</span>
              <span className="quantum-protection__card-title">Key Rotation</span>
            </div>
            <div className="quantum-protection__card-value" data-testid="key-rotation-status">
              {formatDate(lastRotation)}
            </div>
            <div className={`quantum-protection__card-subtitle rotation--${rotationStatus}`} data-testid="next-rotation">
              Next: {formatNextRotation(nextRotation)}
            </div>
          </div>

          <div className="quantum-protection__card" data-testid="quantum-card-protected">
            <div className="quantum-protection__card-header">
              <span className="quantum-protection__card-icon">🛡️</span>
              <span className="quantum-protection__card-title">Protected</span>
            </div>
            <div className="quantum-protection__card-value" data-testid="protected-transactions">
              {formatTransactionCount(protectedTransactions)}
            </div>
            <div className="quantum-protection__card-subtitle">
              transactions secured
            </div>
          </div>

          {encryptionSpeed && (
            <div className="quantum-protection__card" data-testid="quantum-card-performance">
              <div className="quantum-protection__card-header">
                <span className="quantum-protection__card-icon">⚡</span>
                <span className="quantum-protection__card-title">Performance</span>
              </div>
              <div className="quantum-protection__card-value" data-testid="performance-metrics">
                {formatSpeed(encryptionSpeed)}
              </div>
              <div className="quantum-protection__card-subtitle">
                encryption speed
              </div>
            </div>
          )}
        </div>

        <div className="quantum-protection__footer" data-testid="quantum-info-footer">
          <div className="quantum-protection__info">
            <span className="quantum-protection__info-icon">ℹ️</span>
            <span className="quantum-protection__info-text">
              Your transactions are protected with post-quantum cryptography, 
              resistant to both classical and quantum computer attacks.
            </span>
          </div>
        </div>
      </div>
    </div>
  );
};

export default QuantumProtectionDisplay;