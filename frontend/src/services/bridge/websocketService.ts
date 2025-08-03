import { API_CONFIG } from "../api/config";
import {
  transactionService,
  TransactionStatusUpdate,
} from "./transactionService";

export interface WebSocketMessage {
  type: "transaction_update" | "price_update" | "system_status";
  data: unknown;
}

class WebSocketService {
  private ws: WebSocket | null = null;
  private reconnectAttempts = 0;
  private maxReconnectAttempts = 5;
  private reconnectDelay = 1000;
  private subscriptions: Set<string> = new Set();

  /**
   * Connect to WebSocket server
   */
  connect(): Promise<void> {
    return new Promise((resolve, reject) => {
      try {
        this.ws = new WebSocket(API_CONFIG.WS_URL);

        this.ws.onopen = () => {
          console.log("🔌 WebSocket connected");
          this.reconnectAttempts = 0;
          this.resubscribeAll();
          resolve();
        };

        this.ws.onmessage = (event) => {
          try {
            const message: WebSocketMessage = JSON.parse(event.data);
            this.handleMessage(message);
          } catch (error) {
            console.error("Failed to parse WebSocket message:", error);
          }
        };

        this.ws.onclose = (event) => {
          console.log("🔌 WebSocket disconnected:", event.code, event.reason);
          this.handleDisconnection();
        };

        this.ws.onerror = (error) => {
          console.error("🔌 WebSocket error:", error);
          reject(error);
        };
      } catch (error) {
        reject(error);
      }
    });
  }

  /**
   * Disconnect from WebSocket server
   */
  disconnect(): void {
    if (this.ws) {
      this.ws.close();
      this.ws = null;
    }
    this.subscriptions.clear();
  }

  /**
   * Subscribe to transaction updates
   */
  subscribeToTransaction(transactionId: string): void {
    this.subscriptions.add(`transaction:${transactionId}`);

    if (this.ws?.readyState === WebSocket.OPEN) {
      this.ws.send(
        JSON.stringify({
          type: "subscribe",
          channel: "transaction",
          id: transactionId,
        })
      );
    }
  }

  /**
   * Unsubscribe from transaction updates
   */
  unsubscribeFromTransaction(transactionId: string): void {
    this.subscriptions.delete(`transaction:${transactionId}`);

    if (this.ws?.readyState === WebSocket.OPEN) {
      this.ws.send(
        JSON.stringify({
          type: "unsubscribe",
          channel: "transaction",
          id: transactionId,
        })
      );
    }
  }

  /**
   * Check if connected
   */
  isConnected(): boolean {
    return this.ws?.readyState === WebSocket.OPEN;
  }

  /**
   * Handle incoming messages
   */
  private handleMessage(message: WebSocketMessage): void {
    switch (message.type) {
      case "transaction_update":
        this.handleTransactionUpdate(message.data as Record<string, unknown>);
        break;
      case "price_update":
        this.handlePriceUpdate(message.data as Record<string, unknown>);
        break;
      case "system_status":
        this.handleSystemStatus(message.data as Record<string, unknown>);
        break;
      default:
        console.warn("Unknown WebSocket message type:", message.type);
    }
  }

  /**
   * Handle transaction status update
   */
  private handleTransactionUpdate(data: Record<string, unknown>): void {
    const update: TransactionStatusUpdate = {
      id: data.id as string,
      status: data.status as
        | "pending"
        | "confirmed"
        | "completed"
        | "failed"
        | "expired",
      progress: (data.progress as number) || 0,
      currentStep: (data.current_step as string) || "",
      fromTxHash: data.from_tx_hash as string,
      toTxHash: data.to_tx_hash as string,
      errorMessage: data.error_message as string,
    };

    console.log("📡 Transaction update received:", update);
    transactionService.notifyTransactionUpdate(update);
  }

  /**
   * Handle price update
   */
  private handlePriceUpdate(data: Record<string, unknown>): void {
    console.log("📡 Price update received:", data);
    // TODO: Implement price update handling
    // Could emit custom events or update a price store
  }

  /**
   * Handle system status update
   */
  private handleSystemStatus(data: Record<string, unknown>): void {
    console.log("📡 System status update:", data);
    // TODO: Implement system status handling
    // Could show maintenance notices or service alerts
  }

  /**
   * Handle connection loss and reconnection
   */
  private handleDisconnection(): void {
    if (this.reconnectAttempts < this.maxReconnectAttempts) {
      this.reconnectAttempts++;
      console.log(
        `🔄 Attempting to reconnect (${this.reconnectAttempts}/${this.maxReconnectAttempts})...`
      );

      setTimeout(() => {
        this.connect().catch((error) => {
          console.error("Reconnection failed:", error);
        });
      }, this.reconnectDelay * this.reconnectAttempts);
    } else {
      console.error(
        "❌ Max reconnection attempts reached. Please refresh the page."
      );
    }
  }

  /**
   * Resubscribe to all previous subscriptions
   */
  private resubscribeAll(): void {
    this.subscriptions.forEach((subscription) => {
      if (subscription.startsWith("transaction:")) {
        const transactionId = subscription.replace("transaction:", "");
        this.subscribeToTransaction(transactionId);
      }
    });
  }
}

export const websocketService = new WebSocketService();
export default websocketService;
