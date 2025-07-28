/**
 * WebSocket Hooks
 * React hooks for real-time communication
 */

import { useEffect, useState, useCallback } from "react";
import {
  wsClient,
  type TransactionUpdate,
  type RiskAlert,
} from "../../services/websocket/wsClient";
import { useAuthStatus } from "../api/useAuth";

/**
 * Hook for managing WebSocket connection
 */
export const useWebSocketConnection = () => {
  const { isAuthenticated, token } = useAuthStatus();
  const [isConnected, setIsConnected] = useState(false);
  const [connectionState, setConnectionState] =
    useState<string>("disconnected");
  const [error, setError] = useState<string | null>(null);

  // Connect when authenticated
  useEffect(() => {
    if (isAuthenticated && token) {
      console.log("🔌 useWebSocket: Connecting with auth token");

      wsClient
        .connect(token)
        .then(() => {
          setIsConnected(true);
          setConnectionState("connected");
          setError(null);
          console.log("✅ useWebSocket: Connected successfully");
        })
        .catch((err) => {
          setError(err.message || "Failed to connect");
          console.error("❌ useWebSocket: Connection failed:", err);
        });
    } else {
      // Disconnect if not authenticated
      wsClient.disconnect();
      setIsConnected(false);
      setConnectionState("disconnected");
    }

    // Cleanup on unmount
    return () => {
      wsClient.disconnect();
    };
  }, [isAuthenticated, token]);

  // Monitor connection state
  useEffect(() => {
    const updateConnectionState = () => {
      setIsConnected(wsClient.isConnected);
      setConnectionState(wsClient.connectionState);
    };

    // Check state every 5 seconds
    const interval = setInterval(updateConnectionState, 5000);

    return () => clearInterval(interval);
  }, []);

  const reconnect = useCallback(() => {
    if (token) {
      wsClient.connect(token);
    }
  }, [token]);

  return {
    isConnected,
    connectionState,
    error,
    reconnect,
  };
};

/**
 * Hook for tracking transaction updates
 */
export const useTransactionUpdates = (transactionId: string | null) => {
  const [updates, setUpdates] = useState<TransactionUpdate[]>([]);
  const [latestUpdate, setLatestUpdate] = useState<TransactionUpdate | null>(
    null
  );
  const { isConnected } = useWebSocketConnection();

  useEffect(() => {
    if (!transactionId || !isConnected) return;

    console.log(
      "📡 useTransactionUpdates: Subscribing to transaction:",
      transactionId
    );

    // Transaction update handler
    const handleUpdate = (update: TransactionUpdate) => {
      console.log("📨 useTransactionUpdates: Received update:", update);

      if (update.transaction_id === transactionId) {
        setLatestUpdate(update);
        setUpdates((prev) => [...prev, update]);
      }
    };

    // Subscribe to events
    const wrappedHandler = (data: unknown) => {
      handleUpdate(data as TransactionUpdate);
    };
    wsClient.on("transaction_update", wrappedHandler);
    wsClient.subscribeToTransaction(transactionId);

    // Cleanup
    return () => {
      wsClient.off("transaction_update", wrappedHandler);
      wsClient.unsubscribeFromTransaction(transactionId);
      console.log(
        "🔕 useTransactionUpdates: Unsubscribed from transaction:",
        transactionId
      );
    };
  }, [transactionId, isConnected]);

  return {
    updates,
    latestUpdate,
    hasUpdates: updates.length > 0,
  };
};

/**
 * Hook for monitoring multiple transactions
 */
