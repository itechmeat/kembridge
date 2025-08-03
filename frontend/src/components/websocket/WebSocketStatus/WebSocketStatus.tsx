import { FC } from "react";
import cn from "classnames";
import { useWebSocketContext } from "../../../contexts/WebSocketContext";
import styles from "./WebSocketStatus.module.scss";

export interface WebSocketStatusProps {
  className?: string;
  showDetails?: boolean;
  compact?: boolean; // Новый пропс для компактного отображения
}

export const WebSocketStatus: FC<WebSocketStatusProps> = ({
  className = "",
  showDetails = false,
  compact = false,
}) => {
  const { isConnected, connectionQuality, errors, retry } =
    useWebSocketContext();

  // Mapping connectionQuality to connection state for compatibility
  const connectionState = isConnected
    ? "connected"
    : connectionQuality === "unknown"
    ? "disconnected"
    : "connecting";

  // Get latest error if any
  const error = errors.length > 0 ? errors[errors.length - 1].message : null;

  const getStatusIcon = () => {
    switch (connectionState) {
      case "connected":
        return "🟢";
      case "connecting":
        return "🟡";
      case "disconnected":
      default:
        return "🔴";
    }
  };

  const getStatusText = () => {
    switch (connectionState) {
      case "connected":
        return "Connected";
      case "connecting":
        return "Connecting...";
      case "disconnected":
        return "Disconnected";
      default:
        return "Unknown";
    }
  };

  // Компактный режим для отображения в шапке
  if (compact) {
    return (
      <div
        className={cn(
          styles.webSocketStatus,
          styles.compact,
          styles[connectionState],
          className.trim()
        )}
        data-testid="websocket-status"
        title={`WebSocket: ${getStatusText()}`}
      >
        <span className={styles.icon} data-testid="websocket-status-icon">
          {getStatusIcon()}
        </span>
        <span className={styles.text} data-testid="websocket-status-text">
          {getStatusText()}
        </span>
      </div>
    );
  }

  return (
    <div
      className={cn(
        styles.webSocketStatus,
        styles[connectionState],
        className.trim()
      )}
      data-testid="websocket-status"
    >
      <div className={styles.indicator}>
        <span className={styles.icon} data-testid="websocket-status-icon">
          {getStatusIcon()}
        </span>
        <span className={styles.text} data-testid="websocket-status-text">
          {showDetails ? `Real-time: ${getStatusText()}` : getStatusText()}
        </span>
      </div>

      {error && (
        <div className={styles.error} data-testid="websocket-status-error">
          <span className={styles.errorIcon}>⚠️</span>
          <span className={styles.errorText}>{error}</span>
          {!isConnected && (
            <button
              className={styles.retry}
              onClick={retry}
              data-testid="websocket-retry-button"
            >
              Retry
            </button>
          )}
        </div>
      )}

      {showDetails && (
        <div className={styles.details} data-testid="websocket-status-details">
          <div className={styles.detail}>
            <span className={styles.detailLabel}>Status:</span>
            <span className={styles.detailValue}>{getStatusText()}</span>
          </div>
          <div className={styles.detail}>
            <span className={styles.detailLabel}>Real-time Updates:</span>
            <span className={styles.detailValue}>
              {isConnected ? "Enabled" : "Disabled"}
            </span>
          </div>
        </div>
      )}
    </div>
  );
};

export default WebSocketStatus;
