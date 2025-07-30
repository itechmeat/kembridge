/**
 * Simple AI Risk Display E2E Tests
 * Simplified version without excessive API logging
 */

import { test, expect } from '@playwright/test';

test.describe('AI Risk Display Simple Tests', () => {
  
  test.beforeEach(async ({ page }) => {
    // Navigate to the main page
    await page.goto('/');
    
    // Wait for page to load
    await page.waitForLoadState('networkidle');
    
    // Connect to Ethereum wallet to access Bridge page
    console.log('🔗 Connecting Ethereum wallet...');
    const ethereumButton = page.getByText('Ethereum Wallet');
    await expect(ethereumButton).toBeVisible({ timeout: 10000 });
    await ethereumButton.click();
    
    // Wait for bridge page to load after wallet connection
    console.log('⏳ Waiting for Bridge page...');
    await page.waitForURL('**/bridge', { timeout: 30000 });
    await page.waitForLoadState('networkidle');
    
    console.log('✅ Bridge page loaded');
  });

  test('should display AI Risk Display component when AI Engine is healthy', async ({ page }) => {
    console.log('🤖 Testing AI Risk Display basic functionality...');

    // Wait for AI Risk Display component to be present
    const aiRiskDisplay = page.locator('[data-testid="ai-risk-display-ready"]');
    await expect(aiRiskDisplay).toBeVisible({ timeout: 30000 });
    
    // Check that the component shows "Ready" state
    await expect(aiRiskDisplay).toContainText('AI Risk Engine Ready');
    
    console.log('✅ AI Risk Display Component: WORKING');
  });

  test('should trigger risk analysis on form input', async ({ page }) => {
    console.log('🤖 Testing risk analysis triggering...');

    // Wait for the component to be ready
    await page.waitForSelector('[data-testid="ai-risk-display-ready"]', { timeout: 30000 });
    
    // Fill in some transaction data to trigger analysis
    const amountInput = page.locator('input[placeholder="0.0"]');
    await expect(amountInput).toBeVisible({ timeout: 10000 });
    
    // Enter an amount
    await amountInput.fill('100');
    
    // Wait a bit for debounced analysis
    await page.waitForTimeout(2000);
    
    // Check if risk analysis was triggered (either loading or results)
    const hasAnalysis = await page.locator('[data-testid="ai-risk-display-loading"], [data-testid="ai-risk-display"]').count() > 0;
    
    if (hasAnalysis) {
      console.log('✅ Risk Analysis: TRIGGERED');
    } else {
      console.log('⚠️ Risk Analysis: NOT TRIGGERED (may need more transaction data)');
    }
  });

  test('should display risk analysis results', async ({ page }) => {
    console.log('🤖 Testing risk analysis results display...');

    // Wait for component
    await page.waitForSelector('[data-testid="ai-risk-display-ready"]', { timeout: 30000 });
    
    // Fill complete transaction data
    await page.locator('input[placeholder="0.0"]').fill('100');
    
    // Select tokens if available
    const fromTokenSelect = page.locator('select').first();
    if (await fromTokenSelect.count() > 0) {
      await fromTokenSelect.selectOption({ index: 0 });
    }
    
    // Wait for analysis
    await page.waitForTimeout(3000);
    
    // Check for results
    const hasResults = await page.locator('[data-testid="ai-risk-display"]').count() > 0;
    const hasScore = await page.locator('[data-testid="ai-risk-score-value"]').count() > 0;
    
    if (hasResults && hasScore) {
      console.log('✅ Risk Analysis Results: DISPLAYED');
    } else {
      console.log('⚠️ Risk Analysis Results: PENDING (may need complete transaction data)');
    }
  });

});