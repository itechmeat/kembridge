/**
 * Cached Authentication Utility - TypeScript
 * Optimized reusable utility for handling authentication state in tests
 * Features global caching and reuse to minimize initialization time
 * Based on mock-wallet-utility.js pattern but for authentication flows
 */

import { Page } from '@playwright/test';

/**
 * Authentication state cache
 */
interface AuthCache {
  pageStates: Map<string, AuthPageState>;
  lastAccessTime: number;
  hitCount: number;
  missCount: number;
}

interface AuthPageState {
  hasEthereumButton: boolean;
  hasNearButton: boolean;
  hasAuthMessage: boolean;
  isFormAccessible: boolean;
  timestamp: number;
  pageUrl: string;
}

/**
 * Performance tracking
 */
interface PerfTracker {
  totalCheckTime: number;
  checkCount: number;
  cacheHits: number;
  cacheMisses: number;
  avgCheckTime: number;
}

/**
 * Global cache for authentication states
 */
const AUTH_CACHE: AuthCache = {
  pageStates: new Map(),
  lastAccessTime: 0,
  hitCount: 0,
  missCount: 0
};

/**
 * Performance tracking
 */
const PERF_TRACKER: PerfTracker = {
  totalCheckTime: 0,
  checkCount: 0,
  cacheHits: 0,
  cacheMisses: 0,
  avgCheckTime: 0
};

/**
 * Cache expiration time (5 seconds)
 */
const CACHE_EXPIRATION_MS = 5000;

/**
 * Fast authentication state check with caching
 * @param page - Playwright page object
 * @param options - Check options
 * @returns Promise<AuthPageState>
 */
export async function quickAuthStateCheck(
  page: Page, 
  options: { useCache?: boolean; cacheKey?: string } = {}
): Promise<AuthPageState> {
  const startTime = Date.now();
  const { useCache = true, cacheKey } = options;
  
  const url = await page.url();
  const key = cacheKey || url;
  
  // Check cache first
  if (useCache && AUTH_CACHE.pageStates.has(key)) {
    const cached = AUTH_CACHE.pageStates.get(key)!;
    const age = Date.now() - cached.timestamp;
    
    if (age < CACHE_EXPIRATION_MS) {
      PERF_TRACKER.cacheHits++;
      AUTH_CACHE.hitCount++;
      const checkTime = Date.now() - startTime;
      console.log(`⚡ Auth state from cache (${checkTime}ms, age: ${age}ms)`);
      return cached;
    }
  }
  
  // Perform fresh check
  PERF_TRACKER.cacheMisses++;
  AUTH_CACHE.missCount++;
  
  const state: AuthPageState = {
    hasEthereumButton: await page.isVisible('[data-testid="ethereum-wallet-button"]', { timeout: 1000 }),
    hasNearButton: await page.isVisible('[data-testid="near-wallet-button"]', { timeout: 1000 }),
    hasAuthMessage: await page.isVisible('text=Sign in with your wallet', { timeout: 1000 }),
    isFormAccessible: await page.isVisible('.swap-form', { timeout: 1000 }),
    timestamp: Date.now(),
    pageUrl: url
  };
  
  // Cache the result
  if (useCache) {
    AUTH_CACHE.pageStates.set(key, state);
    AUTH_CACHE.lastAccessTime = Date.now();
  }
  
  const checkTime = Date.now() - startTime;
  PERF_TRACKER.totalCheckTime += checkTime;
  PERF_TRACKER.checkCount++;
  PERF_TRACKER.avgCheckTime = PERF_TRACKER.totalCheckTime / PERF_TRACKER.checkCount;
  
  console.log(`✅ Auth state checked (${checkTime}ms) - ETH: ${state.hasEthereumButton}, NEAR: ${state.hasNearButton}, Form: ${state.isFormAccessible}`);
  
  return state;
}

/**
 * Ultra-fast authentication check for tests
 * Uses aggressive caching and minimal timeouts
 */
export async function ultraFastAuthCheck(page: Page): Promise<boolean> {
  const startTime = Date.now();
  
  try {
    const state = await quickAuthStateCheck(page, { useCache: true });
    const authRequired = !state.isFormAccessible && (state.hasEthereumButton || state.hasNearButton || state.hasAuthMessage);
    
    const checkTime = Date.now() - startTime;
    console.log(`⚡ Ultra-fast auth check (${checkTime}ms): ${authRequired ? 'Auth required' : 'Form accessible'}`);
    
    return authRequired;
  } catch (error) {
    const checkTime = Date.now() - startTime;
    console.warn(`⚠️ Ultra-fast auth check failed (${checkTime}ms), assuming auth required`);
    return true;
  }
}

/**
 * Cached page navigation with auth state preloading
 */
export async function navigateWithAuthCache(
  page: Page, 
  url: string, 
  options: { waitUntil?: 'domcontentloaded' | 'load' | 'networkidle'; waitAfter?: number } = {}
): Promise<{ navigationTime: number; authState: AuthPageState }> {
  const { waitUntil = 'domcontentloaded', waitAfter = 500 } = options;
  const startTime = Date.now();
  
  try {
    // Navigate
    await page.goto(url, { waitUntil });
    
    // Minimal wait
    if (waitAfter > 0) {
      await page.waitForTimeout(waitAfter);
    }
    
    // Check auth state with caching
    const authState = await quickAuthStateCheck(page, { cacheKey: url });
    
    const navigationTime = Date.now() - startTime;
    console.log(`🚀 Navigate with auth cache to ${url} (${navigationTime}ms)`);
    
    return { navigationTime, authState };
  } catch (error) {
    const navigationTime = Date.now() - startTime;
    console.error(`❌ Navigate with auth cache failed (${navigationTime}ms):`, error);
    throw error;
  }
}

