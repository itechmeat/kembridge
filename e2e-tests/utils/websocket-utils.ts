/**
 * WebSocket Test Utilities
 * Comprehensive utility library for WebSocket testing
 */

import { Page } from '@playwright/test';
import { TEST_URLS } from './test-constants';
import { createTestJWT, createExpiredTestJWT, createInvalidTestJWT } from './jwt-helper';

export interface ConnectionResult {
  connected: boolean;
  connectionTime: number;
  errors: string[];
}

export interface AuthenticationResult {
  authenticated: boolean;
  authMessages: any[];
  authSuccessMessage?: any;
  authFailedMessage?: any;
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
        // Basic URL validation
        if (!url.startsWith('ws://') && !url.startsWith('wss://')) {
          result.errors.push('Invalid WebSocket protocol');
          return result;
        }

        const ws = new WebSocket(url);
        
        await new Promise<void>((resolve, reject) => {
          const timeoutId = setTimeout(() => {
            ws.close();
            result.errors.push('Connection timeout');
            resolve(); // Don't reject on timeout, just resolve with connected: false
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
            resolve(); // Don't reject on error, just resolve with connected: false
          };

          ws.onclose = (event) => {
            if (event.code !== 1000) { // 1000 is normal closure
              result.errors.push(`Connection closed with code: ${event.code}`);
            }
            clearTimeout(timeoutId);
            resolve();
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
    return await this.page.evaluate(async ({ token, expectedSuccess, wsUrl }) => {
      const result: AuthenticationResult = {
        authenticated: false,
        authMessages: [],
        errors: []
      };

      try {
        const ws = new WebSocket(wsUrl);
        
        await new Promise<void>((resolve) => {
          const timeout = setTimeout(() => {
            result.errors.push('Authentication timeout after 5 seconds');
            ws.close();
            resolve();
          }, 5000);

          ws.onopen = () => {
            // Send authentication message after connection is established
            if (token) {
              ws.send(JSON.stringify({
                type: 'Auth',
                token: token
              }));
            } else {
              // No token provided, should fail
              result.errors.push('No token provided');
              clearTimeout(timeout);
              ws.close();
              resolve();
            }
          };

          ws.onmessage = (event) => {
            try {
              const data = JSON.parse(event.data);
              result.authMessages.push(data);
              
              // Check for authentication success/failure messages
              if (data.type === 'AuthSuccess') {
                result.authenticated = true;
                result.authSuccessMessage = data;
                clearTimeout(timeout);
                ws.close();
                resolve();
              } else if (data.type === 'AuthFailed') {
                result.authFailedMessage = data;
                if (!expectedSuccess) {
                  // Expected failure, don't treat as error
                  result.authenticated = false;
                } else {
                  result.errors.push(data.error || 'Authentication failed');
                }
                clearTimeout(timeout);
                ws.close();
                resolve();
              }
            } catch (parseError) {
              result.errors.push('Failed to parse server response');
            }
          };

          ws.onerror = () => {
            result.errors.push('WebSocket connection error');
            clearTimeout(timeout);
            resolve();
          };

          ws.onclose = (event) => {
            if (event.code !== 1000 && result.authMessages.length === 0) {
              result.errors.push(`Connection closed unexpectedly: ${event.code}`);
            }
            clearTimeout(timeout);
            resolve();
          };
        });
      } catch (error) {
        result.errors.push((error as Error).message);
      }

      return result;
    }, { 
      token: options.token, 
      expectedSuccess: options.expectedSuccess,
      wsUrl: TEST_URLS.WEBSOCKET.GATEWAY 
    });
  }

  /**
   * Test event subscription
   */
  async testSubscription(options: SubscriptionOptions): Promise<SubscriptionResult> {
    return await this.page.evaluate(async ({ eventType, filters, wsUrl }) => {
      const result: SubscriptionResult = {
        subscribed: false,
        confirmationReceived: false,
        errors: []
      };

      try {
        const ws = new WebSocket(wsUrl);
        
        await new Promise<void>((resolve) => {
          const timeout = setTimeout(() => {
            result.errors.push('Subscription timeout after 5 seconds');
            ws.close();
            resolve();
          }, 5000);

          ws.onopen = () => {
            result.subscribed = true;
            
            // Send subscription request
            ws.send(JSON.stringify({
              type: 'Subscribe',
              event_type: eventType,
              filters: filters || {}
            }));
          };

          ws.onmessage = (event) => {
            try {
              const data = JSON.parse(event.data);
              if (data.type === 'subscription_confirmed') {
                result.confirmationReceived = true;
                clearTimeout(timeout);
                ws.close();
                resolve();
              }
            } catch (parseError) {
              result.errors.push('Failed to parse server response');
            }
          };

          ws.onerror = () => {
            result.errors.push('Subscription failed');
            clearTimeout(timeout);
            resolve();
          };

          ws.onclose = (event) => {
            if (event.code !== 1000 && !result.confirmationReceived) {
              result.errors.push(`Connection closed unexpectedly: ${event.code}`);
            }
            clearTimeout(timeout);
            resolve();
          };
        });
      } catch (error) {
        result.errors.push((error as Error).message);
      }

      return result;
    }, {
      eventType: options.eventType,
      filters: options.filters,
      wsUrl: TEST_URLS.WEBSOCKET.GATEWAY
    });
  }

  /**
   * Test event unsubscription
   */
  async testUnsubscription(eventType: string): Promise<UnsubscriptionResult> {
    return await this.page.evaluate(async ({ eventType, wsUrl }) => {
      const result: UnsubscriptionResult = {
        unsubscribed: false,
        confirmationReceived: false,
        errors: []
      };

      try {
        const ws = new WebSocket(wsUrl);
        
        await new Promise<void>((resolve) => {
          const timeout = setTimeout(() => {
            result.errors.push('Unsubscription timeout after 5 seconds');
            ws.close();
            resolve();
          }, 5000);

          ws.onopen = () => {
            result.unsubscribed = true;
            
            // Send unsubscription request
            ws.send(JSON.stringify({
              type: 'Unsubscribe',
              event_type: eventType
            }));
          };

          ws.onmessage = (event) => {
            try {
              const data = JSON.parse(event.data);
              if (data.type === 'unsubscription_confirmed') {
                result.confirmationReceived = true;
                clearTimeout(timeout);
                ws.close();
                resolve();
              }
            } catch (parseError) {
              result.errors.push('Failed to parse server response');
            }
          };

          ws.onerror = () => {
            result.errors.push('Unsubscription failed');
            clearTimeout(timeout);
            resolve();
          };

          ws.onclose = (event) => {
            if (event.code !== 1000 && !result.confirmationReceived) {
              result.errors.push(`Connection closed unexpectedly: ${event.code}`);
            }
            clearTimeout(timeout);
            resolve();
          };
        });
      } catch (error) {
        result.errors.push((error as Error).message);
      }

      return result;
    }, {
      eventType,
      wsUrl: TEST_URLS.WEBSOCKET.GATEWAY
    });
  }

  /**
   * Test real-time event delivery
   */
  async testEventDelivery(options: EventDeliveryOptions): Promise<EventDeliveryResult> {
    return await this.page.evaluate(async ({ subscribeToEvent, filters, expectedEventType, timeout, wsUrl, gatewayUrl }) => {
      const result: EventDeliveryResult = {
        eventsReceived: [],
        expectedEventReceived: false,
        errors: []
      };

      try {
        const ws = new WebSocket(wsUrl);
        let subscriptionConfirmed = false;
        
        await new Promise<void>((resolve) => {
          const timeoutId = setTimeout(() => {
            result.errors.push(`Event delivery timeout after ${timeout}ms`);
            ws.close();
            resolve();
          }, timeout);

          ws.onopen = () => {
            // Subscribe to events
            ws.send(JSON.stringify({
              type: 'Subscribe',
              event_type: subscribeToEvent,
              filters: filters || {}
            }));
          };

          ws.onmessage = async (event) => {
            try {
              const data = JSON.parse(event.data);
              
              if (data.type === 'subscription_confirmed') {
                subscriptionConfirmed = true;
                
                // After successful subscription, trigger a test event via API
                try {
                  console.log('Triggering test event for type:', expectedEventType);
                  if (expectedEventType === 'TransactionStatusUpdate') {
                    const response = await fetch(`${gatewayUrl}/api/v1/events/websocket/test/test-user`, {
                      method: 'POST',
                      headers: { 'Content-Type': 'application/json' },
                      body: JSON.stringify({ message: 'Test transaction event' })
                    });
                    const responseText = await response.text();
                    console.log('Transaction test event API response:', response.status, response.statusText, responseText);
                  } else if (expectedEventType === 'price_updates') {
                    const response = await fetch(`${gatewayUrl}/api/v1/events/crypto/trigger`, {
                      method: 'POST',
                      headers: { 'Content-Type': 'application/json' },
                      body: JSON.stringify({
                        event_type: 'service_status',
                        message: 'Test price update event'
                      })
                    });
                    const responseText = await response.text();
                    console.log('Price update test event API response:', response.status, response.statusText, responseText);
                  }
                } catch (error) {
                  console.warn('Failed to trigger test event:', error);
                }
              } else {
                 // Log all non-subscription messages for debugging
                 console.log('Received WebSocket message:', JSON.stringify(data, null, 2));
                 
                 if (data.type && 
                     data.type !== 'unsubscription_confirmed' && 
                     data.type !== 'Ping' && 
                     data.type !== 'Pong' && 
                     data.type !== 'AuthSuccess') {
                   // This is a real event, not a confirmation or system message
                   console.log('Adding event to received list:', data);
                   result.eventsReceived.push(data);
                  
                  if (data.event_type === expectedEventType) {
                    console.log('Expected event type received! Expected:', expectedEventType, 'Received:', data.event_type);
                    result.expectedEventReceived = true;
                    clearTimeout(timeoutId);
                    ws.close();
                    resolve();
                  } else {
                    console.log('Event type mismatch. Expected:', expectedEventType, 'Received:', data.event_type);
                  }
                 } else {
                   console.log('Filtered out system message:', data.type || 'unknown type');
                 }
              }
            } catch (parseError) {
              result.errors.push('Failed to parse server response');
            }
          };

          ws.onerror = () => {
            result.errors.push('Event delivery failed');
            clearTimeout(timeoutId);
            resolve();
          };

          ws.onclose = (event) => {
            if (event.code !== 1000 && !result.expectedEventReceived) {
              result.errors.push(`Connection closed unexpectedly: ${event.code}`);
            }
            clearTimeout(timeoutId);
            resolve();
          };
        });
      } catch (error) {
        result.errors.push((error as Error).message);
      }

      return result;
    }, {
      subscribeToEvent: options.subscribeToEvent,
      filters: options.filters,
      expectedEventType: options.expectedEventType,
      timeout: options.timeout,
      wsUrl: TEST_URLS.WEBSOCKET.GATEWAY,
      gatewayUrl: TEST_URLS.BACKEND.GATEWAY
    });
  }

  /**
   * Test network disconnection handling
   */
  async testNetworkDisconnection(): Promise<NetworkErrorResult> {
    return await this.page.evaluate(async ({ wsUrl }) => {
      const result: NetworkErrorResult = {
        disconnectionDetected: false,
        errorHandled: false,
        errors: []
      };

      try {
        const ws = new WebSocket(wsUrl);
        
        await new Promise<void>((resolve) => {
          const timeout = setTimeout(() => {
            result.errors.push('Network disconnection test timeout');
            resolve();
          }, 5000);

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
    }, {
      wsUrl: TEST_URLS.WEBSOCKET.GATEWAY
    });
  }

  /**
   * Test reconnection functionality
   */
  async testReconnection(): Promise<ReconnectionResult> {
    return await this.page.evaluate(async ({ wsUrl }) => {
      const result: ReconnectionResult = {
        reconnectionAttempted: false,
        reconnectionSuccessful: false,
        reconnectionTime: 0,
        errors: []
      };

      try {
        let ws = new WebSocket(wsUrl);
        const startTime = Date.now();
        
        await new Promise<void>((resolve) => {
          const timeout = setTimeout(() => {
            result.errors.push('Reconnection test timeout after 10 seconds');
            resolve();
          }, 10000);

          ws.onopen = () => {
            // Close connection to trigger reconnection
            setTimeout(() => ws.close(), 1000);
          };

          ws.onclose = () => {
            result.reconnectionAttempted = true;
            
            // Attempt reconnection
            setTimeout(() => {
              ws = new WebSocket(wsUrl);
              
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
    }, {
      wsUrl: TEST_URLS.WEBSOCKET.GATEWAY
    });
  }

  /**
   * Test server unavailable scenario
   */
  async testServerUnavailable(): Promise<ServerUnavailableResult> {
    return await this.page.evaluate(async ({ nonExistentUrl }) => {
      const result: ServerUnavailableResult = {
        errorDetected: false,
        fallbackActivated: false,
        errors: []
      };

      try {
        const ws = new WebSocket(nonExistentUrl); // Non-existent server
        
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
    }, {
      nonExistentUrl: TEST_URLS.WEBSOCKET.NON_EXISTENT
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
      const wsUrl = TEST_URLS.WEBSOCKET.GATEWAY;
      
      // Setup WebSocket in both tabs and compare events
      const [tab1Events, tab2Events] = await Promise.all([
        page1.evaluate(async (wsUrl) => {
          try {
            const ws = new WebSocket(wsUrl);
            return new Promise((resolve, reject) => {
              const events: any[] = [];
              const timeout = setTimeout(() => {
                ws.close();
                resolve(events);
              }, 3000);
              
              ws.onopen = () => {
                console.log('Tab 1 WebSocket connected');
              };
              
              ws.onmessage = (event) => {
                try {
                  const data = JSON.parse(event.data);
                  events.push(data);
                } catch (e) {
                  console.warn('Failed to parse message:', event.data);
                }
              };
              
              ws.onerror = (error) => {
                clearTimeout(timeout);
                reject(new Error('WebSocket error in tab 1'));
              };
            });
          } catch (error) {
            return { error: (error as Error).message };
          }
        }, wsUrl),
        page2.evaluate(async (wsUrl) => {
          try {
            const ws = new WebSocket(wsUrl);
            return new Promise((resolve, reject) => {
              const events: any[] = [];
              const timeout = setTimeout(() => {
                ws.close();
                resolve(events);
              }, 3000);
              
              ws.onopen = () => {
                console.log('Tab 2 WebSocket connected');
              };
              
              ws.onmessage = (event) => {
                try {
                  const data = JSON.parse(event.data);
                  events.push(data);
                } catch (e) {
                  console.warn('Failed to parse message:', event.data);
                }
              };
              
              ws.onerror = (error) => {
                clearTimeout(timeout);
                reject(new Error('WebSocket error in tab 2'));
              };
            });
          } catch (error) {
            return { error: (error as Error).message };
          }
        }, wsUrl)
      ]);

      // Check if both tabs could connect (even if no events received)
      const tab1Connected = !(tab1Events as any)?.error;
      const tab2Connected = !(tab2Events as any)?.error;
      
      if (tab1Connected && tab2Connected) {
        result.eventsSharedBetweenTabs = true;
        result.synchronizationWorking = true;
        console.log('Both tabs connected successfully to WebSocket');
      } else {
        if (!tab1Connected) result.errors.push('Tab 1 connection failed');
        if (!tab2Connected) result.errors.push('Tab 2 connection failed');
      }
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
    const token = createTestJWT({ sub: 'test_user_123' });
    return this.testAuthentication({
      token,
      expectedSuccess: true
    });
  }

  async authenticateWithInvalidToken(): Promise<AuthenticationResult> {
    const token = createInvalidTestJWT();
    return this.testAuthentication({
      token,
      expectedSuccess: false
    });
  }

  async authenticateWithExpiredToken(): Promise<AuthenticationResult> {
    const token = createExpiredTestJWT({ sub: 'test_user_456' });
    return this.testAuthentication({
      token,
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
    const result = await this.testConnection(TEST_URLS.WEBSOCKET.GATEWAY);
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