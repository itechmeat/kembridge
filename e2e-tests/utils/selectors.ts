/**
 * Unified selectors using Playwright best practices
 * Combines TestSelectors, RealisticSelectors, and ModernSelectors
 * Prioritizes accessibility and role-based selectors over CSS
 */
import { Page, Locator } from "@playwright/test";

export class TestSelectors {
  constructor(private page: Page) {}

  // Authentication selectors (role-based with realistic fallbacks)
  // INFO: Has data-testid
  get ethereumWalletButton(): Locator {
    return this.page
      .getByTestId("ethereum-wallet-button")
      .or(this.page.locator('[data-testid="ethereum-wallet-button"]'))
      .or(this.page.locator(".auth-manager__method--ethereum"))
      .or(
        this.page.locator('button:has-text("🦊Ethereum WalletConnect wallet")')
      )
      .or(this.page.getByRole("button", { name: /ethereum.*wallet/i }));
  }

  // INFO: Has data-testid
  get nearWalletButton(): Locator {
    return this.page
      .getByTestId("near-wallet-button")
      .or(this.page.locator('[data-testid="near-wallet-button"]'))
      .or(this.page.locator(".auth-manager__method--near"))
      .or(this.page.locator('button:has-text("🔷NEAR WalletConnect wallet")'))
      .or(this.page.getByRole("button", { name: /near.*wallet/i }));
  }

  // INFO: Has data-testid
  get connectWalletButton(): Locator {
    return this.page
      .getByTestId("connect-wallet-button")
      .or(this.page.getByRole("button", { name: /connect/i }))
      .or(this.page.locator(".wallet-connect-button"))
      .or(this.page.locator('button:has-text("Connect")'))
      .or(this.page.getByRole("button", { name: /connect.*wallet/i }));
  }

  // INFO: Has data-testid
  get disconnectButton(): Locator {
    return this.page
      .getByTestId("disconnect-button")
      .or(this.page.locator(".auth-manager__disconnect"))
      .or(this.page.getByRole("button", { name: /disconnect|logout/i }))
      .or(this.page.locator(".disconnect-button"))
      .or(this.page.locator('button:has-text("Disconnect")'));
  }

  // Authentication status and profile
  // INFO: Has data-testid
  get authenticationStatus(): Locator {
    return this.page
      .getByTestId("authentication-status")
      .or(this.page.locator(".auth-manager__status"));
  }

  // INFO: Has data-testid
  get userProfile(): Locator {
    return this.page
      .getByTestId("user-profile")
      .or(this.page.locator(".auth-manager__profile"));
  }

  // INFO: Has data-testid
  get authError(): Locator {
    return this.page
      .getByTestId("auth-error")
      .or(this.page.locator(".auth-manager__error"));
  }

  // Navigation selectors (role-based with realistic fallbacks)
  // INFO: Has data-testid
  get homeLink(): Locator {
    return this.page
      .getByTestId("bottom-nav-wallet")
      .or(this.page.getByRole("link", { name: /home/i }));
  }

  // INFO: Has data-testid
  get bridgeLink(): Locator {
    return this.page
      .getByTestId("bottom-nav-swap")
      .or(this.page.getByRole("link", { name: /bridge|swap/i }))
      .or(this.page.locator('.bridge-page__tab:has-text("Bridge")'));
  }

  // INFO: Has data-testid
  get dashboardLink(): Locator {
    return this.page
      .getByTestId("bottom-nav-history")
      .or(this.page.getByRole("link", { name: /dashboard/i }));
  }

  // INFO: Has data-testid
  get swapNavButton(): Locator {
    return this.page
      .getByTestId("bottom-nav-swap")
      .or(this.page.locator('.bottom-nav__item:has-text("Swap")'))
      .or(this.page.locator('button:has-text("🔄Swap")'))
      .or(this.page.locator('.quick-action-btn:has-text("Swap")'))
      .or(this.page.getByRole("button", { name: /swap/i }));
  }

