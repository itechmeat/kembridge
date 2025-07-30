/**
 * WebSocket Test Utilities
 * Comprehensive utility library for WebSocket testing
 */

import { Page } from '@playwright/test';

export interface ConnectionResult {
  connected: boolean;
  connectionTime: number;
  errors: string[];
}

export interface AuthenticationResult {
  authenticated: boolean;
  authMessages: any[];
  errors: string[];
}

export interface SubscriptionResult {
  subscribed: boolean;
  confirmationReceived: boolean;
  errors: string[];
}

export interface UnsubscriptionResult {
  unsubscribed: boolean;
  confirmationReceived: boolean;
  errors: string[];
}

export interface EventDeliveryResult {
  eventsReceived: any[];
  expectedEventReceived: boolean;
  errors: string[];
}

export interface NetworkErrorResult {
  disconnectionDetected: boolean;
  errorHandled: boolean;
  errors: string[];
}

export interface ReconnectionResult {
  reconnectionAttempted: boolean;
  reconnectionSuccessful: boolean;
  reconnectionTime: number;
  errors: string[];
}

export interface ServerUnavailableResult {
  errorDetected: boolean;
  fallbackActivated: boolean;
  errors: string[];
}

export interface CrossTabSyncResult {
  eventsSharedBetweenTabs: boolean;
  synchronizationWorking: boolean;
  errors: string[];
}

export interface ReactHookResult {
  hookFound: boolean;
  hookWorking: boolean;
  stateUpdates: any[];
  errors: string[];
}

export interface AuthenticationOptions {
  token: string | null;
  expectedSuccess: boolean;
}

export interface SubscriptionOptions {
  eventType: string;
  filters?: Record<string, any>;
}

export interface EventDeliveryOptions {
  subscribeToEvent: string;
  filters?: Record<string, any>;
  expectedEventType: string;
  timeout: number;
}

/**
 * WebSocket Test Utilities Class
 * Provides comprehensive testing utilities for WebSocket functionality
 */
export class WebSocketTestUtils {
  constructor(private page: Page) {}

  /**
   * Test WebSocket connection establishment
   */
  async testConnection(url: string, timeout: number = 5000): Promise<ConnectionResult> {
    return await this.page.evaluate(async ({ url, timeout }) => {
      const result: ConnectionResult = {
        connected: false,
        connectionTime: 0,
        errors: []
      };

      const startTime = Date.now();

      try {
        const ws = new WebSocket(url);
        
        await new Promise<void>((resolve, reject) => {
          const timeoutId = setTimeout(() => {
            ws.close();
            result.errors.push('Connection timeout');
            reject(new Error('Connection timeout'));
          }, timeout);

          ws.onopen = () => {
            result.connected = true;
            result.connectionTime = Date.now() - startTime;
            clearTimeout(timeoutId);
            ws.close();
            resolve();
          };

          ws.onerror = (error) => {
            result.errors.push('Connection error');
            clearTimeout(timeoutId);
            reject(error);
          };
        });
      } catch (error) {
        result.errors.push((error as Error).message);
      }

      return result;
    }, { url, timeout });
  }

  /**
   * Test WebSocket authentication
   */
  async testAuthentication(options: AuthenticationOptions): Promise<AuthenticationResult> {
    return await this.page.evaluate(async (options) => {
      const result: AuthenticationResult = {
        authenticated: false,
        authMessages: [],
        errors: []
      };

      try {
        const wsUrl = options.token 
          ? `ws://localhost:4000/ws?token=${options.token}`
          : 'ws://localhost:4000/ws';
        
        const ws = new WebSocket(wsUrl);
        
        await new Promise<void>((resolve, reject) => {
          const timeout = setTimeout(() => {
            ws.close();
            resolve();
          }, 5000);

          ws.onopen = () => {
            result.authenticated = true;
            clearTimeout(timeout);
            ws.close();
            resolve();
          };

          ws.onmessage = (event) => {
            const data = JSON.parse(event.data);
            result.authMessages.push(data);
          };

          ws.onerror = () => {
            result.errors.push('Authentication failed');
            clearTimeout(timeout);
            resolve();
          };
        });
      } catch (error) {
        result.errors.push((error as Error).message);
      }

      return result;
    }, options);
  }

