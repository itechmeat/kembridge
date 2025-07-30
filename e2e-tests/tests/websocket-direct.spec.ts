/**
 * Direct WebSocket Test - No Frontend Navigation
 * Tests WebSocket connection directly without loading the frontend
 */

import { test, expect } from '@playwright/test';
import { TEST_URLS } from '../utils/test-constants';

test.describe('Direct WebSocket Tests', () => {
  test('should connect to WebSocket server directly', async ({ page }) => {
    // Test direct WebSocket connection without navigating to frontend
    const connectionResult = await page.evaluate(async () => {
      return new Promise<{ connected: boolean; error: string | null }>((resolve) => {
        try {
          const ws = new WebSocket(TEST_URLS.WEBSOCKET.GATEWAY);
          
          const timeout = setTimeout(() => {
            ws.close();
            resolve({ connected: false, error: 'Connection timeout' });
          }, 5000);
          
          ws.onopen = () => {
            clearTimeout(timeout);
            ws.close();
            resolve({ connected: true, error: null });
          };
          
          ws.onerror = (error) => {
            clearTimeout(timeout);
            resolve({ connected: false, error: 'WebSocket error' });
          };
          
          ws.onclose = (event) => {
            if (!event.wasClean) {
              clearTimeout(timeout);
              resolve({ connected: false, error: 'Connection closed unexpectedly' });
            }
          };
        } catch (error) {
          resolve({ connected: false, error: `Exception: ${error}` });
        }
      });
    });
    
    expect(connectionResult.connected).toBe(true);
    expect(connectionResult.error).toBeNull();
    
    console.log('✅ Direct WebSocket connection successful');
  });
  
  test('should verify backend health endpoint', async ({ page }) => {
    const response = await page.request.get(`${TEST_URLS.BACKEND.GATEWAY}/health`);
    expect(response.ok()).toBe(true);
    
    const healthData = await response.json();
    expect(healthData).toHaveProperty('success');
    expect(healthData.success).toBe(true);
    expect(healthData.data).toHaveProperty('status');
    expect(healthData.data.status).toBe('healthy');
    
    console.log('✅ Backend health check passed');
  });
});