# Mock Wallet Performance Optimization Guide

## Overview

This guide explains how to optimize mock wallet setup in E2E tests to significantly reduce test execution time. The optimized `mock-wallet-utility.js` includes caching, performance tracking, and smart reuse strategies.

## Performance Improvements

### Before Optimization
- Each test: 3-5 seconds wallet setup
- No caching or reuse
- Fixed long timeouts
- Total test suite time: Very slow

### After Optimization
- First test: 2-5 seconds (fresh installation)
- Subsequent tests: 100-500ms (from cache)
- Smart timeout management
- Expected 60-80% time reduction

## Usage Patterns

### 1. Optimized Test File Structure

```javascript
import { 
  setupMockWalletFast,
  setupMockWalletForTestFile,
  logPerformanceStats 
} from '../utils/mock-wallet-utility.js';

test.describe('Your Test Suite', () => {
  let isFirstTest = true;

  test.afterAll(async () => {
    logPerformanceStats(); // Track performance
  });

  test.beforeEach(async ({ page }) => {
    // Thorough setup for first test, fast for others
    const result = isFirstTest 
      ? await setupMockWalletForTestFile(page, '/')
      : await setupMockWalletFast(page, '/');
    
    isFirstTest = false;
    console.log(`Setup: ${result.totalTime}ms (cached: ${result.fromCache})`);
  });
});
```

### 2. Available Setup Functions

#### `setupMockWalletForTestFile(page, url)`
- **Use for**: First test in each test file
- **Features**: Reliable initialization, proper timeouts
- **Performance**: 2-5 seconds (fresh installation)

#### `setupMockWalletFast(page, url)`
- **Use for**: Subsequent tests in the same file
- **Features**: Minimal timeouts, leverages caching
- **Performance**: 100-500ms (from cache)

#### `setupMockWalletAndNavigate(page, url, options)`
- **Use for**: Custom configurations
- **Features**: Full control over timeouts and behavior
- **Performance**: Configurable

### 3. Cache Management

#### Automatic Caching
```javascript
// Wallet is automatically cached after first installation
// Subsequent calls check if wallet exists before reinstalling
const result = await setupMockWalletFast(page, '/');
console.log(result.fromCache); // true if reused
```

#### Manual Cache Control
```javascript
import { clearWalletCache } from '../utils/mock-wallet-utility.js';

// Force fresh installation (use sparingly)
clearWalletCache();
const result = await setupMockWalletFast(page, '/');
console.log(result.fromCache); // false - fresh installation
```

### 4. Performance Monitoring

#### Get Statistics
```javascript
import { getPerformanceStats } from '../utils/mock-wallet-utility.js';

const stats = getPerformanceStats();
console.log(`Cache hit rate: ${stats.cacheHitRate}%`);
console.log(`Average setup time: ${stats.averageSetupTime}ms`);
```

#### Log Statistics
```javascript
import { logPerformanceStats } from '../utils/mock-wallet-utility.js';

// In test.afterAll()
logPerformanceStats();
// Output:
// 📊 Mock Wallet Performance Statistics:
//    Total setup time: 2500ms
//    Setup operations: 5
//    Average setup time: 500ms
//    Cache hits: 4
//    Cache misses: 1
//    Cache hit rate: 80%
//    Wallet initializations: 1
```

## Migration Guide

### Step 1: Update Imports
```javascript
// Before
import { setupMockWalletAndNavigate } from '../utils/mock-wallet-utility.js';

// After
import { 
  setupMockWalletFast,
  setupMockWalletForTestFile,
  logPerformanceStats 
} from '../utils/mock-wallet-utility.js';
```

### Step 2: Update beforeEach Hook
```javascript
// Before
test.beforeEach(async ({ page }) => {
  await setupMockWalletAndNavigate(page, '/');
  await page.waitForTimeout(2000);
});

// After
test.beforeEach(async ({ page }) => {
  const result = isFirstTest 
    ? await setupMockWalletForTestFile(page, '/')
    : await setupMockWalletFast(page, '/');
  isFirstTest = false;
});
```

### Step 3: Add Performance Tracking
```javascript
test.afterAll(async () => {
  logPerformanceStats();
});
```

## Best Practices

### 1. Test File Organization
- Use `setupMockWalletForTestFile()` for the first test
- Use `setupMockWalletFast()` for subsequent tests
- Add performance logging in `test.afterAll()`

### 2. Cache Management
- Let automatic caching work (don't clear cache unnecessarily)
- Only use `clearWalletCache()` when testing different configurations
- Monitor cache hit rates (target >80%)

### 3. Timeout Optimization
- `setupMockWalletFast()` uses minimal timeouts (500ms)
- `setupMockWalletForTestFile()` uses reliable timeouts (1000-1500ms)
- Customize timeouts only when needed

### 4. Performance Monitoring
- Always log performance stats in test suites
- Monitor average setup times
- Track cache hit rates
- Investigate if cache hit rate <70%

## Expected Performance Gains

### Typical Test Suite (5 tests)

#### Before Optimization
```
Test 1: 4000ms setup
Test 2: 4000ms setup
Test 3: 4000ms setup
Test 4: 4000ms setup
Test 5: 4000ms setup
Total: 20000ms (20 seconds)
```

#### After Optimization
```
Test 1: 3000ms setup (fresh)
Test 2: 300ms setup (cached)
Test 3: 300ms setup (cached)
Test 4: 300ms setup (cached)
Test 5: 300ms setup (cached)
Total: 4200ms (4.2 seconds)
```

**Result: 79% time reduction**

## Troubleshooting

### Low Cache Hit Rate
- Check if `clearWalletCache()` is called too often
- Verify test isolation isn't forcing fresh installations
- Monitor for page context changes

### Inconsistent Performance
- Use `setupMockWalletForTestFile()` for first test reliability
- Check network conditions affecting initial installation
- Monitor system resources during test execution

### Cache Not Working
- Verify wallet detection logic in `isWalletAlreadyAvailable()`
- Check for page navigation clearing wallet state
- Ensure consistent page contexts

## Configuration Options

### Custom Setup Options
```javascript
const result = await setupMockWalletAndNavigate(page, '/', {
  waitAfterSetup: 0,        // Skip wait after setup
  waitAfterNavigation: 500, // Minimal navigation wait
  skipAvailabilityCheck: true, // Skip availability check
  forceReinstall: false     // Use caching
});
```

### Performance Tuning
```javascript
// Ultra-fast setup (use with caution)
const result = await setupMockWalletAndNavigate(page, '/', {
  waitAfterSetup: 0,
  waitAfterNavigation: 0,
  skipAvailabilityCheck: true,
  forceReinstall: false
});

// Reliable setup (for critical tests)
const result = await setupMockWalletAndNavigate(page, '/', {
  waitAfterSetup: 1000,
  waitAfterNavigation: 2000,
  skipAvailabilityCheck: false,
  forceReinstall: false
});
```