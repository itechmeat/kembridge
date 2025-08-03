import { FC, useState } from 'react';
import { 
  SecurityIndicator, 
  RiskAnalysisDisplay, 
  RiskScoreVisualization,
  SecurityAlerts,
  QuantumProtectionDisplay
} from '../../components/security';
import { useRiskAnalysisLogger } from '../../hooks/security';
import { AlertType, AlertPriority } from '../../types/security';
import type { SecurityAlert, RiskAnalysisResult } from '../../types/security';
import classNames from 'classnames';
import styles from './SecurityTestPage.module.scss';

export const SecurityTestPage: FC = () => {
  // Demo state for testing
  const [riskScore, setRiskScore] = useState(0.3);
  const [quantumProtection, setQuantumProtection] = useState(true);
  const [isOnline, setIsOnline] = useState(true);
  const [alerts, setAlerts] = useState<SecurityAlert[]>([]);
  
  // Centralized risk analysis logger
  const { logRiskUpdate } = useRiskAnalysisLogger(1000);

  // Mock risk analysis data
  const mockRiskData: RiskAnalysisResult = {
    riskScore: {
      value: riskScore,
      level: riskScore > 0.7 ? 'high' : riskScore > 0.3 ? 'medium' : 'low',
      confidence: 0.85,
      timestamp: new Date().toISOString()
    },
    factors: [
      {
        type: 'transaction_amount',
        weight: 0.3,
        description: 'Large transaction amount detected',
        impact: riskScore > 0.5 ? 'negative' : 'neutral'
      },
      {
        type: 'address_reputation',
        weight: 0.4,
        description: 'Address reputation analysis',
        impact: 'positive'
      },
      {
        type: 'user_history',
        weight: 0.3,
        description: 'User has good transaction history',
        impact: 'positive'
      }
    ],
    recommendations: riskScore > 0.7 
      ? ['Consider reducing transaction amount', 'Verify recipient address', 'Enable additional security measures']
      : riskScore > 0.3
      ? ['Verify transaction details', 'Check recipient address']
      : ['Transaction appears safe'],
    blacklistStatus: {
      isBlacklisted: riskScore > 0.8,
      reason: riskScore > 0.8 ? 'Address flagged by security provider' : undefined
    },
    analysis: {
      transactionAmount: 1250,
      userHistory: {
        totalTransactions: 45,
        avgAmount: 350,
        riskHistory: [0.2, 0.3, 0.1, 0.4, 0.2]
      },
      addressRisk: {
        from: 0.1,
        to: riskScore * 0.5
      }
    }
  };

  // Generate test alerts
  const generateTestAlert = () => {
    const alertTypes = [AlertType.QUANTUM_OFFLINE, AlertType.HIGH_RISK_TRANSACTION, AlertType.SUSPICIOUS_ADDRESS, AlertType.KEY_ROTATION_DUE];
    const priorities = [AlertPriority.CRITICAL, AlertPriority.HIGH, AlertPriority.MEDIUM, AlertPriority.LOW];
    
    const randomType = alertTypes[Math.floor(Math.random() * alertTypes.length)];
    const randomPriority = priorities[Math.floor(Math.random() * priorities.length)];
    
    const newAlert: SecurityAlert = {
      id: `alert-${Date.now()}`,
      type: randomType,
      priority: randomPriority,
      title: `Test Alert - ${randomType.replace('_', ' ')}`,
      message: `This is a test ${randomPriority} priority alert for demonstration purposes.`,
      timestamp: new Date().toISOString(),
      isRead: false,
      isDismissible: true,
      autoDismissAfter: randomPriority === AlertPriority.LOW ? 5000 : undefined,
      actions: randomPriority === AlertPriority.CRITICAL ? [
        {
          id: 'action-1',
          label: 'Fix Now',
          type: 'danger',
          handler: () => console.log('Fix action clicked')
        },
        {
          id: 'action-2',
          label: 'Learn More',
          type: 'secondary',
          handler: () => console.log('Learn more clicked')
        }
      ] : undefined
    };
    
    setAlerts(prev => [newAlert, ...prev]);
  };

  const dismissAlert = (alertId: string) => {
    setAlerts(prev => prev.map(alert => 
      alert.id === alertId ? { ...alert, isRead: true } : alert
    ));
  };

  const handleAlertAction = (alertId: string, actionId: string) => {
    console.log(`Alert ${alertId} action ${actionId} clicked`);
    dismissAlert(alertId);
  };

  return (
    <div className={styles.securityTestPage}>
      <div className={styles.header}>
        <h1>Security Components Test Page</h1>
        <p>Interactive testing environment for KEMBridge security components</p>
      </div>

      {/* Controls */}
      <div className={styles.controls}>
        <h2>Controls</h2>
        <div className={styles.controlsGrid}>
          <div className={styles.controlGroup}>
            <label>Risk Score: {Math.round(riskScore * 100)}%</label>
            <input
              type="range"
              min="0"
              max="1"
              step="0.1"
              value={riskScore}
              onChange={(e) => setRiskScore(parseFloat(e.target.value))}
            />
          </div>
          
          <div className={styles.controlGroup}>
            <label>
              <input
                type="checkbox"
                checked={quantumProtection}
                onChange={(e) => setQuantumProtection(e.target.checked)}
              />
              Quantum Protection
            </label>
          </div>
          
          <div className={styles.controlGroup}>
            <label>
              <input
                type="checkbox"
                checked={isOnline}
                onChange={(e) => setIsOnline(e.target.checked)}
              />
              System Online
            </label>
          </div>
          
          <div className={styles.controlGroup}>
            <button onClick={generateTestAlert}>Generate Test Alert</button>
          </div>
        </div>
      </div>

      {/* Quantum Protection Display */}
      <div className={styles.section}>
        <h2>Quantum Protection Display</h2>
        <div className={styles.componentDemo}>
          <div className={classNames(styles.demoItem, styles.fullWidth)}>
            <QuantumProtectionDisplay
              isActive={quantumProtection}
              encryptionScheme="ML-KEM-1024"
              keyId="12345678-1234-1234-1234-123456789abc"
              keyStrength={1024}
              lastRotation={new Date(Date.now() - 24 * 60 * 60 * 1000).toISOString()}
              nextRotation={new Date(Date.now() + 7 * 24 * 60 * 60 * 1000).toISOString()}
              protectedTransactions={Math.floor(Math.random() * 10000)}
              encryptionSpeed={15000}
            />
          </div>
        </div>
      </div>

      {/* Security Indicator */}
      <div className={styles.section}>
        <h2>Security Indicator</h2>
        <div className={styles.componentDemo}>
          <div className={styles.demoItem}>
            <h3>Full Version</h3>
            <SecurityIndicator
              quantumProtection={quantumProtection}
              riskScore={riskScore}
              isOnline={isOnline}
              quantumKeyId="12345678-1234-1234-1234-123456789abc"
              encryptionScheme="ML-KEM-1024"
              lastKeyRotation={new Date(Date.now() - 24 * 60 * 60 * 1000).toISOString()}
              transactionCount={Math.floor(Math.random() * 10000)}
            />
          </div>
          
          <div className={styles.demoItem}>
            <h3>Compact Version</h3>
            <SecurityIndicator
              quantumProtection={quantumProtection}
              riskScore={riskScore}
              isOnline={isOnline}
              compact={true}
            />
          </div>
        </div>
      </div>

      {/* Risk Score Visualization */}
      <div className={styles.section}>
        <h2>Risk Score Visualization</h2>
        <div className={styles.componentDemo}>
          <div className={styles.demoItem}>
            <h3>Small</h3>
            <RiskScoreVisualization
              score={riskScore}
              size="small"
              animated={true}
              showLabel={true}
            />
          </div>
          
          <div className={styles.demoItem}>
            <h3>Medium</h3>
            <RiskScoreVisualization
              score={riskScore}
              size="medium"
              animated={true}
              showLabel={true}
              showTrend={true}
              previousScore={riskScore - 0.1}
            />
          </div>
          
          <div className={styles.demoItem}>
            <h3>Large</h3>
            <RiskScoreVisualization
              score={riskScore}
              size="large"
              animated={true}
              showLabel={true}
            />
          </div>
        </div>
      </div>

      {/* Risk Analysis Display */}
      <div className={styles.section}>
        <h2>Risk Analysis Display</h2>
        <div className={styles.componentDemo}>
          <div className={classNames(styles.demoItem, styles.fullWidth)}>
            <RiskAnalysisDisplay
              riskData={mockRiskData}
              realTime={true}
              showDetails={true}
              onUpdate={logRiskUpdate}
            />
          </div>
        </div>
      </div>

      {/* Security Alerts */}
      <SecurityAlerts
        alerts={alerts}
        maxVisible={3}
        position="top"
        onDismiss={dismissAlert}
        onAction={handleAlertAction}
      />
    </div>
  );
};

export default SecurityTestPage;