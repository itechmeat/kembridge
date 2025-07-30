/**
 * Authentication Page Object Model - Enhanced with SOLID principles
 * Single Responsibility: Handles only authentication interactions
 * Open/Closed: Extensible for new authentication methods
 * Liskov Substitution: Can be substituted with specialized auth pages
 * Interface Segregation: Focused interface for auth operations
 * Dependency Inversion: Depends on abstractions (Page interface)
 */

import { Page, expect, Locator } from '@playwright/test';
import { MODERN_SELECTORS } from '../utils/modern-selectors';
import { 
  BaseTestUtility, 
  TEST_CONFIG, 
  AuthResult, 
  AuthStatus,
  createTestUtilities 
} from '../utils/test-utilities';

export class AuthPage extends BaseTestUtility {
  private utilities: ReturnType<typeof createTestUtilities>;

  constructor(page: Page) {
    super(page);
    this.utilities = createTestUtilities(page);
  }

  // Locators using modern selectors
  get ethereumWalletButton(): Locator {
    return this.page.locator(MODERN_SELECTORS.AUTH.ethereumWalletButton);
  }

  get nearWalletButton(): Locator {
    return this.page.locator(MODERN_SELECTORS.AUTH.nearWalletButton);
  }

  get userProfile(): Locator {
    return this.page.locator(MODERN_SELECTORS.AUTH.userProfile);
  }

  get authError(): Locator {
    return this.page.locator(MODERN_SELECTORS.AUTH.authError);
  }

  get loadingSpinner(): Locator {
    return this.page.locator(MODERN_SELECTORS.AUTH.loadingSpinner);
  }

  get disconnectButton(): Locator {
    return this.page.locator(MODERN_SELECTORS.AUTH.disconnectButton);
  }

  /**
   * Navigate to authentication page
   */
  async goto(): Promise<void> {
    console.log('🔐 Navigating to authentication page...');
    await this.page.goto('/auth');
    await this.waitForPageLoad();
  }

  /**
   * Wait for authentication page to load
   */
  async waitForPageLoad(): Promise<void> {
    console.log('⏳ Waiting for auth page to load...');
    
    await this.utilities.navigation.waitForPageReady();
    
    // Wait for at least one wallet button to be visible
    await Promise.race([
      expect(this.ethereumWalletButton).toBeVisible({ timeout: TEST_CONFIG.TIMEOUTS.LONG }),
      expect(this.nearWalletButton).toBeVisible({ timeout: TEST_CONFIG.TIMEOUTS.LONG }),
    ]);
    
    console.log('✅ Auth page loaded successfully');
  }

  /**
   * Check if user is already authenticated
   */
  async isAuthenticated(): Promise<boolean> {
    const status = await this.utilities.auth.getAuthStatus();
    return status.isAuthenticated;
  }

  /**
   * Get current authentication status
   */
  async getAuthStatus(): Promise<AuthStatus> {
    return await this.utilities.auth.getAuthStatus();
  }

  /**
   * Connect Ethereum wallet
   */
  async connectEthereumWallet(): Promise<AuthResult> {
    console.log('🦊 Connecting Ethereum wallet...');
    
    // Check if already authenticated
    if (await this.isAuthenticated()) {
      return { success: true, reason: 'Already authenticated' };
    }

    // Verify button is available and enabled
    await expect(this.ethereumWalletButton).toBeVisible();
    await expect(this.ethereumWalletButton).toBeEnabled();

    const result = await this.utilities.auth.authenticateEthereum();
    
    if (result.success) {
      await this.verifyAuthenticationSuccess();
    }

    return result;
  }

  /**
   * Connect NEAR wallet
   */
  async connectNearWallet(): Promise<AuthResult> {
    console.log('🌌 Connecting NEAR wallet...');
    
    // Check if already authenticated
    if (await this.isAuthenticated()) {
      return { success: true, reason: 'Already authenticated' };
    }

    // Verify button is available and enabled
    await expect(this.nearWalletButton).toBeVisible();
    await expect(this.nearWalletButton).toBeEnabled();

    const result = await this.utilities.auth.authenticateNear();
    
    if (result.success) {
      await this.verifyAuthenticationSuccess();
    }

    return result;
  }

  /**
   * Verify authentication was successful
   */
  private async verifyAuthenticationSuccess(): Promise<void> {
    console.log('✅ Verifying authentication success...');
    
    // Wait for user profile to appear
    await expect(this.userProfile).toBeVisible({ timeout: TEST_CONFIG.TIMEOUTS.LONG });
    
    // Ensure no error messages are present
    const hasError = await this.authError.isVisible();
    if (hasError) {
      const errorMessage = await this.authError.textContent();
      throw new Error(`Authentication error detected: ${errorMessage}`);
    }

    console.log('✅ Authentication verified successfully');
  }

  /**
   * Disconnect wallet
   */
  async disconnect(): Promise<AuthResult> {
    console.log('🔌 Disconnecting wallet...');
    
    // Check if authenticated first
    if (!await this.isAuthenticated()) {
      return { success: true, reason: 'Already disconnected' };
    }

    // Look for disconnect button
    const disconnectVisible = await this.disconnectButton.isVisible();
    if (!disconnectVisible) {
      return { success: false, reason: 'Disconnect button not found' };
    }

    await this.disconnectButton.click();
    await this.page.waitForTimeout(TEST_CONFIG.TIMEOUTS.MEDIUM);

    // Verify disconnection
    const isStillAuthenticated = await this.isAuthenticated();
    if (isStillAuthenticated) {
      return { success: false, reason: 'Disconnection failed - still authenticated' };
    }

    console.log('✅ Wallet disconnected successfully');
    return { success: true, reason: 'Disconnected successfully' };
  }

