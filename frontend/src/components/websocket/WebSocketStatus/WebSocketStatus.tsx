/**
 * WebSocket Status Component
 * Displays real-time WebSocket connection status
 */

import React from 'react';
import { useWebSocketConnection } from '../../../hooks/websocket/useWebSocket';
import './WebSocketStatus.scss';

export interface WebSocketStatusProps {
  className?: string;
  showDetails?: boolean;
}

export const WebSocketStatus: React.FC<WebSocketStatusProps> = ({
  className = '',
  showDetails = false
}) => {
  const { isConnected, connectionState, error, reconnect } = useWebSocketConnection();

  const getStatusIcon = () => {
    switch (connectionState) {
      case 'connected':
        return '🟢';
      case 'connecting':
        return '🟡';
      case 'disconnected':
      case 'closed':
        return '🔴';
      default:
        return '⚪';
    }
  };

  const getStatusText = () => {
    switch (connectionState) {
      case 'connected':
        return 'Connected';
      case 'connecting':
        return 'Connecting...';
      case 'disconnected':
        return 'Disconnected';
      case 'closed':
        return 'Closed';
      default:
        return 'Unknown';
    }
  };

  const getStatusClass = () => {
    return `websocket-status--${connectionState}`;
  };

  return (
    <div className={`websocket-status ${getStatusClass()} ${className}`} data-testid="websocket-status">
      <div className="websocket-status__indicator">
        <span className="websocket-status__icon" data-testid="websocket-status-icon">
          {getStatusIcon()}
        </span>
        <span className="websocket-status__text" data-testid="websocket-status-text">
          {showDetails ? `Real-time: ${getStatusText()}` : getStatusText()}
        </span>
      </div>

      {error && (
        <div className="websocket-status__error" data-testid="websocket-status-error">
          <span className="websocket-status__error-icon">⚠️</span>
          <span className="websocket-status__error-text">{error}</span>
          {!isConnected && (
            <button
              className="websocket-status__retry"
              onClick={reconnect}
              data-testid="websocket-retry-button"
            >
              Retry
            </button>
          )}
        </div>
      )}

      {showDetails && (
        <div className="websocket-status__details" data-testid="websocket-status-details">
          <div className="websocket-status__detail">
            <span className="websocket-status__detail-label">Status:</span>
            <span className="websocket-status__detail-value">{getStatusText()}</span>
          </div>
          <div className="websocket-status__detail">
            <span className="websocket-status__detail-label">Real-time Updates:</span>
            <span className="websocket-status__detail-value">
              {isConnected ? 'Enabled' : 'Disabled'}
            </span>
          </div>
        </div>
      )}
    </div>
  );
};

export default WebSocketStatus;