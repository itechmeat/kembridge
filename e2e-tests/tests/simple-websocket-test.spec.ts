/**
 * Simple WebSocket Connection Test
 * Basic test to verify WebSocket connectivity
 */
import { test, expect } from '@playwright/test';
import { WebSocket } from 'ws';
import { TEST_URLS } from '../utils/test-constants';

test.describe('Simple WebSocket Tests', () => {
  test('should connect to backend health endpoint', async ({ page }) => {
    const response = await page.request.get(`${TEST_URLS.BACKEND.GATEWAY}/health`);expect(response.ok()).toBe(true);
    console.log('✅ Backend health check passed');
  });
  
  test('should connect to WebSocket server directly', async ({ page }) => {
    // Test WebSocket connection without navigating to frontend
    const connectionResult = await page.evaluate(async () => {
      return new Promise<{ connected: boolean; error: string | null }>((resolve) => {
        const ws = new WebSocket(TEST_URLS.WEBSOCKET.GATEWAY);
        const timeout = setTimeout(() => {
          ws.close();
          resolve({ connected: false, error: 'timeout' });
        }, 3000);
        
        ws.onopen = () => {
          clearTimeout(timeout);
          ws.close();
          resolve({ connected: true, error: null });
        };
        
        ws.onerror = (error) => {
          clearTimeout(timeout);
          resolve({ connected: false, error: 'connection_error' });
        };
      });
    });
    
    expect(connectionResult.connected).toBe(true);
    console.log('✅ WebSocket connection test passed');
  });
});