  // INFO: Has data-testid
  get walletNavButton(): Locator {
    return this.page
      .getByTestId("bottom-nav-wallet")
      .or(this.page.locator('.bottom-nav__item[href*="wallet"]'))
      .or(this.page.getByRole("link", { name: /wallet/i }));
  }

  // INFO: Has data-testid
  get historyNavButton(): Locator {
    return this.page
      .getByTestId("bottom-nav-history")
      .or(this.page.locator('.bottom-nav__item[href*="history"]'))
      .or(this.page.getByRole("link", { name: /history/i }));
  }

  // INFO: Has data-testid
  get settingsNavButton(): Locator {
    return this.page
      .getByTestId("bottom-nav-settings")
      .or(this.page.locator('.bottom-nav__item[href*="settings"]'))
      .or(this.page.getByRole("link", { name: /settings/i }));
  }

  // INFO: Has data-testid
  get bridgeNavButton(): Locator {
    return this.page
      .getByTestId("bridge-tab")
      .or(this.page.locator('.bridge-page__tab:has-text("Bridge")'))
      .or(this.page.locator('button:has-text("Bridge")'));
  }

  // INFO: Has data-testid
  get historyNavTab(): Locator {
    return this.page
      .getByTestId("history-tab")
      .or(this.page.locator('.bridge-page__tab:has-text("History")'))
      .or(this.page.getByRole("button", { name: /history/i }));
  }

  // Bridge form elements
  // INFO: Has data-testid
  get bridgeForm(): Locator {
    return this.page
      .getByTestId("bridge-form")
      .or(
        this.page.locator(".bridge-form, .swap-container, .bridge-container")
      );
  }

  // INFO: Has data-testid
  get swapForm(): Locator {
    return this.page
      .getByTestId("swap-form")
      .or(this.page.locator(".swap-form, .bridge-form, .swap-container"));
  }

  // Bridge form selectors (accessible with realistic fallbacks)
  // INFO: Has data-testid
  get amountInput(): Locator {
    return this.page
      .getByTestId("amount-input")
      .or(this.page.locator('[data-testid="amount-input"]'))
      .or(this.page.locator('input[type="number"]'))
      .or(this.page.locator('input[placeholder*="amount"]'))
      .or(this.page.locator('input[placeholder="0.0"]'))
      .or(this.page.locator(".amount-input input"))
      .or(this.page.locator(".swap-form__amount-input input"))
      .or(this.page.getByRole("textbox", { name: /amount/i }))
      .or(this.page.getByPlaceholder(/amount|0\.0/i))
      .or(this.page.getByLabel(/amount/i));
  }

  // INFO: Has data-testid
  get fromTokenSelector(): Locator {
    return this.page
      .getByTestId("token-selector-ethereum")
      .or(this.page.locator('[data-testid^="token-selector-"]').first())
      .or(this.page.locator(".token-selector").first())
      .or(this.page.locator(".swap-form__token-selector").first())
      .or(this.page.locator(".from-token"))
      .or(this.page.locator(".token-select").first())
      .or(
        this.page.getByRole("combobox", { name: /from.*token|source.*token/i })
      )
      .or(this.page.getByLabel(/from.*token/i));
  }

  // INFO: Has data-testid
  get toTokenSelector(): Locator {
    return this.page
      .getByTestId("token-selector-near")
      .or(this.page.locator('[data-testid^="token-selector-"]').last())
      .or(this.page.locator(".token-selector").last())
      .or(this.page.locator(".swap-form__token-selector").last())
      .or(this.page.locator(".to-token"))
      .or(this.page.locator(".token-select").last())
      .or(
        this.page.getByRole("combobox", {
          name: /to.*token|destination.*token/i,
        })
      )
      .or(this.page.getByLabel(/to.*token/i));
  }

