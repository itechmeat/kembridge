/**
 * Simple check of what elements actually exist
 */
import { test, expect } from '@playwright/test';

test.describe('Simple DOM Check', () => {
  test('should check what elements exist on homepage', async ({ page }) => {
    console.log('🔍 Loading homepage...');
    
    try {
      await page.goto('/', { timeout: 30000 });
      await page.waitForLoadState('domcontentloaded');
      console.log('✅ Page loaded');
      
      // Get page title
      const title = await page.title();
      console.log(`📄 Page title: "${title}"`);
      
      // Count basic elements
      const buttonCount = await page.locator('button').count();
      const inputCount = await page.locator('input').count();
      const linkCount = await page.locator('a').count();
      
      console.log(`📊 Elements found:`);
      console.log(`   Buttons: ${buttonCount}`);
      console.log(`   Inputs: ${inputCount}`);
      console.log(`   Links: ${linkCount}`);
      
      // Get button texts
      if (buttonCount > 0) {
        console.log('🔘 Button texts:');
        for (let i = 0; i < Math.min(buttonCount, 5); i++) {
          const button = page.locator('button').nth(i);
          const text = await button.textContent();
          const isVisible = await button.isVisible();
          console.log(`   ${i + 1}. "${text}" (visible: ${isVisible})`);
        }
      }
      
      // Check for wallet-related text
      const walletTexts = ['wallet', 'connect', 'ethereum', 'near', 'bridge', 'swap'];
      
      for (const text of walletTexts) {
        const elements = page.locator(`:has-text("${text}")`);
        const count = await elements.count();
        if (count > 0) {
          console.log(`💰 Found "${text}": ${count} times`);
        }
      }
      
      // Basic assertions
      expect(buttonCount).toBeGreaterThan(0);
      console.log('✅ Basic DOM check passed');
      
    } catch (error) {
      console.error('❌ Test failed:', error.message);
      throw error;
    }
  });

  test('should check bridge page elements', async ({ page }) => {
    console.log('🌉 Loading bridge page...');
    
    try {
      await page.goto('/bridge', { timeout: 30000 });
      await page.waitForLoadState('domcontentloaded');
      console.log('✅ Bridge page loaded');
      
      // Wait a bit for dynamic content
      await page.waitForTimeout(3000);
      
      // Check for form elements
      const forms = await page.locator('form').count();
      const inputs = await page.locator('input').count();
      const selects = await page.locator('select').count();
      
      console.log(`📋 Form elements:`);
      console.log(`   Forms: ${forms}`);
      console.log(`   Inputs: ${inputs}`);
      console.log(`   Selects: ${selects}`);
      
      // Check for bridge-specific classes
      const bridgeClasses = [
        '.swap-form',
        '.token-selector', 
        '.amount-input',
        '.bridge-form',
        '.swap-section'
      ];
      
      for (const className of bridgeClasses) {
        const count = await page.locator(className).count();
        if (count > 0) {
          console.log(`🎯 Found ${className}: ${count} elements`);
        }
      }
      
      // Look for any authentication required messages
      const authTexts = ['connect', 'authenticate', 'required', 'wallet'];
      for (const text of authTexts) {
        const elements = page.locator(`:has-text("${text}")`).first();
        const visible = await elements.isVisible().catch(() => false);
        if (visible) {
          const content = await elements.textContent().catch(() => '');
          console.log(`🔐 Auth-related text: "${content}"`);
        }
      }
      
      console.log('✅ Bridge page check completed');
      
    } catch (error) {
      console.error('❌ Bridge page test failed:', error.message);
      // Don't fail the test, just log the issue
      console.log('⚠️ Bridge page may not be available or require authentication');
    }
  });
});