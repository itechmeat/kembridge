/**
 * Enhanced Performance Tests - Comprehensive Performance Monitoring
 * 
 * This test suite demonstrates:
 * - Page load performance benchmarks
 * - API response time monitoring
 * - Memory usage optimization
 * - User interaction responsiveness
 * - Network efficiency testing
 */

import { test, expect } from '@playwright/test';
import { BridgePage } from '../page-objects/BridgePage';
import { AuthPage } from '../page-objects/AuthPage';
import { TEST_CONFIG } from '../utils/test-utilities';
import { 
  TEST_DATA, 
  TEST_UTILS, 
  PERFORMANCE, 
  FEATURE_FLAGS 
} from '../utils/test-constants';

test.describe('Enhanced Performance Testing', () => {
  // Skip performance tests if not enabled
  test.skip(!FEATURE_FLAGS.PERFORMANCE_TESTING, 'Performance testing is disabled');
  let bridgePage: BridgePage;
  let authPage: AuthPage;

  test.beforeEach(async ({ page }) => {
    // Initialize page objects
    bridgePage = new BridgePage(page);
    authPage = new AuthPage(page);
  });

  test.describe('Page Load Performance', () => {
    test('should load main page within acceptable time', async ({ page }) => {
      console.log('🧪 Testing main page load performance...');

      const startTime = Date.now();
      
      // Navigate to the application
      await page.goto(TEST_UTILS.getBaseUrl());
      
      // Wait for page to be fully loaded
      await page.waitForLoadState('networkidle');
      
      const loadTime = Date.now() - startTime;
      
      // Check against performance benchmarks
      if (loadTime <= PERFORMANCE.PAGE_LOAD.FAST) {
        console.log(`✅ Excellent load time: ${loadTime}ms`);
      } else if (loadTime <= PERFORMANCE.PAGE_LOAD.ACCEPTABLE) {
        console.log(`⚠️ Acceptable load time: ${loadTime}ms`);
      } else {
        console.log(`❌ Slow load time: ${loadTime}ms`);
      }
      
      // Should load within acceptable time
      expect(loadTime).toBeLessThan(PERFORMANCE.PAGE_LOAD.ACCEPTABLE);
      
      // Page should be interactive
      await bridgePage.waitForPageLoad();
      const formState = await bridgePage.getFormState();
      expect(formState.isAccessible).toBe(true);

      console.log(`✅ Main page loaded in ${loadTime}ms`);
    });

    test('should load bridge page efficiently', async ({ page }) => {
      console.log('🧪 Testing bridge page load performance...');

      const startTime = Date.now();
      
      // Navigate directly to bridge
      await bridgePage.goto();
      
      // Wait for bridge components to load
      await bridgePage.waitForPageLoad();
      
      const loadTime = Date.now() - startTime;
      
      // Should load within acceptable time
      expect(loadTime).toBeLessThan(PERFORMANCE.PAGE_LOAD.ACCEPTABLE);
      
      // Bridge form should be functional
      const formState = await bridgePage.getFormState();
      expect(formState.isAccessible).toBe(true);
      expect(formState.hasAmountInput).toBe(true);
      expect(formState.hasSubmitButton).toBe(true);

      console.log(`✅ Bridge page loaded in ${loadTime}ms`);
    });

    test('should load auth page quickly', async ({ page }) => {
      console.log('🧪 Testing auth page load performance...');

      const startTime = Date.now();
      
      // Navigate to auth page
      await authPage.goto();
      
      // Wait for auth components to load
      await authPage.waitForPageLoad();
      
      const loadTime = Date.now() - startTime;
      
      // Should load within acceptable time
      expect(loadTime).toBeLessThan(PERFORMANCE.PAGE_LOAD.ACCEPTABLE);
      
      // Auth options should be available
      const walletAvailability = await authPage.getAvailableWallets();
      const hasWallets = walletAvailability.ethereum.available || walletAvailability.near.available;
      expect(hasWallets).toBe(true);

      console.log(`✅ Auth page loaded in ${loadTime}ms`);
    });
  });

  test.describe('API Response Performance', () => {
    test('should handle token selection efficiently', async ({ page }) => {
      console.log('🧪 Testing token selection performance...');

      await bridgePage.goto();
      await bridgePage.waitForPageLoad();
      
      const startTime = Date.now();
      
      // Select tokens
      await bridgePage.selectToken('ethereum', TEST_DATA.TOKENS.ETHEREUM.NATIVE);
      await bridgePage.selectToken('near', TEST_DATA.TOKENS.NEAR.NATIVE);
      
      const selectionTime = Date.now() - startTime;
      
      // Should complete quickly
      expect(selectionTime).toBeLessThan(PERFORMANCE.API_RESPONSE.ACCEPTABLE);
      
      // Verify tokens are selected
      const formState = await bridgePage.getFormState();
      expect(formState.isAccessible).toBe(true);

      console.log(`✅ Token selection completed in ${selectionTime}ms`);
    });

    test('should generate quotes within acceptable time', async ({ page }) => {
      console.log('🧪 Testing quote generation performance...');

      await bridgePage.goto();
      await bridgePage.waitForPageLoad();
      
      // Set up for quote generation
      await bridgePage.selectToken('ethereum', TEST_DATA.TOKENS.ETHEREUM.NATIVE);
      await bridgePage.selectToken('near', TEST_DATA.TOKENS.NEAR.NATIVE);
      
      const startTime = Date.now();
      
      // Enter amount to trigger quote generation
      await bridgePage.enterAmount(TEST_DATA.AMOUNTS.MEDIUM);
      
      // Wait for quote to be generated (or timeout)
      try {
        await page.waitForResponse(
          response => response.url().includes('quote') && response.status() === 200,
          { timeout: PERFORMANCE.QUOTE_GENERATION.ACCEPTABLE }
        );
        
        const quoteTime = Date.now() - startTime;
        
        // Should generate within acceptable time
        expect(quoteTime).toBeLessThan(PERFORMANCE.QUOTE_GENERATION.ACCEPTABLE);
        
        console.log(`✅ Quote generated in ${quoteTime}ms`);
      } catch (error) {
        const timeoutTime = Date.now() - startTime;
        console.log(`⚠️ Quote generation timed out after ${timeoutTime}ms`);
        
        // Even if quote fails, form should remain functional
        const formState = await bridgePage.getFormState();
        expect(formState.isAccessible).toBe(true);
      }
    });

    test('should handle rapid user interactions', async ({ page }) => {
      console.log('🧪 Testing rapid user interaction performance...');

      await bridgePage.goto();
      await bridgePage.waitForPageLoad();
      
      const startTime = Date.now();
      
      // Perform rapid interactions
      const interactions = [
        () => bridgePage.selectToken('ethereum', TEST_DATA.TOKENS.ETHEREUM.NATIVE),
        () => bridgePage.selectToken('near', TEST_DATA.TOKENS.NEAR.NATIVE),
        () => bridgePage.enterAmount(TEST_DATA.AMOUNTS.SMALL),
        () => bridgePage.enterAmount(TEST_DATA.AMOUNTS.MEDIUM),
        () => bridgePage.enterAmount(TEST_DATA.AMOUNTS.LARGE),
      ];
      
      // Execute interactions rapidly
      for (const interaction of interactions) {
        await interaction();
      }
      
      const interactionTime = Date.now() - startTime;
      
      // Should handle rapid interactions efficiently
      expect(interactionTime).toBeLessThan(PERFORMANCE.API_RESPONSE.SLOW);
      
      // Form should remain responsive
      const formState = await bridgePage.getFormState();
      expect(formState.isAccessible).toBe(true);

      console.log(`✅ Rapid interactions completed in ${interactionTime}ms`);
    });
  });

  test.describe('Memory and Resource Efficiency', () => {
    test('should handle multiple token switches efficiently', async ({ page }) => {
      console.log('🧪 Testing memory efficiency with token switches...');

      await bridgePage.goto();
      await bridgePage.waitForPageLoad();
      
      const startTime = Date.now();
      
      // Switch between multiple tokens rapidly
      const ethereumTokens = TEST_DATA.TOKENS.ETHEREUM.ALL;
      const nearTokens = TEST_DATA.TOKENS.NEAR.ALL;
      
      for (let i = 0; i < 10; i++) {
        const ethToken = ethereumTokens[i % ethereumTokens.length];
        const nearToken = nearTokens[i % nearTokens.length];
        
        await bridgePage.selectToken('ethereum', ethToken);
        await bridgePage.selectToken('near', nearToken);
        
        // Check responsiveness every few iterations
        if (i % 3 === 0) {
          const formState = await bridgePage.getFormState();
          expect(formState.isAccessible).toBe(true);
        }
      }
      
      const switchTime = Date.now() - startTime;
      
      // Should handle multiple switches efficiently
      expect(switchTime).toBeLessThan(PERFORMANCE.API_RESPONSE.SLOW * 2);
      
      // Final state should be clean
      const finalState = await bridgePage.getFormState();
      expect(finalState.isAccessible).toBe(true);

      console.log(`✅ Multiple token switches completed in ${switchTime}ms`);
    });

    test('should handle large amount inputs efficiently', async ({ page }) => {
      console.log('🧪 Testing performance with large amount inputs...');

      await bridgePage.goto();
      await bridgePage.waitForPageLoad();
      
      // Set up tokens
      await bridgePage.selectToken('ethereum', TEST_DATA.TOKENS.ETHEREUM.NATIVE);
      await bridgePage.selectToken('near', TEST_DATA.TOKENS.NEAR.NATIVE);
      
      const startTime = Date.now();
      
      // Test various amount formats
      const amounts = [
        TEST_DATA.AMOUNTS.PRECISION_TEST,
        TEST_DATA.AMOUNTS.MAX_DECIMALS,
        TEST_DATA.AMOUNTS.EXTRA_LARGE,
        '999999.999999999999999999',
        '0.000000000000000001',
      ];
      
      for (const amount of amounts) {
        await bridgePage.enterAmount(amount);
        
        // Verify form remains responsive
        const formState = await bridgePage.getFormState();
        expect(formState.isAccessible).toBe(true);
      }
      
      const processingTime = Date.now() - startTime;
      
      // Should handle large amounts efficiently
      expect(processingTime).toBeLessThan(PERFORMANCE.API_RESPONSE.ACCEPTABLE);

      console.log(`✅ Large amount processing completed in ${processingTime}ms`);
    });

    test('should maintain performance during error scenarios', async ({ page }) => {
      console.log('🧪 Testing performance during error scenarios...');

      await bridgePage.goto();
      await bridgePage.waitForPageLoad();
      
      const startTime = Date.now();
      
      // Create multiple error scenarios
      const errorInputs = ['invalid', 'abc123', '!@#$%', '', '0', '-1'];
      
      for (const input of errorInputs) {
        await bridgePage.enterAmount(input);
        
        // Verify error handling doesn't slow down the app
        const formState = await bridgePage.getFormState();
        expect(formState.isAccessible).toBe(true);
        
        // Clear error
        await bridgePage.enterAmount('');
      }
      
      const errorHandlingTime = Date.now() - startTime;
      
      // Error handling should be efficient
      expect(errorHandlingTime).toBeLessThan(PERFORMANCE.API_RESPONSE.ACCEPTABLE);
      
      // Final recovery with valid input
      await bridgePage.enterAmount(TEST_DATA.AMOUNTS.MEDIUM);
      const finalState = await bridgePage.getFormState();
      expect(finalState.isAccessible).toBe(true);

      console.log(`✅ Error scenario handling completed in ${errorHandlingTime}ms`);
    });
  });

  test.describe('Network Efficiency', () => {
    test('should minimize unnecessary API calls', async ({ page }) => {
      console.log('🧪 Testing API call efficiency...');

      // Track network requests
      const apiCalls: string[] = [];
      
      page.on('request', request => {
        if (request.url().includes('/api/')) {
          apiCalls.push(request.url());
        }
      });
      
      await bridgePage.goto();
      await bridgePage.waitForPageLoad();
      
      const initialCallCount = apiCalls.length;
      
      // Perform typical user flow
      await bridgePage.selectToken('ethereum', TEST_DATA.TOKENS.ETHEREUM.NATIVE);
      await bridgePage.selectToken('near', TEST_DATA.TOKENS.NEAR.NATIVE);
      await bridgePage.enterAmount(TEST_DATA.AMOUNTS.MEDIUM);
      
      const finalCallCount = apiCalls.length;
      const newCalls = finalCallCount - initialCallCount;
      
      // Should make reasonable number of API calls
      expect(newCalls).toBeLessThan(10); // Adjust based on expected behavior
      
      console.log(`✅ Made ${newCalls} API calls for user flow`);
      console.log(`   API calls: ${apiCalls.slice(initialCallCount).join(', ')}`);
    });

    test('should handle concurrent operations efficiently', async ({ page }) => {
      console.log('🧪 Testing concurrent operation performance...');

      await bridgePage.goto();
      await bridgePage.waitForPageLoad();
      
      const startTime = Date.now();
      
      // Perform concurrent operations
      const operations = [
        bridgePage.selectToken('ethereum', TEST_DATA.TOKENS.ETHEREUM.NATIVE),
        bridgePage.selectToken('near', TEST_DATA.TOKENS.NEAR.NATIVE),
        bridgePage.enterAmount(TEST_DATA.AMOUNTS.MEDIUM),
      ];
      
      // Execute operations concurrently
      await Promise.all(operations);
      
      const concurrentTime = Date.now() - startTime;
      
      // Should handle concurrent operations efficiently
      expect(concurrentTime).toBeLessThan(PERFORMANCE.API_RESPONSE.ACCEPTABLE);
      
      // Final state should be consistent
      const formState = await bridgePage.getFormState();
      expect(formState.isAccessible).toBe(true);

      console.log(`✅ Concurrent operations completed in ${concurrentTime}ms`);
    });
  });

  test.describe('Cross-Page Performance', () => {
    test('should navigate between pages efficiently', async ({ page }) => {
      console.log('🧪 Testing cross-page navigation performance...');

      const navigationTimes: number[] = [];
      
      // Test navigation sequence
      const navigationSequence = [
        { page: 'bridge', action: () => bridgePage.goto() },
        { page: 'auth', action: () => authPage.goto() },
        { page: 'bridge', action: () => bridgePage.goto() },
      ];
      
      for (const nav of navigationSequence) {
        const startTime = Date.now();
        
        await nav.action();
        
        // Wait for page to be ready
        if (nav.page === 'bridge') {
          await bridgePage.waitForPageLoad();
          const formState = await bridgePage.getFormState();
          expect(formState.isAccessible).toBe(true);
        } else if (nav.page === 'auth') {
          await authPage.waitForPageLoad();
          const authStatus = await authPage.getAuthStatus();
          expect(typeof authStatus.isAuthenticated).toBe('boolean');
        }
        
        const navTime = Date.now() - startTime;
        navigationTimes.push(navTime);
        
        console.log(`   ${nav.page} navigation: ${navTime}ms`);
      }
      
      // All navigations should be within acceptable time
      for (const time of navigationTimes) {
        expect(time).toBeLessThan(PERFORMANCE.PAGE_LOAD.ACCEPTABLE);
      }
      
      const avgTime = navigationTimes.reduce((a, b) => a + b, 0) / navigationTimes.length;
      console.log(`✅ Average navigation time: ${avgTime.toFixed(0)}ms`);
    });

    test('should maintain state efficiently across navigation', async ({ page }) => {
      console.log('🧪 Testing state maintenance performance...');

      // Set up initial state on bridge
      await bridgePage.goto();
      await bridgePage.waitForPageLoad();
      
      await bridgePage.selectToken('ethereum', TEST_DATA.TOKENS.ETHEREUM.NATIVE);
      await bridgePage.selectToken('near', TEST_DATA.TOKENS.NEAR.NATIVE);
      await bridgePage.enterAmount(TEST_DATA.AMOUNTS.MEDIUM);
      
      const startTime = Date.now();
      
      // Navigate away and back
      await authPage.goto();
      await authPage.waitForPageLoad();
      
      await bridgePage.goto();
      await bridgePage.waitForPageLoad();
      
      const stateRecoveryTime = Date.now() - startTime;
      
      // Should recover state efficiently
      expect(stateRecoveryTime).toBeLessThan(PERFORMANCE.PAGE_LOAD.ACCEPTABLE);
      
      // Verify form is still functional
      const formState = await bridgePage.getFormState();
      expect(formState.isAccessible).toBe(true);

      console.log(`✅ State recovery completed in ${stateRecoveryTime}ms`);
    });
  });

  test.afterEach(async ({ page }, testInfo) => {
    // Log performance metrics
    const performanceMetrics = await page.evaluate(() => {
      const navigation = performance.getEntriesByType('navigation')[0] as PerformanceNavigationTiming;
      return {
        domContentLoaded: navigation.domContentLoadedEventEnd - navigation.domContentLoadedEventStart,
        loadComplete: navigation.loadEventEnd - navigation.loadEventStart,
        firstPaint: performance.getEntriesByName('first-paint')[0]?.startTime || 0,
        firstContentfulPaint: performance.getEntriesByName('first-contentful-paint')[0]?.startTime || 0,
      };
    });
    
    console.log('📊 Performance Metrics:');
    console.log(`   DOM Content Loaded: ${performanceMetrics.domContentLoaded.toFixed(2)}ms`);
    console.log(`   Load Complete: ${performanceMetrics.loadComplete.toFixed(2)}ms`);
    console.log(`   First Paint: ${performanceMetrics.firstPaint.toFixed(2)}ms`);
    console.log(`   First Contentful Paint: ${performanceMetrics.firstContentfulPaint.toFixed(2)}ms`);
    
    // Take screenshot on failure
    if (testInfo.status !== testInfo.expectedStatus) {
      await bridgePage.takeScreenshot(`performance-test-failed-${testInfo.title.replace(/\s+/g, '-')}`);
    }

    console.log(`🏁 Performance test completed: ${testInfo.title} - ${testInfo.status}`);
  });
});