import { useState, useEffect, useCallback, useRef } from "react";
import {
  realTimeEventService,
  type BridgeOperationEvent,
} from "../../services/websocket/realTimeEventService";
import { type TransactionUpdate } from "../../services/websocket/wsClient";
import { useWebSocketConnection } from "../websocket/useWebSocket";

export interface BridgeWebSocketState {
  // Connection state
  isConnected: boolean;
  connectionQuality: "excellent" | "good" | "poor" | "unknown";

  // Bridge operations
  activeBridgeOperations: Map<string, BridgeOperationEvent>;
  activeTransactions: Map<string, TransactionUpdate>;

  // Real-time updates
  latestBridgeUpdate: BridgeOperationEvent | null;
  latestTransactionUpdate: TransactionUpdate | null;

  // Performance metrics
  eventsReceived: number;
  averageLatency: number;

  // Error state
  errors: Array<{ message: string; timestamp: number }>;
}

export interface BridgeWebSocketActions {
  // Subscription management
  subscribeToBridgeOperation: (operationId: string) => void;
  unsubscribeFromBridgeOperation: (operationId: string) => void;
  subscribeToTransaction: (transactionId: string) => void;
  unsubscribeFromTransaction: (transactionId: string) => void;

  // Bridge operation tracking
  trackBridgeOperation: (operationId: string) => void;
  stopTrackingBridgeOperation: (operationId: string) => void;

  // Real-time price monitoring
  subscribeToPriceUpdates: (fromToken: string, toToken: string) => void;
  unsubscribeFromPriceUpdates: () => void;

  // Error handling
  clearErrors: () => void;
  retry: () => void;
}