export const useMultipleTransactionUpdates = (transactionIds: string[]) => {
  const [updates, setUpdates] = useState<Map<string, TransactionUpdate[]>>(
    new Map()
  );
  const [latestUpdates, setLatestUpdates] = useState<
    Map<string, TransactionUpdate>
  >(new Map());
  const { isConnected } = useWebSocketConnection();

  useEffect(() => {
    if (transactionIds.length === 0 || !isConnected) return;

    console.log(
      "📡 useMultipleTransactionUpdates: Subscribing to transactions:",
      transactionIds
    );

    // Update handler
    const handleUpdate = (update: TransactionUpdate) => {
      const txId = update.transaction_id;

      if (transactionIds.includes(txId)) {
        console.log(
          "📨 useMultipleTransactionUpdates: Received update for:",
          txId
        );

        // Update latest updates
        setLatestUpdates((prev) => new Map(prev.set(txId, update)));

        // Add to update history
        setUpdates((prev) => {
          const newMap = new Map(prev);
          const existing = newMap.get(txId) || [];
          newMap.set(txId, [...existing, update]);
          return newMap;
        });
      }
    };

    // Subscribe to events
    const wrappedHandler = (data: unknown) => {
      handleUpdate(data as TransactionUpdate);
    };
    wsClient.on("transaction_update", wrappedHandler);

    // Subscribe to each transaction
    transactionIds.forEach((id) => {
      wsClient.subscribeToTransaction(id);
    });

    // Cleanup
    return () => {
      wsClient.off("transaction_update", wrappedHandler);
      transactionIds.forEach((id) => {
        wsClient.unsubscribeFromTransaction(id);
      });
      console.log(
        "🔕 useMultipleTransactionUpdates: Unsubscribed from all transactions"
      );
    };
  }, [transactionIds, isConnected]);

  return {
    updates,
    latestUpdates,
    getUpdatesForTransaction: (txId: string) => updates.get(txId) || [],
    getLatestUpdateForTransaction: (txId: string) =>
      latestUpdates.get(txId) || null,
  };
};

/**
 * Hook for getting risk alerts
 */
export const useRiskAlerts = () => {
  const [alerts, setAlerts] = useState<RiskAlert[]>([]);
  const [unreadCount, setUnreadCount] = useState(0);
  const { isConnected } = useWebSocketConnection();

  useEffect(() => {
    if (!isConnected) return;

    console.log("📡 useRiskAlerts: Subscribing to risk alerts");

    const handleAlert = (alert: RiskAlert) => {
      console.log("🚨 useRiskAlerts: Received risk alert:", alert);

      setAlerts((prev) => [alert, ...prev]);
      setUnreadCount((prev) => prev + 1);
    };

    const wrappedAlertHandler = (data: unknown) => {
      handleAlert(data as RiskAlert);
    };
    wsClient.on("risk_alert", wrappedAlertHandler);

    return () => {
      wsClient.off("risk_alert", wrappedAlertHandler);
      console.log("🔕 useRiskAlerts: Unsubscribed from risk alerts");
    };
  }, [isConnected]);

  const markAsRead = useCallback(() => {
    setUnreadCount(0);
  }, []);

  const clearAlerts = useCallback(() => {
    setAlerts([]);
    setUnreadCount(0);
  }, []);

  return {
    alerts,
    unreadCount,
    hasUnread: unreadCount > 0,
    markAsRead,
    clearAlerts,
  };
};

/**
 * Hook for system notifications
 */
export const useSystemNotifications = () => {
  const [notifications, setNotifications] = useState<unknown[]>([]);
  const [systemStatus, setSystemStatus] = useState<string>("unknown");
  const { isConnected } = useWebSocketConnection();

  useEffect(() => {
    if (!isConnected) return;

    console.log(
      "📡 useSystemNotifications: Subscribing to system notifications"
    );

    const handleNotification = (notification: unknown) => {
      console.log(
        "📢 useSystemNotifications: Received notification:",
        notification
      );
      setNotifications((prev) => [notification, ...prev.slice(0, 49)]); // Keep last 50
    };

    const handleSystemStatus = (status: unknown) => {
      console.log("📊 useSystemNotifications: System status update:", status);
      setSystemStatus((status as { status?: string })?.status || "unknown");
    };

    wsClient.on("user_notification", handleNotification);
    wsClient.on("system_status", handleSystemStatus);

    // Subscribe to system notifications
    wsClient.subscribeToSystemNotifications();

    return () => {
      wsClient.off("user_notification", handleNotification);
      wsClient.off("system_status", handleSystemStatus);
      wsClient.unsubscribeFromSystemNotifications();
      console.log(
        "🔕 useSystemNotifications: Unsubscribed from system notifications"
      );
    };
  }, [isConnected]);

  return {
    notifications,
    systemStatus,
    hasNotifications: notifications.length > 0,
  };
};
