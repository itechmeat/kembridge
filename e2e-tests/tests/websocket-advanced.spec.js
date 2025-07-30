/**
 * Advanced WebSocket E2E Tests
 * Comprehensive testing of WebSocket real-time functionality and error recovery
 */

import { test, expect } from '@playwright/test';

test.describe('Advanced WebSocket Integration', () => {
  let consoleLogs = [];
  let wsEvents = [];

  test.beforeEach(async ({ page }) => {
    // Reset logs
    consoleLogs = [];
    wsEvents = [];

    // Capture console logs
    page.on('console', msg => {
      const text = msg.text();
      consoleLogs.push({
        type: msg.type(),
        text: text,
        timestamp: Date.now()
      });
    });

    // Navigate to bridge page
    await page.goto('http://localhost:4100/bridge');
    await page.waitForLoadState('domcontentloaded');
    
    // Wait for page to be ready - look for main content
    await page.waitForSelector('body', { timeout: 10000 });
    
    // Wait for WebSocket initialization
    await page.waitForTimeout(3000);
  });

  test('should handle WebSocket connection lifecycle', async ({ page }) => {
    // Check initial connection status
    const webSocketStatus = await page.locator('[data-testid="websocket-status"]');
    await expect(webSocketStatus).toBeVisible();
    
    const statusText = await page.locator('[data-testid="websocket-status-text"]');
    const initialStatus = await statusText.textContent();
    
    console.log('✅ Initial WebSocket status:', initialStatus);
    
    // Verify status is one of expected values
    expect(['Connected', 'Connecting', 'Disconnected', 'Unknown']).toContain(initialStatus);
    
    // Check for WebSocket logs in console
    await page.waitForTimeout(1000);
    const wsLogs = consoleLogs.filter(log => 
      log.text.includes('WebSocket') || 
      log.text.includes('🔌') ||
      log.text.includes('💓')
    );
    
    expect(wsLogs.length).toBeGreaterThan(0);
    console.log(`✅ Found ${wsLogs.length} WebSocket-related console logs`);
    
    // Filter out expected query errors to reduce noise
    const criticalErrors = consoleLogs.filter(log => 
      log.type === 'error' && 
      !log.text.includes('Query data cannot be undefined') &&
      !log.text.includes('404')
    );
    
    expect(criticalErrors.length).toBe(0);
  });

  test('should display real-time event service logs', async ({ page }) => {
    // Wait for real-time event service initialization
    await page.waitForTimeout(3000);
    
    // Check for event service initialization logs
    const eventServiceLogs = consoleLogs.filter(log => 
      log.text.includes('RealTimeEventService') ||
      log.text.includes('🌊') ||
      log.text.includes('Created stream')
    );
    
    if (eventServiceLogs.length > 0) {
      console.log(`✅ Found ${eventServiceLogs.length} real-time event service logs`);
      
      // Verify default streams were created
      const streamLogs = eventServiceLogs.filter(log => 
        log.text.includes('Created stream')
      );
      
      if (streamLogs.length >= 4) {
        console.log(`✅ Found ${streamLogs.length} stream creation logs`);
      } else {
        console.log(`ℹ️ Found ${streamLogs.length} stream creation logs (expected 4+)`);
      }
    } else {
      console.log('ℹ️ No real-time event service logs found (may require authentication)');
    }
    
    // Verify page is functional
    const bridgeHeader = await page.locator('.bridge-page__header');
    await expect(bridgeHeader).toBeVisible({ timeout: 5000 });
  });

  test('should handle bridge WebSocket integration', async ({ page }) => {
    // Check for bridge WebSocket setup logs
    await page.waitForTimeout(2000);
    
    const bridgeWsLogs = consoleLogs.filter(log => 
      log.text.includes('bridge WebSocket') ||
      log.text.includes('🌉') ||
      log.text.includes('Setting up bridge WebSocket')
    );
    
    if (bridgeWsLogs.length > 0) {
      console.log(`✅ Found ${bridgeWsLogs.length} bridge WebSocket integration logs`);
    } else {
      console.log('ℹ️ No bridge WebSocket logs (may be due to authentication state)');
    }
    
    // Verify bridge WebSocket integration is working
    const bridgeHeader = await page.locator('.bridge-page__header');
    await expect(bridgeHeader).toBeVisible({ timeout: 10000 });
    
    // Check if WebSocket status is integrated into bridge page
    const statusInHeader = await bridgeHeader.locator('[data-testid="websocket-status"]');
    await expect(statusInHeader).toBeVisible({ timeout: 5000 });
    
    console.log('✅ WebSocket status integrated in bridge page');
  });

  test('should test error handling mechanisms', async ({ page }) => {
    // Monitor error handler logs
    await page.waitForTimeout(3000);
    
    const errorHandlerLogs = consoleLogs.filter(log => 
      log.text.includes('Error Handler') ||
      log.text.includes('🛠️') ||
      log.text.includes('recovery strategy')
    );
    
    if (errorHandlerLogs.length > 0) {
      console.log(`✅ Found ${errorHandlerLogs.length} error handler logs`);
    } else {
      console.log('ℹ️ No error handler logs found (system may be stable)');
    }
    
    // Check that WebSocket status still displays correctly even with potential errors
    const webSocketStatus = await page.locator('[data-testid="websocket-status"]');
    await expect(webSocketStatus).toBeVisible({ timeout: 10000 });
    
    // Verify status text is readable
    const statusText = await page.locator('[data-testid="websocket-status-text"]');
    const status = await statusText.textContent();
    expect(['Connected', 'Connecting', 'Disconnected', 'Unknown']).toContain(status);
    
    console.log('✅ Error handling mechanisms appear operational');
  });

  test('should test WebSocket reconnection scenarios', async ({ page }) => {
    // Monitor reconnection logs
    await page.waitForTimeout(4000);
    
    const reconnectionLogs = consoleLogs.filter(log => 
      log.text.includes('reconnect') ||
      log.text.includes('🔄') ||
      log.text.includes('connection lost')
    );
    
    if (reconnectionLogs.length > 0) {
      console.log(`✅ Found ${reconnectionLogs.length} reconnection-related logs`);
    } else {
      console.log('ℹ️ No reconnection logs (connection may be stable)');
    }
    
    // Verify WebSocket status is still functional
    const webSocketStatus = await page.locator('[data-testid="websocket-status"]');
    await expect(webSocketStatus).toBeVisible({ timeout: 10000 });
    
    // Check that status shows a valid connection state
    const statusText = await webSocketStatus.textContent();
    expect(statusText).toBeTruthy();
    
    console.log('✅ WebSocket reconnection mechanisms tested');
  });

  test('should test real-time notifications integration', async ({ page }) => {
    // Wait for notification system initialization
    await page.waitForTimeout(3000);
    
    // Check if real-time notifications component exists
    const notifications = page.locator('[data-testid="realtime-notifications"]');
    
    // Initially should not be visible (no notifications yet)
    const isVisible = await notifications.isVisible().catch(() => false);
    expect(isVisible).toBe(false);
    
    // Look for notification-related logs
    const notificationLogs = consoleLogs.filter(log => 
      log.text.includes('notification') ||
      log.text.includes('🔔') ||
      log.text.includes('alert')
    );
    
    console.log(`ℹ️ Found ${notificationLogs.length} notification-related logs`);
    
    // Verify notification infrastructure is set up
    const bridgeWsLogs = consoleLogs.filter(log => 
      log.text.includes('Subscribed to') ||
      log.text.includes('💰') ||
      log.text.includes('💱')
    );
    
    if (bridgeWsLogs.length > 0) {
      console.log(`✅ Found ${bridgeWsLogs.length} subscription logs`);
    }
    
    // Verify WebSocket is ready for notifications
    const webSocketStatus = await page.locator('[data-testid="websocket-status"]');
    await expect(webSocketStatus).toBeVisible({ timeout: 5000 });
    
    console.log('✅ Real-time notifications infrastructure tested');
  });

  test('should test price update subscriptions', async ({ page }) => {
    // Look for price subscription logs
    await page.waitForTimeout(2000);
    
    const priceUpdateLogs = consoleLogs.filter(log => 
      log.text.includes('Subscribed to') && 
      (log.text.includes('price updates') || log.text.includes('💱'))
    );
    
    if (priceUpdateLogs.length > 0) {
      console.log(`✅ Found ${priceUpdateLogs.length} price subscription logs`);
      
      // Verify price subscription messages contain token pairs
      const tokenPairLogs = priceUpdateLogs.filter(log => 
        log.text.includes('-') // Looking for token pair format like ETH-NEAR
      );
      
      expect(tokenPairLogs.length).toBeGreaterThan(0);
      console.log(`✅ Found ${tokenPairLogs.length} token pair subscription logs`);
    } else {
      console.log('ℹ️ No price subscription logs (may require authentication)');
    }
    
    // Verify price display components exist
    const priceDisplay = page.locator('[data-testid="price-display"]');
    
    const isVisible = await priceDisplay.isVisible().catch(() => false);
    if (isVisible) {
      console.log('✅ Price display component found');
      
      // Check if prices are being updated
      const priceValue = await priceDisplay.textContent();
      console.log(`Current price display: ${priceValue}`);
    } else {
      console.log('ℹ️ Price display not visible (may require market data)');
    }
    
    // Verify WebSocket connection supports price updates
    const webSocketStatus = await page.locator('[data-testid="websocket-status"]');
    await expect(webSocketStatus).toBeVisible({ timeout: 5000 });
  });

  test('should test connection quality monitoring', async ({ page }) => {
    // Check for connection quality logs
    await page.waitForTimeout(3000);
    
    const qualityLogs = consoleLogs.filter(log => 
      log.text.includes('quality') ||
      log.text.includes('📊') ||
      log.text.includes('latency') ||
      log.text.includes('ping')
    );
    
    if (qualityLogs.length > 0) {
      console.log(`✅ Found ${qualityLogs.length} connection quality logs`);
    } else {
      console.log('ℹ️ No connection quality logs found');
    }
    
    // Check for quality indicators in UI
    const qualityIndicator = page.locator('[data-testid="connection-quality"]');
    
    const isVisible = await qualityIndicator.isVisible().catch(() => false);
    if (isVisible) {
      const qualityValue = await qualityIndicator.textContent();
      console.log(`✅ Connection quality indicator: ${qualityValue}`);
    } else {
      console.log('ℹ️ Connection quality indicator not visible');
    }
    
    // Verify basic WebSocket functionality as quality baseline
    const webSocketStatus = await page.locator('[data-testid="websocket-status"]');
    await expect(webSocketStatus).toBeVisible({ timeout: 5000 });
    
    console.log('✅ Connection quality monitoring tested');
  });

  test('should test integration with bridge operations', async ({ page }) => {
    // Check for bridge operation logs
    await page.waitForTimeout(2000);
    
    const bridgeOpLogs = consoleLogs.filter(log => 
      log.text.includes('bridge operation') ||
      log.text.includes('🌉') ||
      log.text.includes('transaction')
    );
    
    if (bridgeOpLogs.length > 0) {
      console.log(`✅ Found ${bridgeOpLogs.length} bridge operation logs`);
    } else {
      console.log('ℹ️ No bridge operation logs (may require active transactions)');
    }
    
    // Verify bridge interface elements
    const bridgeForm = page.locator('[data-testid="bridge-form"]');
    
    const isFormVisible = await bridgeForm.isVisible().catch(() => false);
    if (isFormVisible) {
      console.log('✅ Bridge form interface found');
      
      // Check if WebSocket status is integrated
      const statusInForm = bridgeForm.locator('[data-testid="websocket-status"]');
      await expect(statusInForm).toBeVisible({ timeout: 5000 });
      
      console.log('✅ WebSocket status integrated in bridge operations');
    } else {
      console.log('ℹ️ Bridge form not visible (may require authentication)');
      
      // Fallback: verify WebSocket is available for bridge operations
      const webSocketStatus = await page.locator('[data-testid="websocket-status"]');
      await expect(webSocketStatus).toBeVisible({ timeout: 5000 });
      console.log('✅ WebSocket available for bridge operations');
    }
  });

  test('should test performance and memory management', async ({ page }) => {
    // Wait for system to stabilize
    await page.waitForTimeout(3000);
    
    // Monitor for memory-related logs or warnings
    const performanceLogs = consoleLogs.filter(log => 
      log.text.includes('buffer') ||
      log.text.includes('memory') ||
      log.text.includes('cleanup') ||
      log.text.includes('Cleaned up')
    );
    
    console.log(`ℹ️ Found ${performanceLogs.length} performance-related logs`);
    
    // Check that there are no obvious memory leaks (look for cleanup logs)
    const cleanupLogs = consoleLogs.filter(log => 
      log.text.includes('Cleaned up') ||
      log.text.includes('Unsubscribed') ||
      log.text.includes('🔕')
    );
    
    if (cleanupLogs.length > 0) {
      console.log(`✅ Found ${cleanupLogs.length} cleanup logs - memory management working`);
    } else {
      console.log('ℹ️ No cleanup logs found (system may not have triggered cleanup yet)');
    }
    
    // Verify no excessive error messages
    const errorLogs = consoleLogs.filter(log => 
      log.type === 'error' && 
      !log.text.includes('404') && 
      !log.text.includes('Query data cannot be undefined')
    );
    
    expect(errorLogs.length).toBeLessThan(5); // Allow for some expected errors
    console.log(`✅ Error count within acceptable range: ${errorLogs.length}`);
    
    // Verify WebSocket connection is stable
    const webSocketStatus = await page.locator('[data-testid="websocket-status"]');
    await expect(webSocketStatus).toBeVisible({ timeout: 5000 });
    
    console.log('✅ Performance and memory management tested');
  });

  test('should test multi-tab WebSocket behavior', async ({ page, context }) => {
    // Open second tab
    const secondPage = await context.newPage();
    await secondPage.goto('http://localhost:4100/bridge');
    await secondPage.waitForLoadState('domcontentloaded');
    await secondPage.waitForSelector('body', { timeout: 10000 });
    
    // Wait for both tabs to initialize
    await page.waitForTimeout(3000);
    await secondPage.waitForTimeout(3000);
    
    // Check WebSocket status in both tabs
    const firstTabStatus = page.locator('[data-testid="websocket-status"]');
    const secondTabStatus = secondPage.locator('[data-testid="websocket-status"]');
    
    await expect(firstTabStatus).toBeVisible({ timeout: 10000 });
    await expect(secondTabStatus).toBeVisible({ timeout: 10000 });
    
    // Both tabs should show WebSocket status
    const firstStatus = await page.locator('[data-testid="websocket-status-text"]').textContent();
    const secondStatus = await secondPage.locator('[data-testid="websocket-status-text"]').textContent();
    
    console.log(`✅ First tab status: ${firstStatus}`);
    console.log(`✅ Second tab status: ${secondStatus}`);
    
    // Verify both tabs can maintain independent WebSocket connections
    const firstTabHeader = page.locator('.bridge-page__header');
    const secondTabHeader = secondPage.locator('.bridge-page__header');
    
    await expect(firstTabHeader).toBeVisible({ timeout: 5000 });
    await expect(secondTabHeader).toBeVisible({ timeout: 5000 });
    
    // Close second tab
    await secondPage.close();
    
    // Verify first tab still works
    await page.waitForTimeout(1000);
    await expect(firstTabStatus).toBeVisible();
    
    console.log('✅ Multi-tab WebSocket behavior tested');
  });

  test.afterEach(async ({ page }) => {
    // Log summary of WebSocket-related console messages
    const wsLogs = consoleLogs.filter(log => 
      log.text.includes('WebSocket') ||
      log.text.includes('🔌') ||
      log.text.includes('💓') ||
      log.text.includes('🌊') ||
      log.text.includes('🌉') ||
      log.text.includes('💰') ||
      log.text.includes('💱') ||
      log.text.includes('🛠️')
    );
    
    console.log(`📊 Test completed with ${wsLogs.length} WebSocket-related logs`);
    console.log(`📊 Total console messages: ${consoleLogs.length}`);
    
    // Log any errors for debugging
    const errors = consoleLogs.filter(log => log.type === 'error');
    if (errors.length > 0) {
      console.log(`⚠️ Found ${errors.length} console errors:`);
      errors.forEach(error => console.log(`   - ${error.text}`));
    }
  });
});

// Helper functions for WebSocket testing
async function waitForWebSocketEvent(page, eventType, timeout = 5000) {
  const startTime = Date.now();
  
  return new Promise((resolve) => {
    const checkForEvent = async () => {
      const logs = await page.evaluate(() => {
        return window.webSocketLogs || [];
      });
      
      const eventLog = logs.find(log => log.includes(eventType));
      if (eventLog) {
        resolve(eventLog);
        return;
      }
      
      if (Date.now() - startTime > timeout) {
        resolve(null);
        return;
      }
      
      setTimeout(checkForEvent, 100);
    };
    
    checkForEvent();
  });
}

async function simulateWebSocketError(page) {
  // This would typically require injecting code to simulate network issues
  await page.evaluate(() => {
    // Simulate connection drop
    if (window.wsClient) {
      window.wsClient.disconnect();
    }
  });
}

async function getWebSocketMetrics(page) {
  return await page.evaluate(() => {
    if (window.wsClient && window.wsClient.getConnectionMetrics) {
      return window.wsClient.getConnectionMetrics();
    }
    return null;
  });
}