  // INFO: Has data-testid
  get reviewSwapButton(): Locator {
    return this.page
      .getByTestId("swap-button")
      .or(this.page.locator('[data-testid="swap-button"]'))
      .or(this.page.locator('button[type="submit"]'))
      .or(this.page.locator('button:has-text("Review Swap")'))
      .or(this.page.locator('button:has-text("Get Quote")'))
      .or(this.page.locator('button:has-text("Swap")'))
      .or(this.page.locator('button:has-text("Bridge")'))
      .or(this.page.locator('button:has-text("Execute")'))
      .or(this.page.locator(".swap-form__submit"))
      .or(this.page.locator(".submit-btn"))
      .or(
        this.page.getByRole("button", {
          name: /review.*swap|get.*quote|initiate.*bridge/i,
        })
      )
      .or(
        this.page
          .getByRole("button", { name: /swap/i })
          .and(this.page.locator('[type="submit"]'))
      );
  }

  // INFO: Has data-testid
  get swapButton(): Locator {
    return this.page
      .getByTestId("swap-button")
      .or(this.page.getByRole("button", { name: /swap|bridge/i }))
      .or(this.page.locator(".swap-form__submit"))
      .or(this.page.locator('button[type="submit"]'));
  }

  // INFO: Has data-testid
  get confirmSwapButton(): Locator {
    return this.page
      .getByTestId("confirm-swap-button")
      .or(this.page.locator('[data-testid="confirm-swap-button"]'))
      .or(this.page.locator(".swap-confirmation__confirm"))
      .or(this.page.locator(".confirm-button"))
      .or(
        this.page.getByRole("button", { name: /confirm.*swap|execute.*swap/i })
      );
  }

  // INFO: Has data-testid
  get directionSwitch(): Locator {
    return this.page
      .getByTestId("swap-direction-button")
      .or(this.page.locator('[data-testid="swap-direction-button"]'))
      .or(this.page.locator('button:has-text("⇅")'))
      .or(this.page.locator('button:has-text("↔")'))
      .or(this.page.locator('button:has-text("⇄")'))
      .or(this.page.locator(".swap-direction"))
      .or(this.page.locator(".reverse-button"))
      .or(this.page.locator(".swap-form__swap-button"))
      .or(this.page.locator('button[aria-label*="switch"]'))
      .or(this.page.locator('button[aria-label*="reverse"]'))
      .or(this.page.getByRole("button", { name: /switch.*direction|reverse/i }))
      .or(this.page.getByLabel(/switch.*direction/i));
  }

  // Additional form elements from modern selectors
  // INFO: Has data-testid
  get maxButton(): Locator {
    return this.page
      .getByTestId("max-button")
      .or(this.page.getByRole("button", { name: /max/i }))
      .or(this.page.locator(".amount-input__max-button"))
      .or(this.page.locator('button:has-text("MAX")'));
  }

  // INFO: Has data-testid
  get slippageSlider(): Locator {
    return this.page
      .getByTestId("slippage-slider")
      .or(this.page.locator('[data-testid="slippage-slider"]'))
      .or(this.page.locator('input[type="range"]'))
      .or(this.page.locator(".swap-form__slippage-slider"));
  }

  // INFO: Has data-testid
  get securityIndicator(): Locator {
    return this.page
      .getByTestId("security-indicator")
      .or(this.page.locator('[data-testid="security-indicator"]'))
      .or(this.page.locator(".swap-form__security"));
  }

  // INFO: Has data-testid
  get riskAnalysis(): Locator {
    return this.page
      .getByTestId("risk-analysis-display")
      .or(this.page.locator(".swap-form__risk-analysis"));
  }

  // INFO: Has data-testid
  get priceQuote(): Locator {
    return this.page
      .getByTestId("price-quote")
      .or(this.page.locator(".swap-form__price-quote"));
  }

  // Token selection modal selectors
  // INFO: Has data-testid
  get tokenDropdown(): Locator {
    return this.page
      .getByTestId("token-dropdown")
      .or(this.page.locator(".token-selector__dropdown"));
  }

  // INFO: Has data-testid
  get tokenSearchInput(): Locator {
    return this.page
      .getByTestId("token-search-input")
      .or(this.page.locator('[data-testid="token-search-input"]'));
  }

  // INFO: Has data-testid
  get popularTokens(): Locator {
    return this.page
      .getByTestId("popular-tokens")
      .or(this.page.locator(".token-selector__popular"));
  }

