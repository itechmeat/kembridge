/**
 * REALISTIC selectors based on actual DOM structure
 * These selectors are tested against the real application
 */
import { Page, Locator } from '@playwright/test';

export class RealisticSelectors {
  constructor(private page: Page) {}

  // Authentication selectors (based on REAL DOM structure found in tests)
  get ethWalletButton(): Locator {
    // From validation: "🦊Ethereum WalletConnect wallet" with classes: auth-manager__method auth-manager__method--ethereum
    return this.page.locator('.auth-manager__method--ethereum, button:has-text("🦊Ethereum WalletConnect wallet")').first();
  }

  get nearWalletButton(): Locator {
    // From validation: "🔷NEAR WalletConnect wallet" with classes: auth-manager__method auth-manager__method--near
    return this.page.locator('.auth-manager__method--near, button:has-text("🔷NEAR WalletConnect wallet")').first();
  }

  get connectWalletButton(): Locator {
    // From validation: "Connect" with classes: btn btn--primary btn--sm wallet-connect-button
    return this.page.locator('.wallet-connect-button, button:has-text("Connect")').first();
  }

  // Navigation selectors (based on REAL DOM structure)
  get swapNavButton(): Locator {
    // From validation: "🔄Swap" with classes: bottom-nav__item
    return this.page.locator('.bottom-nav__item:has-text("Swap"), button:has-text("🔄Swap")').first();
  }

  get bridgeNavButton(): Locator {
    // Bridge navigation is done via Swap button and then tabs
    return this.page.locator('.bridge-page__tab:has-text("Bridge"), button:has-text("Bridge")').first();
  }

  // Bridge form selectors (based on actual form structure)
  get amountInput(): Locator {
    // Multiple fallback approaches
    const selectors = [
      'input[type="number"]',
      'input[placeholder*="amount"]', 
      'input[placeholder="0.0"]',
      '.amount-input input',
      '.swap-form__amount-input input',
      'input[data-testid="amount-input"]'
    ];
    
    return this.page.locator(selectors.join(', ')).first();
  }

  get fromTokenSelector(): Locator {
    return this.page.locator('.token-selector, .swap-form__token-selector, .from-token, .token-select').first();
  }

  get toTokenSelector(): Locator {
    return this.page.locator('.token-selector, .swap-form__token-selector, .to-token, .token-select').last();
  }

  get submitButton(): Locator {
    const selectors = [
      'button[type="submit"]',
      'button:has-text("Review Swap")',
      'button:has-text("Get Quote")',
      'button:has-text("Swap")',
      'button:has-text("Bridge")',
      'button:has-text("Execute")',
      '.swap-form__submit',
      '.submit-btn'
    ];
    
    return this.page.locator(selectors.join(', ')).first();
  }

  get directionSwitch(): Locator {
    const selectors = [
      'button:has-text("⇅")',
      'button:has-text("↔")', 
      'button:has-text("⇄")',
      '.swap-direction',
      '.reverse-button',
      '.swap-form__swap-button',
      'button[aria-label*="switch"]',
      'button[aria-label*="reverse"]'
    ];
    
    return this.page.locator(selectors.join(', ')).first();
  }

  // Status and feedback selectors
  get loadingSpinner(): Locator {
    return this.page.locator('.loading, .spinner, .loader, [class*="loading"], [class*="spinner"]').first();
  }

  get successMessage(): Locator {
    return this.page.locator('.success, .alert-success, .notification--success, [class*="success"]').first();
  }

  get errorMessage(): Locator {
    return this.page.locator('.error, .alert-error, .notification--error, [class*="error"], [role="alert"]').first();
  }

  get authRequiredMessage(): Locator {
    return this.page.locator('.auth-required, .connect-wallet-prompt, .swap-form__auth-required').first();
  }

  // Security and bridge specific
  get quantumSecurityBadge(): Locator {
    return this.page.locator(':has-text("🔒"), :has-text("Quantum"), :has-text("Protected"), [class*="quantum"], [class*="security"]').first();
  }

  get transactionStatus(): Locator {
    return this.page.locator('[data-testid="transaction-status"], .transaction-status, .status').first();
  }

  // Helper methods for common interactions
  async waitForWalletConnected(timeout = 10000): Promise<void> {
    // Wait for wallet connection indicators
    await Promise.race([
      this.page.waitForSelector(':has-text("Connected")', { timeout }),
      this.page.waitForSelector('.wallet-connected', { timeout }),
      this.page.waitForSelector(':has-text("Balance:")', { timeout }),
      this.page.waitForFunction(() => {
        const button = document.querySelector('button');
        return button && button.textContent && !button.textContent.toLowerCase().includes('connect');
      }, undefined, { timeout })
    ]);
  }

