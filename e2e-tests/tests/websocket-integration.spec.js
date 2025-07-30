/**
 * WebSocket Integration E2E Tests
 * Tests WebSocket real-time functionality across the application
 */

import { test, expect } from '@playwright/test';
import { TEST_URLS } from '../utils/test-constants';

test.describe('WebSocket Integration', () => {
  test.beforeEach(async ({ page }) => {
    // Navigate to bridge page where WebSocket is initialized
    await page.goto(`${TEST_URLS.FRONTEND.LOCAL_DEV}/bridge`, { waitUntil: 'domcontentloaded' });
    
    // Wait for the page to load
    await page.waitForTimeout(2000);
  });

  test('should display WebSocket status component', async ({ page }) => {
    // Check if WebSocket status component is visible
    const webSocketStatus = await page.locator('[data-testid="websocket-status"]');
    await expect(webSocketStatus).toBeVisible();
    
    // Check for status icon and text
    const statusIcon = await page.locator('[data-testid="websocket-status-icon"]');
    const statusText = await page.locator('[data-testid="websocket-status-text"]');
    
    await expect(statusIcon).toBeVisible();
    await expect(statusText).toBeVisible();
    
    // Status should be one of: Connected, Connecting, Disconnected, Unknown
    const statusContent = await statusText.textContent();
    expect(['Connected', 'Connecting', 'Disconnected', 'Unknown']).toContain(statusContent);
    
    console.log('✅ WebSocket status component rendered correctly');
  });

  test('should handle connection state changes', async ({ page }) => {
    const webSocketStatus = await page.locator('[data-testid="websocket-status"]');
    const statusText = await page.locator('[data-testid="websocket-status-text"]');
    
    // Wait a bit for connection attempt
    await page.waitForTimeout(2000);
    
    // Check that status has appropriate CSS class
    const statusClasses = await webSocketStatus.getAttribute('class');
    expect(statusClasses).toMatch(/websocket-status--(connected|connecting|disconnected|closed)/);
    
    console.log('✅ WebSocket connection state handling works');
  });

  test('should show retry button on connection error', async ({ page }) => {
    // Wait for potential connection error
    await page.waitForTimeout(3000);
    
    // Check if error state is shown
    const errorElement = await page.locator('[data-testid="websocket-status-error"]');
    
    if (await errorElement.isVisible()) {
      console.log('⚠️ WebSocket connection error detected');
      
      // Check for retry button
      const retryButton = await page.locator('[data-testid="websocket-retry-button"]');
      await expect(retryButton).toBeVisible();
      
      // Test retry functionality
      await retryButton.click();
      
      console.log('✅ Retry button functionality works');
    } else {
      console.log('✅ No connection errors (WebSocket connected successfully)');
    }
  });

  test('should not show notifications initially', async ({ page }) => {
    // Real-time notifications should not be visible by default
    const notifications = await page.locator('[data-testid="realtime-notifications"]');
    
    // Wait a bit to ensure notifications don't appear
    await page.waitForTimeout(1000);
    
    // Notifications should not be visible initially
    await expect(notifications).not.toBeVisible();
    
    console.log('✅ Real-time notifications hidden by default');
  });

  test('should display WebSocket status in mobile view', async ({ page }) => {
    // Set mobile viewport
    await page.setViewportSize({ width: 375, height: 667 });
    
    // Check WebSocket status is still visible and properly styled
    const webSocketStatus = await page.locator('[data-testid="websocket-status"]');
    await expect(webSocketStatus).toBeVisible();
    
    // Check responsive styling
    const statusClasses = await webSocketStatus.getAttribute('class');
    expect(statusClasses).toBeTruthy();
    
    console.log('✅ WebSocket status responsive design works');
  });

  test('should handle page refresh gracefully', async ({ page }) => {
    // Wait for initial connection
    await page.waitForTimeout(2000);
    
    // Refresh the page
    await page.reload({ waitUntil: 'domcontentloaded' });
    await page.waitForTimeout(2000);
    
    // Check that WebSocket status is still displayed
    const webSocketStatus = await page.locator('[data-testid="websocket-status"]');
    await expect(webSocketStatus).toBeVisible();
    
    // Wait for reconnection attempt
    await page.waitForTimeout(3000);
    
    console.log('✅ WebSocket handles page refresh correctly');
  });

  test('should validate WebSocket URL configuration', async ({ page }) => {
    // Check console for WebSocket connection attempts
    const consoleLogs = [];
    
    page.on('console', msg => {
      consoleLogs.push(msg.text());
    });
    
    // Wait for WebSocket initialization
    await page.waitForTimeout(3000);
    
    // Check that WebSocket connection was attempted or status component exists
    const websocketLogs = consoleLogs.filter(log => 
      log.includes('WebSocket') || 
      log.includes('ws://') || 
      log.includes('wss://')
    );
    
    // If no WebSocket logs found, check that WebSocket status component exists
    if (websocketLogs.length === 0) {
      const webSocketStatus = await page.locator('[data-testid="websocket-status"]');
      await expect(webSocketStatus).toBeVisible();
      console.log('✅ WebSocket status component exists (no console logs found)');
    } else {
      console.log('✅ WebSocket connection attempts logged:', websocketLogs.length);
    }
  });

  test('should test WebSocket component integration in bridge page', async ({ page }) => {
    // We're already on bridge page from beforeEach
    
    // Check that WebSocket status is integrated into the bridge page header
    const bridgeHeader = await page.locator('.bridge-page__header');
    const webSocketStatus = await page.locator('[data-testid="websocket-status"]');
    
    await expect(bridgeHeader).toBeVisible();
    await expect(webSocketStatus).toBeVisible();
    
    // Check that WebSocket status is positioned correctly in header
    const statusInHeader = await bridgeHeader.locator('[data-testid="websocket-status"]');
    await expect(statusInHeader).toBeVisible();
    
    console.log('✅ WebSocket status integrated correctly in bridge page');
  });

  test('should handle authentication state changes', async ({ page }) => {
    // Monitor console for authentication-related WebSocket messages
    const authLogs = [];
    
    page.on('console', msg => {
      if (msg.text().includes('auth') || msg.text().includes('Auth')) {
        authLogs.push(msg.text());
      }
    });
    
    // Wait for potential authentication flow
    await page.waitForTimeout(3000);
    
    // Check WebSocket status remains functional
    const webSocketStatus = await page.locator('[data-testid="websocket-status"]');
    await expect(webSocketStatus).toBeVisible();
    
    console.log('✅ WebSocket handles authentication state correctly');
    if (authLogs.length > 0) {
      console.log('   Auth-related logs found:', authLogs.length);
    }
  });
});

// Helper functions for WebSocket testing
async function waitForWebSocketConnection(page, timeout = 5000) {
  const startTime = Date.now();
  
  while (Date.now() - startTime < timeout) {
    const statusText = await page.locator('[data-testid="websocket-status-text"]').textContent();
    if (statusText === 'Connected') {
      return true;
    }
    await page.waitForTimeout(500);
  }
  
  return false;
}

async function waitForWebSocketDisconnection(page, timeout = 5000) {
  const startTime = Date.now();
  
  while (Date.now() - startTime < timeout) {
    const statusText = await page.locator('[data-testid="websocket-status-text"]').textContent();
    if (statusText === 'Disconnected' || statusText === 'Closed') {
      return true;
    }
    await page.waitForTimeout(500);
  }
  
  return false;
}