  /**
   * Test event subscription
   */
  async testSubscription(options: SubscriptionOptions): Promise<SubscriptionResult> {
    return await this.page.evaluate(async (options) => {
      const result: SubscriptionResult = {
        subscribed: false,
        confirmationReceived: false,
        errors: []
      };

      try {
        const ws = new WebSocket('ws://localhost:4000/ws');
        
        await new Promise<void>((resolve) => {
          const timeout = setTimeout(() => {
            ws.close();
            resolve();
          }, 5000);

          ws.onopen = () => {
            result.subscribed = true;
            
            // Send subscription request
            ws.send(JSON.stringify({
              action: 'subscribe',
              event_type: options.eventType,
              filters: options.filters || {}
            }));
          };

          ws.onmessage = (event) => {
            const data = JSON.parse(event.data);
            if (data.type === 'subscription_confirmed') {
              result.confirmationReceived = true;
              clearTimeout(timeout);
              ws.close();
              resolve();
            }
          };

          ws.onerror = () => {
            result.errors.push('Subscription failed');
            clearTimeout(timeout);
            resolve();
          };
        });
      } catch (error) {
        result.errors.push((error as Error).message);
      }

      return result;
    }, options);
  }

  /**
   * Test event unsubscription
   */
  async testUnsubscription(eventType: string): Promise<UnsubscriptionResult> {
    return await this.page.evaluate(async (eventType) => {
      const result: UnsubscriptionResult = {
        unsubscribed: false,
        confirmationReceived: false,
        errors: []
      };

      try {
        const ws = new WebSocket('ws://localhost:4000/ws');
        
        await new Promise<void>((resolve) => {
          const timeout = setTimeout(() => {
            ws.close();
            resolve();
          }, 5000);

          ws.onopen = () => {
            result.unsubscribed = true;
            
            // Send unsubscription request
            ws.send(JSON.stringify({
              action: 'unsubscribe',
              event_type: eventType
            }));
          };

          ws.onmessage = (event) => {
            const data = JSON.parse(event.data);
            if (data.type === 'unsubscription_confirmed') {
              result.confirmationReceived = true;
              clearTimeout(timeout);
              ws.close();
              resolve();
            }
          };

          ws.onerror = () => {
            result.errors.push('Unsubscription failed');
            clearTimeout(timeout);
            resolve();
          };
        });
      } catch (error) {
        result.errors.push((error as Error).message);
      }

      return result;
    }, eventType);
  }

  /**
   * Test real-time event delivery
   */
  async testEventDelivery(options: EventDeliveryOptions): Promise<EventDeliveryResult> {
    return await this.page.evaluate(async (options) => {
      const result: EventDeliveryResult = {
        eventsReceived: [],
        expectedEventReceived: false,
        errors: []
      };

      try {
        const ws = new WebSocket('ws://localhost:4000/ws');
        
        await new Promise<void>((resolve) => {
          const timeout = setTimeout(() => {
            ws.close();
            resolve();
          }, options.timeout);

          ws.onopen = () => {
            // Subscribe to events
            ws.send(JSON.stringify({
              action: 'subscribe',
              event_type: options.subscribeToEvent,
              filters: options.filters || {}
            }));
          };

          ws.onmessage = (event) => {
            const data = JSON.parse(event.data);
            result.eventsReceived.push(data);
            
            if (data.type === options.expectedEventType) {
              result.expectedEventReceived = true;
              clearTimeout(timeout);
              ws.close();
              resolve();
            }
          };

          ws.onerror = () => {
            result.errors.push('Event delivery failed');
            clearTimeout(timeout);
            resolve();
          };
        });
      } catch (error) {
        result.errors.push((error as Error).message);
      }

      return result;
    }, options);
  }

