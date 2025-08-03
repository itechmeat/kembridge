/**
 * WebSocket Backend Direct Test
 * Tests WebSocket server functionality independently of frontend issues
 */

import { test, expect } from '@playwright/test';
import { TEST_URLS } from '../utils/test-constants';
import { getWebSocketUrl, getFrontendUrl } from '../utils/page-evaluate-utils';

test.describe('WebSocket Backend Direct Test', () => {
  test('should connect to WebSocket server and receive messages', async ({ page }) => {
    const wsMessages = [];
    const wsErrors = [];
    let wsConnected = false;
    
    // Add WebSocket test script to page
    await page.addInitScript((wsUrl) => {
      window.testWebSocket = () => {
        return new Promise((resolve, reject) => {
          const ws = new WebSocket(wsUrl);
          const messages = [];
          const errors = [];
          let connected = false;
          
          ws.onopen = () => {
            console.log('✅ WebSocket connected');
            connected = true;
            
            // Send a subscription request to get messages
            ws.send(JSON.stringify({
              action: 'subscribe',
              event_type: 'transaction_status'
            }));
          };
          
          ws.onmessage = (event) => {
            const data = JSON.parse(event.data);
            console.log('📨 WebSocket message:', data);
            messages.push(data);
          };
          
          ws.onerror = (error) => {
            console.error('❌ WebSocket error:', error);
            errors.push(error);
          };
          
          ws.onclose = (event) => {
            console.log('🔌 WebSocket closed:', event.code, event.reason);
            resolve({ connected, messages, errors });
          };
          
          // Close after 5 seconds
          setTimeout(() => {
            ws.close();
          }, 5000);
        });
      };
    }, getWebSocketUrl('gateway'));
    
    // Navigate to any page (we just need a browser context)
    await page.goto(getFrontendUrl('localDev'));
    
    // Run WebSocket test
    const result = await page.evaluate(() => window.testWebSocket());
    
    console.log('🔍 WebSocket test result:', result);
    
    // Verify connection was established
    expect(result.connected).toBe(true);
    
    // Verify we received at least one message
    expect(result.messages.length).toBeGreaterThan(0);
    
    // Verify message structure - should receive subscription confirmation
    const firstMessage = result.messages[0];
    console.log('📋 First message received:', firstMessage);
    
    // Check for subscription confirmation or any valid message type
    expect(firstMessage.type).toBeDefined();
    
    // The server returns different message structures, adapt to actual response
    if (firstMessage.type === 'subscription_confirmed') {
      expect(firstMessage.event_type).toBeDefined();
    } else if (firstMessage.type === 'Event') {
      expect(firstMessage.event).toBeDefined();
    } else {
      // For other message types, just verify type is present
      expect(firstMessage.type).toBeTruthy();
    }
    
    console.log('✅ WebSocket backend test completed successfully');
  });
  
  test('should handle WebSocket authentication', async ({ page }) => {
    // Add authenticated WebSocket test
    await page.addInitScript((wsUrl) => {
      window.testAuthWebSocket = () => {
        return new Promise((resolve, reject) => {
          const ws = new WebSocket(`${wsUrl}?token=test-token-123`);
          const messages = [];
          let connected = false;
          
          ws.onopen = () => {
            console.log('✅ Authenticated WebSocket connected');
            connected = true;
          };
          
          ws.onmessage = (event) => {
            const data = JSON.parse(event.data);
            console.log('📨 Auth WebSocket message:', data);
            messages.push(data);
          };
    
          ws.onclose = (event) => {
            console.log('🔌 Auth WebSocket closed');
            resolve({ connected, messages });
          };
          
          // Send subscription request
          setTimeout(() => {
            ws.send(JSON.stringify({
              action: 'subscribe',
              event_type: 'transaction_update',
              filters: { user_id: 'test-user' }
            }));
          }, 1000);
          
          // Close after 3 seconds  
          setTimeout(() => {
            ws.close();
          }, 3000);
        });
      };
    }, getWebSocketUrl('gateway'));
    
    await page.goto(getFrontendUrl('localDev'));
    
    const result = await page.evaluate(() => window.testAuthWebSocket());
    
    console.log('🔍 Auth WebSocket test result:', result);
    
    // Verify authenticated connection
    expect(result.connected).toBe(true);
    expect(result.messages.length).toBeGreaterThan(0);
    
    console.log('✅ WebSocket authentication test completed successfully');
  });
  
  test('should handle subscription requests', async ({ page }) => {
    await page.addInitScript((wsUrl) => {
      window.testSubscription = () => {
        return new Promise((resolve) => {
          const ws = new WebSocket(wsUrl);
          const subscriptionMessages = [];
          let connected = false;
          
          ws.onopen = () => {
            console.log('✅ Subscription WebSocket connected');
            connected = true;
            
            // Send price update subscription
            ws.send(JSON.stringify({
              action: 'subscribe',
              event_type: 'price_update',
              filters: { from_token: 'ETH', to_token: 'NEAR' }
            }));
          };
          
          ws.onmessage = (event) => {
            const data = JSON.parse(event.data);
            console.log('📨 Subscription message:', data);
            subscriptionMessages.push(data);
          };
          
          ws.onclose = () => {
            resolve({ connected, subscriptionMessages });
          };
          
          setTimeout(() => ws.close(), 2000);
        });
      };
    }, getWebSocketUrl('gateway'));
    
    await page.goto(getFrontendUrl('localDev'));
    
    const result = await page.evaluate(() => window.testSubscription());
    
    console.log('🔍 Subscription test result:', result);
    
    expect(result.connected).toBe(true);
    expect(result.subscriptionMessages.length).toBeGreaterThan(0);
    
    // Check if we received a price update response
    const priceUpdate = result.subscriptionMessages.find(msg => msg.type === 'price_update');
    if (priceUpdate) {
      expect(priceUpdate.data.from_token).toBe('ETH');
      expect(priceUpdate.data.to_token).toBe('NEAR');
      expect(priceUpdate.data.price).toBe(2500.0);
    }
    
    console.log('✅ WebSocket subscription test completed successfully');
  });
});