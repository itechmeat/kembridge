/**
 * WebSocket Client
 * Real-time connection for monitoring transactions and events
 */

import { API_CONFIG } from "../api/config";

export interface WSMessage {
  type: string;
  data: unknown;
  timestamp: string;
}

export interface TransactionUpdate {
  transaction_id: string;
  status: "pending" | "confirmed" | "completed" | "failed" | "expired";
  from_transaction_hash?: string;
  to_transaction_hash?: string;
  updated_at: string;
}

export interface RiskAlert {
  user_id: string;
  transaction_id: string;
  risk_score: number;
  risk_level: "low" | "medium" | "high";
  flags: string[];
  message: string;
}

export type WSEventType =
  | "transaction_update"
  | "risk_alert"
  | "system_status"
  | "user_notification";

class WebSocketClient {
  private ws: WebSocket | null = null;
  private reconnectAttempts = 0;
  private maxReconnectAttempts = 5;
  private reconnectInterval = 1000; // 1 second
  private listeners: Map<WSEventType, Set<(data: unknown) => void>> = new Map();
  private isConnecting = false;
  private authToken: string | null = null;

  constructor() {
    console.log("🔌 WebSocket Client: Initializing");
  }

  /**
   * Sets up a connection to the WebSocket server
   */
  connect(authToken?: string): Promise<void> {
    return new Promise((resolve, reject) => {
      if (this.ws?.readyState === WebSocket.OPEN) {
        console.log("✅ WebSocket: Already connected");
        resolve();
        return;
      }

      if (this.isConnecting) {
        console.log("⏳ WebSocket: Connection already in progress");
        return;
      }

      this.isConnecting = true;
      this.authToken = authToken || this.authToken;

      try {
        const wsUrl = `${API_CONFIG.WS_URL}${
          this.authToken ? `?token=${this.authToken}` : ""
        }`;
        console.log("🔌 WebSocket: Connecting to:", wsUrl);

        this.ws = new WebSocket(wsUrl);

        this.ws.onopen = () => {
          console.log("✅ WebSocket: Connected successfully");
          this.isConnecting = false;
          this.reconnectAttempts = 0;
          resolve();
        };

        this.ws.onmessage = (event) => {
          try {
            const message: WSMessage = JSON.parse(event.data);
            console.log(
              "📨 WebSocket: Message received:",
              message.type,
              message.data
            );
            this.handleMessage(message);
          } catch (error) {
            console.error("❌ WebSocket: Failed to parse message:", error);
          }
        };

        this.ws.onclose = (event) => {
          console.log(
            "🔌 WebSocket: Connection closed:",
            event.code,
            event.reason
          );
          this.isConnecting = false;

          if (
            !event.wasClean &&
            this.reconnectAttempts < this.maxReconnectAttempts
          ) {
            this.scheduleReconnect();
          }
        };

        this.ws.onerror = (error) => {
          console.error("❌ WebSocket: Connection error:", error);
          this.isConnecting = false;
          reject(error);
        };
      } catch (error) {
        console.error("❌ WebSocket: Failed to create connection:", error);
        this.isConnecting = false;
        reject(error);
      }
    });
  }

  /**
   * Closes WebSocket connection
   */
  disconnect(): void {
    console.log("🔌 WebSocket: Disconnecting");

    if (this.ws) {
      this.ws.close(1000, "User disconnected");
      this.ws = null;
    }

    this.reconnectAttempts = this.maxReconnectAttempts; // Prevent reconnection
  }

  /**
   * Sends message through WebSocket
   */
  send(type: string, data: unknown): void {
    if (this.ws?.readyState !== WebSocket.OPEN) {
      console.warn("⚠️ WebSocket: Cannot send message, connection not open");
      return;
    }

    const message: WSMessage = {
      type,
      data,
      timestamp: new Date().toISOString(),
    };

    console.log("📤 WebSocket: Sending message:", type, data);
    this.ws.send(JSON.stringify(message));
  }

  /**
   * Subscribes to events of specific type
   */
  on(
    eventType: WSEventType,
    handler: (data: TransactionUpdate | RiskAlert | unknown) => void
  ): void {
    if (!this.listeners.has(eventType)) {
      this.listeners.set(eventType, new Set());
    }
    this.listeners.get(eventType)!.add(handler);
    console.log(`🔔 WebSocket: Added listener for ${eventType}`);
  }

  /**
   * Unsubscribes from events
   */
  off(
    eventType: WSEventType,
    handler: (data: TransactionUpdate | RiskAlert | unknown) => void
  ): void {
    const listeners = this.listeners.get(eventType);
    if (listeners) {
      listeners.delete(handler);
      console.log(`🔕 WebSocket: Removed listener for ${eventType}`);
    }
  }

  /**
   * Gets connection status
   */
  get isConnected(): boolean {
    return this.ws?.readyState === WebSocket.OPEN;
  }

  /**
   * Gets connection state
   */
  get connectionState(): string {
    if (!this.ws) return "disconnected";

    switch (this.ws.readyState) {
      case WebSocket.CONNECTING:
        return "connecting";
      case WebSocket.OPEN:
        return "connected";
      case WebSocket.CLOSING:
        return "closing";
      case WebSocket.CLOSED:
        return "disconnected";
      default:
        return "unknown";
    }
  }

  /**
   * Handles incoming messages
   */
  private handleMessage(message: WSMessage): void {
    const listeners = this.listeners.get(message.type as WSEventType);
    if (listeners) {
      listeners.forEach((handler) => {
        try {
          handler(message.data);
        } catch (error) {
          console.error(
            `❌ WebSocket: Error in ${message.type} handler:`,
            error
          );
        }
      });
    } else {
      console.log(
        `📨 WebSocket: No listeners for message type: ${message.type}`
      );
    }
  }

  /**
   * Schedules reconnect
   */
  private scheduleReconnect(): void {
    this.reconnectAttempts++;
    const delay =
      this.reconnectInterval * Math.pow(2, this.reconnectAttempts - 1); // Exponential backoff

    console.log(
      `🔄 WebSocket: Scheduling reconnect attempt ${this.reconnectAttempts}/${this.maxReconnectAttempts} in ${delay}ms`
    );

    setTimeout(() => {
      if (this.reconnectAttempts <= this.maxReconnectAttempts) {
        console.log(
          `🔄 WebSocket: Reconnect attempt ${this.reconnectAttempts}`
        );
        this.connect();
      }
    }, delay);
  }

  /**
   * Subscribes to transaction updates
   */
  subscribeToTransaction(transactionId: string): void {
    this.send("subscribe_transaction", { transaction_id: transactionId });
  }

  /**
   * Unsubscribes from transaction updates
   */
  unsubscribeFromTransaction(transactionId: string): void {
    this.send("unsubscribe_transaction", { transaction_id: transactionId });
  }

  /**
   * Subscribes to system notifications
   */
  subscribeToSystemNotifications(): void {
    this.send("subscribe_system", {});
  }

  /**
   * Unsubscribes from system notifications
   */
  unsubscribeFromSystemNotifications(): void {
    this.send("unsubscribe_system", {});
  }
}

// Create singleton instance
export const wsClient = new WebSocketClient();

// Export for use in components
export default wsClient;
