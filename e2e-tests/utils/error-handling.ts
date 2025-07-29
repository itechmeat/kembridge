/**
 * Comprehensive error handling utilities for E2E tests
 */
import { Page, test, expect } from '@playwright/test';
import { TestSelectors } from './selectors.js';
import { MonitoringData, ApiCall, NetworkError } from '../types/test-types.js';

export interface ErrorHandlingOptions {
  timeout?: number;
  retries?: number;
  throwOnError?: boolean;
  logErrors?: boolean;
}

export class ErrorHandler {
  private selectors: TestSelectors;
  private errors: string[] = [];
  private networkErrors: NetworkError[] = [];

  constructor(private page: Page) {
    this.selectors = new TestSelectors(page);
    this.setupErrorListeners();
  }

  private setupErrorListeners(): void {
    // Capture JavaScript errors
    this.page.on('pageerror', (error) => {
      this.errors.push(`Page Error: ${error.message}`);
      console.warn('🚨 Page Error:', error.message);
    });

    // Capture console errors
    this.page.on('console', (msg) => {
      if (msg.type() === 'error') {
        this.errors.push(`Console Error: ${msg.text()}`);
        console.warn('🚨 Console Error:', msg.text());
      }
    });

    // Capture network failures
    this.page.on('requestfailed', (request) => {
      const error: NetworkError = {
        url: request.url(),
        error: request.failure()?.errorText || 'Network request failed',
        timestamp: Date.now()
      };
      this.networkErrors.push(error);
      console.warn('🚨 Network Error:', error);
    });

    // Capture unhandled rejections
    this.page.on('dialog', async (dialog) => {
      console.warn('🚨 Unexpected Dialog:', dialog.message());
      await dialog.dismiss();
    });
  }

  /**
   * Execute an action with comprehensive error handling
   */
  async withErrorHandling<T>(
    action: () => Promise<T>,
    actionName: string,
    options: ErrorHandlingOptions = {}
  ): Promise<T> {
    const { timeout = 30000, retries = 2, throwOnError = true, logErrors = true } = options;
    
    let lastError: Error | null = null;
    
    for (let attempt = 0; attempt <= retries; attempt++) {
      try {
        if (attempt > 0) {
          console.log(`🔄 Retry attempt ${attempt} for: ${actionName}`);
          await this.page.waitForTimeout(1000 * attempt); // Exponential backoff
        }

        const result = await Promise.race([
          action(),
          this.createTimeoutPromise<T>(timeout, actionName)
        ]);

        if (attempt > 0) {
          console.log(`✅ ${actionName} succeeded on retry ${attempt}`);
        }

        return result;
      } catch (error) {
        lastError = error as Error;
        
        if (logErrors) {
          console.warn(`❌ ${actionName} failed (attempt ${attempt + 1}):`, error);
        }

        // Check for specific error types and handle accordingly
        if (this.isRetryableError(error)) {
          if (attempt < retries) {
            await this.handleRetryableError(error, actionName);
            continue;
          }
        }

        // Non-retryable error or exhausted retries
        if (throwOnError) {
          throw new Error(`${actionName} failed after ${attempt + 1} attempts: ${error}`);
        }
      }
    }

    if (throwOnError && lastError) {
      throw lastError;
    }

    return null as T;
  }

  private createTimeoutPromise<T>(timeout: number, actionName: string): Promise<T> {
    return new Promise((_, reject) => {
      setTimeout(() => {
        reject(new Error(`Timeout: ${actionName} exceeded ${timeout}ms`));
      }, timeout);
    });
  }

  private isRetryableError(error: unknown): boolean {
    const errorMessage = String(error).toLowerCase();
    
    const retryablePatterns = [
      'timeout',
      'network',
      'connection',
      'temporary',
      'rate limit',
      'service unavailable',
      'gateway timeout',
      'bad gateway'
    ];

    return retryablePatterns.some(pattern => errorMessage.includes(pattern));
  }

  private async handleRetryableError(error: unknown, actionName: string): Promise<void> {
    console.log(`🔄 Handling retryable error for ${actionName}`);
    
    // Check if we need to reconnect wallet
    if (String(error).toLowerCase().includes('wallet')) {
      await this.handleWalletError();
    }
    
    // Check if we need to refresh page
    if (String(error).toLowerCase().includes('page')) {
      await this.handlePageError();
    }
    
    // Check if we need to wait for network
    if (String(error).toLowerCase().includes('network')) {
      await this.handleNetworkError();
    }
  }

  private async handleWalletError(): Promise<void> {
    console.log('🔧 Attempting wallet reconnection...');
    
    try {
      // Try to reconnect wallet
      const connectButton = this.selectors.connectWalletButton;
      if (await connectButton.isVisible({ timeout: 2000 })) {
        await connectButton.click();
        await this.selectors.waitForWalletConnected();
      }
    } catch (error) {
      console.warn('⚠️ Wallet reconnection failed:', error);
    }
  }