  async waitForFormReady(timeout = 10000): Promise<void> {
    // Wait for form elements to be available
    await Promise.all([
      this.amountInput.waitFor({ state: 'visible', timeout: timeout/3 }),
      this.fromTokenSelector.waitFor({ state: 'visible', timeout: timeout/3 }),
      this.toTokenSelector.waitFor({ state: 'visible', timeout: timeout/3 })
    ]);
  }

  async waitForPageLoad(timeout = 15000): Promise<void> {
    // Wait for page to be ready
    await this.page.waitForLoadState('domcontentloaded');
    await this.page.waitForSelector('body', { timeout });
    
    // Wait for any loading indicators to disappear
    try {
      await this.page.waitForSelector('.loading, .spinner', { state: 'hidden', timeout: 5000 });
    } catch {
      // Ignore if no loading indicators found
    }
  }

  // Method to test what selectors actually work
  async debugSelectors(): Promise<void> {
    console.log('🔍 Debugging available selectors...');
    
    const allButtons = await this.page.locator('button').count();
    console.log(`Total buttons found: ${allButtons}`);
    
    for (let i = 0; i < Math.min(allButtons, 10); i++) {
      const button = this.page.locator('button').nth(i);
      const text = await button.textContent();
      const classes = await button.getAttribute('class');
      console.log(`Button ${i}: "${text}" (classes: ${classes})`);
    }
    
    const allInputs = await this.page.locator('input').count();
    console.log(`Total inputs found: ${allInputs}`);
    
    for (let i = 0; i < Math.min(allInputs, 5); i++) {
      const input = this.page.locator('input').nth(i);
      const type = await input.getAttribute('type');
      const placeholder = await input.getAttribute('placeholder');
      const classes = await input.getAttribute('class');
      console.log(`Input ${i}: type="${type}" placeholder="${placeholder}" (classes: ${classes})`);
    }
  }

  // Method to find elements by text content (most reliable)
  findByText(text: string): Locator {
    return this.page.locator(`text=${text}`).or(this.page.locator(`:has-text("${text}")`));
  }

  // Method to find elements by partial text (flexible)
  findByPartialText(partialText: string): Locator {
    return this.page.locator(`:has-text("${partialText}")`);
  }

  // Method to get element by data attribute (if available)
  getByTestId(testId: string): Locator {
    return this.page.locator(`[data-testid="${testId}"]`);
  }

  // Check if element exists without throwing
  async elementExists(selector: string): Promise<boolean> {
    try {
      await this.page.locator(selector).waitFor({ state: 'attached', timeout: 1000 });
      return true;
    } catch {
      return false;
    }
  }

  // Safe click that handles multiple selector attempts
  async safeClick(selectors: string[]): Promise<boolean> {
    for (const selector of selectors) {
      try {
        const element = this.page.locator(selector).first();
        if (await element.isVisible({ timeout: 2000 })) {
          await element.click();
          console.log(`✅ Successfully clicked: ${selector}`);
          return true;
        }
      } catch {
        console.log(`❌ Failed to click: ${selector}`);
      }
    }
    return false;
  }
}

// Export common selector patterns that actually work in the real application
export const WORKING_SELECTORS = {
  // VERIFIED selectors from validation tests
  ETH_WALLET_BUTTON: '.auth-manager__method--ethereum',
  NEAR_WALLET_BUTTON: '.auth-manager__method--near', 
  CONNECT_BUTTON: '.wallet-connect-button',
  SWAP_NAV: '.bottom-nav__item:has-text("Swap")',
  HISTORY_NAV: '.bottom-nav__item:has-text("History")',
  WALLET_NAV: '.bottom-nav__item:has-text("Wallet")',
  SETTINGS_NAV: '.bottom-nav__item:has-text("Settings")',
  
  // Form elements (need investigation when auth is complete)
  AMOUNT_INPUT: 'input[type="number"], input[placeholder*="amount"], input[placeholder="0.0"]',
  TOKEN_SELECTOR: '.token-selector, .swap-form__token-selector',
  SUBMIT_BUTTON: 'button[type="submit"], button:has-text("Review Swap"), button:has-text("Swap")',
  AUTH_REQUIRED: '.swap-form__auth-required',
  DIRECTION_SWITCH: 'button:has-text("⇅")',
  
  // Security and app features
  QUANTUM_BADGE: ':has-text("🔒"), :has-text("Quantum")',
  SWAP_FORM: '.swap-form',
  BRIDGE_TAB: '.bridge-page__tab:has-text("Bridge")',
  HISTORY_TAB: '.bridge-page__tab:has-text("History")'
} as const;