  // INFO: Has data-testid
  get tokenList(): Locator {
    return this.page
      .getByTestId("token-list")
      .or(this.page.locator(".token-selector__list"));
  }

  // INFO: Has data-testid
  get noTokenResults(): Locator {
    return this.page
      .getByTestId("no-token-results")
      .or(this.page.locator(".token-selector__no-results"));
  }

  // Dynamic token selectors
  getPopularToken(symbol: string): Locator {
    return this.page.locator(
      `[data-testid="popular-token-${symbol.toLowerCase()}"]`
    );
  }

  getTokenOption(symbol: string): Locator {
    return this.page.locator(
      `[data-testid="token-option-${symbol.toLowerCase()}"]`
    );
  }

  // Status and feedback selectors (with realistic fallbacks)
  // INFO: Has data-testid
  get successMessage(): Locator {
    return this.page
      .getByTestId("success-message")
      .or(this.page.locator(".success"))
      .or(this.page.locator(".alert-success"))
      .or(this.page.locator(".notification--success"))
      .or(this.page.locator('[class*="success"]'))
      .or(
        this.page
          .getByRole("alert")
          .filter({ hasText: /success|complete|confirmed/i })
      )
      .or(this.page.locator('[role="status"]').filter({ hasText: /success/i }));
  }

  // INFO: Has data-testid
  get errorMessage(): Locator {
    return this.page
      .getByTestId("error-message")
      .or(this.page.locator(".error"))
      .or(this.page.locator(".alert-error"))
      .or(this.page.locator(".notification--error"))
      .or(this.page.locator('[class*="error"]'))
      .or(
        this.page
          .getByRole("alert")
          .filter({ hasText: /error|failed|invalid/i })
      )
      .or(this.page.locator('[role="alertdialog"]'))
      .or(this.page.locator(".error, .alert-error"));
  }

  // INFO: Has data-testid
  get loadingSpinner(): Locator {
    return this.page
      .getByTestId("loading-spinner")
      .or(this.page.locator(".loading"))
      .or(this.page.locator(".spinner"))
      .or(this.page.locator(".loader"))
      .or(this.page.locator('[class*="loading"]'))
      .or(this.page.locator('[class*="spinner"]'))
      .or(this.page.getByRole("status", { name: /loading/i }))
      .or(this.page.locator('[aria-label*="loading"]'));
  }

  // INFO: Has data-testid
  get authRequiredMessage(): Locator {
    return this.page
      .getByTestId("auth-required")
      .or(this.page.locator(".auth-required"))
      .or(this.page.locator(".connect-wallet-prompt"))
      .or(this.page.locator(".swap-form__auth-required"))
      .or(
        this.page.getByText(/authentication.*required|connect.*wallet.*first/i)
      );
  }

  // INFO: Has data-testid
  get transactionStatus(): Locator {
    return this.page
      .locator('[data-testid="transaction-status"]')
      .or(this.page.getByText(/pending|processing|completed|failed/i).first());
  }

  // Security indicators
  // INFO: Has data-testid
  get quantumSecurityBadge(): Locator {
    return this.page
      .getByTestId("quantum-protection-status")
      .or(this.page.getByTestId("security-indicator"))
      .or(this.page.getByText(/quantum.*protected|ml-kem|post.*quantum/i))
      .or(this.page.locator('[data-testid*="quantum"]'));
  }

  // INFO: Has data-testid
  get riskScore(): Locator {
    return this.page
      .locator('[data-testid="risk-score"]')
      .or(this.page.getByText(/risk.*score/i).locator(".."))
      .or(this.page.locator(".risk-score"));
  }

  // INFO: Has data-testid
  get securityWarning(): Locator {
    return this.page
      .getByTestId("security-warning")
      .or(
        this.page
          .getByRole("alert")
          .filter({ hasText: /security|warning|risk/i })
      );
  }

  // AI Risk Display selectors
  // INFO: Has data-testid
  get aiRiskDisplay(): Locator {
    return this.page
      .getByTestId("ai-risk-display")
      .or(this.page.getByTestId("ai-risk-display-ready"))
      .or(this.page.locator(".ai-risk-display"));
  }