  private async handlePageError(): Promise<void> {
    console.log('🔧 Attempting page recovery...');
    
    try {
      // Reload page and wait for it to be ready
      await this.page.reload({ waitUntil: 'networkidle' });
      await this.page.waitForLoadState('domcontentloaded');
    } catch (error) {
      console.warn('⚠️ Page recovery failed:', error);
    }
  }

  private async handleNetworkError(): Promise<void> {
    console.log('🔧 Waiting for network recovery...');
    
    // Wait for network to recover
    await this.page.waitForTimeout(3000);
    
    // Check if services are available
    try {
      await this.page.request.get('/health', { timeout: 5000 });
    } catch {
      console.warn('⚠️ Services still unavailable');
    }
  }

  /**
   * Validate API response and handle errors
   */
  async validateApiResponse(
    apiCall: () => Promise<Response>,
    expectedStatus = 200,
    actionName = 'API call'
  ): Promise<any> {
    return this.withErrorHandling(async () => {
      const response = await apiCall();
      
      if (!response.ok) {
        let errorDetails = `HTTP ${response.status}`;
        
        try {
          const errorBody = await response.text();
          errorDetails += `: ${errorBody}`;
        } catch {
          // Ignore if can't read response body
        }
        
        throw new Error(`${actionName} failed - ${errorDetails}`);
      }
      
      if (response.status !== expectedStatus) {
        console.warn(`⚠️ Unexpected status: expected ${expectedStatus}, got ${response.status}`);
      }
      
      return response.json();
    }, actionName);
  }

  /**
   * Handle form validation errors
   */
  async checkFormValidation(formName = 'Form'): Promise<string[]> {
    const errors: string[] = [];
    
    try {
      // Check for validation error messages
      const errorElements = await this.selectors.errorMessage.all();
      
      for (const element of errorElements) {
        if (await element.isVisible()) {
          const errorText = await element.textContent();
          if (errorText) {
            errors.push(errorText.trim());
          }
        }
      }
      
      // Check for invalid form fields
      const invalidFields = await this.page.locator('[aria-invalid="true"], .invalid, .error').all();
      
      for (const field of invalidFields) {
        if (await field.isVisible()) {
          const fieldName = await field.getAttribute('name') || 
                           await field.getAttribute('aria-label') || 
                           'Unknown field';
          errors.push(`Invalid field: ${fieldName}`);
        }
      }
      
      if (errors.length > 0) {
        console.warn(`⚠️ ${formName} validation errors:`, errors);
      }
      
    } catch (error) {
      console.warn(`⚠️ Error checking form validation:`, error);
    }
    
    return errors;
  }

  /**
   * Assert no JavaScript errors occurred
   */
  assertNoJavaScriptErrors(): void {
    if (this.errors.length > 0) {
      test.fail(`JavaScript errors detected: ${this.errors.join(', ')}`);
    }
  }

  /**
   * Assert no network errors occurred
   */
  assertNoNetworkErrors(): void {
    const criticalErrors = this.networkErrors.filter(error => 
      !error.url.includes('/metrics') && // Ignore metrics endpoints
      !error.url.includes('/favicon') && // Ignore favicon
      !error.error.includes('cancelled')  // Ignore cancelled requests
    );
    
    if (criticalErrors.length > 0) {
      test.fail(`Network errors detected: ${JSON.stringify(criticalErrors, null, 2)}`);
    }
  }

  /**
   * Get all captured errors
   */
  getAllErrors(): { jsErrors: string[], networkErrors: NetworkError[] } {
    return {
      jsErrors: [...this.errors],
      networkErrors: [...this.networkErrors]
    };
  }

  /**
   * Clear all captured errors
   */
  clearErrors(): void {
    this.errors = [];
    this.networkErrors = [];
  }

  /**
   * Wait for element with better error messages
   */
  async waitForElement(
    locator: any,
    options: { timeout?: number; state?: 'visible' | 'hidden' | 'attached' } = {}
  ): Promise<void> {
    const { timeout = 10000, state = 'visible' } = options;
    
    try {
      await locator.waitFor({ state, timeout });
    } catch (error) {
      const elementDescription = await this.getElementDescription(locator);
      throw new Error(`Element not found: ${elementDescription}. Original error: ${error}`);
    }
  }

  private async getElementDescription(locator: any): Promise<string> {
    try {
      // Try to get a meaningful description of the locator
      const locatorStr = locator.toString();
      return locatorStr;
    } catch {
      return 'Unknown element';
    }
  }

  /**
   * Validate transaction state with error handling
   */
  async validateTransactionState(
    transactionId: string,
    expectedStatus: string,
    timeout = 30000
  ): Promise<void> {
    return this.withErrorHandling(async () => {
      await this.page.waitForFunction(
        ({ txId, status }) => {
          const statusElement = document.querySelector(`[data-transaction-id="${txId}"] [data-testid="status"]`);
          return statusElement?.textContent?.toLowerCase().includes(status.toLowerCase());
        },
        { transactionId, expectedStatus },
        { timeout }
      );
    }, `Transaction ${transactionId} to reach status ${expectedStatus}`);
  }
}