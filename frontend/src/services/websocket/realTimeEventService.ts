/**
 * Real-time Event Streaming Service
 * Advanced WebSocket event handling with intelligent routing and filtering
 */

import { wsClient, type WSEventType, type TransactionUpdate, type RiskAlert } from './wsClient';

export interface PriceUpdate {
  from_token: string;
  to_token: string;
  price: number;
  change_24h: number;
  timestamp: string;
}

export interface QuantumKeyEvent {
  key_id: string;
  operation: 'rotation' | 'creation' | 'expiration';
  algorithm: string;
  timestamp: string;
}

export interface BridgeOperationEvent {
  operation_id: string;
  status: 'initiated' | 'processing' | 'completed' | 'failed';
  from_chain: string;
  to_chain: string;
  amount: string;
  timestamp: string;
}

export interface UserProfileUpdate {
  user_id: string;
  field: string;
  old_value: unknown;
  new_value: unknown;
  timestamp: string;
}

export interface SystemNotification {
  id: string;
  level: 'info' | 'warning' | 'error' | 'critical';
  title: string;
  message: string;
  category: 'system' | 'security' | 'bridge' | 'ai' | 'quantum';
  timestamp: string;
  actions?: Array<{
    label: string;
    action: string;
    style?: 'primary' | 'secondary' | 'danger';
  }>;
}

export type EventPayload = 
  | TransactionUpdate 
  | RiskAlert 
  | PriceUpdate 
  | QuantumKeyEvent 
  | BridgeOperationEvent 
  | UserProfileUpdate 
  | SystemNotification;

export interface EventSubscription {
  id: string;
  eventType: WSEventType;
  filter?: (payload: EventPayload) => boolean;
  handler: (payload: EventPayload) => void;
  priority: 'low' | 'medium' | 'high' | 'critical';
  rateLimitMs?: number;
  lastTriggered?: number;
}

export interface EventStream {
  id: string;
  name: string;
  subscriptions: Set<string>;
  isActive: boolean;
  buffer: EventPayload[];
  maxBufferSize: number;
}

class RealTimeEventService {
  private subscriptions: Map<string, EventSubscription> = new Map();
  private streams: Map<string, EventStream> = new Map();
  private eventBuffer: Array<{ event: EventPayload; timestamp: number }> = [];
  private maxBufferSize = 1000;
  private eventStats = {
    totalEvents: 0,
    eventsPerType: new Map<WSEventType, number>(),
    lastEventTime: 0,
    droppedEvents: 0,
  };

  constructor() {
    console.log('🌊 RealTimeEventService: Initializing');
    this.initializeDefaultStreams();
    this.setupGlobalEventHandlers();
  }

  /**
   * Initialize default event streams
   */
  private initializeDefaultStreams(): void {
    // Transaction stream for bridge operations
    this.createStream('transactions', 'Transaction Updates', 100);
    
    // Security stream for risk alerts and quantum events
    this.createStream('security', 'Security Events', 50);
    
    // System stream for general notifications
    this.createStream('system', 'System Notifications', 200);
    
    // Price stream for market data
    this.createStream('prices', 'Price Updates', 500);
    
    console.log('🌊 Created default event streams');
  }

  /**
   * Setup global event handlers for all WebSocket events
   */
  private setupGlobalEventHandlers(): void {
    // Transaction updates
    wsClient.on('transaction_update', (data) => {
      this.processEvent('transaction_update', data as TransactionUpdate);
    });

    // Risk alerts
    wsClient.on('risk_alert', (data) => {
      this.processEvent('risk_alert', data as RiskAlert);
    });

    // Price updates
    wsClient.on('price_update', (data) => {
      this.processEvent('price_update', data as PriceUpdate);
    });

    // Quantum key events
    wsClient.on('quantum_key_event', (data) => {
      this.processEvent('quantum_key_event', data as QuantumKeyEvent);
    });

    // Bridge operations
    wsClient.on('bridge_operation', (data) => {
      this.processEvent('bridge_operation', data as BridgeOperationEvent);
    });

    // User profile updates
    wsClient.on('user_profile_update', (data) => {
      this.processEvent('user_profile_update', data as UserProfileUpdate);
    });

    // System notifications
    wsClient.on('system_notification', (data) => {
      this.processEvent('system_notification', data as SystemNotification);
    });

    console.log('🌊 Global event handlers setup complete');
  }

