/**
 * Consolidated exports for all test utilities
 * This provides a single entry point for importing test utilities
 */

// Constants
export * from './constants.js';

// Core utilities
export * from './test-base.js';
export * from './api-helpers.js';
export * from './wallet-helpers.js';
export * from './element-helpers.js';
export * from './mock-wallet-utility.js';

// Page objects
export { BridgePage } from '../page-objects/BridgePage.js';
export { AuthPage } from '../page-objects/AuthPage.js';

// Usage examples:
// import { setupFullTestEnvironment, TEST_DATA, SELECTORS } from '../utils/index.js';
// import { BridgePage, AuthPage } from '../utils/index.js';