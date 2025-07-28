/**
 * Services Index
 * Centralized export of all application services
 *
 * @module Services
 * @description This module exports all services used in the application.
 * @requires api/apiClient
 * @requires api/authService
 * @requires api/userService
 * @requires api/bridgeService
 */

// API Services
export * from "./api";

// WebSocket Services
export { default as wsClient } from "./websocket/wsClient";
export type {
  WSMessage,
  TransactionUpdate,
  RiskAlert,
  WSEventType,
} from "./websocket/wsClient";

// WebSocket Hooks
export {
  useWebSocketConnection,
  useTransactionUpdates,
  useMultipleTransactionUpdates,
  useRiskAlerts,
  useSystemNotifications,
} from "../hooks/websocket/useWebSocket";

// Utility re-exports
export { default as apiClient } from "./api/apiClient";
export { default as authService } from "./api/authService";
export { default as userService } from "./api/userService";
export { default as bridgeService } from "./api/bridgeService";
