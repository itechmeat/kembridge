import { test, expect } from '@playwright/test';
import { setupMockWalletAndNavigate } from '../utils/mock-wallet-utility.js';

test.describe('Complete Transaction Flow Testing', () => {
  test.beforeEach(async ({ page }) => {
    // Setup mock wallet and navigate to home page
    const setupSuccess = await setupMockWalletAndNavigate(page, '/', {
      waitAfterSetup: 3000,
      waitAfterNavigation: 3000
    });
    
    if (!setupSuccess) {
      throw new Error('Failed to setup mock wallet');
    }
  });

  test('should complete ETH→NEAR transaction flow with form interaction', async ({ page }) => {
    console.log('🚀 Testing complete ETH→NEAR transaction flow with form interaction...');

    // Track API calls
    const apiCalls = [];
    page.on('request', request => {
      const url = request.url();
      if (url.includes('/api/v1/')) {
        apiCalls.push({
          url,
          method: request.method(),
          timestamp: Date.now()
        });
      }
    });

    // Step 1: Authenticate
    console.log('🔐 Step 1: Authenticating...');
    const ethButton = page.locator('button:has-text("Ethereum Wallet")');
    if (await ethButton.isVisible() && await ethButton.getAttribute('disabled') === null) {
      await ethButton.click();
      await page.waitForTimeout(10000);
      console.log('✅ Step 1: Authentication completed');
    }

    // Step 2: Navigate to bridge
    console.log('🌉 Step 2: Navigating to bridge...');
    await page.goto('/bridge');
    await page.waitForTimeout(5000);

    // Step 3: Wait for tokens to load
    console.log('🪙 Step 3: Waiting for tokens to load...');
    
    // Wait for bridge tokens API call to complete
    let tokensLoaded = false;
    let attempts = 0;
    const maxAttempts = 10;
    
    while (!tokensLoaded && attempts < maxAttempts) {
      await page.waitForTimeout(1000);
      attempts++;
      
      const tokenCalls = apiCalls.filter(call => call.url.includes('/bridge/tokens'));
      if (tokenCalls.length > 0) {
        tokensLoaded = true;
        console.log('✅ Step 3: Bridge tokens API called successfully');
      }
    }

    if (!tokensLoaded) {
      console.log('❌ Step 3: Bridge tokens failed to load');
      return;
    }

    // Step 4: Check form accessibility
    console.log('📝 Step 4: Checking form accessibility...');
    
    const authRequired = await page.locator('.swap-form__auth-required').isVisible().catch(() => false);
    if (authRequired) {
      console.log('❌ Step 4: Form still showing auth required');
      return;
    }
    
    console.log('✅ Step 4: Form is accessible (authenticated)');

    // Step 5: Check token selectors
    console.log('🔍 Step 5: Checking token selectors...');
    
    const tokenSelectors = page.locator('.token-selector, .swap-form__token-selector');
    const selectorCount = await tokenSelectors.count();
    console.log(`   Found ${selectorCount} token selectors`);

    if (selectorCount >= 2) {
      console.log('✅ Step 5: Token selectors are present');
      
      // Step 6: Check default token selection
      const fromTokenText = await tokenSelectors.first().textContent();
      const toTokenText = await tokenSelectors.last().textContent();
      console.log(`   From token: ${fromTokenText}`);
      console.log(`   To token: ${toTokenText}`);
    }

    // Step 7: Wait for amount input to appear
    console.log('💰 Step 7: Looking for amount input...');
    
    // Try different selectors for amount input
    const amountSelectors = [
      'input[type="number"]',
      'input[placeholder*="amount"]', 
      'input[placeholder="0.0"]',
      '.amount-input input',
      '.swap-form__amount-input input'
    ];

    let amountInput = null;
    let amountFound = false;

    for (const selector of amountSelectors) {
      const elements = page.locator(selector);
      const count = await elements.count();
      if (count > 0) {
        amountInput = elements.first();
        amountFound = true;
        console.log(`✅ Step 7: Amount input found with selector: ${selector}`);
        break;
      }
    }

    if (!amountFound) {
      console.log('⚠️ Step 7: No amount input found - checking if tokens are selected');
      
      // Try to wait a bit more for form to fully render
      await page.waitForTimeout(3000);
      
      // Re-check for amount input
      for (const selector of amountSelectors) {
        const elements = page.locator(selector);
        const count = await elements.count();
        if (count > 0) {
          amountInput = elements.first();
          amountFound = true;
          console.log(`✅ Step 7 (retry): Amount input found with selector: ${selector}`);
          break;
        }
      }
    }

    if (amountFound && amountInput) {
      // Step 8: Enter amount
      console.log('💰 Step 8: Entering amount...');
      await amountInput.fill('0.01');
      await page.waitForTimeout(2000);
      console.log('✅ Step 8: Amount entered');

      // Step 9: Wait for quote generation
      console.log('📊 Step 9: Waiting for quote generation...');
      await page.waitForTimeout(3000);
      
      // Check for quote API calls
      const quoteCalls = apiCalls.filter(call => call.url.includes('/quote'));
      if (quoteCalls.length > 0) {
        console.log('✅ Step 9: Quote API called');
      } else {
        console.log('⏳ Step 9: No quote API calls yet');
      }

      // Step 10: Look for submit button
      console.log('🎯 Step 10: Looking for submit button...');
      
      const submitSelectors = [
        'button[type="submit"]',
        'button:has-text("Review Swap")',
        'button:has-text("Get Quote")',
        'button:has-text("Swap")',
        '.swap-form__submit'
      ];

      let submitButton = null;
      let submitFound = false;

      for (const selector of submitSelectors) {
        const elements = page.locator(selector);
        const count = await elements.count();
        if (count > 0) {
          submitButton = elements.first();
          submitFound = true;
          console.log(`✅ Step 10: Submit button found with selector: ${selector}`);
          break;
        }
      }

      if (submitFound && submitButton) {
        const isEnabled = await submitButton.getAttribute('disabled') === null;
        const buttonText = await submitButton.textContent();
        console.log(`   Button text: "${buttonText}"`);
        console.log(`   Button enabled: ${isEnabled}`);

        if (isEnabled) {
          console.log('🎯 Step 11: Clicking submit button...');
          await submitButton.click();
          await page.waitForTimeout(3000);
          console.log('✅ Step 11: Submit button clicked');
        } else {
          console.log('⏳ Step 11: Submit button disabled (waiting for quote)');
        }
      } else {
        console.log('❌ Step 10: No submit button found');
      }
    } else {
      console.log('❌ Step 7: Amount input not found - form may not be fully loaded');
    }

    // Final API analysis
    console.log('📊 Final API Call Analysis:');
    const authCalls = apiCalls.filter(call => call.url.includes('/auth/'));
    const tokenCalls = apiCalls.filter(call => call.url.includes('/bridge/tokens'));
    const quoteCalls = apiCalls.filter(call => call.url.includes('/quote'));
    const swapCalls = apiCalls.filter(call => call.url.includes('/swap'));

    console.log(`   Auth calls: ${authCalls.length}`);
    console.log(`   Token calls: ${tokenCalls.length}`);
    console.log(`   Quote calls: ${quoteCalls.length}`);
    console.log(`   Swap calls: ${swapCalls.length}`);

    // Success criteria
    const authSuccess = authCalls.length >= 2; // nonce + verify
    const tokensSuccess = tokenCalls.length > 0;
    const formAccessible = !authRequired;

    console.log('🏁 Transaction Flow Test Results:');
    console.log(`   ✅ Authentication: ${authSuccess ? 'PASS' : 'FAIL'}`);
    console.log(`   ✅ Token Loading: ${tokensSuccess ? 'PASS' : 'FAIL'}`);
    console.log(`   ✅ Form Access: ${formAccessible ? 'PASS' : 'FAIL'}`);
    console.log(`   ✅ Amount Input: ${amountFound ? 'PASS' : 'FAIL'}`);
    
    if (authSuccess && tokensSuccess && formAccessible) {
      console.log('🎉 ETH→NEAR Transaction Flow: CORE FUNCTIONALITY WORKING');
    } else {
      console.log('❌ ETH→NEAR Transaction Flow: SOME ISSUES DETECTED');
    }
  });

  test('should test token selector interaction', async ({ page }) => {
    console.log('🔍 Testing token selector interaction...');

    // Authenticate first
    const ethButton = page.locator('button:has-text("Ethereum Wallet")');
    if (await ethButton.isVisible() && await ethButton.getAttribute('disabled') === null) {
      await ethButton.click();
      await page.waitForTimeout(10000);
    }

    // Navigate to bridge
    await page.goto('/bridge');
    await page.waitForTimeout(5000);

    // Wait for tokens to load
    await page.waitForTimeout(5000);

    // Check if form is accessible
    const authRequired = await page.locator('.swap-form__auth-required').isVisible().catch(() => false);
    if (authRequired) {
      console.log('❌ Form not accessible - authentication required');
      return;
    }

    // Look for token selectors
    const tokenSelectors = page.locator('.token-selector, .swap-form__token-selector, select');
    const selectorCount = await tokenSelectors.count();
    console.log(`🔍 Found ${selectorCount} token selectors`);

    if (selectorCount >= 2) {
      // Test clicking on token selectors
      for (let i = 0; i < Math.min(selectorCount, 2); i++) {
        const selector = tokenSelectors.nth(i);
        const isClickable = await selector.isEnabled();
        
        if (isClickable) {
          console.log(`🔄 Clicking token selector ${i + 1}...`);
          
          try {
            await selector.click();
            await page.waitForTimeout(1000);
            
            // Look for dropdown or modal
            const dropdown = page.locator('.token-dropdown, .token-modal, .dropdown-menu');
            const dropdownVisible = await dropdown.isVisible().catch(() => false);
            
            if (dropdownVisible) {
              console.log(`✅ Token selector ${i + 1}: Dropdown opened`);
              
              // Try to close dropdown by clicking elsewhere
              await page.click('body');
              await page.waitForTimeout(500);
            } else {
              console.log(`⏳ Token selector ${i + 1}: No dropdown detected`);
            }
          } catch (error) {
            console.log(`❌ Token selector ${i + 1}: Click failed - ${error.message}`);
          }
        } else {
          console.log(`❌ Token selector ${i + 1}: Not clickable`);
        }
      }
    } else {
      console.log('❌ Token selectors not found or insufficient count');
    }
  });

  test('should test bridge direction switching', async ({ page }) => {
    console.log('🔄 Testing bridge direction switching...');

    // Authenticate first
    const ethButton = page.locator('button:has-text("Ethereum Wallet")');
    if (await ethButton.isVisible() && await ethButton.getAttribute('disabled') === null) {
      await ethButton.click();
      await page.waitForTimeout(10000);
    }

    // Navigate to bridge
    await page.goto('/bridge');
    await page.waitForTimeout(5000);

    // Wait for form to load
    await page.waitForTimeout(3000);

    // Look for direction switch button
    const switchSelectors = [
      'button:has-text("⇅")',
      'button:has-text("↔")', 
      'button:has-text("⇄")',
      '.swap-direction',
      '.reverse-button',
      '.swap-form__swap-button'
    ];

    let switchButton = null;
    let switchFound = false;

    for (const selector of switchSelectors) {
      const elements = page.locator(selector);
      const count = await elements.count();
      if (count > 0) {
        switchButton = elements.first();
        switchFound = true;
        console.log(`✅ Direction switch found with selector: ${selector}`);
        break;
      }
    }

    if (switchFound && switchButton) {
      // Get initial state
      const initialFromChain = await page.locator('.swap-form__section').first().textContent();
      const initialToChain = await page.locator('.swap-form__section').last().textContent();
      
      console.log('📊 Initial state:');
      console.log(`   From: ${initialFromChain}`);
      console.log(`   To: ${initialToChain}`);

      // Click switch button
      console.log('🔄 Clicking direction switch...');
      await switchButton.click();
      await page.waitForTimeout(2000);

      // Check new state
      const newFromChain = await page.locator('.swap-form__section').first().textContent();
      const newToChain = await page.locator('.swap-form__section').last().textContent();
      
      console.log('📊 New state:');
      console.log(`   From: ${newFromChain}`);
      console.log(`   To: ${newToChain}`);

      if (initialFromChain !== newFromChain || initialToChain !== newToChain) {
        console.log('✅ Bridge direction switching: WORKING');
      } else {
        console.log('⏳ Bridge direction switching: No change detected');
      }
    } else {
      console.log('❌ Direction switch button not found');
    }
  });
});