  // INFO: Has data-testid
  get aiRiskDisplayOffline(): Locator {
    return this.page
      .getByTestId("ai-risk-display-offline")
      .or(this.page.getByText(/ai.*risk.*engine.*offline/i))
      .or(this.page.locator(".ai-risk-display--offline"));
  }

  // INFO: Has data-testid
  get aiRiskDisplayLoading(): Locator {
    return this.page
      .getByTestId("ai-risk-display-loading")
      .or(this.page.getByText(/analyzing.*risk/i))
      .or(this.page.locator(".ai-risk-display--loading"));
  }

  // INFO: Has data-testid
  get aiRiskScoreValue(): Locator {
    return this.page
      .getByTestId("ai-risk-score-value")
      .or(this.page.locator('[data-testid*="risk-score"]'));
  }

  // INFO: Has data-testid
  get aiRiskScoreLevel(): Locator {
    return this.page
      .getByTestId("ai-risk-score-level")
      .or(this.page.getByText(/risk.*level/i).locator(".."));
  }

  // INFO: Has data-testid
  get aiRiskApprovalStatus(): Locator {
    return this.page
      .getByTestId("ai-risk-approval-status")
      .or(this.page.getByText(/approved|blocked/i));
  }

  // INFO: Has data-testid
  get aiRiskWarning(): Locator {
    return this.page
      .getByTestId("risk-warning")
      .or(
        this.page
          .getByRole("alert")
          .filter({ hasText: /risk|warning|blocked/i })
      )
      .or(this.page.locator(".ai-risk-display__approval.blocked"));
  }

  // INFO: Has data-testid
  get aiRiskToggleDetails(): Locator {
    return this.page
      .getByTestId("ai-risk-toggle-details")
      .or(this.page.getByRole("button", { name: /details|expand/i }))
      .or(this.page.locator('button:has-text("▼"), button:has-text("▶")'));
  }

  // INFO: Has data-testid
  get aiRiskDisplayDetails(): Locator {
    return this.page
      .getByTestId("ai-risk-display-details")
      .or(this.page.getByText(/risk.*factors/i).locator(".."))
      .or(this.page.locator(".ai-risk-display__details"));
  }

  // INFO: Has data-testid
  get aiRiskRefreshButton(): Locator {
    return this.page
      .getByTestId("ai-risk-refresh-button")
      .or(this.page.getByRole("button", { name: /refresh/i }))
      .or(this.page.locator('button:has-text("🔄")'));
  }

  // INFO: Has data-testid
  get aiRiskDisplayError(): Locator {
    return this.page
      .getByTestId("ai-risk-display-error")
      .or(this.page.getByText(/risk.*analysis.*failed/i))
      .or(this.page.locator(".ai-risk-display--error"));
  }

  // Form validation
  // INFO: Has data-testid
  get signInMessage(): Locator {
    return this.page
      .getByText(/sign in with your wallet/i)
      .or(this.page.getByTestId("sign-in-message"))
      .or(this.page.locator(".sign-in-message"));
  }

  get insufficientBalanceError(): Locator {
    return this.page.getByText(/insufficient.*balance|not.*enough.*funds/i);
  }

  // Modal and dialog selectors
  get modal(): Locator {
    return this.page
      .getByRole("dialog")
      .or(this.page.locator('[role="modal"]'))
      .or(this.page.locator(".modal"));
  }

  get modalCloseButton(): Locator {
    return this.page
      .getByTestId("modal-close-button")
      .or(this.page.getByRole("button", { name: /close|×/i }))
      .or(this.page.locator(".modal__close"))
      .or(this.page.locator('button:has-text("×")'));
  }

  // Utility methods for complex selections
  getTokenSelectorByIndex(index: number): Locator {
    return this.page.locator(".token-selector").nth(index);
  }

  getTransactionById(id: string): Locator {
    return this.page
      .locator(`[data-transaction-id="${id}"]`)
      .or(this.page.getByText(id).locator(".."));
  }

  getChainOption(chain: "ethereum" | "near"): Locator {
    return this.page
      .getByRole("option", { name: new RegExp(chain, "i") })
      .or(this.page.locator(`[data-chain="${chain}"]`));
  }

