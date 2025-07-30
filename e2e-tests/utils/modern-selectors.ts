/**
 * Modern Playwright Selectors - Best Practices Implementation
 * Prioritizes accessibility and role-based selectors over CSS
 */

export const MODERN_SELECTORS = {
  // Authentication selectors using data-testid and role-based approaches
  AUTH: {
    ethereumWalletButton: '[data-testid="ethereum-wallet-button"]',
    nearWalletButton: '[data-testid="near-wallet-button"]',
    authenticationStatus: '.auth-manager__status',
    userProfile: '.auth-manager__profile',
    authError: '.auth-manager__error',
    loadingSpinner: '.auth-manager__loading',
    disconnectButton: '.auth-manager__disconnect, [data-testid="disconnect-button"]',
  } as const,

  // Bridge form selectors with improved specificity
  BRIDGE: {
    form: '.swap-form',
    authRequired: '.swap-form__auth-required',
    
    // Token selectors
    fromTokenSelector: '[data-testid="token-selector-ethereum"]',
    toTokenSelector: '[data-testid="token-selector-near"]',
    tokenSearchInput: '[data-testid="token-search-input"]',
    
    // Amount input
    amountInput: '[data-testid="amount-input"]',
    maxButton: '.amount-input__max-button',
    
    // Direction controls
    swapDirectionButton: '[data-testid="swap-direction-button"]',
    
    // Settings
    slippageSlider: '[data-testid="slippage-slider"]',
    
    // Submit button
    submitButton: '[data-testid="swap-button"]',
    
    // Status indicators
    securityIndicator: '.swap-form__security',
    riskAnalysis: '.swap-form__risk-analysis',
    priceQuote: '.swap-form__price-quote',
  },

  // Token selection modal
  TOKEN_MODAL: {
    dropdown: '.token-selector__dropdown',
    searchInput: '[data-testid="token-search-input"]',
    popularTokens: '.token-selector__popular',
    tokenList: '.token-selector__list',
    noResults: '.token-selector__no-results',
    
    // Dynamic token selectors (use with token symbol)
    popularToken: (symbol: string) => `[data-testid="popular-token-${symbol.toLowerCase()}"]`,
    tokenOption: (symbol: string) => `[data-testid="token-option-${symbol.toLowerCase()}"]`,
  },

  // Navigation
  NAVIGATION: {
    swapLink: 'text="Swap"',
    bridgeLink: 'text="Bridge"',
    bottomNavSwap: '.bottom-nav__item:has-text("Swap")',
    quickActionSwap: '.quick-action-btn:has-text("Swap")',
  },

  // Common UI elements
  COMMON: {
    loadingSpinner: '.spinner, [data-testid="loading"]',
    errorMessage: '.error-message, .auth-manager__error',
    successMessage: '.success-message',
    modal: '.modal, .dropdown',
    closeButton: '[aria-label="Close"], .close-button',
  },

  // Security and status indicators
  SECURITY: {
    quantumBadge: '.swap-form__quantum-badge',
    websocketStatus: '.swap-form__websocket-status',
    securityNote: '.auth-manager__security-note',
    riskIndicator: '.risk-indicator',
  },
} as const;

/**
 * Helper functions for dynamic selectors
 */
export const selectorHelpers = {
  /**
   * Get token selector by chain type
   */
  tokenSelectorByChain: (chain: 'ethereum' | 'near') => 
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
    
    return modernPatterns.some(pattern => pattern.test(selector));
  },

  /**
   * Get alternative selectors for fallback
   */
  getAlternatives: (primarySelector: string): string[] => {
    const alternatives: string[] = [];
    
    // If primary is data-testid, add class-based fallback
    if (primarySelector.includes('data-testid')) {
      const testId = primarySelector.match(/data-testid="([^"]+)"/)?.[1];
      if (testId) {
        alternatives.push(`.${testId.replace(/-/g, '__')}`);
      }
    }
    
    return alternatives;
  },
};