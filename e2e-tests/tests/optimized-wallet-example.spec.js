/**
 * EXAMPLE: Optimized wallet setup for maximum performance
 * Demonstrates best practices for fast test execution
 */
import { test, expect } from '@playwright/test';
import { 
  setupMockWalletFast,
  setupMockWalletForTestFile,
  clearWalletCache,
  logPerformanceStats,
  getPerformanceStats
} from '../utils/mock-wallet-utility.js';

test.describe('Optimized Wallet Performance Example', () => {
  // Use setupMockWalletForTestFile for the first test in each file
  // This ensures reliable initialization with proper timeouts
  test.beforeAll(async () => {
    console.log('🚀 Starting optimized wallet test suite');
  });

  test.afterAll(async () => {
    // Log performance statistics at the end of test suite
    logPerformanceStats();
  });

  test('first test - uses thorough setup for reliability', async ({ page }) => {
    console.log('🔧 First test: Using setupMockWalletForTestFile for reliable initialization');
    
    const result = await setupMockWalletForTestFile(page, '/');
    
    expect(result.success).toBeTruthy();
    console.log(`⏱️ Setup completed in ${result.totalTime}ms (from cache: ${result.fromCache})`);
    
    // Your test logic here
    await expect(page).toHaveTitle(/KEMBridge/i);
  });

  test('second test - uses fast setup for speed', async ({ page }) => {
    console.log('⚡ Second test: Using setupMockWalletFast for maximum speed');
    
    const result = await setupMockWalletFast(page, '/');
    
    expect(result.success).toBeTruthy();
    console.log(`⏱️ Setup completed in ${result.totalTime}ms (from cache: ${result.fromCache})`);
    
    // Your test logic here
    await expect(page).toHaveTitle(/KEMBridge/i);
  });

  test('third test - also uses fast setup', async ({ page }) => {
    console.log('⚡ Third test: Using setupMockWalletFast for maximum speed');
    
    const result = await setupMockWalletFast(page, '/');
    
    expect(result.success).toBeTruthy();
    console.log(`⏱️ Setup completed in ${result.totalTime}ms (from cache: ${result.fromCache})`);
    
    // Your test logic here
    await expect(page).toHaveTitle(/KEMBridge/i);
  });

  test('performance validation test', async ({ page }) => {
    console.log('📊 Performance validation test');
    
    const startTime = Date.now();
    const result = await setupMockWalletFast(page, '/');
    const endTime = Date.now();
    
    expect(result.success).toBeTruthy();
    
    // Validate that cached setup is significantly faster
    if (result.fromCache) {
      expect(result.totalTime).toBeLessThan(1000); // Should be under 1 second from cache
      console.log(`✅ Cache performance validated: ${result.totalTime}ms`);
    }
    
    // Get current performance stats
    const stats = getPerformanceStats();
    console.log(`📈 Current cache hit rate: ${stats.cacheHitRate}%`);
    
    // Your test logic here
    await expect(page).toHaveTitle(/KEMBridge/i);
  });

  test('cache clearing test - demonstrates fresh installation', async ({ page }) => {
    console.log('🧹 Cache clearing test: Demonstrating fresh installation');
    
    // Clear cache to force fresh installation
    clearWalletCache();
    
    const result = await setupMockWalletFast(page, '/');
    
    expect(result.success).toBeTruthy();
    expect(result.fromCache).toBeFalsy(); // Should not be from cache
    console.log(`⏱️ Fresh setup completed in ${result.totalTime}ms`);
    
    // Your test logic here
    await expect(page).toHaveTitle(/KEMBridge/i);
  });
});

/**
 * PERFORMANCE TIPS:
 * 
 * 1. Use setupMockWalletForTestFile() for the first test in each file
 *    - More reliable with proper timeouts
 *    - Ensures wallet is properly initialized
 * 
 * 2. Use setupMockWalletFast() for subsequent tests
 *    - Minimal wait times
 *    - Leverages caching for speed
 * 
 * 3. Call logPerformanceStats() in test.afterAll() to monitor performance
 * 
 * 4. Use clearWalletCache() only when you need fresh installation
 *    - For testing different configurations
 *    - For test isolation when needed
 * 
 * 5. Expected performance improvements:
 *    - First setup: ~2-5 seconds (fresh installation)
 *    - Cached setups: ~100-500ms (significant speedup)
 *    - Cache hit rate should be >80% in typical test suites
 */