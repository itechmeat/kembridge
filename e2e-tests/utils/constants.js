/**
 * Test constants and configuration
 */

import { TEST_URLS } from './test-constants';

export const API_ENDPOINTS = {
  BASE: TEST_URLS.BACKEND.GATEWAY,
  HEALTH: '/health',
  AUTH_NONCE: '/api/v1/auth/nonce',
  AUTH_VERIFY: '/api/v1/auth/verify',
  BRIDGE_TOKENS: '/api/v1/bridge/tokens',
  BRIDGE_QUOTE: '/api/v1/bridge/quote',
  BRIDGE_SWAP: '/api/v1/bridge/swap',
  RISK_ANALYSIS: '/api/v1/risk/analysis',
  SECURITY_CHECK: '/api/v1/security/check'
};

export const SERVICE_ENDPOINTS = {
  GATEWAY: TEST_URLS.BACKEND.GATEWAY,
  ONEINCH: TEST_URLS.BACKEND.ONEINCH, 
  BLOCKCHAIN: TEST_URLS.BACKEND.BLOCKCHAIN,
  CRYPTO: TEST_URLS.BACKEND.CRYPTO,
  AUTH: TEST_URLS.BACKEND.AUTH,
  AI_ENGINE: TEST_URLS.BACKEND.AI_ENGINE
};

export const FRONTEND_URL = TEST_URLS.FRONTEND.LOCAL_DEV;

export const TIMEOUTS = {
  SHORT: 1000,
  MEDIUM: 3000,
  LONG: 5000,
  VERY_LONG: 10000,
  AUTH_FLOW: 10000,
  PAGE_LOAD: 15000
};

export const TEST_DATA = {
  PRIVATE_KEY: "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80",
  ETHEREUM_ADDRESS: "0x1234567890123456789012345678901234567890",
  NEAR_ADDRESS: "kembridge.testnet",
  TEST_AMOUNTS: {
    SMALL: "0.01",
    MEDIUM: "1.0", 
    LARGE: "100",
    INVALID: ["abc", "-1", "0", "999999999999"]
  }
};

export const SELECTORS = {
  // Authentication
  ETH_WALLET_BUTTON: 'button:has-text("Ethereum Wallet")',
  NEAR_WALLET_BUTTON: 'button:has-text("NEAR Wallet")',
  ONBOARDING_TITLE: '.onboarding__title:has-text("Welcome to KEMBridge")',
  
  // Navigation
  SWAP_NAV_BUTTON: '.bottom-nav__item:has-text("Swap"), .quick-action-btn:has-text("Swap")',
  
  // Bridge Form
  AUTH_REQUIRED: '.swap-form__auth-required',
  TOKEN_SELECTOR: '.token-selector, .swap-form__token-selector',
  AMOUNT_INPUT: [
    'input[type="number"]',
    'input[placeholder*="amount"]', 
    'input[placeholder="0.0"]',
    '.amount-input input',
    '.swap-form__amount-input input'
  ],
  SUBMIT_BUTTON: [
    'button[type="submit"]',
    'button:has-text("Review Swap")',
    'button:has-text("Get Quote")',
    'button:has-text("Swap")',
    '.swap-form__submit'
  ],
  
  // Direction Switch
  DIRECTION_SWITCH: [
    'button:has-text("⇅")',
    'button:has-text("↔")', 
    'button:has-text("⇄")',
    '.swap-direction',
    '.reverse-button',
    '.swap-form__swap-button'
  ],
  
  // Security & Risk
  SECURITY_INDICATORS: [
    'text=Quantum Protected',
    'text=ML-KEM',
    'text=Post-quantum',
    '.quantum-badge',
    '.security-indicator',
    '[data-testid*="quantum"]'
  ],
  RISK_INDICATORS: [
    '.risk-analysis',
    '.risk-score',
    '.security-alert',
    '[data-testid*="risk"]',
    'text=Risk Score',
    'text=Risk Analysis'
  ],
  
  // Error Handling
  ERROR_ELEMENTS: '.error, [role="alert"], .notification--error, .invalid'
};

export const EXPECTED_PATTERNS = {
  NONCE_FORMAT: /^[a-f0-9]{64}$/,
  AUTH_MESSAGE_CONTENT: [
    'KEMBridge Authentication',
    'ethereum',
    'near'
  ]
};

// AI Risk Engine constants
export const SERVICE_URLS = {
  AI_ENGINE: process.env.AI_ENGINE_URL || TEST_URLS.BACKEND.AI_ENGINE,
  BACKEND_GATEWAY: process.env.BACKEND_URL || TEST_URLS.BACKEND.GATEWAY,
};

export const RISK_ANALYSIS = {
  THRESHOLDS: {
    LOW: 0.3,
    MEDIUM: 0.6,
    HIGH: 0.8,
    AUTO_BLOCK: 0.9,
  },
  LEVELS: {
    LOW: 'low',
    MEDIUM: 'medium', 
    HIGH: 'high',
  }
};

export const DEFAULT_USER_ID = "test_user_e2e";