  /**
   * Process incoming events with intelligent routing
   */
  private processEvent(eventType: WSEventType, payload: EventPayload): void {
    const now = Date.now();
    
    // Update statistics
    this.eventStats.totalEvents++;
    this.eventStats.eventsPerType.set(
      eventType, 
      (this.eventStats.eventsPerType.get(eventType) || 0) + 1
    );
    this.eventStats.lastEventTime = now;

    // Add to buffer
    this.addToBuffer({ event: payload, timestamp: now });

    // Route to appropriate streams
    this.routeEventToStreams(eventType, payload);

    // Process subscriptions
    this.processSubscriptions(eventType, payload, now);

    console.log(`🌊 Processed ${eventType} event:`, payload);
  }

  /**
   * Add event to global buffer with size management
   */
  private addToBuffer(item: { event: EventPayload; timestamp: number }): void {
    this.eventBuffer.push(item);
    
    // Maintain buffer size
    if (this.eventBuffer.length > this.maxBufferSize) {
      this.eventBuffer.shift();
      this.eventStats.droppedEvents++;
    }
  }

  /**
   * Route events to appropriate streams
   */
  private routeEventToStreams(eventType: WSEventType, payload: EventPayload): void {
    let targetStream: string;

    switch (eventType) {
      case 'transaction_update':
      case 'bridge_operation':
        targetStream = 'transactions';
        break;
      case 'risk_alert':
      case 'quantum_key_event':
        targetStream = 'security';
        break;
      case 'price_update':
        targetStream = 'prices';
        break;
      case 'system_notification':
      case 'user_profile_update':
      default:
        targetStream = 'system';
        break;
    }

    const stream = this.streams.get(targetStream);
    if (stream && stream.isActive) {
      stream.buffer.push(payload);
      
      // Maintain stream buffer size
      if (stream.buffer.length > stream.maxBufferSize) {
        stream.buffer.shift();
      }
    }
  }

  /**
   * Process active subscriptions with rate limiting
   */
  private processSubscriptions(eventType: WSEventType, payload: EventPayload, now: number): void {
    for (const subscription of this.subscriptions.values()) {
      if (subscription.eventType !== eventType) continue;

      // Check rate limiting
      if (subscription.rateLimitMs && subscription.lastTriggered) {
        if (now - subscription.lastTriggered < subscription.rateLimitMs) {
          continue; // Skip due to rate limiting
        }
      }

      // Apply filter if present
      if (subscription.filter && !subscription.filter(payload)) {
        continue; // Skip due to filter
      }

      try {
        subscription.handler(payload);
        subscription.lastTriggered = now;
      } catch (error) {
        console.error(`🌊 Error in subscription ${subscription.id}:`, error);
      }
    }
  }

  /**
   * Create a new event stream
   */
  createStream(id: string, name: string, maxBufferSize = 100): EventStream {
    const stream: EventStream = {
      id,
      name,
      subscriptions: new Set(),
      isActive: true,
      buffer: [],
      maxBufferSize,
    };

    this.streams.set(id, stream);
    console.log(`🌊 Created stream: ${name} (${id})`);
    return stream;
  }

