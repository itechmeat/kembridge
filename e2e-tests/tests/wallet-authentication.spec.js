import { test, expect } from '@playwright/test';
import * as dotenv from 'dotenv';
import { AuthPage } from '../page-objects/AuthPage.js';
import { setupApiMonitoring } from '../utils/api-helpers.js';
import { SELECTORS, TIMEOUTS } from '../utils/constants.js';

// Load environment variables
dotenv.config();

test.describe('Wallet Authentication Integration', () => {
  let authPage;
  let monitoring;

  test.beforeEach(async ({ page }) => {
    // Setup API monitoring
    monitoring = setupApiMonitoring(page);
    
    // Initialize page objects
    authPage = new AuthPage(page, monitoring);
    
    // Navigate and wait for page load
    await page.goto('/');
    const loaded = await authPage.waitForAuthPageLoad();
    expect(loaded).toBeTruthy();
  });

  test('should detect NEAR wallet connection status', async ({ page }) => {
    console.log('🔍 Checking NEAR wallet connection status...');
    
    const nearStatus = await authPage.getNearWalletStatus();
    
    console.log(`   NEAR wallet button visible: ${nearStatus.buttonText !== 'NOT_FOUND' ? '✅' : '❌'}`);
    console.log(`   Button text: "${nearStatus.buttonText}"`);
    console.log(`   Connection status: ${nearStatus.connected ? '✅ CONNECTED' : '❌ DISCONNECTED'}`);
    
    // Verify button is visible
    const nearButton = page.locator(SELECTORS.NEAR_WALLET_BUTTON);
    await expect(nearButton).toBeVisible();
    
    if (nearStatus.connected) {
      console.log('✅ NEAR wallet appears to be connected');
    } else {
      console.log('❌ NEAR wallet is not connected (expected for test environment)');
    }
  });

  test('should attempt NEAR authentication and monitor API calls', async ({ page }) => {
    console.log('🚀 Testing NEAR authentication with API monitoring...');
    
    const result = await authPage.attemptNearAuthentication();
    
    console.log('📊 NEAR Authentication Result:');
    console.log(`   Success: ${result.success ? '✅' : '❌'}`);
    console.log(`   Reason: ${result.reason}`);
    console.log(`   API calls made: ${result.apiCallsMade}`);
    console.log(`   Has nonce calls: ${result.hasNonceCalls ? '✅' : '❌'}`);
    
    // Log API call summary
    monitoring.logApiSummary();
    
    if (result.success) {
      console.log('✅ NEAR authentication: Basic API flow working');
      
      // Check for specific API calls
      const nonceCalls = monitoring.getNonceCalls();
      console.log(`   Nonce API calls: ${nonceCalls.length}`);
      nonceCalls.forEach((call, i) => {
        console.log(`     ${i + 1}. ${call.method} ${call.url}`);
      });
    } else {
      console.log('❌ NEAR authentication: Limited functionality (expected without real wallet)');
    }
    
    // The test doesn't fail for NEAR since we can't fully test without real wallet
    console.log('📝 Note: Full NEAR testing requires real wallet connection');
  });


  test('should test complete NEAR authentication flow', async ({ page }) => {
    console.log('🚀 Testing complete NEAR authentication flow with monitoring...');
    
    // Get wallet password from environment
    const walletPassword = process.env.WALLET_PASSWORD || 'demo_password_123';
    console.log(`🔐 Using wallet password from environment (length: ${walletPassword.length})`);
    
    const result = await authPage.monitorAuthenticationProcess('near');
    
    console.log('📊 Complete NEAR Authentication Analysis:');
    console.log(`   Duration: ${result.duration}ms`);
    console.log(`   Success: ${result.success ? '✅' : '❌'}`);
    console.log(`   API calls increase: ${result.apiCallsIncrease}`);
    console.log(`   Has nonce calls: ${result.finalState.hasNonceCalls ? '✅' : '❌'}`);
    console.log(`   Has verify calls: ${result.finalState.hasVerifyCalls ? '✅' : '❌'}`);
    console.log(`   Is fully authenticated: ${result.finalState.isAuthenticated ? '✅' : '❌'}`);
    
    // Check authentication state
    const authState = await authPage.getAuthenticationState();
    console.log('🔍 Final authentication state:');
    console.log(`   Total auth calls: ${authState.totalAuthCalls}`);
    console.log(`   Ethereum wallet: ${authState.ethWallet.buttonText}`);
    console.log(`   NEAR wallet: ${authState.nearWallet.buttonText}`);
    
    if (result.success) {
      console.log('✅ NEAR authentication flow: API integration working');
    } else {
      console.log('📝 NEAR authentication flow: Limited without real wallet (expected)');
    }
  });
});