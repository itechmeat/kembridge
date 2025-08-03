/**
 * Mock Wallet Utility
 * Optimized reusable utility for setting up mock wallet in tests
 * Features global caching and reuse to minimize initialization time
 */

import { installMockWallet } from '@johanneskares/wallet-mock';
import { privateKeyToAccount } from 'viem/accounts';
import { http } from 'viem';
import { sepolia } from 'viem/chains';

/**
 * Default configuration for mock wallet
 */
const DEFAULT_MOCK_WALLET_CONFIG = {
  account: privateKeyToAccount(
    "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
  ),
  defaultChain: sepolia,
  transports: { [sepolia.id]: http() },
};

/**
 * Global cache for mock wallet state
 * Allows reusing initialized wallet across tests
 */
const WALLET_CACHE = {
  isInitialized: false,
  initializationPromise: null,
  config: null,
  lastPage: null,
  initCount: 0
};

/**
 * Performance tracking
 */
const PERF_TRACKER = {
  totalSetupTime: 0,
  setupCount: 0,
  cacheHits: 0,
  cacheMisses: 0
};

/**
 * Checks if wallet is already available in the page
 * @param {Page} page - Playwright page object
 * @returns {Promise<boolean>}
 */
async function isWalletAlreadyAvailable(page) {
  try {
    return await page.evaluate(() => {
      return typeof window.ethereum !== 'undefined' && 
             window.ethereum.isMetaMask === true;
    });
  } catch (error) {
    return false;
  }
}

/**
 * Fast wallet setup with caching and reuse
 * @param {Page} page - Playwright page object
 * @param {Object} customConfig - Optional custom configuration to override defaults
 * @param {boolean} forceReinstall - Force reinstallation even if wallet exists
 * @returns {Promise<{success: boolean, fromCache: boolean, setupTime: number}>}
 */
export async function setupMockWallet(page, customConfig = {}, forceReinstall = false) {
  const startTime = Date.now();
  const config = { ...DEFAULT_MOCK_WALLET_CONFIG, ...customConfig };
  
  try {
    // Check if wallet is already available and we don't need to force reinstall
    if (!forceReinstall && await isWalletAlreadyAvailable(page)) {
      PERF_TRACKER.cacheHits++;
      const setupTime = Date.now() - startTime;
      console.log(`⚡ Mock wallet already available (${setupTime}ms)`);
      return { success: true, fromCache: true, setupTime };
    }

    // Install mock wallet
    await installMockWallet({
      page,
      ...config
    });
    
    // Update cache
    WALLET_CACHE.isInitialized = true;
    WALLET_CACHE.config = config;
    WALLET_CACHE.lastPage = page;
    WALLET_CACHE.initCount++;
    
    PERF_TRACKER.cacheMisses++;
    PERF_TRACKER.setupCount++;
    
    const setupTime = Date.now() - startTime;
    PERF_TRACKER.totalSetupTime += setupTime;
    
    console.log(`✅ Mock wallet installed successfully (${setupTime}ms)`);
    return { success: true, fromCache: false, setupTime };
  } catch (error) {
    const setupTime = Date.now() - startTime;
    console.error(`❌ Failed to install mock wallet (${setupTime}ms):`, error);
    return { success: false, fromCache: false, setupTime, error };
  }
}

/**
 * Waits for mock wallet to be available in the page
 * @param {Page} page - Playwright page object
 * @param {number} timeout - Timeout in milliseconds (default: 10000)
 * @returns {Promise<boolean>} - Returns true if wallet is available, false otherwise
 */
export async function waitForMockWalletAvailable(page, timeout = 10000) {
  const startTime = Date.now();
  
  while (Date.now() - startTime < timeout) {
    try {
      const isAvailable = await page.evaluate(() => {
        return typeof window.ethereum !== 'undefined' && 
               window.ethereum.isMetaMask === true;
      });
      
      if (isAvailable) {
        console.log('✅ Mock wallet is available');
        return true;
      }
    } catch (error) {
      // Continue waiting
    }
    
    await page.waitForTimeout(500);
  }
  
  console.log('❌ Mock wallet not available after timeout');
  return false;
}

