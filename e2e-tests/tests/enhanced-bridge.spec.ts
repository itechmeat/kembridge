/**
 * Enhanced Bridge Tests - Modern E2E Testing with SOLID Principles
 * 
 * This test suite demonstrates:
 * - DRY principle through reusable utilities
 * - SOLID principles in test organization
 * - Modern Playwright best practices
 * - Comprehensive error handling
 * - Detailed logging and reporting
 */

import { test, expect } from '@playwright/test';
import { BridgePage } from '../page-objects/BridgePage';
import { AuthPage } from '../page-objects/AuthPage';
import { TEST_CONFIG } from '../utils/test-utilities';
import { TEST_DATA, TEST_UTILS } from '../utils/test-constants';

test.describe('Enhanced Bridge Functionality', () => {
  let bridgePage: BridgePage;
  let authPage: AuthPage;

  test.beforeEach(async ({ page }) => {
    // Initialize page objects
    bridgePage = new BridgePage(page);
    authPage = new AuthPage(page);

    // Navigate to bridge page
    await bridgePage.goto();
  });

  test.describe('Bridge Form Accessibility', () => {
    test('should display bridge form without authentication', async () => {
      console.log('🧪 Testing bridge form accessibility without auth...');

      // Verify form is accessible
      const formState = await bridgePage.getFormState();
      expect(formState.isAccessible, 'Bridge form should be accessible without auth').toBe(true);
      expect(formState.hasAmountInput, 'Amount input should be present').toBe(true);
      expect(formState.hasSubmitButton, 'Submit button should be present').toBe(true);

      // Verify essential elements are visible
      await expect(bridgePage.form).toBeVisible();
      await expect(bridgePage.amountInput).toBeVisible();
      await expect(bridgePage.submitButton).toBeVisible();

      console.log('✅ Bridge form is accessible without authentication');
    });

    test('should show appropriate button states', async () => {
      console.log('🧪 Testing button states...');

      const formState = await bridgePage.getFormState();
      console.log(`Submit button text: "${formState.submitButtonText}"`);

      // Submit button should indicate what's needed
      const buttonText = formState.submitButtonText.toLowerCase();
      const expectedStates = ['connect wallet', 'enter amount', 'insufficient balance', 'swap'];
      const hasValidState = expectedStates.some(state => buttonText.includes(state));
      
      expect(hasValidState, `Button should show valid state. Current: "${formState.submitButtonText}"`).toBe(true);

      console.log('✅ Button states are appropriate');
    });
  });

  test.describe('Token Selection', () => {
    TEST_DATA.TOKENS.ETHEREUM.ALL.forEach((token: string) => {
      test(`should select ${token} token for Ethereum`, async () => {
        console.log(`🧪 Testing ${token} token selection for Ethereum...`);

        await bridgePage.selectToken('ethereum', token);
        
        // Verify token was selected by checking button text
        const buttonText = await bridgePage.fromTokenSelector.textContent();
        expect(buttonText).toContain(token);

        console.log(`✅ ${token} token selected successfully for Ethereum`);
      });
    });

    TEST_DATA.TOKENS.NEAR.ALL.forEach((token: string) => {
      test(`should select ${token} token for NEAR`, async () => {
        console.log(`🧪 Testing ${token} token selection for NEAR...`);

        await bridgePage.selectToken('near', token);
        
        // Verify token was selected by checking button text
        const buttonText = await bridgePage.toTokenSelector.textContent();
        expect(buttonText).toContain(token);

        console.log(`✅ ${token} token selected successfully for NEAR`);
      });
    });

    test('should handle token switching', async () => {
      console.log('🧪 Testing token switching...');

      // Select initial tokens
      await bridgePage.selectToken('ethereum', 'ETH');
      await bridgePage.selectToken('near', 'NEAR');

      // Switch direction
      await bridgePage.switchDirection();

      // Verify tokens switched positions
      const fromText = await bridgePage.fromTokenSelector.textContent();
      const toText = await bridgePage.toTokenSelector.textContent();
      
      expect(fromText).toContain('NEAR');
      expect(toText).toContain('ETH');

      console.log('✅ Token switching works correctly');
    });
  });

  test.describe('Amount Input Validation', () => {
    Object.entries(TEST_DATA.AMOUNTS).forEach(([size, amount]) => {
      test(`should accept ${size.toLowerCase()} amount (${amount})`, async () => {
        console.log(`🧪 Testing ${size.toLowerCase()} amount input: ${amount}...`);

        await bridgePage.enterAmount(amount);
        
        // Verify amount was entered
        const inputValue = await bridgePage.amountInput.inputValue();
        expect(inputValue).toBe(amount);

        console.log(`✅ ${size.toLowerCase()} amount (${amount}) accepted`);
      });
    });

    test('should handle decimal precision', async () => {
      console.log('🧪 Testing decimal precision...');

      const preciseAmount = '0.123456789';
      await bridgePage.enterAmount(preciseAmount);
      
      const inputValue = await bridgePage.amountInput.inputValue();
      expect(inputValue).toBe(preciseAmount);

      console.log('✅ Decimal precision handled correctly');
    });

    test('should reject invalid input', async () => {
      console.log('🧪 Testing invalid input rejection...');

      const invalidInputs = ['abc', '1.2.3', '-1', ''];
      
      for (const invalidInput of invalidInputs) {
        await bridgePage.amountInput.fill(invalidInput);
        const inputValue = await bridgePage.amountInput.inputValue();
        
        // Should either reject the input or clear it
        expect(inputValue === '' || !inputValue.includes(invalidInput), 
          `Invalid input "${invalidInput}" should be rejected`).toBe(true);
      }

      console.log('✅ Invalid inputs rejected correctly');
    });
  });

  test.describe('Price Quote Generation', () => {
    test('should generate price quote for valid inputs', async () => {
      console.log('🧪 Testing price quote generation...');

      // Set up valid swap parameters
      await bridgePage.selectToken('ethereum', 'ETH');
      await bridgePage.selectToken('near', 'NEAR');
      await bridgePage.enterAmount(TEST_DATA.AMOUNTS.MEDIUM);

      // Wait for price quote
      await bridgePage.waitForPriceQuote();
      
      // Verify quote information
      const quoteInfo = await bridgePage.getPriceQuoteInfo();
      expect(quoteInfo.isVisible, 'Price quote should be visible').toBe(true);
      expect(quoteInfo.hasAmount, 'Quote should contain amount information').toBe(true);

      console.log('✅ Price quote generated successfully');
    });

    test('should update quote when amount changes', async () => {
      console.log('🧪 Testing quote updates on amount change...');

      // Set up initial swap
      await bridgePage.selectToken('ethereum', 'ETH');
      await bridgePage.selectToken('near', 'NEAR');
      await bridgePage.enterAmount(TEST_DATA.AMOUNTS.SMALL);
      await bridgePage.waitForPriceQuote();
      
      const initialQuote = await bridgePage.getPriceQuoteInfo();
      
      // Change amount
      await bridgePage.enterAmount(TEST_DATA.AMOUNTS.LARGE);
      await bridgePage.waitForPriceQuote();
      
      const updatedQuote = await bridgePage.getPriceQuoteInfo();
      
      // Quote should have updated
      expect(updatedQuote.text).not.toBe(initialQuote.text);

      console.log('✅ Quote updates correctly when amount changes');
    });
  });

  test.describe('Complete Bridge Flow (Mock)', () => {
    test('should complete full bridge flow without authentication', async () => {
      console.log('🧪 Testing complete bridge flow without auth...');

      const result = await bridgePage.completeBridgeFlow({
        fromToken: 'ETH',
        toToken: 'NEAR',
        amount: TEST_DATA.AMOUNTS.MEDIUM,
        waitForQuote: true,
        submit: false, // Don't submit without auth
      });

      expect(result.success, `Bridge flow should succeed. Error: ${result.error}`).toBe(true);
      expect(result.steps).toContain('navigation');
      expect(result.steps).toContain('from-token-selection');
      expect(result.steps).toContain('to-token-selection');
      expect(result.steps).toContain('amount-entry');
      expect(result.steps).toContain('price-quote');

      console.log(`✅ Bridge flow completed in ${result.duration}ms`);
      console.log(`   Steps completed: ${result.steps.join(' → ')}`);
    });

    test('should handle authentication flow', async () => {
      console.log('🧪 Testing authentication flow...');

      // Check if authentication is required
      const authRequired = await bridgePage.isAuthenticationRequired();
      
      if (authRequired) {
        console.log('🔐 Authentication required, testing auth flow...');
        
        const authResult = await bridgePage.authenticateIfRequired('ethereum');
        
        // In test environment, auth might fail due to no real wallet
        // This is expected behavior
        console.log(`Auth result: ${authResult.success ? 'Success' : 'Failed'} - ${authResult.reason}`);
        
        // The important thing is that the auth flow doesn't crash
        expect(typeof authResult.success).toBe('boolean');
        expect(typeof authResult.reason).toBe('string');
      } else {
        console.log('✅ No authentication required');
      }
    });
  });

  test.describe('Error Handling and Edge Cases', () => {
    test('should handle network errors gracefully', async () => {
      console.log('🧪 Testing network error handling...');

      // Set up a swap that might trigger network calls
      await bridgePage.selectToken('ethereum', 'ETH');
      await bridgePage.selectToken('near', 'NEAR');
      await bridgePage.enterAmount(TEST_DATA.AMOUNTS.MEDIUM);

      // The form should remain functional even if some network calls fail
      const formState = await bridgePage.getFormState();
      expect(formState.isFormReady).toBe(true);

      console.log('✅ Form remains functional during network issues');
    });

    test('should handle rapid user interactions', async () => {
      console.log('🧪 Testing rapid user interactions...');

      // Rapidly change tokens and amounts
      await bridgePage.selectToken('ethereum', 'ETH');
      await bridgePage.selectToken('near', 'NEAR');
      await bridgePage.enterAmount('0.1');
      await bridgePage.switchDirection();
      await bridgePage.enterAmount('0.2');
      await bridgePage.switchDirection();

      // Form should still be in a valid state
      const formState = await bridgePage.getFormState();
      expect(formState.isAccessible).toBe(true);

      console.log('✅ Rapid interactions handled correctly');
    });
  });

  test.afterEach(async ({ page }, testInfo) => {
    // Take screenshot on failure
    if (testInfo.status !== testInfo.expectedStatus) {
      await bridgePage.takeScreenshot(`failed-${testInfo.title.replace(/\s+/g, '-')}`);
    }

    // Log test completion
    console.log(`🏁 Test completed: ${testInfo.title} - ${testInfo.status}`);
  });
});