  /**
   * Subscribe to specific event type with advanced options
   */
  subscribe(
    eventType: WSEventType,
    handler: (payload: EventPayload) => void,
    options: Partial<Pick<EventSubscription, 'filter' | 'priority' | 'rateLimitMs'>> = {}
  ): string {
    const subscriptionId = `sub_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
    
    const subscription: EventSubscription = {
      id: subscriptionId,
      eventType,
      handler,
      priority: options.priority || 'medium',
      filter: options.filter,
      rateLimitMs: options.rateLimitMs,
    };

    this.subscriptions.set(subscriptionId, subscription);
    
    // Subscribe to WebSocket event type if not already subscribed
    wsClient.subscribeToEventType(eventType);
    
    console.log(`🌊 Created subscription ${subscriptionId} for ${eventType}`);
    return subscriptionId;
  }

  /**
   * Unsubscribe from events
   */
  unsubscribe(subscriptionId: string): boolean {
    const subscription = this.subscriptions.get(subscriptionId);
    if (!subscription) return false;

    this.subscriptions.delete(subscriptionId);
    
    // Check if we need to unsubscribe from WebSocket event type
    const hasOtherSubscriptions = Array.from(this.subscriptions.values())
      .some(sub => sub.eventType === subscription.eventType);
    
    if (!hasOtherSubscriptions) {
      wsClient.unsubscribeFromEventType(subscription.eventType);
    }

    console.log(`🌊 Removed subscription ${subscriptionId}`);
    return true;
  }

  /**
   * Subscribe to a specific stream
   */
  subscribeToStream(streamId: string, subscriptionId: string): boolean {
    const stream = this.streams.get(streamId);
    if (!stream) return false;

    stream.subscriptions.add(subscriptionId);
    console.log(`🌊 Added subscription ${subscriptionId} to stream ${streamId}`);
    return true;
  }

  /**
   * Get events from a specific stream
   */
  getStreamEvents(streamId: string, limit = 50): EventPayload[] {
    const stream = this.streams.get(streamId);
    if (!stream) return [];

    return stream.buffer.slice(-limit);
  }

  /**
   * Get recent events from global buffer
   */
  getRecentEvents(limit = 100, eventType?: WSEventType): EventPayload[] {
    let events = this.eventBuffer.slice(-limit);

    if (eventType) {
      // This is a simplified filter - in reality we'd need type info
      events = events.filter(() => {
        // Check if event matches type (simplified)
        return true; // Would need proper type checking
      });
    }

    return events.map(item => item.event);
  }

  /**
   * Get service statistics
   */
  getStatistics() {
    return {
      ...this.eventStats,
      activeSubscriptions: this.subscriptions.size,
      activeStreams: Array.from(this.streams.values()).filter(s => s.isActive).length,
      bufferSize: this.eventBuffer.length,
      connectionMetrics: wsClient.getConnectionMetrics(),
    };
  }

  /**
   * Enable/disable a stream
   */
  setStreamActive(streamId: string, active: boolean): boolean {
    const stream = this.streams.get(streamId);
    if (!stream) return false;

    stream.isActive = active;
    console.log(`🌊 Stream ${streamId} ${active ? 'activated' : 'deactivated'}`);
    return true;
  }

  /**
   * Clear stream buffer
   */
  clearStreamBuffer(streamId: string): boolean {
    const stream = this.streams.get(streamId);
    if (!stream) return false;

    stream.buffer.length = 0;
    console.log(`🌊 Cleared buffer for stream ${streamId}`);
    return true;
  }

  /**
   * Clear all events and reset
   */
  reset(): void {
    this.eventBuffer.length = 0;
    this.subscriptions.clear();
    
    // Reset all stream buffers
    for (const stream of this.streams.values()) {
      stream.buffer.length = 0;
      stream.subscriptions.clear();
    }

    // Reset statistics
    this.eventStats = {
      totalEvents: 0,
      eventsPerType: new Map(),
      lastEventTime: 0,
      droppedEvents: 0,
    };

    console.log('🌊 RealTimeEventService reset');
  }
}

// Create singleton instance
export const realTimeEventService = new RealTimeEventService();

// Export for use in components
export default realTimeEventService;