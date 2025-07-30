/**
 * Real-time Notifications Component
 * Displays live notifications from WebSocket events
 */

import React, { useState, useEffect } from 'react';
import { useRiskAlerts, useSystemNotifications } from '../../../hooks/websocket/useWebSocket';
import './RealTimeNotifications.scss';

export interface RealTimeNotificationsProps {
  className?: string;
  maxNotifications?: number;
  autoHide?: boolean;
  autoHideDelay?: number;
}

export const RealTimeNotifications: React.FC<RealTimeNotificationsProps> = ({
  className = '',
  maxNotifications = 5,
  autoHide = true,
  autoHideDelay = 5000,
}) => {
  const { alerts, unreadCount: alertCount, markAsRead: markAlertsRead } = useRiskAlerts();
  const { notifications, systemStatus } = useSystemNotifications();
  const [isVisible, setIsVisible] = useState(false);
  const [hiddenNotifications, setHiddenNotifications] = useState<Set<string>>(new Set());

  // Show notifications when there are new alerts or notifications
  useEffect(() => {
    if (alertCount > 0 || notifications.length > 0) {
      setIsVisible(true);
      
      if (autoHide) {
        const timer = setTimeout(() => {
          setIsVisible(false);
        }, autoHideDelay);
        
        return () => clearTimeout(timer);
      }
    }
  }, [alertCount, notifications.length, autoHide, autoHideDelay]);

  const handleClose = () => {
    setIsVisible(false);
    markAlertsRead();
  };

  const hideNotification = (id: string) => {
    setHiddenNotifications(prev => new Set(prev).add(id));
  };

  interface SystemNotification {
    id?: string;
    level?: string;
    title?: string;
    message?: string;
  }

  const getNotificationIcon = (level: string) => {
    switch (level) {
      case 'critical':
        return '🚨';
      case 'error':
        return '❌';
      case 'warning':
        return '⚠️';
      case 'info':
      default:
        return 'ℹ️';
    }
  };

  const getRiskIcon = (level: string) => {
    switch (level) {
      case 'high':
      case 'critical':
        return '🔴';
      case 'medium':
        return '🟡';
      case 'low':
      default:
        return '🟢';
    }
  };

  if (!isVisible) {
    return null;
  }

  const visibleAlerts = alerts.slice(0, maxNotifications);
  const visibleNotifications = notifications
    .filter(n => !hiddenNotifications.has((n as SystemNotification)?.id || ''))
    .slice(0, maxNotifications) as SystemNotification[];

  return (
    <div className={`realtime-notifications ${className}`} data-testid="realtime-notifications">
      <div className="realtime-notifications__header">
        <h3 className="realtime-notifications__title">
          🔔 Real-time Alerts
          {(alertCount > 0 || visibleNotifications.length > 0) && (
            <span className="realtime-notifications__count">
              {alertCount + visibleNotifications.length}
            </span>
          )}
        </h3>
        <button
          className="realtime-notifications__close"
          onClick={handleClose}
          data-testid="notifications-close-button"
        >
          ✕
        </button>
      </div>

      <div className="realtime-notifications__content">
        {/* Risk Alerts */}
        {visibleAlerts.map((alert, index) => (
          <div
            key={`alert-${index}`}
            className={`realtime-notifications__item realtime-notifications__item--alert realtime-notifications__item--${alert.risk_level}`}
            data-testid="risk-alert-item"
          >
            <div className="realtime-notifications__item-icon">
              {getRiskIcon(alert.risk_level)}
            </div>
            <div className="realtime-notifications__item-content">
              <div className="realtime-notifications__item-title">
                Risk Alert: {alert.risk_level.toUpperCase()}
              </div>
              <div className="realtime-notifications__item-message">
                {alert.message}
              </div>
              <div className="realtime-notifications__item-meta">
                Risk Score: {Math.round(alert.risk_score * 100)}%
                {alert.transaction_id && ` • TX: ${alert.transaction_id.slice(0, 8)}...`}
              </div>
            </div>
            <button
              className="realtime-notifications__item-dismiss"
              onClick={() => hideNotification(`alert-${index}`)}
              data-testid="dismiss-alert-button"
            >
              ✕
            </button>
          </div>
        ))}

        {/* System Notifications */}
        {visibleNotifications.map((notification, index) => (
          <div
            key={`notification-${index}`}
            className={`realtime-notifications__item realtime-notifications__item--notification realtime-notifications__item--${notification.level || 'info'}`}
            data-testid="system-notification-item"
          >
            <div className="realtime-notifications__item-icon">
              {getNotificationIcon(notification.level || 'info')}
            </div>
            <div className="realtime-notifications__item-content">
              <div className="realtime-notifications__item-title">
                {notification.title || 'System Notification'}
              </div>
              <div className="realtime-notifications__item-message">
                {notification.message || JSON.stringify(notification)}
              </div>
              <div className="realtime-notifications__item-meta">
                System Status: {systemStatus}
              </div>
            </div>
            <button
              className="realtime-notifications__item-dismiss"
              onClick={() => hideNotification(`notification-${index}`)}
              data-testid="dismiss-notification-button"
            >
              ✕
            </button>
          </div>
        ))}

        {visibleAlerts.length === 0 && visibleNotifications.length === 0 && (
          <div className="realtime-notifications__empty" data-testid="notifications-empty">
            No new notifications
          </div>
        )}
      </div>
    </div>
  );
};

export default RealTimeNotifications;