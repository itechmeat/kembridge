import { useState, useEffect, FC } from "react";
import cn from "classnames";
import {
  useRiskAlerts,
  useSystemNotifications,
} from "../../../hooks/websocket/useWebSocket";
import styles from "./RealTimeNotifications.module.scss";

export interface RealTimeNotificationsProps {
  className?: string;
  maxNotifications?: number;
  autoHide?: boolean;
  autoHideDelay?: number;
}

export const RealTimeNotifications: FC<RealTimeNotificationsProps> = ({
  className = "",
  maxNotifications = 5,
  autoHide = true,
  autoHideDelay = 5000,
}) => {
  const {
    alerts,
    unreadCount: alertCount,
    markAsRead: markAlertsRead,
  } = useRiskAlerts();
  const { notifications, systemStatus } = useSystemNotifications();
  const [isVisible, setIsVisible] = useState(false);
  const [hiddenNotifications, setHiddenNotifications] = useState<Set<string>>(
    new Set()
  );

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
    setHiddenNotifications((prev) => new Set(prev).add(id));
  };

  interface SystemNotification {
    id?: string;
    level?: string;
    title?: string;
    message?: string;
  }

  const getNotificationIcon = (level: string) => {
    switch (level) {
      case "critical":
        return "🚨";
      case "error":
        return "❌";
      case "warning":
        return "⚠️";
      case "info":
      default:
        return "ℹ️";
    }
  };

  const getRiskIcon = (level: string) => {
    switch (level) {
      case "high":
      case "critical":
        return "🔴";
      case "medium":
        return "🟡";
      case "low":
      default:
        return "🟢";
    }
  };

  if (!isVisible) {
    return null;
  }

  const visibleAlerts = alerts.slice(0, maxNotifications);
  const visibleNotifications = notifications
    .filter(
      (n) => !hiddenNotifications.has((n as SystemNotification)?.id || "")
    )
    .slice(0, maxNotifications) as SystemNotification[];

  return (
    <div
      className={cn(styles.realTimeNotifications, className.trim())}
      data-testid="realtime-notifications"
    >
      <div className={styles.header}>
        <h3 className={styles.title}>
          🔔 Real-time Alerts
          {(alertCount > 0 || visibleNotifications.length > 0) && (
            <span className={styles.count}>
              {alertCount + visibleNotifications.length}
            </span>
          )}
        </h3>
        <button
          className={styles.close}
          onClick={handleClose}
          data-testid="notifications-close-button"
        >
          ✕
        </button>
      </div>

      <div className={styles.content}>
        {/* Risk Alerts */}
        {visibleAlerts.map((alert, index) => (
          <div
            key={`alert-${index}`}
            className={cn(styles.item, styles.alert, styles[alert.risk_level])}
            data-testid="risk-alert-item"
          >
            <div className={styles.itemIcon}>
              {getRiskIcon(alert.risk_level)}
            </div>
            <div className={styles.itemContent}>
              <div className={styles.itemTitle}>
                Risk Alert: {alert.risk_level.toUpperCase()}
              </div>
              <div className={styles.itemMessage}>{alert.message}</div>
              <div className={styles.itemMeta}>
                Risk Score: {Math.round(alert.risk_score * 100)}%
                {alert.transaction_id &&
                  ` • TX: ${alert.transaction_id.slice(0, 8)}...`}
              </div>
            </div>
            <button
              className={styles.itemDismiss}
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
            className={cn(
              styles.item,
              styles.notification,
              styles[notification.level || "info"]
            )}
            data-testid="system-notification-item"
          >
            <div className={styles.itemIcon}>
              {getNotificationIcon(notification.level || "info")}
            </div>
            <div className={styles.itemContent}>
              <div className={styles.itemTitle}>
                {notification.title || "System Notification"}
              </div>
              <div className={styles.itemMessage}>
                {notification.message || JSON.stringify(notification)}
              </div>
              <div className={styles.itemMeta}>
                System Status: {systemStatus}
              </div>
            </div>
            <button
              className={styles.itemDismiss}
              onClick={() => hideNotification(`notification-${index}`)}
              data-testid="dismiss-notification-button"
            >
              ✕
            </button>
          </div>
        ))}

        {visibleAlerts.length === 0 && visibleNotifications.length === 0 && (
          <div className={styles.empty} data-testid="notifications-empty">
            No new notifications
          </div>
        )}
      </div>
    </div>
  );
};

export default RealTimeNotifications;
