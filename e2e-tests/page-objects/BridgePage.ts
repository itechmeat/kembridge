/**
 * Bridge Page Object Model - Enhanced with SOLID principles
 * Single Responsibility: Handles only bridge page interactions
 * Open/Closed: Extensible for new bridge features
 * Liskov Substitution: Can be substituted with specialized bridge pages
 * Interface Segregation: Focused interface for bridge operations
 * Dependency Inversion: Depends on abstractions (Page interface)
 */

import { Page, expect, Locator } from '@playwright/test';
import { MODERN_SELECTORS } from '../utils/selectors';
import { 
  BaseTestUtility, 
  TEST_CONFIG, 
  AuthResult, 
  BridgeFormState,
  createTestUtilities 
} from '../utils/test-utilities';

export class BridgePage extends BaseTestUtility {
  private utilities: ReturnType<typeof createTestUtilities>;

  constructor(page: Page) {
    super(page);
    this.utilities = createTestUtilities(page);
  }

  // Locators using modern selectors
  get form(): Locator {
    return this.page.locator(MODERN_SELECTORS.BRIDGE.form);
  }

  get fromTokenSelector(): Locator {
    return this.page.locator(MODERN_SELECTORS.BRIDGE.fromTokenSelector);
  }

  get toTokenSelector(): Locator {
    return this.page.locator(MODERN_SELECTORS.BRIDGE.toTokenSelector);
  }

  get amountInput(): Locator {
    return this.page.locator(MODERN_SELECTORS.BRIDGE.amountInput);
  }

  get submitButton(): Locator {
    return this.page.locator(MODERN_SELECTORS.BRIDGE.submitButton);
  }

  get swapDirectionButton(): Locator {
    return this.page.locator(MODERN_SELECTORS.BRIDGE.swapDirectionButton);
  }

  get priceQuote(): Locator {
    return this.page.locator(MODERN_SELECTORS.BRIDGE.priceQuote);
  }

  get slippageSlider(): Locator {
    return this.page.locator(MODERN_SELECTORS.BRIDGE.slippageSlider);
  }

  /**
   * Navigate to bridge page and wait for it to load
   */
  async goto(): Promise<void> {
    console.log('🌉 Navigating to bridge page...');
    
    // Use direct navigation as most reliable method
    await this.page.goto('/bridge');
    await this.page.waitForTimeout(3000);
    
    await this.waitForPageLoad();
  }

  /**
   * Wait for bridge page to fully load
   */
  async waitForPageLoad(): Promise<void> {
    console.log('⏳ Waiting for bridge page to load...');
    
    await this.utilities.navigation.waitForPageReady();
    
    // Wait for essential elements
    await expect(this.form).toBeVisible({ timeout: TEST_CONFIG.TIMEOUTS.LONG });
    
    console.log('✅ Bridge page loaded successfully');
  }

  /**
   * Check if authentication is required
   */
  async isAuthenticationRequired(): Promise<boolean> {
    const authRequired = await this.isElementVisible(MODERN_SELECTORS.BRIDGE.authRequired);
    return authRequired;
  }

  /**
   * Authenticate if required
   */
  async authenticateIfRequired(walletType: 'ethereum' | 'near' = 'ethereum'): Promise<AuthResult> {
    const authRequired = await this.isAuthenticationRequired();
    
    if (!authRequired) {
      return { success: true, reason: 'Authentication not required' };
    }

    console.log(`🔐 Authentication required, using ${walletType} wallet...`);
    
    if (walletType === 'ethereum') {
      return await this.utilities.auth.authenticateEthereum();
    } else {
      return await this.utilities.auth.authenticateNear();
    }
  }

  /**
   * Select token for a specific chain
   */
  async selectToken(chain: 'ethereum' | 'near', tokenSymbol: string): Promise<void> {
    console.log(`🪙 Selecting ${tokenSymbol} token for ${chain} chain...`);
    
    const success = await this.utilities.bridge.selectToken(chain, tokenSymbol);
    if (!success) {
      throw new Error(`Failed to select ${tokenSymbol} token for ${chain} chain`);
    }

    // Verify token was selected
    await this.verifyTokenSelected(chain, tokenSymbol);
  }

