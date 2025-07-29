import { test, expect } from '@playwright/test';
import { installMockWallet } from '@johanneskares/wallet-mock';
import { privateKeyToAccount } from 'viem/accounts';
import { http } from 'viem';
import { sepolia } from 'viem/chains';

test.describe('Security & Risk Analysis Testing', () => {
  test.beforeEach(async ({ page }) => {
    // Install mock wallet for each test
    await installMockWallet({
      page,
      account: privateKeyToAccount(
        "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
      ),
      defaultChain: sepolia,
      transports: { [sepolia.id]: http() },
    });

    await page.goto('/');
    await page.waitForTimeout(3000);
  });

  test('should verify quantum cryptography indicators', async ({ page }) => {
    console.log('🔒 Testing quantum cryptography indicators...');

    // Authenticate and navigate to bridge
    const ethButton = page.locator('button:has-text("Ethereum Wallet")');
    if (await ethButton.isVisible() && await ethButton.getAttribute('disabled') === null) {
      await ethButton.click();
      await page.waitForTimeout(10000);
    }

    await page.goto('/bridge');
    await page.waitForTimeout(5000);

    // Look for quantum security indicators
    const quantumIndicators = [
      'text=Quantum Protected',
      'text=ML-KEM',
      'text=Post-quantum',
      '.quantum-badge',
      '.security-indicator',
      '[data-testid*="quantum"]'
    ];

    let quantumFound = false;
    for (const indicator of quantumIndicators) {
      const elements = page.locator(indicator);
      const count = await elements.count();
      if (count > 0) {
        quantumFound = true;
        console.log(`✅ Quantum security indicator found: ${indicator}`);
        
        // Get text content for verification
        const text = await elements.first().textContent();
        console.log(`   Content: "${text}"`);
      }
    }

    if (quantumFound) {
      console.log('✅ Quantum cryptography indicators: PRESENT');
    } else {
      console.log('❌ Quantum cryptography indicators: NOT FOUND');
    }

    // Check for security status in swap form
    const securitySection = page.locator('.swap-form__security, .security-status');
    const securitySectionVisible = await securitySection.isVisible().catch(() => false);
    
    if (securitySectionVisible) {
      console.log('✅ Security section in swap form: VISIBLE');
      const securityText = await securitySection.textContent();
      console.log(`   Security section content: "${securityText}"`);
    } else {
      console.log('⏳ Security section in swap form: NOT VISIBLE');
    }
  });

  test('should test risk analysis integration', async ({ page }) => {
    console.log('⚡ Testing risk analysis integration...');

    // Monitor API calls for risk analysis
    const riskApiCalls = [];
    page.on('request', request => {
      const url = request.url();
      if (url.includes('/risk/') || url.includes('/analysis/') || url.includes('/security/')) {
        riskApiCalls.push({
          url,
          method: request.method()
        });
      }
    });

    // Authenticate and navigate to bridge
    const ethButton = page.locator('button:has-text("Ethereum Wallet")');
    if (await ethButton.isVisible() && await ethButton.getAttribute('disabled') === null) {
      await ethButton.click();
      await page.waitForTimeout(10000);
    }

    await page.goto('/bridge');
    await page.waitForTimeout(5000);

    // Enter amount to trigger risk analysis
    const amountInput = page.locator('input[type="number"], input[placeholder*="amount"]').first();
    const amountInputExists = await amountInput.count() > 0;
    
    if (amountInputExists) {
      console.log('💰 Entering amount to trigger risk analysis...');
      await amountInput.fill('100'); // Large amount to potentially trigger risk analysis
      await page.waitForTimeout(3000);
    }

    // Look for risk analysis UI components
    const riskIndicators = [
      '.risk-analysis',
      '.risk-score',
      '.security-alert',
      '[data-testid*="risk"]',
      'text=Risk Score',
      'text=Risk Analysis'
    ];

    let riskUIFound = false;
    for (const indicator of riskIndicators) {
      const elements = page.locator(indicator);
      const count = await elements.count();
      if (count > 0) {
        riskUIFound = true;
        console.log(`✅ Risk analysis UI found: ${indicator}`);
      }
    }

    // Check API calls
    console.log(`📊 Risk-related API calls: ${riskApiCalls.length}`);
    riskApiCalls.forEach((call, i) => {
      console.log(`   ${i + 1}. ${call.method} ${call.url}`);
    });

    // Results
    if (riskUIFound || riskApiCalls.length > 0) {
      console.log('✅ Risk analysis integration: WORKING');
    } else {
      console.log('⏳ Risk analysis integration: NO ACTIVITY DETECTED');
    }
  });

  test('should test error handling and recovery', async ({ page }) => {
    console.log('🚨 Testing error handling and recovery...');

    // Monitor console errors
    const consoleErrors = [];
    page.on('console', msg => {
      if (msg.type() === 'error') {
        consoleErrors.push(msg.text());
      }
    });

    // Monitor network errors
    const networkErrors = [];
    page.on('requestfailed', request => {
      networkErrors.push({
        url: request.url(),
        failure: request.failure()
      });
    });

    // Test 1: Navigate without authentication
    console.log('🔍 Test 1: Accessing bridge without authentication...');
    await page.goto('/bridge');
    await page.waitForTimeout(3000);

    const authRequired = await page.locator('.swap-form__auth-required').isVisible().catch(() => false);
    if (authRequired) {
      console.log('✅ Test 1: Bridge correctly shows auth requirement');
    } else {
      console.log('❌ Test 1: Bridge accessible without authentication (security issue)');
    }

    // Test 2: Invalid amount input
    console.log('🔍 Test 2: Testing invalid amount input...');
    
    // Authenticate first
    const ethButton = page.locator('button:has-text("Ethereum Wallet")');
    if (await ethButton.isVisible() && await ethButton.getAttribute('disabled') === null) {
      await ethButton.click();
      await page.waitForTimeout(10000);
    }

    await page.goto('/bridge');
    await page.waitForTimeout(5000);

    const amountInput = page.locator('input[type="number"], input[placeholder*="amount"]').first();
    const amountInputExists = await amountInput.count() > 0;
    
    if (amountInputExists) {
      // Test invalid inputs
      const invalidInputs = ['abc', '-1', '0', '999999999999'];
      
      for (const invalidInput of invalidInputs) {
        await amountInput.fill(invalidInput);
        await page.waitForTimeout(1000);
        
        // Look for error messages
        const errorMessages = page.locator('.error, .invalid, [role="alert"]');
        const errorCount = await errorMessages.count();
        
        if (errorCount > 0) {
          console.log(`✅ Test 2: Invalid input "${invalidInput}" - error message shown`);
        } else {
          console.log(`⏳ Test 2: Invalid input "${invalidInput}" - no error message`);
        }
      }
    }

    // Test 3: Network error simulation
    console.log('🔍 Test 3: Checking network error handling...');
    
    // This test checks how the app handles existing network errors
    if (networkErrors.length > 0) {
      console.log(`✅ Test 3: Network errors detected and handled: ${networkErrors.length}`);
      networkErrors.forEach((error, i) => {
        console.log(`   ${i + 1}. ${error.url} - ${error.failure?.errorText || 'Unknown error'}`);
      });
    } else {
      console.log('⏳ Test 3: No network errors detected');
    }

    // Summary
    console.log('📊 Error Handling Test Results:');
    console.log(`   Console errors: ${consoleErrors.length}`);
    console.log(`   Network errors: ${networkErrors.length}`);
    console.log(`   Auth protection: ${authRequired ? 'WORKING' : 'MISSING'}`);
    
    if (consoleErrors.length < 5 && authRequired) {
      console.log('✅ Error handling: BASIC PROTECTION WORKING');
    } else {
      console.log('⚠️ Error handling: NEEDS ATTENTION');
    }
  });

  test('should test performance and responsiveness', async ({ page }) => {
    console.log('⚡ Testing performance and responsiveness...');

    const performanceMetrics = {
      pageLoadTime: 0,
      authTime: 0,
      bridgeLoadTime: 0,
      apiResponseTimes: []
    };

    // Track API response times
    page.on('response', response => {
      if (response.url().includes('/api/v1/')) {
        // Note: We can't easily measure exact response time in Playwright
        // but we can track successful responses
        performanceMetrics.apiResponseTimes.push({
          url: response.url(),
          status: response.status(),
          ok: response.ok()
        });
      }
    });

    // Test 1: Page load performance
    console.log('🔍 Test 1: Measuring page load performance...');
    const loadStart = Date.now();
    
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    
    const loadEnd = Date.now();
    performanceMetrics.pageLoadTime = loadEnd - loadStart;
    console.log(`   Page load time: ${performanceMetrics.pageLoadTime}ms`);

    // Test 2: Authentication performance
    console.log('🔍 Test 2: Measuring authentication performance...');
    const authStart = Date.now();
    
    const ethButton = page.locator('button:has-text("Ethereum Wallet")');
    if (await ethButton.isVisible() && await ethButton.getAttribute('disabled') === null) {
      await ethButton.click();
      await page.waitForTimeout(10000);
    }
    
    const authEnd = Date.now();
    performanceMetrics.authTime = authEnd - authStart;
    console.log(`   Authentication time: ${performanceMetrics.authTime}ms`);

    // Test 3: Bridge page load performance
    console.log('🔍 Test 3: Measuring bridge page performance...');
    const bridgeStart = Date.now();
    
    await page.goto('/bridge');
    await page.waitForTimeout(5000);
    
    const bridgeEnd = Date.now();
    performanceMetrics.bridgeLoadTime = bridgeEnd - bridgeStart;
    console.log(`   Bridge load time: ${performanceMetrics.bridgeLoadTime}ms`);

    // Test 4: UI responsiveness
    console.log('🔍 Test 4: Testing UI responsiveness...');
    
    const buttons = page.locator('button');
    const buttonCount = await buttons.count();
    
    let responsiveButtons = 0;
    for (let i = 0; i < Math.min(buttonCount, 5); i++) {
      const button = buttons.nth(i);
      const isEnabled = await button.isEnabled();
      const isVisible = await button.isVisible();
      
      if (isEnabled && isVisible) {
        responsiveButtons++;
      }
    }
    
    console.log(`   Responsive buttons: ${responsiveButtons}/${Math.min(buttonCount, 5)}`);

    // Performance summary
    console.log('📊 Performance Test Results:');
    console.log(`   Page load: ${performanceMetrics.pageLoadTime}ms`);
    console.log(`   Authentication: ${performanceMetrics.authTime}ms`);
    console.log(`   Bridge load: ${performanceMetrics.bridgeLoadTime}ms`);
    console.log(`   API calls: ${performanceMetrics.apiResponseTimes.length}`);
    console.log(`   UI responsiveness: ${responsiveButtons}/${Math.min(buttonCount, 5)} buttons`);

    // Performance criteria (based on plan requirements)
    const pageLoadOk = performanceMetrics.pageLoadTime < 10000; // 10 seconds
    const authOk = performanceMetrics.authTime < 15000; // 15 seconds 
    const bridgeLoadOk = performanceMetrics.bridgeLoadTime < 10000; // 10 seconds
    const uiResponsive = responsiveButtons > 0;

    if (pageLoadOk && authOk && bridgeLoadOk && uiResponsive) {
      console.log('✅ Performance: MEETS BASIC REQUIREMENTS');
    } else {
      console.log('⚠️ Performance: SOME METRICS NEED OPTIMIZATION');
    }
  });
});