/**
 * Simple WebSocket Connection Test
 * Basic test to verify WebSocket connectivity
 */

import { test, expect } from '@playwright/test';

test.describe('Simple WebSocket Test', () => {
  test('should verify backend health endpoint', async ({ page }) => {
    const response = await page.request.get('http://localhost:4000/health');
    expect(response.ok()).toBe(true);
    console.log('✅ Backend health check passed');
  });
  
  test('should connect to WebSocket server directly', async ({ page }) => {
    // Test WebSocket connection without navigating to frontend
    const connectionResult = await page.evaluate(async () => {
      return new Promise<{ connected: boolean; error: string | null }>((resolve) => {
        const ws = new WebSocket('ws://localhost:4000/ws');
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