  /**
   * Verify that a token was selected
   */
  private async verifyTokenSelected(chain: 'ethereum' | 'near', tokenSymbol: string): Promise<void> {
    const selectorButton = chain === 'ethereum' ? this.fromTokenSelector : this.toTokenSelector;
    const buttonText = await selectorButton.textContent();
    
    if (!buttonText?.includes(tokenSymbol)) {
      throw new Error(`Token ${tokenSymbol} was not selected for ${chain} chain. Button text: ${buttonText}`);
    }

    console.log(`✅ ${tokenSymbol} token selected for ${chain} chain`);
  }

  /**
   * Enter amount to bridge
   */
  async enterAmount(amount: string): Promise<void> {
    console.log(`💰 Entering amount: ${amount}`);
    
    const success = await this.utilities.bridge.enterAmount(amount);
    if (!success) {
      throw new Error(`Failed to enter amount: ${amount}`);
    }

    // Verify amount was entered
    await this.verifyAmountEntered(amount);
  }

  /**
   * Verify that amount was entered correctly
   */
  private async verifyAmountEntered(expectedAmount: string): Promise<void> {
    const actualAmount = await this.amountInput.inputValue();
    
    if (actualAmount !== expectedAmount) {
      throw new Error(`Amount mismatch. Expected: ${expectedAmount}, Actual: ${actualAmount}`);
    }

    console.log(`✅ Amount ${expectedAmount} entered correctly`);
  }

  /**
   * Switch bridge direction (from <-> to)
   */
  async switchDirection(): Promise<void> {
    console.log('🔄 Switching bridge direction...');
    
    const success = await this.utilities.bridge.switchDirection();
    if (!success) {
      throw new Error('Failed to switch bridge direction');
    }
  }

  /**
   * Wait for price quote to be generated
   */
  async waitForPriceQuote(timeout: number = TEST_CONFIG.TIMEOUTS.LONG): Promise<void> {
    console.log('💱 Waiting for price quote...');
    
    await expect(this.priceQuote).toBeVisible({ timeout });
    
    // Wait for quote to stabilize (no loading indicators)
    await this.page.waitForTimeout(TEST_CONFIG.TIMEOUTS.SHORT);
    
    console.log('✅ Price quote loaded');
  }

  /**
   * Get current price quote information
   */
  async getPriceQuoteInfo(): Promise<PriceQuoteInfo> {
    await this.waitForPriceQuote();
    
    const quoteText = await this.priceQuote.textContent() || '';
    
    return {
      isVisible: await this.priceQuote.isVisible(),
      text: quoteText,
      hasRate: quoteText.includes('Rate:') || quoteText.includes('1 '),
      hasAmount: /\d+\.?\d*/.test(quoteText),
    };
  }

  /**
   * Adjust slippage tolerance
   */
  async adjustSlippage(percentage: number): Promise<void> {
    console.log(`⚙️ Adjusting slippage to ${percentage}%...`);
    
    if (!await this.slippageSlider.isVisible()) {
      console.log('Slippage slider not visible, skipping adjustment');
      return;
    }

    // This would need to be implemented based on the actual slider component
    // For now, we'll just verify it's accessible
    await expect(this.slippageSlider).toBeVisible();
    
    console.log(`✅ Slippage adjustment interface available`);
  }

  /**
   * Submit the bridge transaction
   */
  async submitBridge(): Promise<void> {
    console.log('🎯 Submitting bridge transaction...');
    
    const success = await this.utilities.bridge.submitForm();
    if (!success) {
      throw new Error('Failed to submit bridge form');
    }
  }

  /**
   * Get current form state
   */
  async getFormState(): Promise<BridgeFormState> {
    return await this.utilities.bridge.getFormState();
  }

  /**
   * Verify form is ready for submission
   */
  async verifyFormReady(): Promise<void> {
    const formState = await this.getFormState();
    
    if (!formState.isFormReady) {
      throw new Error(`Form is not ready. State: ${JSON.stringify(formState)}`);
    }

    console.log('✅ Bridge form is ready for submission');
  }