/**
 * Complete mock wallet setup with availability check and performance tracking
 * @param {Page} page - Playwright page object
 * @param {Object} options - Configuration options
 * @param {Object} options.config - Custom wallet configuration
 * @param {number} options.timeout - Timeout for availability check
 * @param {boolean} options.skipAvailabilityCheck - Skip waiting for availability
 * @param {boolean} options.forceReinstall - Force reinstallation even if wallet exists
 * @returns {Promise<{success: boolean, fromCache: boolean, setupTime: number, availabilityTime?: number}>}
 */
export async function setupMockWalletComplete(page, options = {}) {
  const {
    config = {},
    timeout = 10000,
    skipAvailabilityCheck = false,
    forceReinstall = false
  } = options;
  
  const startTime = Date.now();
  
  try {
    // Install mock wallet with caching
    const setupResult = await setupMockWallet(page, config, forceReinstall);
    
    if (!setupResult.success) {
      return setupResult;
    }
    
    let availabilityTime = 0;
    
    // Wait for it to be available (unless skipped)
    if (!skipAvailabilityCheck) {
      const availabilityStart = Date.now();
      const isAvailable = await waitForMockWalletAvailable(page, timeout);
      availabilityTime = Date.now() - availabilityStart;
      
      if (!isAvailable) {
        console.warn(`⚠️ Mock wallet setup completed but availability check failed (${availabilityTime}ms)`);
        return { ...setupResult, success: false, availabilityTime };
      }
    }
    
    const totalTime = Date.now() - startTime;
    console.log(`✅ Mock wallet setup completed successfully (total: ${totalTime}ms, setup: ${setupResult.setupTime}ms${availabilityTime ? `, availability: ${availabilityTime}ms` : ''})`);
    
    return { 
      ...setupResult, 
      totalTime,
      availabilityTime: availabilityTime || undefined 
    };
  } catch (error) {
    const totalTime = Date.now() - startTime;
    console.error(`❌ Mock wallet setup failed (${totalTime}ms):`, error);
    return { success: false, fromCache: false, setupTime: 0, totalTime, error };
  }
}

/**
 * Checks if mock wallet is currently available
 * @param {Page} page - Playwright page object
 * @returns {Promise<boolean>} - Returns true if wallet is available
 */
export async function isMockWalletAvailable(page) {
  try {
    return await page.evaluate(() => {
      return typeof window.ethereum !== 'undefined' && 
             window.ethereum.isMetaMask === true;
    });
  } catch (error) {
    return false;
  }
}

/**
 * Optimized mock wallet test helper that sets up wallet and navigates to page
 * @param {Page} page - Playwright page object
 * @param {string} url - URL to navigate to (default: '/')
 * @param {Object} options - Setup options
 * @param {number} options.waitAfterSetup - Wait time after wallet setup (default: 500, reduced from 1000)
 * @param {number} options.waitAfterNavigation - Wait time after navigation (default: 1000, reduced from 2000)
 * @param {boolean} options.skipAvailabilityCheck - Skip availability check for faster setup (default: true)
 * @param {boolean} options.forceReinstall - Force wallet reinstallation (default: false)
 * @returns {Promise<{success: boolean, fromCache: boolean, setupTime: number, navigationTime: number, totalTime: number}>}
 */
export async function setupMockWalletAndNavigate(page, url = '/', options = {}) {
  const {
    waitAfterSetup = 500, // Reduced from 1000ms
    waitAfterNavigation = 1000, // Reduced from 2000ms
    skipAvailabilityCheck = true, // Skip availability check by default for faster setup
    forceReinstall = false,
    ...setupOptions
  } = options;
  
  const startTime = Date.now();
  
  try {
    // Setup mock wallet with caching and performance tracking
    const setupResult = await setupMockWalletComplete(page, {
      ...setupOptions,
      skipAvailabilityCheck,
      forceReinstall
    });
    
    if (!setupResult.success) {
      console.warn(`⚠️ Mock wallet setup failed (${setupResult.setupTime}ms), continuing anyway`);
    }
    
    // Optimized wait after setup (only if wallet was actually installed)
    if (waitAfterSetup > 0 && !setupResult.fromCache) {
      await page.waitForTimeout(waitAfterSetup);
    }
    
    // Navigate to page
    const navStart = Date.now();
    await page.goto(url);
    const navigationTime = Date.now() - navStart;
    
    // Optimized wait after navigation
    if (waitAfterNavigation > 0) {
      await page.waitForTimeout(waitAfterNavigation);
    }
    
    const totalTime = Date.now() - startTime;
    
    console.log(`✅ Mock wallet setup and navigation to ${url} completed (total: ${totalTime}ms, setup: ${setupResult.setupTime}ms, nav: ${navigationTime}ms${setupResult.fromCache ? ', from cache' : ''})`);
    
    return {
      success: true,
      fromCache: setupResult.fromCache,
      setupTime: setupResult.setupTime,
      navigationTime,
      totalTime,
      setupResult
    };
  } catch (error) {
    const totalTime = Date.now() - startTime;
    console.error(`❌ Mock wallet setup and navigation failed (${totalTime}ms):`, error);
    return {
      success: false,
      fromCache: false,
      setupTime: 0,
      navigationTime: 0,
      totalTime,
      error
    };
  }
}