/**
 * Optimized beforeEach setup for auth tests
 */
export async function setupAuthTestFast(
  page: Page, 
  baseUrl: string = 'http://localhost:4100/'
): Promise<{ setupTime: number; authRequired: boolean; fromCache: boolean }> {
  const startTime = Date.now();
  
  try {
    // Navigate with auth cache
    const { authState } = await navigateWithAuthCache(page, baseUrl, {
      waitUntil: 'domcontentloaded',
      waitAfter: 300 // Reduced wait time
    });
    
    const authRequired = !authState.isFormAccessible && 
      (authState.hasEthereumButton || authState.hasNearButton || authState.hasAuthMessage);
    
    const setupTime = Date.now() - startTime;
    const fromCache = (Date.now() - authState.timestamp) < CACHE_EXPIRATION_MS;
    
    console.log(`⚡ Fast auth test setup (${setupTime}ms) - Auth required: ${authRequired}${fromCache ? ' (cached)' : ''}`);
    
    return { setupTime, authRequired, fromCache };
  } catch (error) {
    const setupTime = Date.now() - startTime;
    console.error(`❌ Fast auth test setup failed (${setupTime}ms):`, error);
    return { setupTime, authRequired: true, fromCache: false };
  }
}

/**
 * Batch auth state checks for multiple elements
 * More efficient than checking elements individually
 */
export async function batchAuthElementCheck(
  page: Page,
  elements: string[]
): Promise<Record<string, boolean>> {
  const startTime = Date.now();
  const result: Record<string, boolean> = {};
  
  try {
    // Check all elements in parallel
    const checks = elements.map(async (selector) => {
      const isVisible = await page.isVisible(selector, { timeout: 500 });
      return { selector, isVisible };
    });
    
    const results = await Promise.all(checks);
    
    results.forEach(({ selector, isVisible }) => {
      result[selector] = isVisible;
    });
    
    const checkTime = Date.now() - startTime;
    console.log(`📊 Batch auth element check (${checkTime}ms) - ${elements.length} elements`);
    
    return result;
  } catch (error) {
    const checkTime = Date.now() - startTime;
    console.error(`❌ Batch auth element check failed (${checkTime}ms):`, error);
    
    // Return false for all elements on error
    elements.forEach(selector => {
      result[selector] = false;
    });
    
    return result;
  }
}

/**
 * Clear auth cache
 */
export function clearAuthCache(): void {
  AUTH_CACHE.pageStates.clear();
  AUTH_CACHE.lastAccessTime = 0;
  console.log('🧹 Auth cache cleared');
}

/**
 * Get performance statistics
 */
export function getAuthPerformanceStats() {
  const cacheTotal = AUTH_CACHE.hitCount + AUTH_CACHE.missCount;
  const cacheHitRate = cacheTotal > 0 ? Math.round((AUTH_CACHE.hitCount / cacheTotal) * 100) : 0;
  
  return {
    totalCheckTime: PERF_TRACKER.totalCheckTime,
    checkCount: PERF_TRACKER.checkCount,
    averageCheckTime: Math.round(PERF_TRACKER.avgCheckTime),
    cacheHits: PERF_TRACKER.cacheHits,
    cacheMisses: PERF_TRACKER.cacheMisses,
    cacheHitRate,
    cacheSize: AUTH_CACHE.pageStates.size,
    lastAccessTime: AUTH_CACHE.lastAccessTime
  };
}

/**
 * Log performance statistics
 */
export function logAuthPerformanceStats(): void {
  const stats = getAuthPerformanceStats();
  console.log('📊 Auth Cache Performance Statistics:');
  console.log(`   Total check time: ${stats.totalCheckTime}ms`);
  console.log(`   Auth checks: ${stats.checkCount}`);
  console.log(`   Average check time: ${stats.averageCheckTime}ms`);
  console.log(`   Cache hits: ${stats.cacheHits}`);
  console.log(`   Cache misses: ${stats.cacheMisses}`);
  console.log(`   Cache hit rate: ${stats.cacheHitRate}%`);
  console.log(`   Cache size: ${stats.cacheSize} entries`);
}

/**
 * Pre-warm cache for common URLs
 */
export async function preWarmAuthCache(page: Page, urls: string[]): Promise<void> {
  console.log(`🔥 Pre-warming auth cache for ${urls.length} URLs...`);
  
  const startTime = Date.now();
  
  for (const url of urls) {
    try {
      await navigateWithAuthCache(page, url, { waitAfter: 200 });
    } catch (error) {
      console.warn(`⚠️ Failed to pre-warm cache for ${url}:`, error);
    }
  }
  
  const totalTime = Date.now() - startTime;
  console.log(`✅ Auth cache pre-warmed in ${totalTime}ms`);
}

/**
 * Default export with all utilities
 */
export default {
  quickAuthStateCheck,
  ultraFastAuthCheck,
  navigateWithAuthCache,
  setupAuthTestFast,
  batchAuthElementCheck,
  clearAuthCache,
  getAuthPerformanceStats,
  logAuthPerformanceStats,
  preWarmAuthCache
};