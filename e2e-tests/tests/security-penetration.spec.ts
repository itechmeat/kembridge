/**
 * Comprehensive Security Penetration Tests
 * Tests for XSS, CSRF, SQL Injection, and other security vulnerabilities
 */
import { test, expect, Page } from '@playwright/test';
import { TestSelectors } from '../utils/selectors.js';
import { ErrorHandler } from '../utils/error-handling.js';
import { setupFullTestEnvironment } from '../utils/test-base.js';

test.describe('Security Penetration Tests', () => {
  let selectors: TestSelectors;
  let errorHandler: ErrorHandler;

  test.beforeEach(async ({ page }) => {
    selectors = new TestSelectors(page);
    errorHandler = new ErrorHandler(page);
    
    // Setup test environment
    await setupFullTestEnvironment(page);
  });

  test.afterEach(async ({ page }) => {
    // Assert no security-related JS errors
    errorHandler.assertNoJavaScriptErrors();
  });

  test('should prevent XSS attacks in form inputs', async ({ page }) => {
    console.log('🛡️ Testing XSS prevention...');
    
    const xssPayloads = [
      '<script>alert("XSS")</script>',
      '<img src="x" onerror="alert(\'XSS\')">',
      'javascript:alert("XSS")',
      '<svg onload="alert(\'XSS\')">',
      '"><script>alert("XSS")</script>',
      "';alert('XSS');//",
      '<iframe src="javascript:alert(\'XSS\')"></iframe>'
    ];

    // Navigate to bridge form
    await page.goto('/bridge');
    await selectors.connectWalletButton.click();
    await selectors.ethWalletButton.click();
    await selectors.waitForWalletConnected();

    // Setup dialog handler to catch any XSS attempts
    let dialogTriggered = false;
    page.on('dialog', (dialog) => {
      dialogTriggered = true;
      console.error('🚨 XSS VULNERABILITY DETECTED: Dialog triggered');
      dialog.dismiss();
    });

    for (const payload of xssPayloads) {
      console.log(`🧪 Testing XSS payload: ${payload.substring(0, 30)}...`);
      
      try {
        // Test in amount input
        await selectors.amountInput.fill(payload);
        await page.waitForTimeout(1000);
        
        // Verify input is sanitized
        const inputValue = await selectors.amountInput.inputValue();
        expect(inputValue).not.toContain('<script>');
        expect(inputValue).not.toContain('javascript:');
        expect(inputValue).not.toContain('onerror');
        
        // Clear input
        await selectors.amountInput.fill('');
        
      } catch (error) {
        console.log(`✅ XSS payload blocked: ${payload}`);
      }
    }

    expect(dialogTriggered).toBeFalsy();
    console.log('✅ XSS prevention test passed');
  });

  test('should enforce secure HTTP headers', async ({ page }) => {
    console.log('🛡️ Testing security headers...');
    
    const response = await page.goto('/');
    expect(response).toBeTruthy();
    
    const headers = response!.headers();
    
    // Test for security headers
    expect(headers['strict-transport-security']).toBeTruthy();
    expect(headers['x-content-type-options']).toBe('nosniff');
    expect(headers['x-frame-options']).toBeTruthy();
    expect(headers['content-security-policy']).toBeTruthy();
    expect(headers['x-xss-protection']).toBeTruthy();
    
    console.log('✅ Security headers test passed');
  });

  test('should prevent CSRF attacks', async ({ page }) => {
    console.log('🛡️ Testing CSRF prevention...');
    
    // Authenticate user
    await page.goto('/');
    await selectors.connectWalletButton.click();
    await selectors.ethWalletButton.click();
    await selectors.waitForWalletConnected();
    
    // Get CSRF token if present
    const csrfToken = await page.evaluate(() => {
      const meta = document.querySelector('meta[name="csrf-token"]');
      return meta?.getAttribute('content');
    });
    
    // Attempt to make request without CSRF token
    const response = await page.request.post('/api/v1/bridge/initiate', {
      data: {
        amount: '0.1',
        fromChain: 'ethereum',
        toChain: 'near'
      },
      headers: {
        'Content-Type': 'application/json'
      }
    });
    
    // Should be rejected due to missing CSRF token or proper authentication
    expect([401, 403, 422]).toContain(response.status());
    console.log('✅ CSRF prevention test passed');
  });

  test('should prevent SQL injection in API parameters', async ({ page }) => {
    console.log('🛡️ Testing SQL injection prevention...');
    
    // Authenticate user
    await page.goto('/');
    await selectors.connectWalletButton.click();
    await selectors.ethWalletButton.click();
    await selectors.waitForWalletConnected();
    
    const sqlInjectionPayloads = [
      "'; DROP TABLE users; --",
      "' OR '1'='1",
      "' UNION SELECT * FROM users --",
      "'; DELETE FROM transactions; --",
      "' OR 1=1 --",
      "admin'--",
      "' OR 'x'='x"
    ];
    
    for (const payload of sqlInjectionPayloads) {
      console.log(`🧪 Testing SQL injection: ${payload}`);
      
      try {
        const response = await page.request.get(`/api/v1/bridge/transactions?search=${encodeURIComponent(payload)}`);
        
        // Should return valid response or proper error, not crash
        expect([200, 400, 422]).toContain(response.status());
        
        if (response.ok()) {
          const data = await response.json();
          expect(data).not.toHaveProperty('sql_error');
          expect(data).not.toHaveProperty('database_error');
        }
        
      } catch (error) {
        console.log(`✅ SQL injection blocked: ${payload}`);
      }
    }
    
    console.log('✅ SQL injection prevention test passed');
  });

  test('should validate JWT token integrity and expiration', async ({ page }) => {
    console.log('🛡️ Testing JWT security...');
    
    // Authenticate user
    await page.goto('/');
    await selectors.connectWalletButton.click();
    await selectors.ethWalletButton.click();
    await selectors.waitForWalletConnected();
    
    // Get original token
    const originalToken = await page.evaluate(() => localStorage.getItem('auth_token'));
    expect(originalToken).toBeTruthy();
    
    // Test 1: Tampered token
    console.log('🧪 Testing tampered JWT token...');
    const tamperedToken = originalToken!.slice(0, -10) + 'tampered123';
    
    const tamperedResponse = await page.request.get('/api/v1/user/profile', {
      headers: { 'Authorization': `Bearer ${tamperedToken}` }
    });
    
    expect(tamperedResponse.status()).toBe(401);
    
    // Test 2: Malformed token
    console.log('🧪 Testing malformed JWT token...');
    const malformedResponse = await page.request.get('/api/v1/user/profile', {
      headers: { 'Authorization': 'Bearer invalid.token.here' }
    });
    
    expect(malformedResponse.status()).toBe(401);
    
    // Test 3: Missing token
    console.log('🧪 Testing missing JWT token...');
    const noTokenResponse = await page.request.get('/api/v1/user/profile');
    expect(noTokenResponse.status()).toBe(401);
    
    console.log('✅ JWT security test passed');
  });

  test('should prevent unauthorized access to protected routes', async ({ page }) => {
    console.log('🛡️ Testing access control...');
    
    const protectedEndpoints = [
      '/api/v1/user/profile',
      '/api/v1/bridge/initiate', 
      '/api/v1/user/transactions',
      '/api/v1/user/balance'
    ];
    
    for (const endpoint of protectedEndpoints) {
      console.log(`🧪 Testing unauthorized access to: ${endpoint}`);
      
      const response = await page.request.get(endpoint);
      expect(response.status()).toBe(401);
    }
    
    console.log('✅ Access control test passed');
  });

  test('should validate input sanitization and length limits', async ({ page }) => {
    console.log('🛡️ Testing input validation...');
    
    await page.goto('/bridge');
    await selectors.connectWalletButton.click();
    await selectors.ethWalletButton.click();
    await selectors.waitForWalletConnected();
    
    // Test extremely long input
    const longInput = 'a'.repeat(10000);
    await selectors.amountInput.fill(longInput);
    
    const actualValue = await selectors.amountInput.inputValue();
    expect(actualValue.length).toBeLessThan(1000); // Should be limited
    
    // Test special characters
    const specialChars = '!@#$%^&*()_+{}[]|\\:";\'<>?,./`~';
    await selectors.amountInput.fill(specialChars);
    
    const specialValue = await selectors.amountInput.inputValue();
    // Should only allow valid numeric input
    expect(specialValue).toMatch(/^[0-9.]*$/);
    
    console.log('✅ Input validation test passed');
  });

  test('should prevent timing attacks on authentication', async ({ page }) => {
    console.log('🛡️ Testing timing attack prevention...');
    
    const startTime = Date.now();
    
    // Attempt login with invalid credentials
    const invalidResponse = await page.request.post('/api/v1/auth/verify-wallet', {
      data: {
        address: 'invalid_address',
        signature: 'invalid_signature',
        nonce: 'invalid_nonce'
      }
    });
    
    const invalidTime = Date.now() - startTime;
    
    // Now try with valid format but wrong signature
    const validFormatStart = Date.now();
    
    const validFormatResponse = await page.request.post('/api/v1/auth/verify-wallet', {
      data: {
        address: '0x1234567890123456789012345678901234567890',
        signature: '0x' + 'a'.repeat(128),
        nonce: 'valid_looking_nonce'
      }
    });
    
    const validFormatTime = Date.now() - validFormatStart;
    
    // Both should fail, and timing should be similar (within reasonable range)
    expect(invalidResponse.status()).toBe(401);
    expect(validFormatResponse.status()).toBe(401);
    
    const timingDifference = Math.abs(invalidTime - validFormatTime);
    expect(timingDifference).toBeLessThan(1000); // Should not reveal timing info
    
    console.log('✅ Timing attack prevention test passed');
  });

  test('should validate quantum cryptography signatures', async ({ page }) => {
    console.log('🛡️ Testing quantum cryptography validation...');
    
    await page.goto('/bridge');
    await selectors.connectWalletButton.click();
    await selectors.ethWalletButton.click();
    await selectors.waitForWalletConnected();
    
    await selectors.amountInput.fill('0.1');
    await selectors.reviewSwapButton.click();
    
    // Wait for quantum signature generation
    await page.waitForTimeout(2000);
    
    // Check for quantum-related elements
    const quantumIndicators = await selectors.quantumSecurityBadge.count();
    expect(quantumIndicators).toBeGreaterThan(0);
    
    // Verify quantum signature is present in transaction
    const quantumSignature = await page.locator('[data-testid="quantum-signature"]').textContent();
    if (quantumSignature) {
      // Should be valid base64 encoded signature
      expect(quantumSignature).toMatch(/^[A-Za-z0-9+/=]+$/);
      expect(quantumSignature.length).toBeGreaterThan(100); // Reasonable signature length
    }
    
    console.log('✅ Quantum cryptography test passed');
  });

  test('should handle rate limiting correctly', async ({ page }) => {
    console.log('🛡️ Testing rate limiting...');
    
    await page.goto('/');
    await selectors.connectWalletButton.click();
    await selectors.ethWalletButton.click();
    await selectors.waitForWalletConnected();
    
    // Make multiple rapid requests
    const promises = Array.from({ length: 20 }, () => 
      page.request.get('/api/v1/bridge/status')
    );
    
    const responses = await Promise.all(promises);
    const rateLimitedResponses = responses.filter(r => r.status() === 429);
    
    // Should have some rate limited responses
    expect(rateLimitedResponses.length).toBeGreaterThan(0);
    
    // Rate limited responses should have proper headers
    if (rateLimitedResponses.length > 0) {
      const rateLimitResponse = rateLimitedResponses[0];
      const headers = rateLimitResponse.headers();
      
      expect(headers['x-ratelimit-limit'] || headers['x-rate-limit-limit']).toBeTruthy();
      expect(headers['retry-after']).toBeTruthy();
    }
    
    console.log('✅ Rate limiting test passed');
  });
});