  // Wait helpers with role-based selectors
  async waitForWalletConnected(timeout = 10000): Promise<void> {
    await this.page.waitForFunction(
      () => {
        const buttons = document.querySelectorAll(
          'button[aria-label*="wallet"], button'
        );
        for (const button of buttons) {
          if (
            button.textContent?.toLowerCase().includes("connected") &&
            !button.textContent?.toLowerCase().includes("connect")
          ) {
            return true;
          }
        }
        return false;
      },
      undefined,
      { timeout }
    );
  }

  async waitForFormReady(timeout = 10000): Promise<void> {
    await Promise.all([
      this.amountInput.waitFor({ state: "visible", timeout }),
      this.fromTokenSelector.waitFor({ state: "visible", timeout }),
      this.toTokenSelector.waitFor({ state: "visible", timeout }),
    ]);
  }

  async waitForTransactionComplete(
    transactionId: string,
    timeout = 60000
  ): Promise<void> {
    await this.page.waitForFunction(
      (txId) => {
        const statusElement = document.querySelector(
          `[data-transaction-id="${txId}"] [data-testid="status"]`
        );
        return statusElement?.textContent?.toLowerCase().includes("completed");
      },
      transactionId,
      { timeout }
    );
  }

  // Additional helper methods from RealisticSelectors
  async waitForPageLoad(timeout = 15000): Promise<void> {
    await this.page.waitForLoadState("domcontentloaded");
    await this.page.waitForSelector("body", { timeout });

    try {
      await this.page.waitForSelector(".loading, .spinner", {
        state: "hidden",
        timeout: 5000,
      });
    } catch {
      // Ignore if no loading indicators found
    }
  }

  async debugSelectors(): Promise<void> {
    console.log("🔍 Debugging available selectors...");

    const allButtons = await this.page.locator("button").count();
    console.log(`Total buttons found: ${allButtons}`);

    for (let i = 0; i < Math.min(allButtons, 10); i++) {
      const button = this.page.locator("button").nth(i);
      const text = await button.textContent();
      const classes = await button.getAttribute("class");
      console.log(`Button ${i}: "${text}" (classes: ${classes})`);
    }

    const allInputs = await this.page.locator("input").count();
    console.log(`Total inputs found: ${allInputs}`);

    for (let i = 0; i < Math.min(allInputs, 5); i++) {
      const input = this.page.locator("input").nth(i);
      const type = await input.getAttribute("type");
      const placeholder = await input.getAttribute("placeholder");
      const classes = await input.getAttribute("class");
      console.log(
        `Input ${i}: type="${type}" placeholder="${placeholder}" (classes: ${classes})`
      );
    }
  }

  findByText(text: string): Locator {
    return this.page
      .locator(`text=${text}`)
      .or(this.page.locator(`:has-text("${text}")`));
  }

  findByPartialText(partialText: string): Locator {
    return this.page.locator(`:has-text("${partialText}")`);
  }

  getByTestId(testId: string): Locator {
    return this.page.locator(`[data-testid="${testId}"]`);
  }

  async elementExists(selector: string): Promise<boolean> {
    try {
      await this.page
        .locator(selector)
        .waitFor({ state: "attached", timeout: 1000 });
      return true;
    } catch {
      return false;
    }
  }
}

/**
 * Modern Playwright Selectors - Best Practices Implementation
 * Prioritizes accessibility and role-based selectors over CSS
 */
