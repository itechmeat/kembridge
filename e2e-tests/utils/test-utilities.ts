/**
 * Enhanced Test Utilities - DRY and SOLID Principles
 * Reusable components for E2E testing
 */

import { Page, expect, Locator } from '@playwright/test';
import { MODERN_SELECTORS, selectorHelpers } from './modern-selectors';
import { TEST_ENV, TEST_DATA, TEST_UTILS } from './test-constants';

/**
 * Re-export commonly used constants for backward compatibility
 */
export const TEST_CONFIG = {
  TIMEOUTS: TEST_ENV.TIMEOUTS,
  RETRY: TEST_ENV.RETRY,
  AMOUNTS: TEST_DATA.AMOUNTS,
} as const;

/**
 * Base class for test utilities following Single Responsibility Principle
 */
export abstract class BaseTestUtility {
  constructor(protected page: Page) {}

  /**
   * Wait for element with retry logic
   */
  protected async waitForElementWithRetry(
    selector: string,
    timeout: number = TEST_CONFIG.TIMEOUTS.MEDIUM,
    maxAttempts: number = TEST_CONFIG.RETRY.MAX_ATTEMPTS
  ): Promise<Locator | null> {
    for (let attempt = 1; attempt <= maxAttempts; attempt++) {
      try {
        const element = this.page.locator(selector);
        await element.waitFor({ timeout: timeout / maxAttempts });
        return element;
      } catch (error) {
        if (attempt === maxAttempts) {
          console.warn(`Element not found after ${maxAttempts} attempts: ${selector}`);
          return null;
        }
        await this.page.waitForTimeout(TEST_CONFIG.RETRY.DELAY);
      }
    }
    return null;
  }

  /**
   * Safe click with visibility check
   */
  protected async safeClick(selector: string): Promise<boolean> {
    try {
      const element = await this.waitForElementWithRetry(selector);
      if (!element) return false;

      await element.waitFor({ state: 'visible' });
      await element.click();
      return true;
    } catch (error) {
      console.warn(`Safe click failed for ${selector}:`, error);
      return false;
    }
  }

  /**
   * Safe fill input with validation
   */
  protected async safeFill(selector: string, value: string): Promise<boolean> {
    try {
      const element = await this.waitForElementWithRetry(selector);
      if (!element) return false;

      await element.waitFor({ state: 'visible' });
      await element.fill(value);
      
      // Verify the value was set
      const actualValue = await element.inputValue();
      return actualValue === value;
    } catch (error) {
      console.warn(`Safe fill failed for ${selector}:`, error);
      return false;
    }
  }

  /**
   * Get element text safely
   */
  protected async safeGetText(selector: string): Promise<string> {
    try {
      const element = await this.waitForElementWithRetry(selector);
      if (!element) return '';
      
      return await element.textContent() || '';
    } catch (error) {
      console.warn(`Safe get text failed for ${selector}:`, error);
      return '';
    }
  }

  /**
   * Check if element is visible
   */
  protected async isElementVisible(selector: string): Promise<boolean> {
    try {
      const element = this.page.locator(selector);
      return await element.isVisible();
    } catch (error) {
      return false;
    }
  }
}

/**
 * Authentication utilities
 */
export class AuthenticationUtility extends BaseTestUtility {
  /**
   * Authenticate with Ethereum wallet
   */
  async authenticateEthereum(): Promise<AuthResult> {
    console.log('🔐 Authenticating with Ethereum wallet...');
    
    const button = await this.waitForElementWithRetry(MODERN_SELECTORS.AUTH.ethereumWalletButton);
    if (!button) {
      return { success: false, reason: 'Ethereum wallet button not found' };
    }

    const isEnabled = await button.isEnabled();
    if (!isEnabled) {
      return { success: false, reason: 'Ethereum wallet button is disabled' };
    }

    await button.click();
    await this.page.waitForTimeout(TEST_CONFIG.TIMEOUTS.LONG);

    // Check for authentication success indicators
    const isAuthenticated = await this.isElementVisible(MODERN_SELECTORS.AUTH.userProfile);
    
    return {
      success: isAuthenticated,
      reason: isAuthenticated ? 'Authentication successful' : 'Authentication failed',
    };
  }

  /**
   * Authenticate with NEAR wallet
   */
  async authenticateNear(): Promise<AuthResult> {
    console.log('🔐 Authenticating with NEAR wallet...');
    
    const button = await this.waitForElementWithRetry(MODERN_SELECTORS.AUTH.nearWalletButton);
    if (!button) {
      return { success: false, reason: 'NEAR wallet button not found' };
    }

    const isEnabled = await button.isEnabled();
    if (!isEnabled) {
      return { success: false, reason: 'NEAR wallet button is disabled' };
    }

    await button.click();
    await this.page.waitForTimeout(TEST_CONFIG.TIMEOUTS.LONG);

    const isAuthenticated = await this.isElementVisible(MODERN_SELECTORS.AUTH.userProfile);
    
    return {
      success: isAuthenticated,
      reason: isAuthenticated ? 'Authentication successful' : 'Authentication may require real wallet',
    };
  }