/**
 * Clears the wallet cache and forces fresh installation
 * Useful for test isolation or when switching configurations
 */
export function clearWalletCache() {
  WALLET_CACHE.isInitialized = false;
  WALLET_CACHE.initializationPromise = null;
  WALLET_CACHE.config = null;
  WALLET_CACHE.lastPage = null;
  console.log('🧹 Wallet cache cleared');
}

/**
 * Gets performance statistics for wallet setup operations
 * @returns {Object} Performance statistics
 */
export function getPerformanceStats() {
  const avgSetupTime = PERF_TRACKER.setupCount > 0 
    ? Math.round(PERF_TRACKER.totalSetupTime / PERF_TRACKER.setupCount)
    : 0;
    
  return {
    totalSetupTime: PERF_TRACKER.totalSetupTime,
    setupCount: PERF_TRACKER.setupCount,
    averageSetupTime: avgSetupTime,
    cacheHits: PERF_TRACKER.cacheHits,
    cacheMisses: PERF_TRACKER.cacheMisses,
    cacheHitRate: PERF_TRACKER.cacheHits + PERF_TRACKER.cacheMisses > 0 
      ? Math.round((PERF_TRACKER.cacheHits / (PERF_TRACKER.cacheHits + PERF_TRACKER.cacheMisses)) * 100)
      : 0,
    walletInitCount: WALLET_CACHE.initCount
  };
}

/**
 * Prints performance statistics to console
 */
export function logPerformanceStats() {
  const stats = getPerformanceStats();
  console.log('📊 Mock Wallet Performance Statistics:');
  console.log(`   Total setup time: ${stats.totalSetupTime}ms`);
  console.log(`   Setup operations: ${stats.setupCount}`);
  console.log(`   Average setup time: ${stats.averageSetupTime}ms`);
  console.log(`   Cache hits: ${stats.cacheHits}`);
  console.log(`   Cache misses: ${stats.cacheMisses}`);
  console.log(`   Cache hit rate: ${stats.cacheHitRate}%`);
  console.log(`   Wallet initializations: ${stats.walletInitCount}`);
}

/**
 * Fast setup for test suites - optimized for speed
 * Skips all availability checks and uses minimal wait times
 * @param {Page} page - Playwright page object
 * @param {string} url - URL to navigate to
 * @returns {Promise<{success: boolean, fromCache: boolean, totalTime: number}>}
 */
export async function setupMockWalletFast(page, url = '/') {
  return await setupMockWalletAndNavigate(page, url, {
    waitAfterSetup: 0,
    waitAfterNavigation: 500, // Minimal wait
    skipAvailabilityCheck: true,
    forceReinstall: false
  });
}

/**
 * Setup for test file initialization - runs once per test file
 * Uses longer timeouts for reliability but caches for subsequent tests
 * @param {Page} page - Playwright page object
 * @param {string} url - URL to navigate to
 * @returns {Promise<{success: boolean, fromCache: boolean, totalTime: number}>}
 */
export async function setupMockWalletForTestFile(page, url = '/') {
  return await setupMockWalletAndNavigate(page, url, {
    waitAfterSetup: 1000,
    waitAfterNavigation: 1500,
    skipAvailabilityCheck: false, // More thorough check for first setup
    forceReinstall: false
  });
}

/**
 * Default export with all utilities
 */
export default {
  setupMockWallet,
  waitForMockWalletAvailable,
  setupMockWalletComplete,
  isMockWalletAvailable,
  setupMockWalletAndNavigate,
  setupMockWalletFast,
  setupMockWalletForTestFile,
  clearWalletCache,
  getPerformanceStats,
  logPerformanceStats,
  DEFAULT_MOCK_WALLET_CONFIG
};