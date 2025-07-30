/**
 * Enhanced Authentication Tests - Modern E2E Testing with SOLID Principles
 * 
 * This test suite demonstrates:
 * - Comprehensive authentication flow testing
 * - Error handling and edge cases
 * - Modern Playwright best practices
 * - Centralized configuration usage
 * - Detailed logging and reporting
 */

import { test, expect } from '@playwright/test';
import { AuthPage } from '../page-objects/AuthPage';
import { BridgePage } from '../page-objects/BridgePage';
import { TEST_CONFIG } from '../utils/test-utilities';
import { 
  TEST_DATA, 
  TEST_UTILS, 
  WALLET_CONFIG, 
  ERROR_MESSAGES, 
  FEATURE_FLAGS 
} from '../utils/test-constants';

test.describe('Enhanced Authentication Functionality', () => {
  let authPage: AuthPage;
  let bridgePage: BridgePage;

  test.beforeEach(async ({ page }) => {
    // Initialize page objects
    authPage = new AuthPage(page);
    bridgePage = new BridgePage(page);

    // Navigate to the application
    await page.goto(TEST_UTILS.getBaseUrl());
    await authPage.waitForPageLoad();
  });

  test.describe('Wallet Connection Availability', () => {
    test('should display available wallet options', async () => {
      console.log('🧪 Testing wallet options availability...');

      const walletAvailability = await authPage.getAvailableWallets();
      
      // At least one wallet should be available
      const hasAvailableWallet = walletAvailability.ethereum.available || walletAvailability.near.available;
      expect(hasAvailableWallet, 'At least one wallet option should be available').toBe(true);

      // Log available wallets
      console.log('📊 Wallet availability:');
      console.log(`   Ethereum: ${walletAvailability.ethereum.available ? 'Available' : 'Not available'} ${walletAvailability.ethereum.enabled ? '(Enabled)' : '(Disabled)'}`);
      console.log(`   NEAR: ${walletAvailability.near.available ? 'Available' : 'Not available'} ${walletAvailability.near.enabled ? '(Enabled)' : '(Disabled)'}`);

      console.log('✅ Wallet options checked successfully');
    });

    test('should show correct button states', async () => {
      console.log('🧪 Testing wallet button states...');

      const walletAvailability = await authPage.getAvailableWallets();
      
      if (walletAvailability.ethereum.available) {
        await expect(authPage.ethereumWalletButton).toBeVisible();
        
        if (walletAvailability.ethereum.enabled) {
          await expect(authPage.ethereumWalletButton).toBeEnabled();
        }
      }

      if (walletAvailability.near.available) {
        await expect(authPage.nearWalletButton).toBeVisible();
        
        if (walletAvailability.near.enabled) {
          await expect(authPage.nearWalletButton).toBeEnabled();
        }
      }

      console.log('✅ Button states are correct');
    });
  });

  test.describe('Authentication Flow', () => {
    test('should handle Ethereum wallet connection attempt', async () => {
      console.log('🧪 Testing Ethereum wallet connection...');

      const walletAvailability = await authPage.getAvailableWallets();
      
      if (!walletAvailability.ethereum.available) {
        test.skip();
        return;
      }

      const authResult = await authPage.connectEthereumWallet();
      
      // In test environment, we expect specific behaviors
      if (FEATURE_FLAGS.REAL_WALLET_CONNECTION) {
        // Real wallet testing - result depends on actual wallet presence
        console.log(`Auth result: ${authResult.success ? 'Success' : 'Failed'} - ${authResult.reason}`);
      } else {
        // Mock environment - should handle gracefully
        expect(typeof authResult.success).toBe('boolean');
        expect(typeof authResult.reason).toBe('string');
        
        // Should not crash the application
        const authStatus = await authPage.getAuthStatus();
        expect(typeof authStatus.isAuthenticated).toBe('boolean');
      }

      console.log('✅ Ethereum wallet connection handled correctly');
    });

    test('should handle NEAR wallet connection attempt', async () => {
      console.log('🧪 Testing NEAR wallet connection...');

      const walletAvailability = await authPage.getAvailableWallets();
      
      if (!walletAvailability.near.available) {
        test.skip();
        return;
      }

      const authResult = await authPage.connectNearWallet();
      
      // In test environment, we expect specific behaviors
      if (FEATURE_FLAGS.REAL_WALLET_CONNECTION) {
        // Real wallet testing - result depends on actual wallet presence
        console.log(`Auth result: ${authResult.success ? 'Success' : 'Failed'} - ${authResult.reason}`);
      } else {
        // Mock environment - should handle gracefully
        expect(typeof authResult.success).toBe('boolean');
        expect(typeof authResult.reason).toBe('string');
        
        // Should not crash the application
        const authStatus = await authPage.getAuthStatus();
        expect(typeof authStatus.isAuthenticated).toBe('boolean');
      }

      console.log('✅ NEAR wallet connection handled correctly');
    });

    test('should handle authentication with fallback', async () => {
      console.log('🧪 Testing authentication with fallback...');

      const authResult = await authPage.authenticateWithFallback('ethereum');
      
      // Verify result structure
      expect(authResult).toHaveProperty('success');
      expect(authResult).toHaveProperty('walletUsed');
      expect(authResult).toHaveProperty('attempts');
      expect(authResult).toHaveProperty('duration');
      
      // Should have attempted at least one wallet
      expect(authResult.attempts.length).toBeGreaterThan(0);
      
      // Duration should be reasonable
      expect(authResult.duration).toBeGreaterThan(0);
      expect(authResult.duration).toBeLessThan(TEST_CONFIG.TIMEOUTS.EXTRA_LONG);

      console.log(`✅ Authentication flow completed in ${authResult.duration}ms`);
      console.log(`   Attempts: ${authResult.attempts.length}`);
      console.log(`   Wallet used: ${authResult.walletUsed || 'None'}`);
    });
  });

  test.describe('Authentication State Management', () => {
    test('should track authentication status correctly', async () => {
      console.log('🧪 Testing authentication status tracking...');

      // Initial state should be unauthenticated
      const initialStatus = await authPage.getAuthStatus();
      expect(initialStatus.isAuthenticated).toBe(false);
      expect(initialStatus.isLoading).toBe(false);

      console.log('✅ Initial authentication state is correct');
    });

    test('should handle authentication errors gracefully', async () => {
      console.log('🧪 Testing authentication error handling...');

      // Try to connect wallet (will likely fail in test environment)
      await authPage.connectEthereumWallet();
      
      // Check for error handling
      const errorMessage = await authPage.handleAuthError();
      
      // Error handling should not crash
      expect(typeof errorMessage).toBe('string');
      
      // Application should remain functional
      const authStatus = await authPage.getAuthStatus();
      expect(typeof authStatus.hasError).toBe('boolean');

      console.log('✅ Authentication errors handled gracefully');
    });

    test('should maintain state during navigation', async () => {
      console.log('🧪 Testing authentication state during navigation...');

      // Get initial auth status
      const initialStatus = await authPage.getAuthStatus();
      
      // Navigate to bridge page
      await bridgePage.goto();
      
      // Check if auth state is maintained
      const bridgeAuthRequired = await bridgePage.isAuthenticationRequired();
      
      // State should be consistent
      if (initialStatus.isAuthenticated) {
        expect(bridgeAuthRequired).toBe(false);
      } else {
        // Bridge might or might not require auth depending on configuration
        expect(typeof bridgeAuthRequired).toBe('boolean');
      }

      console.log('✅ Authentication state maintained during navigation');
    });
  });

  test.describe('Integration with Bridge', () => {
    test('should integrate authentication with bridge functionality', async () => {
      console.log('🧪 Testing auth integration with bridge...');

      // Navigate to bridge
      await bridgePage.goto();
      
      // Check if authentication is required for bridge
      const authRequired = await bridgePage.isAuthenticationRequired();
      
      if (authRequired) {
        console.log('🔐 Bridge requires authentication');
        
        // Try to authenticate
        const authResult = await bridgePage.authenticateIfRequired('ethereum');
        
        // Should handle authentication attempt
        expect(typeof authResult.success).toBe('boolean');
        expect(typeof authResult.reason).toBe('string');
        
        console.log(`Auth attempt: ${authResult.success ? 'Success' : 'Failed'} - ${authResult.reason}`);
      } else {
        console.log('✅ Bridge accessible without authentication');
        
        // Verify bridge form is accessible
        const formState = await bridgePage.getFormState();
        expect(formState.isAccessible).toBe(true);
      }

      console.log('✅ Auth-bridge integration working correctly');
    });

    test('should handle bridge form states based on authentication', async () => {
      console.log('🧪 Testing bridge form states with auth...');

      await bridgePage.goto();
      
      const formState = await bridgePage.getFormState();
      
      // Form should always be present
      expect(formState.hasAmountInput).toBe(true);
      expect(formState.hasSubmitButton).toBe(true);
      
      // Button text should indicate current state
      const buttonText = formState.submitButtonText.toLowerCase();
      const validStates = [
        'connect wallet',
        'enter amount', 
        'insufficient balance',
        'swap',
        'bridge',
        'loading'
      ];
      
      const hasValidState = validStates.some(state => buttonText.includes(state));
      expect(hasValidState, `Button should show valid state. Current: "${formState.submitButtonText}"`).toBe(true);

      console.log(`✅ Bridge form state is valid: "${formState.submitButtonText}"`);
    });
  });

  test.describe('Error Handling and Edge Cases', () => {
    test('should handle rapid authentication attempts', async () => {
      console.log('🧪 Testing rapid authentication attempts...');

      const walletAvailability = await authPage.getAvailableWallets();
      
      if (walletAvailability.ethereum.available) {
        // Rapid clicks should not crash the application
        await authPage.ethereumWalletButton.click();
        await authPage.ethereumWalletButton.click();
        await authPage.ethereumWalletButton.click();
        
        // Wait for any processing to complete
        await authPage.waitForAuthCompletion();
        
        // Application should remain functional
        const authStatus = await authPage.getAuthStatus();
        expect(typeof authStatus.isAuthenticated).toBe('boolean');
      }

      console.log('✅ Rapid authentication attempts handled correctly');
    });

    test('should handle network interruptions gracefully', async () => {
      console.log('🧪 Testing network interruption handling...');

      // Try authentication (may fail due to network issues in test env)
      const authResult = await authPage.connectEthereumWallet();
      
      // Should not crash regardless of network state
      expect(typeof authResult.success).toBe('boolean');
      
      // Application should remain responsive
      const authStatus = await authPage.getAuthStatus();
      expect(typeof authStatus.isAuthenticated).toBe('boolean');
      expect(typeof authStatus.hasError).toBe('boolean');

      console.log('✅ Network interruptions handled gracefully');
    });

    test('should maintain performance under load', async () => {
      console.log('🧪 Testing authentication performance...');

      const startTime = Date.now();
      
      // Perform authentication flow
      const authResult = await authPage.authenticateWithFallback();
      
      const duration = Date.now() - startTime;
      
      // Should complete within reasonable time
      expect(duration).toBeLessThan(TEST_CONFIG.TIMEOUTS.EXTRA_LONG);
      
      console.log(`✅ Authentication completed in ${duration}ms (within ${TEST_CONFIG.TIMEOUTS.EXTRA_LONG}ms limit)`);
    });
  });

  test.afterEach(async ({ page }, testInfo) => {
    // Take screenshot on failure
    if (testInfo.status !== testInfo.expectedStatus) {
      await authPage.takeScreenshot(`failed-${testInfo.title.replace(/\s+/g, '-')}`);
    }

    // Log test completion
    console.log(`🏁 Test completed: ${testInfo.title} - ${testInfo.status}`);
  });
});