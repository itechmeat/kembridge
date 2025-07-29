/**
 * Improved selectors using Playwright best practices
 * Prioritizes accessibility and role-based selectors over CSS
 */
import { Page, Locator } from '@playwright/test';

export class TestSelectors {
  constructor(private page: Page) {}

  // Authentication selectors (role-based)
  get ethWalletButton(): Locator {
    return this.page.getByRole('button', { name: /ethereum.*wallet/i });
  }

  get nearWalletButton(): Locator {
    return this.page.getByRole('button', { name: /near.*wallet/i });
  }

  get connectWalletButton(): Locator {
    return this.page.getByRole('button', { name: /connect.*wallet/i });
  }

  get disconnectButton(): Locator {
    return this.page.getByRole('button', { name: /disconnect/i });
  }

  // Navigation selectors (role-based with aria-labels)
  get homeLink(): Locator {
    return this.page.getByRole('link', { name: /home/i });
  }

  get bridgeLink(): Locator {
    return this.page.getByRole('link', { name: /bridge|swap/i });
  }

  get dashboardLink(): Locator {
    return this.page.getByRole('link', { name: /dashboard/i });
  }

  get swapNavButton(): Locator {
    return this.page.getByRole('button', { name: /swap/i });
  }

  // Bridge form selectors (accessible)
  get amountInput(): Locator {
    return this.page.getByRole('textbox', { name: /amount/i })
      .or(this.page.getByPlaceholder(/amount|0\.0/i))
      .or(this.page.getByLabel(/amount/i));
  }

  get fromTokenSelector(): Locator {
    return this.page.getByRole('combobox', { name: /from.*token|source.*token/i })
      .or(this.page.getByLabel(/from.*token/i))
      .or(this.page.locator('[data-testid="from-token-selector"]'));
  }

  get toTokenSelector(): Locator {
    return this.page.getByRole('combobox', { name: /to.*token|destination.*token/i })
      .or(this.page.getByLabel(/to.*token/i))
      .or(this.page.locator('[data-testid="to-token-selector"]'));
  }

  get reviewSwapButton(): Locator {
    return this.page.getByRole('button', { name: /review.*swap|get.*quote|initiate.*bridge/i })
      .or(this.page.getByRole('button', { name: /swap/i }).and(this.page.locator('[type="submit"]')));
  }

  get confirmSwapButton(): Locator {
    return this.page.getByRole('button', { name: /confirm.*swap|execute.*swap/i });
  }

  get directionSwitch(): Locator {
    return this.page.getByRole('button', { name: /switch.*direction|reverse/i })
      .or(this.page.getByLabel(/switch.*direction/i))
      .or(this.page.locator('button:has-text("⇅"), button:has-text("↔"), button:has-text("⇄")'));
  }

  // Status and feedback selectors
  get successMessage(): Locator {
    return this.page.getByRole('alert').filter({ hasText: /success|complete|confirmed/i })
      .or(this.page.locator('[role="status"]').filter({ hasText: /success/i }));
  }

  get errorMessage(): Locator {
    return this.page.getByRole('alert').filter({ hasText: /error|failed|invalid/i })
      .or(this.page.locator('[role="alertdialog"]'))
      .or(this.page.locator('.error, .alert-error'));
  }

  get loadingSpinner(): Locator {
    return this.page.getByRole('status', { name: /loading/i })
      .or(this.page.locator('[aria-label*="loading"]'))
      .or(this.page.locator('.loading, .spinner'));
  }

  get transactionStatus(): Locator {
    return this.page.locator('[data-testid="transaction-status"]')
      .or(this.page.getByText(/pending|processing|completed|failed/i).first());
  }

  // Security indicators
  get quantumSecurityBadge(): Locator {
    return this.page.getByText(/quantum.*protected|ml-kem|post.*quantum/i)
      .or(this.page.locator('[data-testid*="quantum"]'));
  }

  get riskScore(): Locator {
    return this.page.locator('[data-testid="risk-score"]')
      .or(this.page.getByText(/risk.*score/i).locator('..'))
      .or(this.page.locator('.risk-score'));
  }

  get securityWarning(): Locator {
    return this.page.getByRole('alert').filter({ hasText: /security|warning|risk/i });
  }

  // Form validation
  get authRequiredMessage(): Locator {
    return this.page.getByText(/authentication.*required|connect.*wallet.*first/i)
      .or(this.page.locator('.auth-required, .connect-wallet-prompt'));
  }

  get insufficientBalanceError(): Locator {
    return this.page.getByText(/insufficient.*balance|not.*enough.*funds/i);
  }

  // Modal and dialog selectors
  get modal(): Locator {
    return this.page.getByRole('dialog')
      .or(this.page.locator('[role="modal"]'))
      .or(this.page.locator('.modal'));
  }

  get modalCloseButton(): Locator {
    return this.modal.getByRole('button', { name: /close|×/i });
  }

  // Utility methods for complex selections
  getTokenSelectorByIndex(index: number): Locator {
    return this.page.locator('.token-selector').nth(index);
  }

  getTransactionById(id: string): Locator {
    return this.page.locator(`[data-transaction-id="${id}"]`)
      .or(this.page.getByText(id).locator('..'));
  }

  getChainOption(chain: 'ethereum' | 'near'): Locator {
    return this.page.getByRole('option', { name: new RegExp(chain, 'i') })
      .or(this.page.locator(`[data-chain="${chain}"]`));
  }

  // Wait helpers with role-based selectors
  async waitForWalletConnected(timeout = 10000): Promise<void> {
    await this.page.waitForFunction(
      () => {
        const button = document.querySelector('button[aria-label*="wallet"], button:has-text("Connected")');
        return button && !button.textContent?.toLowerCase().includes('connect');
      },
      undefined,
      { timeout }
    );
  }

  async waitForFormReady(timeout = 10000): Promise<void> {
    await Promise.all([
      this.amountInput.waitFor({ state: 'visible', timeout }),
      this.fromTokenSelector.waitFor({ state: 'visible', timeout }),
      this.toTokenSelector.waitFor({ state: 'visible', timeout })
    ]);
  }

  async waitForTransactionComplete(transactionId: string, timeout = 60000): Promise<void> {
    await this.page.waitForFunction(
      (txId) => {
        const statusElement = document.querySelector(`[data-transaction-id="${txId}"] [data-testid="status"]`);
        return statusElement?.textContent?.toLowerCase().includes('completed');
      },
      transactionId,
      { timeout }
    );
  }
}

/**
 * Legacy CSS selector fallbacks (to be phased out)
 * @deprecated Use TestSelectors class instead
 */
export const LEGACY_SELECTORS = {
  ETH_WALLET_BUTTON: 'button:has-text("Ethereum Wallet")',
  NEAR_WALLET_BUTTON: 'button:has-text("NEAR Wallet")',
  SWAP_NAV_BUTTON: '.bottom-nav__item:has-text("Swap"), .quick-action-btn:has-text("Swap")',
  AMOUNT_INPUT: 'input[type="number"], input[placeholder*="amount"]',
  TOKEN_SELECTOR: '.token-selector, .swap-form__token-selector',
  SUBMIT_BUTTON: 'button[type="submit"], button:has-text("Review Swap")',
  AUTH_REQUIRED: '.swap-form__auth-required',
  ERROR_ELEMENTS: '.error, [role="alert"], .notification--error'
} as const;