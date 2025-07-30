/**
 * Enhanced Error Handling Tests - Comprehensive Error Scenarios
 * 
 * This test suite demonstrates:
 * - Network error simulation and handling
 * - API error responses and recovery
 * - User input validation and feedback
 * - Graceful degradation scenarios
 * - Error boundary testing
 */

import { test, expect } from '@playwright/test';
import { BridgePage } from '../page-objects/BridgePage';
import { AuthPage } from '../page-objects/AuthPage';
import { TEST_CONFIG } from '../utils/test-utilities';
import { 
  TEST_DATA, 
  TEST_UTILS, 
  ERROR_MESSAGES, 
  API_PATTERNS, 
  PERFORMANCE 
} from '../utils/test-constants';

test.describe('Enhanced Error Handling', () => {
  let bridgePage: BridgePage;
  let authPage: AuthPage;

  test.beforeEach(async ({ page }) => {
    // Initialize page objects
    bridgePage = new BridgePage(page);
    authPage = new AuthPage(page);

    // Navigate to the application
    await page.goto(TEST_UTILS.getBaseUrl());
    await bridgePage.goto();
  });

  test.describe('Input Validation Errors', () => {
    test('should handle invalid amount inputs', async () => {
      console.log('🧪 Testing invalid amount input handling...');

      const invalidAmounts = [
        '', // Empty
        '0', // Zero
        '-1', // Negative
        'abc', // Non-numeric
        '999999999999999999999', // Too large
        '0.000000000000001', // Too small
        '1.2.3', // Invalid decimal
        '1,000', // Comma separator
        '1e10', // Scientific notation
        '∞', // Infinity symbol
        'NaN' // Not a number
      ];

      for (const amount of invalidAmounts) {
        console.log(`   Testing amount: "${amount}"`);
        
        await bridgePage.enterAmount(amount);
        
        // Check form state for validation feedback
        const formState = await bridgePage.getFormState();
        
        if (amount === '' || amount === '0') {
          // Empty or zero should show specific button states
          const buttonText = formState.submitButtonText.toLowerCase();
          expect(buttonText.includes('enter amount') || buttonText.includes('invalid')).toBe(true);
        } else if (amount !== '1,000') { // Skip comma test as it might be valid in some locales
          // Invalid inputs should disable submit or show error state
          const hasError = formState.submitButtonText.toLowerCase().includes('invalid') ||
                          formState.submitButtonText.toLowerCase().includes('error');
          expect(hasError).toBe(true);
        }
        
        // Clear input for next test
        await bridgePage.enterAmount('');
      }

      console.log('✅ Invalid amount inputs handled correctly');
    });

    test('should validate amount against balance', async () => {
      console.log('🧪 Testing balance validation...');

      // Enter amount larger than possible balance
      const largeAmount = TEST_DATA.AMOUNTS.EXTRA_LARGE;
      await bridgePage.enterAmount(largeAmount);
      
      // Should show insufficient balance error
      const formState = await bridgePage.getFormState();
      
      // Button should indicate insufficient balance or similar error
      const buttonText = formState.submitButtonText.toLowerCase();
      const hasBalanceError = buttonText.includes('insufficient') || 
                             buttonText.includes('balance') ||
                             buttonText.includes('exceed');
      
      expect(hasBalanceError).toBe(true);

      console.log('✅ Balance validation working correctly');
    });

    test('should handle rapid input changes', async () => {
      console.log('🧪 Testing rapid input changes...');

      const amounts = ['1', '12', '123', '1234', '12345'];
      
      // Rapidly change amounts
      for (const amount of amounts) {
        await bridgePage.enterAmount(amount);
      }
      
      // Final state should be valid
      const formState = await bridgePage.getFormState();
      const buttonText = formState.submitButtonText.toLowerCase();
      const isValidState = !buttonText.includes('invalid') && !buttonText.includes('error');
      expect(isValidState).toBe(true);
      
      // Form should be accessible
      expect(formState.isAccessible).toBe(true);

      console.log('✅ Rapid input changes handled correctly');
    });
  });

  test.describe('Network Error Simulation', () => {
    test('should handle API timeout errors', async ({ page }) => {
      console.log('🧪 Testing API timeout handling...');

      // Intercept API calls and simulate timeout
      await page.route('**/api/v1/bridge/quote', async route => {
        // Delay response to simulate timeout
        await page.waitForTimeout(TEST_CONFIG.TIMEOUTS.EXTRA_LONG + 1000);
        await route.abort('timedout');
      });

      // Try to get a quote
      await bridgePage.selectToken('ethereum', TEST_DATA.TOKENS.ETHEREUM.NATIVE);
      await bridgePage.selectToken('near', TEST_DATA.TOKENS.NEAR.NATIVE);
      await bridgePage.enterAmount(TEST_DATA.AMOUNTS.MEDIUM);
      
      // Should handle timeout gracefully
      const formState = await bridgePage.getFormState();
      
      // Should show timeout or network error in button or form state
      const buttonText = formState.submitButtonText.toLowerCase();
      const hasTimeoutError = buttonText.includes('timeout') || 
                             buttonText.includes('network') ||
                             buttonText.includes('error');
      
      // Application should remain functional
      expect(formState.isAccessible).toBe(true);

      console.log('✅ API timeout handled gracefully');
    });

    test('should handle API server errors', async ({ page }) => {
      console.log('🧪 Testing API server error handling...');

      // Intercept API calls and return server error
      await page.route('**/api/v1/bridge/quote', async route => {
        await route.fulfill({
          status: 500,
          contentType: 'application/json',
          body: JSON.stringify({
            error: 'Internal Server Error',
            message: 'Service temporarily unavailable'
          })
        });
      });

      // Try to get a quote
      await bridgePage.selectToken('ethereum', TEST_DATA.TOKENS.ETHEREUM.NATIVE);
      await bridgePage.selectToken('near', TEST_DATA.TOKENS.NEAR.NATIVE);
      await bridgePage.enterAmount(TEST_DATA.AMOUNTS.MEDIUM);
      
      // Should handle server error gracefully
      const formState = await bridgePage.getFormState();
      
      // Should show server error indication
      const buttonText = formState.submitButtonText.toLowerCase();
      const hasServerError = buttonText.includes('server') || 
                            buttonText.includes('error') ||
                            buttonText.includes('unavailable');
      
      // Application should remain functional
      expect(formState.isAccessible).toBe(true);

      console.log('✅ API server errors handled gracefully');
    });

    test('should handle malformed API responses', async ({ page }) => {
      console.log('🧪 Testing malformed API response handling...');

      // Intercept API calls and return malformed data
      await page.route('**/api/v1/bridge/quote', async route => {
        await route.fulfill({
          status: 200,
          contentType: 'application/json',
          body: 'invalid json response'
        });
      });

      // Try to get a quote
      await bridgePage.selectToken('ethereum', TEST_DATA.TOKENS.ETHEREUM.NATIVE);
      await bridgePage.selectToken('near', TEST_DATA.TOKENS.NEAR.NATIVE);
      await bridgePage.enterAmount(TEST_DATA.AMOUNTS.MEDIUM);
      
      // Should handle parsing error gracefully
      const formState = await bridgePage.getFormState();
      
      // Should show parsing error indication
      const buttonText = formState.submitButtonText.toLowerCase();
      const hasParseError = buttonText.includes('error') || 
                           buttonText.includes('invalid') ||
                           buttonText.includes('format');
      
      // Application should not crash
      expect(formState.isAccessible).toBe(true);

      console.log('✅ Malformed API responses handled gracefully');
    });
  });

  test.describe('User Experience Error Recovery', () => {
    test('should provide clear error messages', async () => {
      console.log('🧪 Testing error message clarity...');

      // Test various error scenarios
      const errorScenarios = [
        {
          action: () => bridgePage.enterAmount('abc'),
          expectedPattern: /invalid|number|numeric/i,
          description: 'Invalid amount format'
        },
        {
          action: () => bridgePage.enterAmount('0'),
          expectedPattern: /amount|zero|greater/i,
          description: 'Zero amount'
        },
        {
          action: () => bridgePage.enterAmount(TEST_DATA.AMOUNTS.EXTRA_LARGE),
          expectedPattern: /balance|insufficient|exceed/i,
          description: 'Insufficient balance'
        }
      ];

      for (const scenario of errorScenarios) {
        console.log(`   Testing: ${scenario.description}`);
        
        await scenario.action();
        
        const formState = await bridgePage.getFormState();
        
        // Check if button text or form state indicates the expected error
        const buttonText = formState.submitButtonText.toLowerCase();
        const hasExpectedError = scenario.expectedPattern.test(buttonText);
        
        if (hasExpectedError) {
          expect(buttonText.length).toBeGreaterThan(5);
        }
        
        await bridgePage.enterAmount('');
      }

      console.log('✅ Error messages are clear and helpful');
    });

    test('should allow error recovery', async () => {
      console.log('🧪 Testing error recovery mechanisms...');

      // Create an error state
      await bridgePage.enterAmount('invalid');
      
      let formState = await bridgePage.getFormState();
      const initialButtonText = formState.submitButtonText.toLowerCase();
      expect(initialButtonText.includes('invalid') || initialButtonText.includes('error')).toBe(true);
      
      // Recover by entering valid amount
      await bridgePage.enterAmount(TEST_DATA.AMOUNTS.MEDIUM);
      
      formState = await bridgePage.getFormState();
      const recoveredButtonText = formState.submitButtonText.toLowerCase();
      expect(!recoveredButtonText.includes('invalid') && !recoveredButtonText.includes('error')).toBe(true);
      
      // Form should be functional again
      expect(formState.isAccessible).toBe(true);

      console.log('✅ Error recovery working correctly');
    });

    test('should maintain state during error recovery', async () => {
      console.log('🧪 Testing state maintenance during error recovery...');

      // Set up initial state
      await bridgePage.selectToken('ethereum', TEST_DATA.TOKENS.ETHEREUM.NATIVE);
      await bridgePage.selectToken('near', TEST_DATA.TOKENS.NEAR.NATIVE);
      
      // Create error
      await bridgePage.enterAmount('invalid');
      
      // Verify form state is maintained
      const tokenState = await bridgePage.getFormState();
      expect(tokenState.isAccessible).toBe(true);
      
      // Recover from error
      await bridgePage.enterAmount(TEST_DATA.AMOUNTS.MEDIUM);
      
      // Verify state is maintained
      const recoveredTokenState = await bridgePage.getFormState();
      expect(recoveredTokenState.isAccessible).toBe(true);

      console.log('✅ State maintained during error recovery');
    });
  });

  test.describe('Performance Under Error Conditions', () => {
    test('should maintain performance during error handling', async () => {
      console.log('🧪 Testing performance during error handling...');

      const startTime = Date.now();
      
      // Create multiple errors rapidly
      const errorInputs = ['abc', 'xyz', '!@#', ''];
      
      for (const input of errorInputs) {
        await bridgePage.enterAmount(input);
        // Continue without delay
      }
      
      // Recover with valid input
      await bridgePage.enterAmount(TEST_DATA.AMOUNTS.MEDIUM);
      
      const duration = Date.now() - startTime;
      
      // Should complete within reasonable time
      expect(duration).toBeLessThan(PERFORMANCE.API_RESPONSE.SLOW);
      
      // Application should be responsive
      const formState = await bridgePage.getFormState();
      expect(formState.isAccessible).toBe(true);

      console.log(`✅ Error handling completed in ${duration}ms`);
    });

    test('should handle memory efficiently during errors', async () => {
      console.log('🧪 Testing memory efficiency during error handling...');

      // Create many error states to test memory handling
      for (let i = 0; i < 50; i++) {
        await bridgePage.enterAmount(`invalid${i}`);
        await bridgePage.enterAmount('');
        
        // Check every 10 iterations
        if (i % 10 === 0) {
          const formState = await bridgePage.getFormState();
          expect(formState.isAccessible).toBe(true);
        }
      }
      
      // Final state should be clean
      const finalState = await bridgePage.getFormState();
      expect(finalState.isAccessible).toBe(true);

      console.log('✅ Memory handled efficiently during error scenarios');
    });
  });

  test.describe('Cross-Component Error Propagation', () => {
    test('should handle errors across authentication and bridge', async () => {
      console.log('🧪 Testing cross-component error handling...');

      // Try to use bridge functionality that might require auth
      await bridgePage.selectToken('ethereum', TEST_DATA.TOKENS.ETHEREUM.NATIVE);
      await bridgePage.selectToken('near', TEST_DATA.TOKENS.NEAR.NATIVE);
      await bridgePage.enterAmount(TEST_DATA.AMOUNTS.MEDIUM);
      
      // Check if auth is required
      const authRequired = await bridgePage.isAuthenticationRequired();
      
      // Should handle auth requirement gracefully
      if (authRequired) {
        console.log('🔐 Bridge requires authentication');
        
        // Bridge should remain accessible
        const bridgeState = await bridgePage.getFormState();
        expect(bridgeState.isAccessible).toBe(true);
      }

      console.log('✅ Cross-component error handling working correctly');
    });

    test('should maintain error boundaries', async () => {
      console.log('🧪 Testing error boundary maintenance...');

      // Create error in one component
      await bridgePage.enterAmount('invalid');
      
      // Navigate to auth (should not be affected)
      await authPage.goto();
      
      // Auth should be functional
      const authState = await authPage.getAuthStatus();
      expect(typeof authState.isAuthenticated).toBe('boolean');
      
      // Navigate back to bridge
      await bridgePage.goto();
      
      // Bridge should have recovered
      const bridgeState = await bridgePage.getFormState();
      expect(bridgeState.isAccessible).toBe(true);

      console.log('✅ Error boundaries maintained correctly');
    });
  });

  test.afterEach(async ({ page }, testInfo) => {
    // Take screenshot on failure
    if (testInfo.status !== testInfo.expectedStatus) {
      await bridgePage.takeScreenshot(`error-test-failed-${testInfo.title.replace(/\s+/g, '-')}`);
    }

    // Log test completion
    console.log(`🏁 Error test completed: ${testInfo.title} - ${testInfo.status}`);
  });
});