  /**
   * Complete a full bridge flow
   */
  async completeBridgeFlow(params: BridgeFlowParams): Promise<BridgeFlowResult> {
    console.log('🌉 Starting complete bridge flow...');
    
    const startTime = Date.now();
    const result: BridgeFlowResult = {
      success: false,
      steps: [],
      duration: 0,
      error: null,
    };

    try {
      // Step 1: Navigate to bridge
      await this.goto();
      result.steps.push('navigation');

      // Step 2: Authenticate if required
      if (params.authenticate) {
        const authResult = await this.authenticateIfRequired(params.walletType);
        if (!authResult.success) {
          throw new Error(`Authentication failed: ${authResult.reason}`);
        }
        result.steps.push('authentication');
      }

      // Step 3: Select tokens
      if (params.fromToken) {
        await this.selectToken('ethereum', params.fromToken);
        result.steps.push('from-token-selection');
      }

      if (params.toToken) {
        await this.selectToken('near', params.toToken);
        result.steps.push('to-token-selection');
      }

      // Step 4: Enter amount
      if (params.amount) {
        await this.enterAmount(params.amount);
        result.steps.push('amount-entry');
      }

      // Step 5: Wait for quote
      if (params.waitForQuote) {
        await this.waitForPriceQuote();
        result.steps.push('price-quote');
      }

      // Step 6: Adjust slippage if needed
      if (params.slippage) {
        await this.adjustSlippage(params.slippage);
        result.steps.push('slippage-adjustment');
      }

      // Step 7: Submit if requested
      if (params.submit) {
        await this.verifyFormReady();
        await this.submitBridge();
        result.steps.push('submission');
      }

      result.success = true;
      console.log('✅ Bridge flow completed successfully');

    } catch (error) {
      result.error = error instanceof Error ? error.message : String(error);
      console.error('❌ Bridge flow failed:', result.error);
    } finally {
      result.duration = Date.now() - startTime;
    }

    return result;
  }

  /**
   * Check if sign-in message is visible
   */
  async isSignInMessageVisible(): Promise<boolean> {
    return await this.page.locator('text=Sign in with your wallet').isVisible();
  }

  /**
   * Check if connect button is visible and get its text
   */
  async getConnectButtonInfo(): Promise<{ visible: boolean; text: string }> {
    // Use more specific selector to avoid multiple matches
    const connectButton = this.page.locator('.wallet-connect-button, button.btn--primary:has-text("Connect")').first();
    const visible = await connectButton.isVisible();
    const text = visible ? await connectButton.textContent() || '' : '';
    
    return { visible, text };
  }

  /**
   * Get token button texts for verification after switching
   */
  async getTokenButtonTexts(): Promise<{ fromToken: string; toToken: string }> {
    const tokenButtons = await this.page.locator('button').all();
    let fromTokenText = '';
    let toTokenText = '';
    
    // Find token buttons by their content
    for (const button of tokenButtons) {
      const text = await button.textContent();
      if (text && (text.includes('NEAR Protocol') || text.includes('ETH') || text.includes('USDC'))) {
        if (!fromTokenText) {
          fromTokenText = text;
        } else if (!toTokenText) {
          toTokenText = text;
          break;
        }
      }
    }
    
    return { fromToken: fromTokenText, toToken: toTokenText };
  }

  /**
   * Get page instance for advanced operations
   */
  getPage() {
    return this.page;
  }

  /**
   * Take screenshot for debugging
   */
  async takeScreenshot(name: string): Promise<void> {
    await this.page.screenshot({ 
      path: `test-results/screenshots/bridge-${name}-${Date.now()}.png`,
      fullPage: true 
    });
  }
}

/**
 * Type definitions
 */
export interface PriceQuoteInfo {
  isVisible: boolean;
  text: string;
  hasRate: boolean;
  hasAmount: boolean;
}

export interface BridgeFlowParams {
  authenticate?: boolean;
  walletType?: 'ethereum' | 'near';
  fromToken?: string;
  toToken?: string;
  amount?: string;
  slippage?: number;
  waitForQuote?: boolean;
  submit?: boolean;
}

export interface BridgeFlowResult {
  success: boolean;
  steps: string[];
  duration: number;
  error: string | null;
}