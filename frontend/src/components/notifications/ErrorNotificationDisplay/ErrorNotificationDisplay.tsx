import { FC, useEffect, useState } from "react";
import cn from "classnames";
import { ErrorNotification } from "../../../services/errorHandlingService";
import { errorHandlingService } from "../../../services/errorHandlingService";
import styles from "./ErrorNotificationDisplay.module.scss";

interface Props {
  maxVisible?: number;
  position?: "top-right" | "top-left" | "bottom-right" | "bottom-left";
}

const ErrorNotificationDisplay: FC<Props> = ({
  maxVisible = 5,
  position = "top-right",
}) => {
  const [notifications, setNotifications] = useState<ErrorNotification[]>([]);
  const [collapsed, setCollapsed] = useState(false);

  useEffect(() => {
    const unsubscribe = errorHandlingService.subscribe(
      (updatedNotifications) => {
        setNotifications(updatedNotifications);
      }
    );

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
        return styles.urgencyCritical;
      case "high":
        return styles.urgencyHigh;
      case "medium":
        return styles.urgencyMedium;
      case "low":
        return styles.urgencyLow;
      default:
        return styles.urgencyMedium;
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
      className={cn(styles.errorNotificationDisplay, {
        [styles.positionTopRight]: position === "top-right",
        [styles.positionTopLeft]: position === "top-left",
        [styles.positionBottomRight]: position === "bottom-right",
        [styles.positionBottomLeft]: position === "bottom-left",
      })}
      data-testid="error-notification-display"
    >
      {/* Header with controls */}
      <div className={styles.notificationHeader}>
        <div className={styles.notificationHeaderInfo}>
          <span className={styles.notificationCount}>
            {notifications.length} notification
            {notifications.length !== 1 ? "s" : ""}
          </span>
          {hiddenCount > 0 && (
            <span className={styles.hiddenCount}>(+{hiddenCount} more)</span>
          )}
        </div>

        <div className={styles.notificationHeaderControls}>
          <button
            type="button"
            className={styles.controlButton}
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
            className={styles.controlButton}
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
        <div className={styles.notificationList}>
          {visibleNotifications.map((notification) => {
            const urgencyClass = getUrgencyClass(
              notification.error.severity || "medium"
            );
            const categoryClass =
              notification.error.category.toLowerCase() === "error"
                ? styles.typeError
                : notification.error.category.toLowerCase() === "warning"
                ? styles.typeWarning
                : notification.error.category.toLowerCase() === "success"
                ? styles.typeSuccess
                : notification.error.category.toLowerCase() === "info"
                ? styles.typeInfo
                : notification.error.severity === "critical"
                ? styles.typeCritical
                : undefined;

            return (
              <div
                key={notification.id}
                className={cn(styles.notification, urgencyClass, categoryClass)}
                data-testid={
                  notification.title === "Success" ||
                  notification.message.toLowerCase().includes("success") ||
                  notification.message.toLowerCase().includes("complete")
                    ? "success-message"
                    : notification.error.severity === "critical" ||
                      notification.error.severity === "high" ||
                      notification.message.toLowerCase().includes("error") ||
                      notification.message.toLowerCase().includes("failed")
                    ? "error-message"
                    : `notification-${notification.id}`
                }
              >
                {/* Notification header */}
                <div className={styles.notificationHeaderRow}>
                  <div className={styles.notificationIcon}>
                    {getNotificationIcon(
                      notification.error.severity || "medium"
                    )}
                  </div>

                  <div className={styles.notificationTitleSection}>
                    <h4 className={styles.notificationTitle}>
                      {notification.title}
                    </h4>

                    <div className={styles.notificationMeta}>
                      <span className={styles.notificationCategory}>
                        {notification.category.replace("_", " ")}
                      </span>
                      <span className={styles.notificationTimestamp}>
                        {formatTimestamp(new Date(notification.timestamp))}
                      </span>
                    </div>
                  </div>

                  <button
                    type="button"
                    className={styles.notificationDismiss}
                    onClick={() => handleDismiss(notification.id)}
                    title="Dismiss notification"
                    data-testid={`dismiss-${notification.id}`}
                  >
                    ✕
                  </button>
                </div>

                {/* Notification content */}
                <div className={styles.notificationContent}>
                  <p className={styles.notificationMessage}>
                    {notification.message}
                  </p>

                  {notification.details && (
                    <details className={styles.notificationDetails}>
                      <summary>Details</summary>
                      <p>{notification.details}</p>
                    </details>
                  )}

                  {/* Transaction info */}
                  {notification.transactionId && (
                    <div className={styles.notificationTransactionInfo}>
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
                      <div className={styles.notificationRetryInfo}>
                        <small>
                          Retry {notification.retryCount} of{" "}
                          {notification.maxRetries}
                        </small>
                      </div>
                    )}
                </div>

                {/* Notification actions */}
                {notification.actions && notification.actions.length > 0 && (
                  <div className={styles.notificationActions}>
                    {notification.actions.map((action, index) => {
                      const actionClass =
                        action.style === "primary"
                          ? styles.notificationActionPrimary
                          : action.style === "danger"
                          ? styles.notificationActionDanger
                          : styles.notificationActionSecondary;

                      return (
                        <button
                          key={index}
                          type="button"
                          className={cn(styles.notificationAction, actionClass)}
                          onClick={() => handleAction(action.action)}
                          data-testid={`action-${notification.id}-${index}`}
                        >
                          {action.label}
                        </button>
                      );
                    })}
                  </div>
                )}

                {/* Progress bar for auto-hide */}
                {notification.autoHide && notification.hideAfter && (
                  <div className={styles.notificationProgress}>
                    <div
                      className={styles.notificationProgressBar}
                      style={{
                        animationDuration: `${notification.hideAfter}ms`,
                      }}
                    />
                  </div>
                )}
              </div>
            );
          })}
        </div>
      )}

      {/* Show more button */}
      {hiddenCount > 0 && !collapsed && (
        <div className={styles.notificationFooter}>
          <button
            type="button"
            className={styles.showMoreButton}
            onClick={() => {
              /* TODO: Show all notifications in modal */
            }}
            data-testid="show-more-button"
          >
            Show {hiddenCount} more notification{hiddenCount !== 1 ? "s" : ""}
          </button>
        </div>
      )}
    </div>
  );
};

export default ErrorNotificationDisplay;
