/**
 * Page Evaluate Utilities
 * Utilities for safely passing constants and configurations to page.evaluate() context
 */

import { TEST_URLS } from './test-constants';

/**
 * Configuration object that can be safely passed to page.evaluate()
 * Contains all necessary URLs and constants for browser context
 */
export interface PageEvaluateConfig {
  urls: {
    websocket: {
      gateway: string;
      frontend: string;
      nonExistent: string;
    };
    backend: {
      gateway: string;
      oneinch: string;
      blockchain: string;
      crypto: string;
      auth: string;
      aiEngine: string;
    };
    frontend: {
      localDev: string;
      staging: string;
      production: string;
    };
  };
  timeouts: {
    short: number;
    medium: number;
    long: number;
    extraLong: number;
  };
}

/**
 * Creates a configuration object that can be safely passed to page.evaluate()
 * This solves the problem where TEST_URLS is not available in browser context
 * 
 * @returns Configuration object with all necessary URLs and constants
 */
export function createPageEvaluateConfig(): PageEvaluateConfig {
  return {
    urls: {
      websocket: {
        gateway: TEST_URLS.WEBSOCKET.GATEWAY,
        frontend: TEST_URLS.WEBSOCKET.FRONTEND,
        nonExistent: TEST_URLS.WEBSOCKET.NON_EXISTENT,
      },
      backend: {
        gateway: TEST_URLS.BACKEND.GATEWAY,
        oneinch: TEST_URLS.BACKEND.ONEINCH,
        blockchain: TEST_URLS.BACKEND.BLOCKCHAIN,
        crypto: TEST_URLS.BACKEND.CRYPTO,
        auth: TEST_URLS.BACKEND.AUTH,
        aiEngine: TEST_URLS.BACKEND.AI_ENGINE,
      },
      frontend: {
        localDev: TEST_URLS.FRONTEND.LOCAL_DEV,
        staging: TEST_URLS.FRONTEND.STAGING,
        production: TEST_URLS.FRONTEND.PRODUCTION,
      },
    },
    timeouts: {
      short: 2000,
      medium: 5000,
      long: 10000,
      extraLong: 30000,
    },
  };
}

/**
 * Helper function to get WebSocket URL for page.evaluate()
 * 
 * @param type - Type of WebSocket connection ('gateway' | 'frontend' | 'nonExistent')
 * @returns WebSocket URL string
 */
export function getWebSocketUrl(type: 'gateway' | 'frontend' | 'nonExistent' = 'gateway'): string {
  const config = createPageEvaluateConfig();
  return config.urls.websocket[type];
}

/**
 * Helper function to get backend URL for page.evaluate()
 * 
 * @param service - Backend service name
 * @returns Backend service URL string
 */
export function getBackendUrl(service: keyof PageEvaluateConfig['urls']['backend'] = 'gateway'): string {
  const config = createPageEvaluateConfig();
  return config.urls.backend[service];
}

/**
 * Helper function to get frontend URL for page.evaluate()
 * 
 * @param env - Environment type
 * @returns Frontend URL string
 */
export function getFrontendUrl(env: keyof PageEvaluateConfig['urls']['frontend'] = 'localDev'): string {
  const config = createPageEvaluateConfig();
  return config.urls.frontend[env];
}

/**
 * Type-safe wrapper for page.evaluate() with configuration
 * Automatically injects the configuration as the first parameter
 * 
 * @param page - Playwright page object
 * @param evaluateFunction - Function to execute in browser context
 * @param additionalArgs - Additional arguments to pass to the function
 * @returns Promise with the result of page.evaluate()
 */
export async function evaluateWithConfig<T, Args extends any[]>(
  page: any,
  evaluateFunction: (config: PageEvaluateConfig, ...args: Args) => T | Promise<T>,
  ...additionalArgs: Args
): Promise<T> {
  const config = createPageEvaluateConfig();
  return page.evaluate(evaluateFunction, config, ...additionalArgs);
}

/**
 * Simplified wrapper for WebSocket-specific page.evaluate() calls
 * Automatically passes the WebSocket gateway URL as the first parameter
 * 
 * @param page - Playwright page object
 * @param evaluateFunction - Function to execute in browser context
 * @param additionalArgs - Additional arguments to pass to the function
 * @returns Promise with the result of page.evaluate()
 */
export async function evaluateWithWebSocket<T, Args extends any[]>(
  page: any,
  evaluateFunction: (wsUrl: string, ...args: Args) => T | Promise<T>,
  ...additionalArgs: Args
): Promise<T> {
  const wsUrl = getWebSocketUrl('gateway');
  return page.evaluate(evaluateFunction, wsUrl, ...additionalArgs);
}