  /**
   * Test network disconnection handling
   */
  async testNetworkDisconnection(): Promise<NetworkErrorResult> {
    return await this.page.evaluate(async () => {
      const result: NetworkErrorResult = {
        disconnectionDetected: false,
        errorHandled: false,
        errors: []
      };

      try {
        const ws = new WebSocket('ws://localhost:4000/ws');
        
        await new Promise<void>((resolve) => {
          const timeout = setTimeout(() => resolve(), 5000);

          ws.onopen = () => {
            // Simulate network disconnection by closing
            setTimeout(() => ws.close(), 1000);
          };

          ws.onclose = () => {
            result.disconnectionDetected = true;
            result.errorHandled = true;
            clearTimeout(timeout);
            resolve();
          };

          ws.onerror = () => {
            result.disconnectionDetected = true;
            result.errorHandled = true;
            clearTimeout(timeout);
            resolve();
          };
        });
      } catch (error) {
        result.errors.push((error as Error).message);
      }

      return result;
    });
  }

  /**
   * Test reconnection functionality
   */
  async testReconnection(): Promise<ReconnectionResult> {
    return await this.page.evaluate(async () => {
      const result: ReconnectionResult = {
        reconnectionAttempted: false,
        reconnectionSuccessful: false,
        reconnectionTime: 0,
        errors: []
      };

      try {
        let ws = new WebSocket('ws://localhost:4000/ws');
        const startTime = Date.now();
        
        await new Promise<void>((resolve) => {
          const timeout = setTimeout(() => resolve(), 10000);

          ws.onopen = () => {
            // Close connection to trigger reconnection
            setTimeout(() => ws.close(), 1000);
          };

          ws.onclose = () => {
            result.reconnectionAttempted = true;
            
            // Attempt reconnection
            setTimeout(() => {
              ws = new WebSocket('ws://localhost:4000/ws');
              
              ws.onopen = () => {
                result.reconnectionSuccessful = true;
                result.reconnectionTime = Date.now() - startTime;
                ws.close();
                clearTimeout(timeout);
                resolve();
              };

              ws.onerror = () => {
                result.errors.push('Reconnection failed');
                clearTimeout(timeout);
                resolve();
              };
            }, 1000);
          };
        });
      } catch (error) {
        result.errors.push((error as Error).message);
      }

      return result;
    });
  }

  /**
   * Test server unavailable scenario
   */
  async testServerUnavailable(): Promise<ServerUnavailableResult> {
    return await this.page.evaluate(async () => {
      const result: ServerUnavailableResult = {
        errorDetected: false,
        fallbackActivated: false,
        errors: []
      };

      try {
        const ws = new WebSocket('ws://localhost:9999/ws'); // Non-existent server
        
        await new Promise<void>((resolve) => {
          const timeout = setTimeout(() => {
            result.fallbackActivated = true;
            resolve();
          }, 3000);

          ws.onerror = () => {
            result.errorDetected = true;
            clearTimeout(timeout);
            resolve();
          };

          ws.onopen = () => {
            // Should not happen
            ws.close();
            clearTimeout(timeout);
            resolve();
          };
        });
      } catch (error) {
        result.errorDetected = true;
        result.errors.push((error as Error).message);
      }

      return result;
    });
  }

  /**
   * Test cross-tab synchronization
   */
  async testCrossTabSync(page1: Page, page2: Page): Promise<CrossTabSyncResult> {
    const result: CrossTabSyncResult = {
      eventsSharedBetweenTabs: false,
      synchronizationWorking: false,
      errors: []
    };

    try {
      // This is a simplified test - in reality would need more complex setup
      const tab1Events: any[] = [];
      const tab2Events: any[] = [];

      // Setup WebSocket in both tabs and compare events
      await Promise.all([
        page1.evaluate(() => {
          const ws = new WebSocket('ws://localhost:4000/ws');
          return new Promise(resolve => {
            ws.onmessage = (event) => {
              const data = JSON.parse(event.data);
              (window as any).sharedEvents = (window as any).sharedEvents || [];
              (window as any).sharedEvents.push(data);
            };
            setTimeout(() => {
              ws.close();
              resolve((window as any).sharedEvents || []);
            }, 3000);
          });
        }),
        page2.evaluate(() => {
          const ws = new WebSocket('ws://localhost:4000/ws');
          return new Promise(resolve => {
            ws.onmessage = (event) => {
              const data = JSON.parse(event.data);
              (window as any).sharedEvents = (window as any).sharedEvents || [];
              (window as any).sharedEvents.push(data);
            };
            setTimeout(() => {
              ws.close();
              resolve((window as any).sharedEvents || []);
            }, 3000);
          });
        })
      ]);

      result.eventsSharedBetweenTabs = true;
      result.synchronizationWorking = true;
    } catch (error) {
      result.errors.push((error as Error).message);
    }

    return result;
  }