  /**
   * Wait for authentication to complete (loading state)
   */
  async waitForAuthCompletion(timeout: number = TEST_CONFIG.TIMEOUTS.EXTRA_LONG): Promise<void> {
    console.log('⏳ Waiting for authentication to complete...');
    
    // Wait for loading spinner to disappear
    if (await this.loadingSpinner.isVisible()) {
      await expect(this.loadingSpinner).toBeHidden({ timeout });
    }

    // Wait a bit more for state to stabilize
    await this.page.waitForTimeout(TEST_CONFIG.TIMEOUTS.SHORT);
    
    console.log('✅ Authentication process completed');
  }

  /**
   * Handle authentication errors
   */
  async handleAuthError(): Promise<string> {
    const hasError = await this.authError.isVisible();
    if (!hasError) {
      return '';
    }

    const errorMessage = await this.authError.textContent() || 'Unknown authentication error';
    console.warn(`⚠️ Authentication error: ${errorMessage}`);
    
    return errorMessage;
  }

  /**
   * Retry authentication with different wallet
   */
  async retryAuthentication(walletType: 'ethereum' | 'near'): Promise<AuthResult> {
    console.log(`🔄 Retrying authentication with ${walletType} wallet...`);
    
    // Clear any existing errors
    await this.handleAuthError();
    
    // Wait a moment before retry
    await this.page.waitForTimeout(TEST_CONFIG.TIMEOUTS.SHORT);
    
    if (walletType === 'ethereum') {
      return await this.connectEthereumWallet();
    } else {
      return await this.connectNearWallet();
    }
  }

  /**
   * Complete authentication flow with fallback
   */
  async authenticateWithFallback(preferredWallet: 'ethereum' | 'near' = 'ethereum'): Promise<AuthFlowResult> {
    console.log('🔐 Starting authentication flow with fallback...');
    
    const startTime = Date.now();
    const result: AuthFlowResult = {
      success: false,
      walletUsed: null,
      attempts: [],
      duration: 0,
      error: null,
    };

    try {
      // Check if already authenticated
      if (await this.isAuthenticated()) {
        result.success = true;
        result.walletUsed = 'existing';
        result.attempts.push({ wallet: 'existing', success: true, error: null });
        console.log('✅ Already authenticated');
        return result;
      }

      // Try preferred wallet first
      console.log(`🎯 Trying preferred wallet: ${preferredWallet}`);
      const primaryResult = preferredWallet === 'ethereum' 
        ? await this.connectEthereumWallet()
        : await this.connectNearWallet();
      
      result.attempts.push({ 
        wallet: preferredWallet, 
        success: primaryResult.success, 
        error: primaryResult.success ? null : primaryResult.reason 
      });

      if (primaryResult.success) {
        result.success = true;
        result.walletUsed = preferredWallet;
        console.log(`✅ Authentication successful with ${preferredWallet}`);
        return result;
      }

      // Try fallback wallet
      const fallbackWallet = preferredWallet === 'ethereum' ? 'near' : 'ethereum';
      console.log(`🔄 Trying fallback wallet: ${fallbackWallet}`);
      
      const fallbackResult = fallbackWallet === 'ethereum'
        ? await this.connectEthereumWallet()
        : await this.connectNearWallet();
      
      result.attempts.push({ 
        wallet: fallbackWallet, 
        success: fallbackResult.success, 
        error: fallbackResult.success ? null : fallbackResult.reason 
      });

      if (fallbackResult.success) {
        result.success = true;
        result.walletUsed = fallbackWallet;
        console.log(`✅ Authentication successful with fallback ${fallbackWallet}`);
      } else {
        result.error = `Both wallets failed. ${preferredWallet}: ${primaryResult.reason}, ${fallbackWallet}: ${fallbackResult.reason}`;
        console.error('❌ All authentication methods failed');
      }

    } catch (error) {
      result.error = error instanceof Error ? error.message : String(error);
      console.error('❌ Authentication flow error:', result.error);
    } finally {
      result.duration = Date.now() - startTime;
    }

    return result;
  }

  /**
   * Get available wallet options
   */
  async getAvailableWallets(): Promise<WalletAvailability> {
    const ethereumAvailable = await this.ethereumWalletButton.isVisible();
    const nearAvailable = await this.nearWalletButton.isVisible();
    
    const ethereumEnabled = ethereumAvailable ? await this.ethereumWalletButton.isEnabled() : false;
    const nearEnabled = nearAvailable ? await this.nearWalletButton.isEnabled() : false;

    return {
      ethereum: { available: ethereumAvailable, enabled: ethereumEnabled },
      near: { available: nearAvailable, enabled: nearEnabled },
    };
  }

  /**
   * Take screenshot for debugging
   */
  async takeScreenshot(name: string): Promise<void> {
    await this.page.screenshot({ 
      path: `test-results/screenshots/auth-${name}-${Date.now()}.png`,
      fullPage: true 
    });
  }
}

/**
 * Type definitions
 */
export interface AuthFlowResult {
  success: boolean;
  walletUsed: 'ethereum' | 'near' | 'existing' | null;
  attempts: AuthAttempt[];
  duration: number;
  error: string | null;
}

export interface AuthAttempt {
  wallet: 'ethereum' | 'near' | 'existing';
  success: boolean;
  error: string | null;
}

export interface WalletAvailability {
  ethereum: { available: boolean; enabled: boolean };
  near: { available: boolean; enabled: boolean };
}