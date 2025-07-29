import { test, expect } from '@playwright/test';
import { setupFullTestEnvironment, performAuthenticationFlow } from '../utils/test-base.js';
import { AuthPage } from '../page-objects/AuthPage.js';
import { logTestResults } from '../utils/test-base.js';

test.describe('Wallet Mock Testing', () => {
  let testEnv;
  let authPage;

  test.beforeEach(async ({ page }) => {
    // Setup complete test environment with mock wallet
    testEnv = await setupFullTestEnvironment(page);
    authPage = new AuthPage(page, testEnv.monitoring);
  });

  test('should connect MetaMask with mock wallet', async ({ page }) => {
    console.log('🦊 Testing MetaMask connection with mock wallet...');
    
    // Check if mock wallet is available
    const hasEthereum = await page.evaluate(() => typeof window.ethereum !== 'undefined');
    console.log(`🔍 Mock Ethereum Provider Available: ${hasEthereum ? '✅' : '❌'}`);
    expect(hasEthereum).toBeTruthy();
    
    // Test authentication with monitoring
    const result = await authPage.monitorAuthenticationProcess('ethereum');
    
    logTestResults({
      testName: 'MetaMask Mock Wallet Connection',
      success: result.success,
      authResult: result,
      apiCallCounts: {
        'Total auth calls': result.finalState.totalAuthCalls,
        'Nonce calls': testEnv.monitoring.getNonceCalls().length,
        'Verify calls': testEnv.monitoring.getVerifyCalls().length
      },
      metrics: {
        'Authentication duration': `${result.duration}ms`,
        'API calls increase': result.apiCallsIncrease
      }
    });
    
    // Verify authentication worked
    expect(result.success).toBeTruthy();
    expect(result.finalState.hasNonceCalls).toBeTruthy();
    
    if (result.success) {
      console.log('✅ MetaMask mock wallet integration: WORKING');
    } else {
      console.log('❌ MetaMask mock wallet integration: FAILED');
    }
  });

  test('should test complete authentication flow with mock wallet', async ({ page }) => {
    console.log('🚀 Testing complete authentication flow with mock wallet...');

    const authResult = await performAuthenticationFlow(page, testEnv.monitoring);
    
    console.log('📊 Complete Authentication Flow Results:');
    console.log(`   Success: ${authResult.success ? '✅' : '❌'}`);
    console.log(`   Has nonce calls: ${authResult.hasNonceCalls ? '✅' : '❌'}`);
    console.log(`   Has verify calls: ${authResult.hasVerifyCalls ? '✅' : '❌'}`);
    console.log(`   Is complete: ${authResult.isComplete ? '✅' : '❌'}`);
    console.log(`   Total auth calls: ${authResult.totalAuthCalls}`);
    
    // Detailed API call analysis
    console.log('🔍 API Call Details:');
    const nonceCalls = testEnv.monitoring.getNonceCalls();
    const verifyCalls = testEnv.monitoring.getVerifyCalls();
    
    console.log(`   Nonce calls: ${nonceCalls.length}`);
    nonceCalls.forEach((call, i) => {
      console.log(`     ${i + 1}. ${call.method} ${call.url}`);
    });
    
    console.log(`   Verify calls: ${verifyCalls.length}`);
    verifyCalls.forEach((call, i) => {
      console.log(`     ${i + 1}. ${call.method} ${call.url}`);
    });
    
    // Assertions
    expect(authResult.success).toBeTruthy();
    expect(authResult.hasNonceCalls).toBeTruthy();
    expect(nonceCalls.length).toBeGreaterThan(0);
    
    if (authResult.isComplete) {
      console.log('✅ Complete authentication flow: FULLY WORKING');
      expect(authResult.hasVerifyCalls).toBeTruthy();
    } else {
      console.log('✅ Partial authentication flow: NONCE WORKING (verify may be pending)');
    }
  });
});