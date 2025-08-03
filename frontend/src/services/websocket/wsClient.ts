import { WEBSOCKET_CONFIG } from "../../constants/services";

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
  | "price_update"
  | "system_notification"
  | "bridge_operation"
  | "quantum_key_event"
  | "user_profile_update";

class WebSocketClient {
  private ws: WebSocket | null = null;
  private reconnectAttempts = 0;
  private maxReconnectAttempts = WEBSOCKET_CONFIG.MAX_RECONNECT_ATTEMPTS;
  private reconnectInterval = WEBSOCKET_CONFIG.RECONNECT_INTERVAL_MS;
  private listeners: Map<WSEventType, Set<(data: unknown) => void>> = new Map();
  private isConnecting = false;
  private authToken: string | null = null;
  private pingInterval: ReturnType<typeof setInterval> | null = null;
  private lastConnectAttempt = 0;
  private connectionQuality: "excellent" | "good" | "poor" | "unknown" =
    "unknown";
  private connectionMetrics = {
    connected: 0,
    disconnected: 0,
    errors: 0,
    lastPongTime: 0,
    averageLatency: 0,
  };

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

      // Prevent too frequent reconnection attempts
      const now = Date.now();
      if (now - this.lastConnectAttempt < 1000) {
        console.log("⚠️ WebSocket: Rate limiting connection attempts");
        setTimeout(() => this.connect(authToken).catch(console.error), 1000);
        return;
      }
      this.lastConnectAttempt = now;

      this.isConnecting = true;
      this.authToken = authToken || this.authToken;

      try {
        const wsUrl = `${WEBSOCKET_CONFIG.URL}${
          this.authToken ? `?token=${this.authToken}` : ""
        }`;
        console.log("🔌 WebSocket: Connecting to:", wsUrl);

        this.ws = new WebSocket(wsUrl);

        this.ws.onopen = () => {
          console.log("✅ WebSocket: Connected successfully");
          this.isConnecting = false;
          this.reconnectAttempts = 0;
          this.connectionMetrics.connected++;
          this.updateConnectionQuality("excellent");
          this.startHeartbeat();
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
          this.connectionMetrics.disconnected++;
          this.updateConnectionQuality("poor");

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
          this.connectionMetrics.errors++;
          this.updateConnectionQuality("poor");
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

    this.stopHeartbeat();

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
    // Handle ping/pong messages for heartbeat
    if (message.type === "Ping" || message.type === "ping") {
      console.log("💓 WebSocket: Received ping, sending pong");
      this.send("pong", {});
      this.connectionMetrics.lastPongTime = Date.now();
      return;
    }

    if (message.type === "Pong" || message.type === "pong") {
      console.log("💓 WebSocket: Received pong");
      this.connectionMetrics.lastPongTime = Date.now();
      return;
    }

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
      console.warn(
        `Unknown WebSocket message type: ${message.type}`
      );
    }
  }

  /**
   * Schedules reconnect
   */
  private scheduleReconnect(): void {
    this.reconnectAttempts++;
    const delay = Math.min(
      this.reconnectInterval * Math.pow(2, this.reconnectAttempts - 1), // Exponential backoff
      30000 // Max 30 seconds
    );

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
   * Starts heartbeat ping
   */
  private startHeartbeat(): void {
    this.stopHeartbeat(); // Clear any existing heartbeat

    this.pingInterval = setInterval(() => {
      if (this.ws?.readyState === WebSocket.OPEN) {
        this.send("ping", {});
      }
    }, WEBSOCKET_CONFIG.PING_INTERVAL_MS);

    console.log("💓 WebSocket: Heartbeat started");
  }

  /**
   * Stops heartbeat ping
   */
  private stopHeartbeat(): void {
    if (this.pingInterval) {
      clearInterval(this.pingInterval);
      this.pingInterval = null;
      console.log("💓 WebSocket: Heartbeat stopped");
    }
  }

  /**
   * Updates connection quality based on metrics
   */
  private updateConnectionQuality(
    quality: "excellent" | "good" | "poor" | "unknown"
  ): void {
    if (this.connectionQuality !== quality) {
      this.connectionQuality = quality;
      console.log(`📊 WebSocket: Connection quality updated to ${quality}`);
    }
  }

  /**
   * Gets connection metrics for monitoring
   */
  getConnectionMetrics() {
    return {
      ...this.connectionMetrics,
      quality: this.connectionQuality,
      reconnectAttempts: this.reconnectAttempts,
      maxReconnectAttempts: this.maxReconnectAttempts,
    };
  }

  /**
   * Force reconnection (useful for testing)
   */
  forceReconnect(): void {
    console.log("🔄 WebSocket: Force reconnecting...");
    this.disconnect();
    this.reconnectAttempts = 0;
    setTimeout(() => this.connect(), 1000);
  }

  /**
   * Subscribes to event type
   */
  subscribeToEventType(eventType: WSEventType): void {
    this.send("subscribe", { event_type: eventType });
  }

  /**
   * Unsubscribes from event type
   */
  unsubscribeFromEventType(eventType: WSEventType): void {
    this.send("unsubscribe", { event_type: eventType });
  }

  /**
   * Subscribes to transaction updates
   */
  subscribeToTransaction(transactionId: string): void {
    this.subscribeToEventType("transaction_update");
    // Store transaction ID for filtering
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
    this.subscribeToEventType("system_notification");
  }

  /**
   * Unsubscribes from system notifications
   */
  unsubscribeFromSystemNotifications(): void {
    this.unsubscribeFromEventType("system_notification");
  }

  /**
   * Authenticates with JWT token
   */
  authenticate(token: string): void {
    this.send("auth", { token });
  }
}

// Create singleton instance
export const wsClient = new WebSocketClient();

// Export for use in components
export default wsClient;
