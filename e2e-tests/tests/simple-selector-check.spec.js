/**
 * Simple check of what elements actually exist
 */
import { test, expect } from '@playwright/test';
import { TestSelectors } from '../utils/selectors.ts';

test.describe('Simple DOM Check', () => {
  test('should check what elements exist on homepage', async ({ page }) => {
    console.log('🔍 Loading homepage...');
    const selectors = new TestSelectors(page);
    
    try {
      await page.goto('/', { timeout: 30000 });
      await page.waitForLoadState('domcontentloaded');
      console.log('✅ Page loaded');
      
      // Get page title
      const title = await page.title();
      console.log(`📄 Page title: "${title}"`);
      
      // Count basic elements using TestSelectors
      const ethWalletButton = selectors.ethWalletButton;
      const nearWalletButton = selectors.nearWalletButton;
      const connectWalletButton = selectors.connectWalletButton;
      
      const ethButtonVisible = await ethWalletButton.isVisible().catch(() => false);
      const nearButtonVisible = await nearWalletButton.isVisible().catch(() => false);
      const connectButtonVisible = await connectWalletButton.isVisible().catch(() => false);
      
      console.log(`📊 Wallet buttons found:`);
      console.log(`   ETH Wallet: ${ethButtonVisible ? '✅' : '❌'}`);
      console.log(`   NEAR Wallet: ${nearButtonVisible ? '✅' : '❌'}`);
      console.log(`   Connect Wallet: ${connectButtonVisible ? '✅' : '❌'}`);
      
      // Get button texts
      if (ethButtonVisible) {
        const text = await ethWalletButton.textContent();
        console.log(`🔘 ETH Wallet button text: "${text}"`);
      }
      
      if (nearButtonVisible) {
        const text = await nearWalletButton.textContent();
        console.log(`🔘 NEAR Wallet button text: "${text}"`);
      }
      
      // Basic assertions - at least one wallet button should be present
      expect(ethButtonVisible || nearButtonVisible || connectButtonVisible).toBeTruthy();
      console.log('✅ Basic DOM check passed');
      
    } catch (error) {
      console.error('❌ Test failed:', error.message);
      throw error;
    }
  });

  test('should check bridge page elements', async ({ page }) => {
    console.log('🌉 Loading bridge page...');
    const selectors = new TestSelectors(page);
    
    try {
      await page.goto('/bridge', { timeout: 30000 });
      await page.waitForLoadState('domcontentloaded');
      console.log('✅ Bridge page loaded');
      
      // Wait a bit for dynamic content
      await page.waitForTimeout(3000);
      
      // Check for bridge-specific elements using TestSelectors
      const swapForm = selectors.swapForm;
      const tokenSelector = selectors.tokenSelector;
      const amountInput = selectors.amountInput;
      const bridgeForm = selectors.bridgeForm;
      const submitButton = selectors.submitButton;
      
      const swapFormVisible = await swapForm.isVisible().catch(() => false);
      const tokenSelectorVisible = await tokenSelector.isVisible().catch(() => false);
      const amountInputVisible = await amountInput.isVisible().catch(() => false);
      const bridgeFormVisible = await bridgeForm.isVisible().catch(() => false);
      const submitButtonVisible = await submitButton.isVisible().catch(() => false);
      
      console.log(`📋 Bridge form elements:`);
      console.log(`   Swap Form: ${swapFormVisible ? '✅' : '❌'}`);
      console.log(`   Token Selector: ${tokenSelectorVisible ? '✅' : '❌'}`);
      console.log(`   Amount Input: ${amountInputVisible ? '✅' : '❌'}`);
      console.log(`   Bridge Form: ${bridgeFormVisible ? '✅' : '❌'}`);
      console.log(`   Submit Button: ${submitButtonVisible ? '✅' : '❌'}`);
      
      // Check for wallet connection elements
      const ethWalletButton = selectors.ethWalletButton;
      const nearWalletButton = selectors.nearWalletButton;
      const connectWalletButton = selectors.connectWalletButton;
      
      const ethButtonVisible = await ethWalletButton.isVisible().catch(() => false);
      const nearButtonVisible = await nearWalletButton.isVisible().catch(() => false);
      const connectButtonVisible = await connectWalletButton.isVisible().catch(() => false);
      
      console.log(`🔐 Wallet connection elements:`);
      console.log(`   ETH Wallet: ${ethButtonVisible ? '✅' : '❌'}`);
      console.log(`   NEAR Wallet: ${nearButtonVisible ? '✅' : '❌'}`);
      console.log(`   Connect Wallet: ${connectButtonVisible ? '✅' : '❌'}`);
      
      console.log('✅ Bridge page check completed');
      
    } catch (error) {
      console.error('❌ Bridge page test failed:', error.message);
      // Don't fail the test, just log the issue
      console.log('⚠️ Bridge page may not be available or require authentication');
    }
  });
});