export const MODERN_SELECTORS = {
  // Authentication selectors using data-testid and role-based approaches
  AUTH: {
    ethereumWalletButton: '[data-testid="ethereum-wallet-button"]',
    nearWalletButton: '[data-testid="near-wallet-button"]',
    // INFO: Has data-testid
    authenticationStatus: '[data-testid="authentication-status"]',
    // INFO: Has data-testid
    userProfile: '[data-testid="user-profile"]',
    // INFO: Has data-testid
    authError: '[data-testid="auth-error"]',
    // INFO: Has data-testid
    loadingSpinner: '[data-testid="loading-spinner"]',
    // INFO: Has data-testid
    disconnectButton:
      '.auth-manager__disconnect, [data-testid="disconnect-button"]',
  } as const,

  // Bridge form selectors with improved specificity
  BRIDGE: {
    // INFO: Has data-testid
    form: '[data-testid="swap-form"]',
    // INFO: Has data-testid
    authRequired: '[data-testid="auth-required"]',

    // Token selectors
    fromTokenSelector: '[data-testid="token-selector-ethereum"]',
    toTokenSelector: '[data-testid="token-selector-near"]',
    tokenSearchInput: '[data-testid="token-search-input"]',

    // Amount input
    amountInput: '[data-testid="amount-input"]',
    // INFO: Has data-testid
    maxButton: '[data-testid="max-button"]',

    // Direction controls
    swapDirectionButton: '[data-testid="swap-direction-button"]',

    // Settings
    slippageSlider: '[data-testid="slippage-slider"]',

    // Submit button
    submitButton: '[data-testid="swap-button"]',

    // Status indicators
    // INFO: Has data-testid
    securityIndicator: '[data-testid="security-indicator"]',
    // INFO: Has data-testid
    riskAnalysis: '[data-testid="risk-analysis-display"]',
    // INFO: Has data-testid
    priceQuote: '[data-testid="price-quote"]',
  },

  // Token selection modal
  TOKEN_MODAL: {
    // INFO: Has data-testid
    dropdown: '[data-testid="token-dropdown"]',
    searchInput: '[data-testid="token-search-input"]',
    // INFO: Has data-testid
    popularTokens: '[data-testid="popular-tokens"]',
    // INFO: Has data-testid
    tokenList: '[data-testid="token-list"]',
    // INFO: Has data-testid
    noResults: '[data-testid="no-token-results"]',

    // Dynamic token selectors (use with token symbol)
    popularToken: (symbol: string) =>
      `[data-testid="popular-token-${symbol.toLowerCase()}"]`,
    tokenOption: (symbol: string) =>
      `[data-testid="token-option-${symbol.toLowerCase()}"]`,
  },

  // Navigation
  NAVIGATION: {
    // INFO: Has data-testid
    swapLink: '[data-testid="bottom-nav-swap"]',
    // INFO: Has data-testid
    bridgeLink: '[data-testid="bridge-tab"]',
    // INFO: Has data-testid
    bottomNavSwap: '[data-testid="bottom-nav-swap"]',
    // INFO: Has data-testid
    quickActionSwap: '[data-testid="quick-action-swap"]',
  },

  // Common UI elements
  COMMON: {
    loadingSpinner: '.spinner, [data-testid="loading"]',
    errorMessage: ".error-message, .auth-manager__error",
    successMessage: ".success-message",
    // INFO: Has data-testid
    modal: '[data-testid="modal"]',
    // INFO: Has data-testid
    closeButton: '[data-testid="modal-close-button"]',
  },

  // Security and status indicators
  SECURITY: {
    // INFO: Has data-testid
    quantumBadge: '[data-testid="quantum-protection-display"]',
    // INFO: Has data-testid
    websocketStatus: '[data-testid="websocket-status"]',
    // INFO: Has data-testid
    securityNote: '[data-testid="auth-security-note"]',
    // INFO: Has data-testid
    riskIndicator: "[data-testid='risk-analysis-display']",
  },
} as const;

/**
 * Helper functions for dynamic selectors
 */
export const selectorHelpers = {
  /**
   * Get token selector by chain type
   */
  tokenSelectorByChain: (chain: "ethereum" | "near") =>
    `[data-testid="token-selector-${chain}"]`,

  /**
   * Get token option by symbol
   */
  tokenBySymbol: (symbol: string) =>
    `[data-testid="token-option-${symbol.toLowerCase()}"]`,

  /**
   * Get popular token by symbol
   */
  popularTokenBySymbol: (symbol: string) =>
    `[data-testid="popular-token-${symbol.toLowerCase()}"]`,

  /**
   * Get button by text content (fallback for dynamic content)
   */
  buttonByText: (text: string) => `button:has-text("${text}")`,

  /**
   * Get element by role and name (accessibility-first)
   */
  byRole: (role: string, name?: string) =>
    name ? `role=${role}[name="${name}"]` : `role=${role}`,

  /**
   * Get element by label (form inputs)
   */
  byLabel: (label: string) => `label:has-text("${label}")`,

  /**
   * Get element by placeholder text
   */
  byPlaceholder: (placeholder: string) => `[placeholder="${placeholder}"]`,
};