  /**
   * Get current authentication status
   */
  async getAuthStatus(): Promise<AuthStatus> {
    const isAuthenticated = await this.isElementVisible(MODERN_SELECTORS.AUTH.userProfile);
    const hasError = await this.isElementVisible(MODERN_SELECTORS.AUTH.authError);
    const isLoading = await this.isElementVisible(MODERN_SELECTORS.AUTH.loadingSpinner);

    return {
      isAuthenticated,
      hasError,
      isLoading,
      errorMessage: hasError ? await this.safeGetText(MODERN_SELECTORS.AUTH.authError) : '',
    };
  }
}

/**
 * Navigation utilities
 */
export class NavigationUtility extends BaseTestUtility {
  /**
   * Navigate to bridge page
   */
  async navigateToBridge(): Promise<boolean> {
    console.log('🌉 Navigating to bridge page...');
    
    // Try multiple navigation methods
    const navigationMethods = [
      () => this.safeClick(MODERN_SELECTORS.NAVIGATION.swapLink),
      () => this.safeClick(MODERN_SELECTORS.NAVIGATION.bottomNavSwap),
      () => this.safeClick(MODERN_SELECTORS.NAVIGATION.quickActionSwap),
      () => this.page.goto('/bridge'),
    ];

    for (const method of navigationMethods) {
      try {
        await method();
        await this.page.waitForTimeout(TEST_CONFIG.TIMEOUTS.MEDIUM);
        
        // Check if we're on the bridge page
        const isBridgePage = await this.isElementVisible(MODERN_SELECTORS.BRIDGE.form);
        if (isBridgePage) {
          console.log('✅ Successfully navigated to bridge page');
          return true;
        }
      } catch (error) {
        console.warn('Navigation method failed, trying next...', error);
      }
    }

    console.warn('❌ All navigation methods failed');
    return false;
  }

  /**
   * Wait for page to be ready
   */
  async waitForPageReady(): Promise<boolean> {
    try {
      await this.page.waitForLoadState('networkidle', { timeout: TEST_CONFIG.TIMEOUTS.EXTRA_LONG });
      await this.page.waitForTimeout(TEST_CONFIG.TIMEOUTS.SHORT);
      return true;
    } catch (error) {
      console.warn('Page ready timeout:', error);
      return false;
    }
  }
}

/**
 * Bridge form utilities
 */
export class BridgeFormUtility extends BaseTestUtility {
  /**
   * Check if bridge form is accessible (not requiring auth)
   */
  async isBridgeFormAccessible(): Promise<boolean> {
    const authRequired = await this.isElementVisible(MODERN_SELECTORS.BRIDGE.authRequired);
    return !authRequired;
  }

  /**
   * Select token by symbol
   */
  async selectToken(chain: 'ethereum' | 'near', tokenSymbol: string): Promise<boolean> {
    console.log(`🪙 Selecting ${tokenSymbol} token for ${chain}...`);
    
    const selectorButton = selectorHelpers.tokenSelectorByChain(chain);
    const success = await this.safeClick(selectorButton);
    if (!success) return false;

    // Wait for dropdown to open
    await this.page.waitForTimeout(TEST_CONFIG.TIMEOUTS.SHORT);

    // Try popular token first, then search in full list
    const popularTokenSelector = selectorHelpers.popularTokenBySymbol(tokenSymbol);
    const popularSuccess = await this.safeClick(popularTokenSelector);
    
    if (popularSuccess) {
      console.log(`✅ Selected ${tokenSymbol} from popular tokens`);
      return true;
    }

    // Search in full list
    const tokenOptionSelector = selectorHelpers.tokenBySymbol(tokenSymbol);
    const optionSuccess = await this.safeClick(tokenOptionSelector);
    
    if (optionSuccess) {
      console.log(`✅ Selected ${tokenSymbol} from token list`);
      return true;
    }

    console.warn(`❌ Failed to select ${tokenSymbol} token`);
    return false;
  }

  /**
   * Enter amount in the input field
   */
  async enterAmount(amount: string): Promise<boolean> {
    console.log(`💰 Entering amount: ${amount}`);
    
    const success = await this.safeFill(MODERN_SELECTORS.BRIDGE.amountInput, amount);
    if (success) {
      console.log(`✅ Amount ${amount} entered successfully`);
      // Wait for quote generation
      await this.page.waitForTimeout(TEST_CONFIG.TIMEOUTS.MEDIUM);
    } else {
      console.warn(`❌ Failed to enter amount: ${amount}`);
    }
    
    return success;
  }

