/**
 * Authentication Page Object Model
 * Handles all authentication-related interactions
 */
import { 
  authenticateEthereumWallet, 
  getWalletStatus, 
  waitForAuthenticationComplete 
} from '../utils/wallet-helpers.js';
import { SELECTORS, TIMEOUTS } from '../utils/constants.js';

export class AuthPage {
  constructor(page, monitoring) {
    this.page = page;
    this.monitoring = monitoring;
  }
  
  /**
   * Check if we're on the onboarding/auth page
   */
  async isOnAuthPage() {
    const title = this.page.locator(SELECTORS.ONBOARDING_TITLE);
    return await title.isVisible().catch(() => false);
  }
  
  /**
   * Wait for auth page to load
   */
  async waitForAuthPageLoad() {
    const title = this.page.locator(SELECTORS.ONBOARDING_TITLE);
    try {
      await title.waitFor({ timeout: TIMEOUTS.PAGE_LOAD });
      await this.page.waitForTimeout(TIMEOUTS.SHORT); // Wait for wallet initialization
      return true;
    } catch (error) {
      console.log('❌ Auth page failed to load:', error.message);
      return false;
    }
  }
  
  /**
   * Get Ethereum wallet status
   */
  async getEthereumWalletStatus() {
    return await getWalletStatus(this.page, 'ethereum');
  }
  
  /**
   * Get NEAR wallet status
   */
  async getNearWalletStatus() {
    return await getWalletStatus(this.page, 'near');
  }
  
  /**
   * Authenticate with Ethereum wallet
   */
  async authenticateEthereum() {
    console.log('🔐 Starting Ethereum authentication...');
    
    const initialApiCalls = this.monitoring.getAuthCalls().length;
    const success = await authenticateEthereumWallet(this.page);
    
    if (!success) {
      return {
        success: false,
        reason: 'Button not available or clickable'
      };
    }
    
    // Wait for authentication to complete
    const authResult = await waitForAuthenticationComplete(this.page, this.monitoring.getApiCalls());
    
    const finalApiCalls = this.monitoring.getAuthCalls().length;
    const newApiCalls = finalApiCalls - initialApiCalls;
    
    console.log(`📊 Authentication completed. New API calls: ${newApiCalls}`);
    
    return {
      success: authResult.success,
      nonceCalled: authResult.nonceCalled,
      verifyCalled: authResult.verifyCalled,
      apiCallsMade: newApiCalls,
      isComplete: authResult.success
    };
  }
  
  /**
   * Attempt NEAR wallet authentication (limited without real wallet)
   */
  async attemptNearAuthentication() {
    console.log('🔍 Attempting NEAR wallet authentication...');
    
    const nearButton = this.page.locator(SELECTORS.NEAR_WALLET_BUTTON);
    const isVisible = await nearButton.isVisible();
    
    if (!isVisible) {
      return {
        success: false,
        reason: 'NEAR wallet button not visible'
      };
    }
    
    const isEnabled = await nearButton.getAttribute('disabled') === null;
    
    if (!isEnabled) {
      return {
        success: false,
        reason: 'NEAR wallet button is disabled'
      };
    }
    
    const initialApiCalls = this.monitoring.getAuthCalls().length;
    
    // Click the button
    await nearButton.click();
    await this.page.waitForTimeout(TIMEOUTS.LONG);
    
    const finalApiCalls = this.monitoring.getAuthCalls().length;
    const apiCallsMade = finalApiCalls - initialApiCalls;
    
    // Check for nonce calls
    const nonceCalls = this.monitoring.getNonceCalls();
    const hasNonceCalls = nonceCalls.length > 0;
    
    const result = {
      success: hasNonceCalls,
      apiCallsMade,
      hasNonceCalls,
      reason: hasNonceCalls ? 'Nonce API called' : 'No authentication API calls detected'
    };
    
    if (hasNonceCalls) {
      console.log('✅ NEAR authentication: Nonce API called successfully');
    } else {
      console.log('❌ NEAR authentication: No API activity detected');
    }
    
    return result;
  }
  
