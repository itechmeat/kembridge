/**
 * Error Notification Display Component
 * Visual display of error notifications with user interaction capabilities
 */

import React, { useState, useEffect } from "react";
import { ErrorNotification } from "../../services/errorHandlingService";
import { errorHandlingService } from "../../services/errorHandlingService";
import "./ErrorNotificationDisplay.scss";

interface Props {
  maxVisible?: number;
  position?: "top-right" | "top-left" | "bottom-right" | "bottom-left";
}

const ErrorNotificationDisplay: React.FC<Props> = ({
  maxVisible = 5,
  position = "top-right",
}) => {
  const [notifications, setNotifications] = useState<ErrorNotification[]>([]);
  const [collapsed, setCollapsed] = useState(false);

  useEffect(() => {
    // Subscribe to notification updates
    const unsubscribe = errorHandlingService.subscribe(
      (updatedNotifications) => {
        setNotifications(updatedNotifications);
      }
    );

    // Load initial notifications
    setNotifications(errorHandlingService.getNotifications());

    return unsubscribe;
  }, []);

  const handleDismiss = (id: string) => {
    errorHandlingService.dismissNotification(id);
  };

  const handleDismissAll = () => {
    errorHandlingService.dismissAllNotifications();
  };

  const handleAction = async (action: () => void | Promise<void>) => {
    try {
      await action();
    } catch (error) {
      console.error("Error executing notification action:", error);
    }
  };

  const getNotificationIcon = (type: string): string => {
    switch (type) {
      case "error":
      case "critical":
        return "❌";
      case "warning":
        return "⚠️";
      case "success":
        return "✅";
      case "info":
        return "ℹ️";
      default:
        return "📝";
    }
  };

  const getUrgencyClass = (urgency: string): string => {
    switch (urgency) {
      case "critical":
        return "notification--critical";
      case "high":
        return "notification--high";
      case "medium":
        return "notification--medium";
      case "low":
        return "notification--low";
      default:
        return "notification--medium";
    }
  };

  const formatTimestamp = (timestamp: Date): string => {
    const now = new Date();
    const diffMs = now.getTime() - timestamp.getTime();
    const diffMins = Math.floor(diffMs / 60000);
    const diffHours = Math.floor(diffMins / 60);

    if (diffMins < 1) return "Just now";
    if (diffMins < 60) return `${diffMins}m ago`;
    if (diffHours < 24) return `${diffHours}h ago`;

    return timestamp.toLocaleDateString();
  };

  const visibleNotifications = notifications.slice(0, maxVisible);
  const hiddenCount = Math.max(0, notifications.length - maxVisible);

  if (notifications.length === 0) {
    return null;
  }

  return (
    <div
      className={`error-notification-display error-notification-display--${position}`}
      data-testid="error-notification-display"
    >
      {/* Header with controls */}
      <div className="notification-header">
        <div className="notification-header__info">
          <span className="notification-count">
            {notifications.length} notification
            {notifications.length !== 1 ? "s" : ""}
          </span>
          {hiddenCount > 0 && (
            <span className="hidden-count">(+{hiddenCount} more)</span>
          )}
        </div>

        <div className="notification-header__controls">
          <button
            type="button"
            className="control-button"
            onClick={() => setCollapsed(!collapsed)}
            title={
              collapsed ? "Expand notifications" : "Collapse notifications"
            }
            data-testid="toggle-collapse-button"
          >
            {collapsed ? "📋" : "📋"}
          </button>

          <button
            type="button"
            className="control-button"
            onClick={handleDismissAll}
            title="Dismiss all notifications"
            data-testid="dismiss-all-button"
          >
            🗑️
          </button>
        </div>
      </div>

      {/* Notification list */}
      {!collapsed && (
        <div className="notification-list">
          {visibleNotifications.map((notification) => (
            <div
              key={notification.id}
              className={`notification ${getUrgencyClass(
                notification.error.severity || "medium"
              )} notification--${notification.error.category.toLowerCase()}`}
              data-testid={`notification-${notification.id}`}
            >
              {/* Notification header */}
              <div className="notification__header">
                <div className="notification__icon">
                  {getNotificationIcon(notification.error.severity || "medium")}
                </div>

                <div className="notification__title-section">
                  <h4 className="notification__title">{notification.title}</h4>

                  <div className="notification__meta">
                    <span className="notification__category">
                      {notification.category.replace("_", " ")}
                    </span>
                    <span className="notification__timestamp">
                      {formatTimestamp(new Date(notification.timestamp))}
                    </span>
                  </div>
                </div>

                <button
                  type="button"
                  className="notification__dismiss"
                  onClick={() => handleDismiss(notification.id)}
                  title="Dismiss notification"
                  data-testid={`dismiss-${notification.id}`}
                >
                  ✕
                </button>
              </div>

              {/* Notification content */}
              <div className="notification__content">
                <p className="notification__message">{notification.message}</p>

                {notification.details && (
                  <details className="notification__details">
                    <summary>Details</summary>
                    <p>{notification.details}</p>
                  </details>
                )}

                {/* Transaction info */}
                {notification.transactionId && (
                  <div className="notification__transaction-info">
                    <small>
                      Transaction: {notification.transactionId.slice(0, 8)}...
                      {notification.operationType &&
                        ` (${notification.operationType})`}
                    </small>
                  </div>
                )}

                {/* Retry info */}
                {notification.retryCount !== undefined &&
                  notification.maxRetries !== undefined && (
                    <div className="notification__retry-info">
                      <small>
                        Retry {notification.retryCount} of{" "}
                        {notification.maxRetries}
                      </small>
                    </div>
                  )}
              </div>

              {/* Notification actions */}
              {notification.actions && notification.actions.length > 0 && (
                <div className="notification__actions">
                  {notification.actions.map((action, index) => (
                    <button
                      key={index}
                      type="button"
                      className={`notification__action notification__action--${
                        action.style || "secondary"
                      }`}
                      onClick={() => handleAction(action.action)}
                      data-testid={`action-${notification.id}-${index}`}
                    >
                      {action.label}
                    </button>
                  ))}
                </div>
              )}

              {/* Progress bar for auto-hide */}
              {notification.autoHide && notification.hideAfter && (
                <div className="notification__progress">
                  <div
                    className="notification__progress-bar"
                    style={{
                      animationDuration: `${notification.hideAfter}ms`,
                    }}
                  />
                </div>
              )}
            </div>
          ))}
        </div>
      )}

      {/* Show more button */}
      {hiddenCount > 0 && !collapsed && (
        <div className="notification-footer">
          <button
            type="button"
            className="show-more-button"
            onClick={() => {
              /* TODO: Show all notifications in modal */
            }}
            data-testid="show-more-button"
          >
            Show {hiddenCount} more notification{hiddenCount !== 1 ? "s" : ""}
          </button>
        </div>
      )}

      {/* Statistics display */}
    </div>
  );
};

export default ErrorNotificationDisplay;