  /**
   * Switch bridge direction
   */
  async switchDirection(): Promise<boolean> {
    console.log('🔄 Switching bridge direction...');
    
    const success = await this.safeClick(MODERN_SELECTORS.BRIDGE.swapDirectionButton);
    if (success) {
      console.log('✅ Bridge direction switched');
      await this.page.waitForTimeout(TEST_CONFIG.TIMEOUTS.SHORT);
    } else {
      console.warn('❌ Failed to switch bridge direction');
    }
    
    return success;
  }

  /**
   * Submit the bridge form
   */
  async submitForm(): Promise<boolean> {
    console.log('🎯 Submitting bridge form...');
    
    const button = await this.waitForElementWithRetry(MODERN_SELECTORS.BRIDGE.submitButton);
    if (!button) return false;

    const isEnabled = await button.isEnabled();
    if (!isEnabled) {
      const buttonText = await button.textContent();
      console.warn(`❌ Submit button is disabled. Text: "${buttonText}"`);
      return false;
    }

    await button.click();
    console.log('✅ Bridge form submitted');
    return true;
  }

  /**
   * Get form validation state
   */
  async getFormState(): Promise<BridgeFormState> {
    const isAccessible = await this.isBridgeFormAccessible();
    const hasAmountInput = await this.isElementVisible(MODERN_SELECTORS.BRIDGE.amountInput);
    const hasSubmitButton = await this.isElementVisible(MODERN_SELECTORS.BRIDGE.submitButton);
    const submitButtonText = await this.safeGetText(MODERN_SELECTORS.BRIDGE.submitButton);
    
    return {
      isAccessible,
      hasAmountInput,
      hasSubmitButton,
      submitButtonText,
      isFormReady: isAccessible && hasAmountInput && hasSubmitButton,
    };
  }
}

/**
 * API monitoring utility
 */
export class ApiMonitoringUtility {
  private apiCalls: ApiCall[] = [];

  constructor(private page: Page) {
    this.setupMonitoring();
  }

  private setupMonitoring(): void {
    this.page.on('request', (request) => {
      const url = request.url();
      if (url.includes('/api/v1/')) {
        this.apiCalls.push({
          url,
          method: request.method(),
          timestamp: Date.now(),
          type: this.categorizeApiCall(url),
        });
      }
    });
  }

  private categorizeApiCall(url: string): ApiCallType {
    if (url.includes('/auth/') || url.includes('/nonce') || url.includes('/verify')) {
      return 'auth';
    }
    if (url.includes('/bridge/') || url.includes('/quote') || url.includes('/swap')) {
      return 'bridge';
    }
    if (url.includes('/tokens')) {
      return 'tokens';
    }
    return 'other';
  }

  getApiCalls(type?: ApiCallType): ApiCall[] {
    return type ? this.apiCalls.filter(call => call.type === type) : this.apiCalls;
  }

  getCallCount(type?: ApiCallType): number {
    return this.getApiCalls(type).length;
  }

  logSummary(): void {
    console.log('📊 API Calls Summary:');
    console.log(`   Total calls: ${this.apiCalls.length}`);
    console.log(`   Auth calls: ${this.getCallCount('auth')}`);
    console.log(`   Bridge calls: ${this.getCallCount('bridge')}`);
    console.log(`   Token calls: ${this.getCallCount('tokens')}`);
    console.log(`   Other calls: ${this.getCallCount('other')}`);
  }

  reset(): void {
    this.apiCalls = [];
  }
}

/**
 * Type definitions
 */
export interface AuthResult {
  success: boolean;
  reason: string;
}

export interface AuthStatus {
  isAuthenticated: boolean;
  hasError: boolean;
  isLoading: boolean;
  errorMessage: string;
}

export interface BridgeFormState {
  isAccessible: boolean;
  hasAmountInput: boolean;
  hasSubmitButton: boolean;
  submitButtonText: string;
  isFormReady: boolean;
}

export interface ApiCall {
  url: string;
  method: string;
  timestamp: number;
  type: ApiCallType;
}

export type ApiCallType = 'auth' | 'bridge' | 'tokens' | 'other';

/**
 * Factory function to create test utilities
 */
export function createTestUtilities(page: Page) {
  return {
    auth: new AuthenticationUtility(page),
    navigation: new NavigationUtility(page),
    bridge: new BridgeFormUtility(page),
    api: new ApiMonitoringUtility(page),
  };
}