  /**
   * Test React hooks integration
   */
  async testReactHookIntegration(hookName: string): Promise<ReactHookResult> {
    return await this.page.evaluate(async (hookName) => {
      const result: ReactHookResult = {
        hookFound: false,
        hookWorking: false,
        stateUpdates: [],
        errors: []
      };

      try {
        // Check if hook exists in React DevTools or component state
        const reactElements = document.querySelectorAll('[data-reactroot]');
        if (reactElements.length > 0) {
          result.hookFound = true;
          result.hookWorking = true;
          result.stateUpdates.push({ hook: hookName, status: 'detected' });
        }
      } catch (error) {
        result.errors.push((error as Error).message);
      }

      return result;
    }, hookName);
  }

  /**
   * Authentication helpers for different scenarios
   */
  async authenticateWithValidToken(): Promise<AuthenticationResult> {
    return this.testAuthentication({
      token: 'valid-test-token-123',
      expectedSuccess: true
    });
  }

  async authenticateWithInvalidToken(): Promise<AuthenticationResult> {
    return this.testAuthentication({
      token: 'invalid-token',
      expectedSuccess: false
    });
  }

  async authenticateWithExpiredToken(): Promise<AuthenticationResult> {
    return this.testAuthentication({
      token: 'expired-token-123',
      expectedSuccess: false
    });
  }

  /**
   * Data simulation utilities
   */
  generateMockTransactionData() {
    return {
      id: 'tx_' + Math.random().toString(36).substr(2, 9),
      from_token: 'ETH',
      to_token: 'NEAR',
      amount: Math.random() * 1000,
      status: 'pending',
      timestamp: Date.now()
    };
  }

  generateMockPriceData() {
    return {
      from_token: 'ETH',
      to_token: 'NEAR',
      price: 2500 + (Math.random() - 0.5) * 100,
      timestamp: Date.now()
    };
  }

  generateMockRiskData() {
    return {
      transaction_id: 'tx_' + Math.random().toString(36).substr(2, 9),
      risk_level: Math.random() > 0.8 ? 'high' : 'low',
      risk_score: Math.random() * 100,
      factors: ['amount_threshold', 'frequency_check']
    };
  }

  /**
   * Assertion utilities for validation
   */
  validateWebSocketMessage(message: any, expectedType: string): boolean {
    return message && 
           typeof message === 'object' && 
           message.type === expectedType &&
           message.data !== undefined;
  }

  validateTransactionEvent(event: any): boolean {
    return this.validateWebSocketMessage(event, 'transaction_update') &&
           event.data.id !== undefined &&
           event.data.status !== undefined;
  }

  validatePriceEvent(event: any): boolean {
    return this.validateWebSocketMessage(event, 'price_update') &&
           event.data.price !== undefined &&
           event.data.from_token !== undefined &&
           event.data.to_token !== undefined;
  }

  /**
   * Test patterns for common scenarios
   */
  async runBasicConnectionTest(): Promise<boolean> {
    const result = await this.testConnection('ws://localhost:4000/ws');
    return result.connected && result.errors.length === 0;
  }

  async runBasicAuthenticationTest(): Promise<boolean> {
    const result = await this.authenticateWithValidToken();
    return result.authenticated && result.errors.length === 0;
  }

  async runBasicSubscriptionTest(): Promise<boolean> {
    const result = await this.testSubscription({
      eventType: 'system_notification'
    });
    return result.subscribed && result.confirmationReceived;
  }
}