  /**
   * Check authentication state by examining API calls and UI
   */
  async getAuthenticationState() {
    const authCalls = this.monitoring.getAuthCalls();
    const nonceCalls = this.monitoring.getNonceCalls();
    const verifyCalls = this.monitoring.getVerifyCalls();
    
    // Check wallet button states
    const ethStatus = await this.getEthereumWalletStatus();
    const nearStatus = await this.getNearWalletStatus();
    
    return {
      hasAuthCalls: authCalls.length > 0,
      hasNonceCalls: nonceCalls.length > 0,
      hasVerifyCalls: verifyCalls.length > 0,
      totalAuthCalls: authCalls.length,
      ethWallet: ethStatus,
      nearWallet: nearStatus,
      isAuthenticated: nonceCalls.length > 0 && verifyCalls.length > 0
    };
  }
  
  /**
   * Monitor authentication process with detailed logging
   */
  async monitorAuthenticationProcess(walletType = 'ethereum') {
    console.log(`🔍 Monitoring ${walletType} authentication process...`);
    
    const startTime = Date.now();
    const initialState = await this.getAuthenticationState();
    
    // Perform authentication
    const authResult = walletType === 'ethereum' 
      ? await this.authenticateEthereum()
      : await this.attemptNearAuthentication();
    
    const endTime = Date.now();
    const duration = endTime - startTime;
    
    const finalState = await this.getAuthenticationState();
    
    // Log detailed results
    console.log('📊 Authentication Process Analysis:');
    console.log(`   Duration: ${duration}ms`);
    console.log(`   Success: ${authResult.success ? '✅' : '❌'}`);
    console.log(`   API calls before: ${initialState.totalAuthCalls}`);
    console.log(`   API calls after: ${finalState.totalAuthCalls}`);
    console.log(`   New API calls: ${finalState.totalAuthCalls - initialState.totalAuthCalls}`);
    console.log(`   Nonce calls: ${finalState.hasNonceCalls ? '✅' : '❌'} (${this.monitoring.getNonceCalls().length})`);
    console.log(`   Verify calls: ${finalState.hasVerifyCalls ? '✅' : '❌'} (${this.monitoring.getVerifyCalls().length})`);
    
    return {
      ...authResult,
      duration,
      initialState,
      finalState,
      apiCallsIncrease: finalState.totalAuthCalls - initialState.totalAuthCalls
    };
  }
  
  /**
   * Test authentication error handling
   */
  async testAuthenticationErrorHandling() {
    console.log('🧪 Testing authentication error handling...');
    
    const results = {
      consoleErrors: [],
      networkErrors: [],
      uiErrors: []
    };
    
    // Monitor errors during authentication
    const errorListener = (msg) => {
      if (msg.type() === 'error') {
        results.consoleErrors.push(msg.text());
      }
    };
    
    const networkErrorListener = (request) => {
      results.networkErrors.push({
        url: request.url(),
        failure: request.failure()
      });
    };
    
    this.page.on('console', errorListener);
    this.page.on('requestfailed', networkErrorListener);
    
    try {
      // Attempt authentication
      const authResult = await this.authenticateEthereum();
      
      // Check for UI error messages
      const errorElements = this.page.locator('.error, [role="alert"], .notification--error');
      const errorCount = await errorElements.count();
      
      for (let i = 0; i < Math.min(errorCount, 3); i++) {
        const errorText = await errorElements.nth(i).textContent();
        if (errorText && errorText.trim()) {
          results.uiErrors.push(errorText.trim());
        }
      }
      
      results.authenticationResult = authResult;
      
    } finally {
      this.page.off('console', errorListener);
      this.page.off('requestfailed', networkErrorListener);
    }
    
    console.log('📊 Error Handling Test Results:');
    console.log(`   Console errors: ${results.consoleErrors.length}`);
    console.log(`   Network errors: ${results.networkErrors.length}`);
    console.log(`   UI errors: ${results.uiErrors.length}`);
    
    if (results.consoleErrors.length > 0) {
      console.log('   Console errors:');
      results.consoleErrors.slice(0, 3).forEach((error, i) => {
        console.log(`     ${i + 1}. ${error}`);
      });
    }
    
    return results;
  }
}