export const useBridgeWebSocket = (): BridgeWebSocketState &
  BridgeWebSocketActions => {
  // WebSocket connection state
  const { isConnected, connectionState } = useWebSocketConnection();

  // Bridge state
  const [state, setState] = useState<BridgeWebSocketState>({
    isConnected: false,
    connectionQuality: "unknown",
    activeBridgeOperations: new Map(),
    activeTransactions: new Map(),
    latestBridgeUpdate: null,
    latestTransactionUpdate: null,
    eventsReceived: 0,
    averageLatency: 0,
    errors: [],
  });

  // Subscription tracking
  const subscriptionsRef = useRef<{
    bridgeOperations: Set<string>;
    transactions: Set<string>;
    priceUpdates: string | null;
  }>({
    bridgeOperations: new Set(),
    transactions: new Set(),
    priceUpdates: null,
  });

  // Event handlers
  const handleBridgeOperationUpdate = useCallback(
    (event: BridgeOperationEvent) => {
      console.log("🌉 Bridge operation update:", event);

      setState((prev) => {
        const newOperations = new Map(prev.activeBridgeOperations);
        newOperations.set(event.operation_id, event);

        return {
          ...prev,
          activeBridgeOperations: newOperations,
          latestBridgeUpdate: event,
          eventsReceived: prev.eventsReceived + 1,
        };
      });
    },
    []
  );

  const handleTransactionUpdate = useCallback((event: TransactionUpdate) => {
    console.log("💰 Transaction update:", event);

    setState((prev) => {
      const newTransactions = new Map(prev.activeTransactions);
      newTransactions.set(event.transaction_id, event);

      return {
        ...prev,
        activeTransactions: newTransactions,
        latestTransactionUpdate: event,
        eventsReceived: prev.eventsReceived + 1,
      };
    });
  }, []);

  // Handle errors (placeholder for future implementation)
  // const handleError = useCallback((error: string) => {
  //   setState(prev => ({
  //     ...prev,
  //     errors: [...prev.errors, { message: error, timestamp: Date.now() }].slice(-10),
  //   }));
  // }, []);

  // Subscribe to WebSocket events
  useEffect(() => {
    if (!isConnected) return;

    console.log("🌉 Setting up bridge WebSocket subscriptions");

    // Subscribe to bridge operations
    const bridgeSubscriptionId = realTimeEventService.subscribe(
      "bridge_operation",
      (payload) => handleBridgeOperationUpdate(payload as BridgeOperationEvent),
      { priority: "high" }
    );

    // Subscribe to transaction updates
    const transactionSubscriptionId = realTimeEventService.subscribe(
      "transaction_update",
      (payload) => handleTransactionUpdate(payload as TransactionUpdate),
      { priority: "high" }
    );

    // Cleanup subscriptions
    return () => {
      realTimeEventService.unsubscribe(bridgeSubscriptionId);
      realTimeEventService.unsubscribe(transactionSubscriptionId);
      console.log("🌉 Cleaned up bridge WebSocket subscriptions");
    };
  }, [isConnected, handleBridgeOperationUpdate, handleTransactionUpdate]);

  // Update connection state
  useEffect(() => {
    setState((prev) => ({
      ...prev,
      isConnected,
      connectionQuality:
        connectionState === "connected"
          ? "excellent"
          : connectionState === "connecting"
          ? "good"
          : "poor",
    }));
  }, [isConnected, connectionState]);

  // Actions
  const subscribeToBridgeOperation = useCallback((operationId: string) => {
    if (subscriptionsRef.current.bridgeOperations.has(operationId)) return;

    subscriptionsRef.current.bridgeOperations.add(operationId);
    console.log(`🌉 Subscribed to bridge operation: ${operationId}`);
  }, []);

  const unsubscribeFromBridgeOperation = useCallback((operationId: string) => {
    subscriptionsRef.current.bridgeOperations.delete(operationId);

    setState((prev) => {
      const newOperations = new Map(prev.activeBridgeOperations);
      newOperations.delete(operationId);
      return { ...prev, activeBridgeOperations: newOperations };
    });

    console.log(`🌉 Unsubscribed from bridge operation: ${operationId}`);
  }, []);

  const subscribeToTransaction = useCallback((transactionId: string) => {
    if (subscriptionsRef.current.transactions.has(transactionId)) return;

    subscriptionsRef.current.transactions.add(transactionId);
    console.log(`💰 Subscribed to transaction: ${transactionId}`);
  }, []);

  const unsubscribeFromTransaction = useCallback((transactionId: string) => {
    subscriptionsRef.current.transactions.delete(transactionId);

    setState((prev) => {
      const newTransactions = new Map(prev.activeTransactions);
      newTransactions.delete(transactionId);
      return { ...prev, activeTransactions: newTransactions };
    });

    console.log(`💰 Unsubscribed from transaction: ${transactionId}`);
  }, []);

  const trackBridgeOperation = useCallback(
    (operationId: string) => {
      subscribeToBridgeOperation(operationId);

      // Request current status from server
      // This would typically send a WebSocket message to get current state
      console.log(`🔍 Tracking bridge operation: ${operationId}`);
    },
    [subscribeToBridgeOperation]
  );

  const stopTrackingBridgeOperation = useCallback(
    (operationId: string) => {
      unsubscribeFromBridgeOperation(operationId);
      console.log(`🛑 Stopped tracking bridge operation: ${operationId}`);
    },
    [unsubscribeFromBridgeOperation]
  );

  const subscribeToPriceUpdates = useCallback(
    (fromToken: string, toToken: string) => {
      const pairKey = `${fromToken}-${toToken}`;

      if (subscriptionsRef.current.priceUpdates === pairKey) return;

      // Unsubscribe from previous pair if exists
      if (subscriptionsRef.current.priceUpdates) {
        // Would unsubscribe from previous price updates
      }

      subscriptionsRef.current.priceUpdates = pairKey;

      // Subscribe to price updates for this pair
      realTimeEventService.subscribe(
        "price_update",
        (payload) => {
          const priceEvent = payload as {
            from_token: string;
            to_token: string;
            price: number;
          };
          if (
            priceEvent.from_token === fromToken &&
            priceEvent.to_token === toToken
          ) {
            console.log(
              `💱 Price update for ${fromToken}-${toToken}:`,
              priceEvent
            );
          }
        },
        {
          priority: "medium",
          rateLimitMs: 1000, // Limit to 1 update per second
          filter: (payload) => {
            const priceEvent = payload as {
              from_token: string;
              to_token: string;
            };
            return (
              priceEvent.from_token === fromToken &&
              priceEvent.to_token === toToken
            );
          },
        }
      );

      console.log(`💱 Subscribed to price updates: ${pairKey}`);
    },
    []
  );

  const unsubscribeFromPriceUpdates = useCallback(() => {
    if (subscriptionsRef.current.priceUpdates) {
      // Would unsubscribe from price updates
      subscriptionsRef.current.priceUpdates = null;
      console.log("💱 Unsubscribed from price updates");
    }
  }, []);

  const clearErrors = useCallback(() => {
    setState((prev) => ({ ...prev, errors: [] }));
  }, []);

  const retry = useCallback(() => {
    // Force WebSocket reconnection if needed
    if (!isConnected) {
      console.log("🔄 Attempting to retry WebSocket connection");
      // Would trigger reconnection
    }
    clearErrors();
  }, [isConnected, clearErrors]);

  return {
    // State
    ...state,

    // Actions
    subscribeToBridgeOperation,
    unsubscribeFromBridgeOperation,
    subscribeToTransaction,
    unsubscribeFromTransaction,
    trackBridgeOperation,
    stopTrackingBridgeOperation,
    subscribeToPriceUpdates,
    unsubscribeFromPriceUpdates,
    clearErrors,
    retry,
  };
};

export default useBridgeWebSocket;
