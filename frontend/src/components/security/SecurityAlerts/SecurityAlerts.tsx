import React, { useEffect, useState, useCallback } from 'react';
import { SecurityAlertsProps, SecurityAlert, AlertPriority } from '../../../types/security';
import './SecurityAlerts.scss';

export const SecurityAlerts: React.FC<SecurityAlertsProps> = ({
  alerts,
  maxVisible = 5,
  position = 'top',
  onDismiss,
  onAction,
}) => {
  const [visibleAlerts, setVisibleAlerts] = useState<SecurityAlert[]>([]);
  const [dismissingAlerts, setDismissingAlerts] = useState<Set<string>>(new Set());

  const handleDismiss = useCallback((alertId: string) => {
    setDismissingAlerts(prev => new Set(prev).add(alertId));
    
    // Delay to allow animation
    setTimeout(() => {
      onDismiss(alertId);
      setDismissingAlerts(prev => {
        const newSet = new Set(prev);
        newSet.delete(alertId);
        return newSet;
      });
    }, 300);
  }, [onDismiss]);

  // Auto-dismiss alerts based on autoDismissAfter
  useEffect(() => {
    const timers: Record<string, number> = {};

    alerts.forEach(alert => {
      if (alert.autoDismissAfter && !alert.isRead) {
        timers[alert.id] = window.setTimeout(() => {
          handleDismiss(alert.id);
        }, alert.autoDismissAfter);
      }
    });

    return () => {
      Object.values(timers).forEach(timer => window.clearTimeout(timer));
    };
  }, [alerts, handleDismiss]);

  // Update visible alerts based on priority and maxVisible
  useEffect(() => {
    const sortedAlerts = [...alerts]
      .filter(alert => !alert.isRead)
      .sort((a, b) => {
        // Sort by priority first
        const priorityOrder: Record<AlertPriority, number> = {
          critical: 0,
          high: 1,
          medium: 2,
          low: 3,
        };
        
        const priorityDiff = priorityOrder[a.priority] - priorityOrder[b.priority];
        if (priorityDiff !== 0) return priorityDiff;
        
        // Then by timestamp
        return new Date(b.timestamp).getTime() - new Date(a.timestamp).getTime();
      })
      .slice(0, maxVisible);

    setVisibleAlerts(sortedAlerts);
  }, [alerts, maxVisible]);

  const handleAction = (alertId: string, actionId: string) => {
    onAction(alertId, actionId);
  };

  const getAlertIcon = (alert: SecurityAlert): string => {
    switch (alert.type) {
      case 'quantum_offline':
        return '🔒';
      case 'high_risk_transaction':
        return '⚠️';
      case 'suspicious_address':
        return '🚨';
      case 'rate_limit_warning':
        return '⏱️';
      case 'system_maintenance':
        return '🔧';
      case 'key_rotation_due':
        return '🔄';
      case 'blacklist_detected':
        return '🛡️';
      default:
        return 'ℹ️';
    }
  };

  const getTimeAgo = (timestamp: string): string => {
    const now = new Date();
    const alertTime = new Date(timestamp);
    const diffInMinutes = Math.floor((now.getTime() - alertTime.getTime()) / (1000 * 60));
    
    if (diffInMinutes < 1) return 'Just now';
    if (diffInMinutes < 60) return `${diffInMinutes}m ago`;
    
    const diffInHours = Math.floor(diffInMinutes / 60);
    if (diffInHours < 24) return `${diffInHours}h ago`;
    
    const diffInDays = Math.floor(diffInHours / 24);
    return `${diffInDays}d ago`;
  };

  if (visibleAlerts.length === 0) {
    return null;
  }

  return (
    <div className={`security-alerts security-alerts--${position}`}>
      <div className="security-alerts__container">
        {visibleAlerts.map(alert => (
          <div
            key={alert.id}
            className={`
              security-alert 
              security-alert--${alert.priority}
              ${dismissingAlerts.has(alert.id) ? 'security-alert--dismissing' : ''}
            `}
          >
            <div className="security-alert__content">
              <div className="security-alert__header">
                <span className="security-alert__icon">
                  {getAlertIcon(alert)}
                </span>
                <div className="security-alert__info">
                  <h4 className="security-alert__title">{alert.title}</h4>
                  <span className="security-alert__time">
                    {getTimeAgo(alert.timestamp)}
                  </span>
                </div>
                {alert.isDismissible && (
                  <button
                    className="security-alert__dismiss"
                    onClick={() => handleDismiss(alert.id)}
                    aria-label="Dismiss alert"
                  >
                    ✕
                  </button>
                )}
              </div>
              
              <p className="security-alert__message">{alert.message}</p>
              
              {alert.actions && alert.actions.length > 0 && (
                <div className="security-alert__actions">
                  {alert.actions.map(action => (
                    <button
                      key={action.id}
                      className={`security-alert__action security-alert__action--${action.type}`}
                      onClick={() => handleAction(alert.id, action.id)}
                    >
                      {action.label}
                    </button>
                  ))}
                </div>
              )}
            </div>
          </div>
        ))}
      </div>
      
      {alerts.filter(a => !a.isRead).length > maxVisible && (
        <div className="security-alerts__overflow">
          <span className="security-alerts__overflow-text">
            +{alerts.filter(a => !a.isRead).length - maxVisible} more alerts
          </span>
        </div>
      )}
    </div>
  );
};

export default SecurityAlerts;