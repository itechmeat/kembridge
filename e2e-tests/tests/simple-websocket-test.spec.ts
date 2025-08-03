/**
 * Simple WebSocket Connection Test
 * Basic test to verify WebSocket connectivity
 */
import { test, expect } from '@playwright/test';
import { WebSocket } from 'ws';
import { TEST_URLS } from '../utils/test-constants';
import { getWebSocketUrl, getBackendUrl } from '../utils/page-evaluate-utils';

test.describe('Simple WebSocket Tests', () => {
  test('should connect to backend health endpoint', async ({ page }) => {
    const response = await page.request.get(`${getBackendUrl('gateway')}/health`);expect(response.ok()).toBe(true);
    console.log('✅ Backend health check passed');
  });
  
  test('should connect to WebSocket server directly', async ({ page }) => {
    // Test WebSocket connection without navigating to frontend
    const wsUrl = getWebSocketUrl('gateway');
    const connectionResult = await page.evaluate(async (url) => {
      return new Promise<{ connected: boolean; error: string | null }>((resolve) => {
        const ws = new WebSocket(url);
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
    }, wsUrl);
    
    expect(connectionResult.connected).toBe(true);
    console.log('✅ WebSocket connection test passed');
  });
});