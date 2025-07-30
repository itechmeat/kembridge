/**
 * Base test utilities and common setup
 */
import { setupMockWallet, authenticateEthereumWallet } from './wallet-helpers.js';
import { setupApiMonitoring } from './api-helpers.js';  
import { navigateToBridge, waitForBridgeFormAccessible } from './element-helpers.js';
import { TIMEOUTS, SELECTORS } from './constants.js';

/**
 * Complete test setup with mock wallet and monitoring
 * @param {import('@playwright/test').Page} page 
 * @returns {Promise<Object>} Test utilities
 */
export async function setupFullTestEnvironment(page) {
  console.log('🚀 Setting up full test environment...');
  
  // Navigate to app first
  await page.goto('/');
  await page.waitForTimeout(TIMEOUTS.SHORT);
  
  // Setup mock wallet after page load
  await setupMockWallet(page);
  
  // Setup API monitoring
  const monitoring = setupApiMonitoring(page);
  
  // Wait for everything to be ready
  await page.waitForTimeout(TIMEOUTS.MEDIUM);
  
  console.log('✅ Test environment setup complete');
  
  return {
    monitoring,
    
    // Common actions
    authenticate: () => authenticateEthereumWallet(page),
    navigateToBridge: () => navigateToBridge(page),
    waitForBridgeAccessible: (timeout) => waitForBridgeFormAccessible(page, timeout),
    setupMockWallet: () => setupMockWallet(page),
    
    // Monitoring shortcuts
    getApiCalls: monitoring.getApiCalls,
    getAuthCalls: monitoring.getAuthCalls,
    getBridgeCalls: monitoring.getBridgeCalls,
    logApiSummary: monitoring.logApiSummary,
    
    // Assertion helpers
    expectAuthenticated: async () => {
      const authCalls = monitoring.getAuthCalls();
      const hasNonce = authCalls.some(call => call.url.includes('/nonce'));
      const hasVerify = authCalls.some(call => call.url.includes('/verify'));
      
      return {
        hasNonceCalls: hasNonce,
        hasVerifyCalls: hasVerify,
        isFullyAuthenticated: hasNonce && hasVerify,
        totalAuthCalls: authCalls.length
      };
    },
    
    expectBridgeFormAccessible: async () => {
      const authRequired = await page.locator(SELECTORS.AUTH_REQUIRED).isVisible().catch(() => false);
      return !authRequired;
    }
  };
}

/**
 * Common authentication flow for tests
 * @param {import('@playwright/test').Page} page 
 * @param {Object} monitoring 
 * @returns {Promise<Object>} Authentication result
 */
export async function performAuthenticationFlow(page, monitoring) {
  console.log('🔐 Starting authentication flow...');
  
  const success = await authenticateEthereumWallet(page);
  
  if (!success) {
    return {
      success: false,
      reason: 'Authentication button not available'
    };
  }
  
  // Wait for auth calls to complete
  await page.waitForTimeout(TIMEOUTS.MEDIUM);
  
  const authCalls = monitoring.getAuthCalls();
  const hasNonce = authCalls.some(call => call.url.includes('/nonce'));
  const hasVerify = authCalls.some(call => call.url.includes('/verify'));
  
  const result = {
    success: hasNonce,
    hasNonceCalls: hasNonce,
    hasVerifyCalls: hasVerify,
    totalAuthCalls: authCalls.length,
    isComplete: hasNonce && hasVerify
  };
  
  if (result.success) {
    console.log('✅ Authentication flow completed successfully');
  } else {
    console.log('❌ Authentication flow failed');
  }
  
  return result;
}

/**
 * Standard bridge flow setup (authenticate + navigate + wait for form)
 * @param {import('@playwright/test').Page} page 
 * @param {Object} monitoring 
 * @returns {Promise<Object>} Bridge setup result
 */
export async function setupBridgeFlow(page, monitoring) {
  console.log('🌉 Setting up bridge flow...');
  
  // Step 1: Authenticate
  const authResult = await performAuthenticationFlow(page, monitoring);
  if (!authResult.success) {
    return {
      success: false,
      step: 'authentication',
      error: 'Authentication failed',
      authResult
    };
  }
  
  // Step 2: Navigate to bridge
  const navSuccess = await navigateToBridge(page);
  if (!navSuccess) {
    return {
      success: false,
      step: 'navigation',
      error: 'Bridge navigation failed'
    };
  }
  
  // Step 3: Wait for form to be accessible
  const formAccessible = await waitForBridgeFormAccessible(page);
  if (!formAccessible) {
    return {
      success: false,
      step: 'form_access',
      error: 'Bridge form not accessible'
    };
  }
  
  console.log('✅ Bridge flow setup complete');
  
  return {
    success: true,
    authResult,
    formAccessible: true
  };
}

/**
 * Log comprehensive test results
 * @param {Object} testData 
 */
export function logTestResults(testData) {
  const {
    testName,
    success,
    authResult,
    apiCallCounts,
    errors = [],
    metrics = {}
  } = testData;
  
  console.log(`\n📊 ${testName} - Test Results Summary:`);
  console.log(`   Overall Success: ${success ? '✅ PASS' : '❌ FAIL'}`);
  
  if (authResult) {
    console.log(`   Authentication: ${authResult.success ? '✅ PASS' : '❌ FAIL'}`);
    console.log(`     - Nonce calls: ${authResult.hasNonceCalls ? '✅' : '❌'}`);
    console.log(`     - Verify calls: ${authResult.hasVerifyCalls ? '✅' : '❌'}`);
    console.log(`     - Total auth calls: ${authResult.totalAuthCalls}`);
  }
  
  if (apiCallCounts) {
    console.log(`   API Activity:`);
    Object.entries(apiCallCounts).forEach(([type, count]) => {
      console.log(`     - ${type}: ${count}`);
    });
  }
  
  if (errors.length > 0) {
    console.log(`   Errors Found: ${errors.length}`);
    errors.slice(0, 3).forEach((error, i) => {
      console.log(`     ${i + 1}. ${error}`);
    });
  }
  
  if (Object.keys(metrics).length > 0) {
    console.log(`   Performance Metrics:`);
    Object.entries(metrics).forEach(([metric, value]) => {
      console.log(`     - ${metric}: ${value}`);
    });
  }
  
  console.log(''); // Empty line for readability
}