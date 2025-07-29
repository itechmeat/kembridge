/**
 * Selector Validation Test
 * This test validates what selectors actually work in the real application
 */
import { test, expect } from '@playwright/test';
import { RealisticSelectors, WORKING_SELECTORS } from '../utils/realistic-selectors.js';
import { setupMockWallet } from '../utils/wallet-helpers.js';

test.describe('Selector Validation', () => {
  let selectors;

  test.beforeEach(async ({ page }) => {
    // Setup mock wallet
    await setupMockWallet(page);
    
    // Initialize realistic selectors
    selectors = new RealisticSelectors(page);
    
    // Navigate to app
    await page.goto('/');
    await selectors.waitForPageLoad();
  });

  test('should validate authentication selectors', async ({ page }) => {
    console.log('🔍 Validating authentication selectors...');
    
    // Debug what's actually available
    await selectors.debugSelectors();
    
    // Test wallet buttons
    const ethButtonExists = await selectors.elementExists(WORKING_SELECTORS.ETH_WALLET_BUTTON);
    const nearButtonExists = await selectors.elementExists(WORKING_SELECTORS.NEAR_WALLET_BUTTON);
    
    console.log(`ETH wallet button exists: ${ethButtonExists ? '✅' : '❌'}`);
    console.log(`NEAR wallet button exists: ${nearButtonExists ? '✅' : '❌'}`);
    
    if (ethButtonExists) {
      const ethButton = selectors.ethWalletButton;
      const isVisible = await ethButton.isVisible();
      const buttonText = await ethButton.textContent();
      console.log(`ETH button visible: ${isVisible}, text: "${buttonText}"`);
      
      expect(isVisible).toBeTruthy();
    }
    
    if (nearButtonExists) {
      const nearButton = selectors.nearWalletButton;
      const isVisible = await nearButton.isVisible();
      const buttonText = await nearButton.textContent();
      console.log(`NEAR button visible: ${isVisible}, text: "${buttonText}"`);
      
      expect(isVisible).toBeTruthy();
    }
    
    // At least one wallet button should exist
    expect(ethButtonExists || nearButtonExists).toBeTruthy();
  });

  test('should validate bridge form selectors after authentication', async ({ page }) => {
    console.log('🌉 Validating bridge form selectors...');
    
    // Try to authenticate first
    const authSuccess = await selectors.safeClick([
      WORKING_SELECTORS.ETH_WALLET_BUTTON,
      'button:has-text("Connect")',
      'button:has-text("Wallet")'
    ]);
    
    if (authSuccess) {
      console.log('✅ Authentication clicked, waiting for connection...');
      await page.waitForTimeout(5000);
    }
    
    // Navigate to bridge page
    const bridgeNavSuccess = await selectors.safeClick([
      WORKING_SELECTORS.SWAP_NAV,
      'a[href="/bridge"]',
      'button:has-text("Bridge")',
      'button:has-text("Swap")'
    ]);
    
    if (bridgeNavSuccess) {
      console.log('✅ Bridge navigation successful');
      await page.waitForTimeout(3000);
    } else {
      // Try direct navigation
      await page.goto('/bridge');
      await page.waitForTimeout(3000);
    }
    
    // Test form elements
    const formSelectors = [
      { name: 'Amount Input', selector: WORKING_SELECTORS.AMOUNT_INPUT },
      { name: 'Token Selector', selector: WORKING_SELECTORS.TOKEN_SELECTOR },
      { name: 'Submit Button', selector: WORKING_SELECTORS.SUBMIT_BUTTON },
      { name: 'Direction Switch', selector: WORKING_SELECTORS.DIRECTION_SWITCH }
    ];
    
    let foundElements = 0;
    
    for (const { name, selector } of formSelectors) {
      const exists = await selectors.elementExists(selector);
      const count = await page.locator(selector).count();
      
      console.log(`${name}: ${exists ? '✅' : '❌'} (count: ${count})`);
      
      if (exists) {
        foundElements++;
        
        // Get more details about the element
        const element = page.locator(selector).first();
        const isVisible = await element.isVisible({ timeout: 1000 }).catch(() => false);
        const classes = await element.getAttribute('class').catch(() => 'N/A');
        
        console.log(`  - Visible: ${isVisible}, Classes: ${classes}`);
      }
    }
    
    console.log(`📊 Bridge form elements found: ${foundElements}/${formSelectors.length}`);
    
    // At least some form elements should be present
    expect(foundElements).toBeGreaterThan(0);
  });

  test('should test text-based selectors (most reliable)', async ({ page }) => {
    console.log('📝 Testing text-based selectors...');
    
    // Test common text patterns that should exist
    const textPatterns = [
      'Welcome',
      'KEMBridge', 
      'Connect',
      'Wallet',
      'Ethereum',
      'NEAR'
    ];
    
    let foundTexts = 0;
    
    for (const text of textPatterns) {
      const element = selectors.findByPartialText(text);
      const count = await element.count();
      
      if (count > 0) {
        foundTexts++;
        console.log(`✅ Found text "${text}": ${count} occurrences`);
        
        // Get first occurrence details
        const firstElement = element.first();
        const isVisible = await firstElement.isVisible({ timeout: 1000 }).catch(() => false);
        const fullText = await firstElement.textContent().catch(() => 'N/A');
        
        console.log(`  - Visible: ${isVisible}, Full text: "${fullText}"`);
      } else {
        console.log(`❌ Text "${text}" not found`);
      }
    }
    
    console.log(`📊 Text patterns found: ${foundTexts}/${textPatterns.length}`);
    
    // Should find most text patterns
    expect(foundTexts).toBeGreaterThan(textPatterns.length / 2);
  });

  test('should identify what accessibility attributes exist', async ({ page }) => {
    console.log('♿ Checking accessibility attributes...');
    
    // Check for aria-labels
    const ariaLabels = await page.locator('[aria-label]').count();
    console.log(`Elements with aria-label: ${ariaLabels}`);
    
    if (ariaLabels > 0) {
      const firstLabel = await page.locator('[aria-label]').first().getAttribute('aria-label');
      console.log(`First aria-label: "${firstLabel}"`);
    }
    
    // Check for roles
    const rolesMap = {};
    const roleElements = await page.locator('[role]').all();
    
    for (const element of roleElements) {
      const role = await element.getAttribute('role');
      rolesMap[role] = (rolesMap[role] || 0) + 1;
    }
    
    console.log('Roles found:', rolesMap);
    
    // Check for data-testid attributes
    const testIds = await page.locator('[data-testid]').count();
    console.log(`Elements with data-testid: ${testIds}`);
    
    if (testIds > 0) {
      const testIdElements = await page.locator('[data-testid]').all();
      const testIdList = [];
      
      for (const element of testIdElements.slice(0, 5)) {
        const testId = await element.getAttribute('data-testid');
        testIdList.push(testId);
      }
      
      console.log('Test IDs found:', testIdList);
    }
    
    // Report accessibility readiness
    const accessibilityScore = ariaLabels + Object.keys(rolesMap).length + testIds;
    console.log(`📊 Accessibility readiness score: ${accessibilityScore}`);
    
    if (accessibilityScore > 10) {
      console.log('✅ Good accessibility attribute coverage');
    } else if (accessibilityScore > 5) {
      console.log('⚠️ Moderate accessibility attribute coverage');
    } else {
      console.log('❌ Low accessibility attribute coverage - recommend adding more aria-labels and data-testids');
    }
  });

  test('should test CSS class-based selectors (current approach)', async ({ page }) => {
    console.log('🎨 Testing CSS class-based selectors...');
    
    // Test common CSS patterns from our existing tests
    const cssPatterns = [
      '.btn, button',
      '.input, input',
      '.form, form',
      '.nav, nav',
      '.content, .main',
      '.header, header',
      '.footer, footer'
    ];
    
    let workingPatterns = 0;
    
    for (const pattern of cssPatterns) {
      const count = await page.locator(pattern).count();
      
      if (count > 0) {
        workingPatterns++;
        console.log(`✅ CSS pattern "${pattern}": ${count} elements`);
        
        // Get sample classes
        const firstElement = page.locator(pattern).first();
        const classes = await firstElement.getAttribute('class').catch(() => 'N/A');
        console.log(`  - Sample classes: ${classes}`);
      } else {
        console.log(`❌ CSS pattern "${pattern}": not found`);
      }
    }
    
    console.log(`📊 Working CSS patterns: ${workingPatterns}/${cssPatterns.length}`);
    
    // Test specific selectors from our working tests
    const specificSelectors = [
      '.bottom-nav__item',
      '.swap-form__auth-required',
      '.token-selector',
      '.swap-form__token-selector'
    ];
    
    let workingSpecific = 0;
    
    for (const selector of specificSelectors) {
      const exists = await selectors.elementExists(selector);
      
      if (exists) {
        workingSpecific++;
        console.log(`✅ Specific selector "${selector}": found`);
      } else {
        console.log(`❌ Specific selector "${selector}": not found`);
      }
    }
    
    console.log(`📊 Working specific selectors: ${workingSpecific}/${specificSelectors.length}`);
  });
});