/**
 * Validation helpers for selector existence
 */
export const selectorValidation = {
  /**
   * Check if selector follows modern best practices
   */
  isModernSelector: (selector: string): boolean => {
    // Prefer data-testid, role, or semantic selectors
    const modernPatterns = [
      /^\[data-testid=/, // data-testid attributes
      /^role=/, // role-based selectors
      /^text=/, // text content selectors
      /^\[aria-/, // ARIA attributes
      /^\[placeholder=/, // placeholder attributes
    ];

    return modernPatterns.some((pattern) => pattern.test(selector));
  },

  /**
   * Get alternative selectors for fallback
   */
  getAlternatives: (primarySelector: string): string[] => {
    const alternatives: string[] = [];

    // If primary is data-testid, add class-based fallback
    if (primarySelector.includes("data-testid")) {
      const testId = primarySelector.match(/data-testid="([^"]+)"/)?.[1];
      if (testId) {
        alternatives.push(`.${testId.replace(/-/g, "__")}`);
      }
    }

    return alternatives;
  },
};

/**
 * Legacy CSS selector fallbacks (to be phased out)
 * @deprecated Use TestSelectors class instead
 */
export const LEGACY_SELECTORS = {
  ETH_WALLET_BUTTON:
    'button[aria-label*="ethereum"], button[title*="ethereum"]',
  NEAR_WALLET_BUTTON: 'button[aria-label*="near"], button[title*="near"]',
  SWAP_NAV_BUTTON: ".bottom-nav__item, .quick-action-btn",
  AMOUNT_INPUT: 'input[type="number"], input[placeholder*="amount"]',
  TOKEN_SELECTOR: ".token-selector, .swap-form__token-selector",
  SUBMIT_BUTTON: 'button[type="submit"], button[aria-label*="review"]',
  AUTH_REQUIRED: ".swap-form__auth-required",
  ERROR_ELEMENTS: '.error, [role="alert"], .notification--error',
} as const;

// Export common selector patterns that actually work in the real application
export const WORKING_SELECTORS = {
  // VERIFIED selectors from validation tests
  ETH_WALLET_BUTTON: ".auth-manager__method--ethereum",
  NEAR_WALLET_BUTTON: ".auth-manager__method--near",
  CONNECT_BUTTON: ".wallet-connect-button",
  SWAP_NAV: '.bottom-nav__item:has-text("Swap")',
  HISTORY_NAV: '.bottom-nav__item:has-text("History")',
  WALLET_NAV: '.bottom-nav__item:has-text("Wallet")',
  SETTINGS_NAV: '.bottom-nav__item:has-text("Settings")',

  // Form elements (need investigation when auth is complete)
  AMOUNT_INPUT:
    'input[type="number"], input[placeholder*="amount"], input[placeholder="0.0"]',
  TOKEN_SELECTOR: ".token-selector, .swap-form__token-selector",
  SUBMIT_BUTTON:
    'button[type="submit"], button:has-text("Review Swap"), button:has-text("Swap")',
  AUTH_REQUIRED: ".swap-form__auth-required",
  DIRECTION_SWITCH: 'button:has-text("⇅")',

  // Security and app features
  QUANTUM_BADGE: ':has-text("🔒"), :has-text("Quantum")',
  SWAP_FORM: ".swap-form",
  BRIDGE_TAB: '.bridge-page__tab:has-text("Bridge")',
  HISTORY_TAB: '.bridge-page__tab:has-text("History")',
} as const;

// Export RealisticSelectors class for backward compatibility
export class RealisticSelectors extends TestSelectors {
  constructor(page: Page) {
    super(page);
  }
}
