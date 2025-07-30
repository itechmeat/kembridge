/**
 * Comprehensive WebSocket E2E Tests
 * Tests core WebSocket functionality including connection, authentication, and event handling
 */

import { test, expect } from '@playwright/test';
import { WebSocketTestUtils } from '../utils/websocket-utils';

test.describe('WebSocket Comprehensive Tests', () => {
  let wsUtils: WebSocketTestUtils;

  test.beforeEach(async ({ page }) => {
    wsUtils = new WebSocketTestUtils(page);
    // No frontend navigation - testing WebSocket directly
  });

  test.describe('Connection Establishment', () => {
    test('should establish WebSocket connection to /ws endpoint', async ({ page }) => {
      const connectionResult = await wsUtils.testConnection('ws://localhost:4000/ws');
      
      expect(connectionResult.connected).toBe(true);
      expect(connectionResult.connectionTime).toBeLessThan(5000);
      expect(connectionResult.errors).toHaveLength(0);
      
      console.log('✅ WebSocket connection established successfully');
    });

    test('should handle connection timeout gracefully', async ({ page }) => {
      const connectionResult = await wsUtils.testConnection('ws://localhost:9999/ws', 2000);
      
      expect(connectionResult.connected).toBe(false);
      expect(connectionResult.errors.length).toBeGreaterThan(0);
      
      console.log('✅ Connection timeout handled correctly');
    });

    test('should validate WebSocket URL format', async ({ page }) => {
      const invalidUrls = [
        'http://localhost:4000/ws', // Wrong protocol
        'ws://invalid-host/ws',     // Invalid host
        'ws://localhost:4000/invalid' // Invalid endpoint
      ];

      for (const url of invalidUrls) {
        const result = await wsUtils.testConnection(url, 1000);
        expect(result.connected).toBe(false);
      }
      
      console.log('✅ Invalid WebSocket URLs rejected correctly');
    });
  });

  test.describe('JWT Authentication', () => {
    test('should authenticate with valid JWT token', async ({ page }) => {
      const authResult = await wsUtils.testAuthentication({
        token: 'valid-test-token-123',
        expectedSuccess: true
      });
      
      expect(authResult.authenticated).toBe(true);
      expect(authResult.authMessages.length).toBeGreaterThan(0);
      
      console.log('✅ JWT authentication successful');
    });

    test('should reject invalid JWT token', async ({ page }) => {
      const authResult = await wsUtils.testAuthentication({
        token: 'invalid-token',
        expectedSuccess: false
      });
      
      expect(authResult.authenticated).toBe(false);
      expect(authResult.errors.some((e: string) => e.includes('authentication'))).toBe(true);
      
      console.log('✅ Invalid JWT token rejected');
    });

    test('should handle expired JWT token', async ({ page }) => {
      const authResult = await wsUtils.testAuthentication({
        token: 'expired-token-123',
        expectedSuccess: false
      });
      
      expect(authResult.authenticated).toBe(false);
      expect(authResult.errors.some((e: string) => e.includes('expired') || e.includes('invalid'))).toBe(true);
      
      console.log('✅ Expired JWT token handled correctly');
    });

    test('should handle missing JWT token', async ({ page }) => {
      const authResult = await wsUtils.testAuthentication({
        token: null,
        expectedSuccess: false
      });
      
      expect(authResult.authenticated).toBe(false);
      
      console.log('✅ Missing JWT token handled correctly');
    });
  });

  test.describe('Event Subscription System', () => {
    test('should subscribe to transaction updates', async ({ page }) => {
      const subscriptionResult = await wsUtils.testSubscription({
        eventType: 'transaction_update',
        filters: { user_id: 'test-user' }
      });
      
      expect(subscriptionResult.subscribed).toBe(true);
      expect(subscriptionResult.confirmationReceived).toBe(true);
      
      console.log('✅ Transaction update subscription successful');
    });

    test('should subscribe to price updates', async ({ page }) => {
      const subscriptionResult = await wsUtils.testSubscription({
        eventType: 'price_update',
        filters: { from_token: 'ETH', to_token: 'NEAR' }
      });
      
      expect(subscriptionResult.subscribed).toBe(true);
      expect(subscriptionResult.confirmationReceived).toBe(true);
      
      console.log('✅ Price update subscription successful');
    });

    test('should unsubscribe from events', async ({ page }) => {
      // First subscribe
      await wsUtils.testSubscription({
        eventType: 'transaction_update',
        filters: { user_id: 'test-user' }
      });
      
      // Then unsubscribe
      const unsubscribeResult = await wsUtils.testUnsubscription('transaction_update');
      
      expect(unsubscribeResult.unsubscribed).toBe(true);
      expect(unsubscribeResult.confirmationReceived).toBe(true);
      
      console.log('✅ Event unsubscription successful');
    });

    test('should handle multiple subscriptions', async ({ page }) => {
      const subscriptions = [
        { eventType: 'transaction_update', filters: { user_id: 'test-user' } },
        { eventType: 'price_update', filters: { from_token: 'ETH' } },
        { eventType: 'system_notification', filters: {} }
      ];
      
      const results = await Promise.all(
        subscriptions.map(sub => wsUtils.testSubscription(sub))
      );
      
      results.forEach((result: any) => {
        expect(result.subscribed).toBe(true);
      });
      
      console.log('✅ Multiple subscriptions handled correctly');
    });
  });

  test.describe('Real-time Event Delivery', () => {
    test('should receive transaction update events', async ({ page }) => {
      const eventResult = await wsUtils.testEventDelivery({
        subscribeToEvent: 'transaction_update',
        expectedEventType: 'transaction_update',
        timeout: 10000
      });
      
      expect(eventResult.eventsReceived.length).toBeGreaterThan(0);
      expect(eventResult.eventsReceived[0].type).toBe('transaction_update');
      expect(eventResult.eventsReceived[0].data).toBeDefined();
      
      console.log('✅ Transaction update events delivered correctly');
    });

    test('should receive price update events', async ({ page }) => {
      const eventResult = await wsUtils.testEventDelivery({
        subscribeToEvent: 'price_update',
        expectedEventType: 'price_update',
        timeout: 10000
      });
      
      expect(eventResult.eventsReceived.length).toBeGreaterThan(0);
      expect(eventResult.eventsReceived[0].type).toBe('price_update');
      expect(eventResult.eventsReceived[0].data.price).toBeDefined();
      
      console.log('✅ Price update events delivered correctly');
    });

    test('should filter events correctly', async ({ page }) => {
      const eventResult = await wsUtils.testEventDelivery({
        subscribeToEvent: 'transaction_update',
        filters: { user_id: 'specific-user' },
        expectedEventType: 'transaction_update',
        timeout: 5000
      });
      
      // Should only receive events for the specific user
      eventResult.eventsReceived.forEach((event: any) => {
        expect(event.data.user_id).toBe('specific-user');
      });
      
      console.log('✅ Event filtering working correctly');
    });
  });

  test.describe('Connection Error Handling', () => {
    test('should handle network disconnection', async ({ page }) => {
      const errorResult = await wsUtils.testNetworkDisconnection();
      
      expect(errorResult.disconnectionDetected).toBe(true);
      expect(errorResult.errorHandled).toBe(true);
      
      console.log('✅ Network disconnection handled correctly');
    });

    test('should attempt reconnection after disconnection', async ({ page }) => {
      const reconnectionResult = await wsUtils.testReconnection();
      
      expect(reconnectionResult.reconnectionAttempted).toBe(true);
      expect(reconnectionResult.reconnectionSuccessful).toBe(true);
      expect(reconnectionResult.reconnectionTime).toBeLessThan(10000);
      
      console.log('✅ Reconnection successful');
    });

    test('should handle server unavailable scenario', async ({ page }) => {
      const serverDownResult = await wsUtils.testServerUnavailable();
      
      expect(serverDownResult.errorDetected).toBe(true);
      expect(serverDownResult.fallbackActivated).toBe(true);
      
      console.log('✅ Server unavailable scenario handled');
    });
  });

  test.describe('Cross-tab Event Synchronization', () => {
    test('should synchronize events across multiple tabs', async ({ page, context }) => {
      const secondPage = await context.newPage();
      await secondPage.goto('http://localhost:4100/bridge');
      
      const syncResult = await wsUtils.testCrossTabSync(page, secondPage);
      
      expect(syncResult.eventsSharedBetweenTabs).toBe(true);
      expect(syncResult.synchronizationWorking).toBe(true);
      
      await secondPage.close();
      console.log('✅ Cross-tab synchronization working');
    });
  });

  test.describe('React Hooks Integration', () => {
    test('should integrate with useWebSocketConnection hook', async ({ page }) => {
      const hookResult = await wsUtils.testReactHookIntegration('useWebSocketConnection');
      
      expect(hookResult.hookFound).toBe(true);
      expect(hookResult.hookWorking).toBe(true);
      expect(hookResult.stateUpdates.length).toBeGreaterThan(0);
      
      console.log('✅ useWebSocketConnection hook integration working');
    });

    test('should integrate with useTransactionUpdates hook', async ({ page }) => {
      const hookResult = await wsUtils.testReactHookIntegration('useTransactionUpdates');
      
      expect(hookResult.hookFound).toBe(true);
      expect(hookResult.hookWorking).toBe(true);
      
      console.log('✅ useTransactionUpdates hook integration working');
    });
  });
});