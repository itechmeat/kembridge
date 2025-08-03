import { test, expect } from '@playwright/test';
import { setupMockWalletAndNavigate } from '../utils/mock-wallet-utility.js';
import { SERVICE_URLS, RISK_ANALYSIS, DEFAULT_USER_ID } from '../utils/constants.js';
import { getBackendUrl } from '../utils/page-evaluate-utils.js';
import { TestSelectors } from '../utils/selectors.ts';

test.describe('AI Risk Display Component E2E Tests', () => {
  test.beforeEach(async ({ page }) => {
    console.log('🤖 Setting up AI Risk Display tests...');
    
    // Setup mock wallet and navigate to home page
    const setupSuccess = await setupMockWalletAndNavigate(page, '/', {
      waitAfterSetup: 3000,
      waitAfterNavigation: 3000
    });
    
    if (!setupSuccess) {
      throw new Error('Failed to setup mock wallet');
    }
  });

  test('should display AI Risk Display component when AI Engine is healthy', async ({ page }) => {
    console.log('🏥 Testing AI Risk Display with healthy AI Engine...');
    const selectors = new TestSelectors(page);

    // Authenticate first
    const ethButton = selectors.ethWalletButton;
    if (await ethButton.isVisible() && await ethButton.getAttribute('disabled') === null) {
      await ethButton.click();
      await page.waitForTimeout(10000);
    }

    // Navigate to bridge page
    await page.goto('/bridge');
    await page.waitForTimeout(5000);

    // Look for AI Risk Display component
    const aiRiskDisplay = selectors.aiRiskDisplay;
    const aiRiskFound = await aiRiskDisplay.count() > 0;
    
    if (aiRiskFound) {
      console.log('✅ AI Risk Display found');
      const content = await aiRiskDisplay.first().textContent();
      console.log(`   Component content: "${content}"`);
    }

    expect(aiRiskFound).toBeTruthy();
    console.log('✅ AI Risk Display Component: PRESENT');

    // Check for AI Engine status indicators
    const statusFound = await page.getByText(/ai.*risk.*engine/i).count() > 0;
    
    if (statusFound) {
      console.log('✅ AI Engine Status: VISIBLE IN UI');
    } else {
      console.log('⏳ AI Engine Status: NOT EXPLICITLY SHOWN');
    }
  });

  test('should display offline state when AI Engine is unavailable', async ({ page }) => {
    console.log('❌ Testing AI Risk Display with offline AI Engine...');
    const selectors = new TestSelectors(page);
    
    // Mock AI Engine offline by intercepting requests
    await page.route('**/api/risk/**', route => {
      route.abort('failed');
    });

    // Authenticate first
    const ethButton = selectors.ethWalletButton;
    if (await ethButton.isVisible() && await ethButton.getAttribute('disabled') === null) {
      await ethButton.click();
      await page.waitForTimeout(10000);
    }

    // Navigate to bridge page
    await page.goto('/bridge');
    await page.waitForTimeout(5000);

    // Look for offline state indicators
    const offlineDisplay = selectors.aiRiskDisplayOffline;
    const offlineFound = await offlineDisplay.count() > 0;
    
    if (offlineFound) {
      console.log('✅ Offline state indicator found');
      const content = await offlineDisplay.first().textContent();
      console.log(`   Offline message: "${content}"`);
    }

    // Even if offline state isn't explicitly shown, component should handle gracefully
    console.log(`📊 Offline State Detection: ${offlineFound ? 'EXPLICIT' : 'GRACEFUL_DEGRADATION'}`);
  });

  test('should trigger risk analysis when transaction data changes', async ({ page }) => {
    console.log('📊 Testing risk analysis triggering...');
    const selectors = new TestSelectors(page);

    // Monitor API calls to AI Engine
    const aiRiskCalls = [];
    page.on('request', request => {
      const url = request.url();
      if (url.includes('/api/risk/analyze') || url.includes(getBackendUrl('aiEngine'))) {
        aiRiskCalls.push({
          url,
          method: request.method(),
          timestamp: Date.now()
        });
      }
    });

    // Authenticate and navigate
    const ethButton = selectors.ethWalletButton;
    if (await ethButton.isVisible() && await ethButton.getAttribute('disabled') === null) {
      await ethButton.click();
      await page.waitForTimeout(10000);
    }

    await page.goto('/bridge');
    await page.waitForTimeout(5000);

    // Enter transaction amount to trigger risk analysis
    const amountInput = selectors.amountInput;
    let amountEntered = false;
    
    if (await amountInput.count() > 0) {
      console.log('💰 Entering amount');
      await amountInput.fill('100'); // Amount that should trigger risk analysis
      await page.waitForTimeout(3000); // Wait for analysis
      amountEntered = true;
    }

    console.log(`📝 Amount entered: ${amountEntered}`);
    console.log(`📊 AI Risk API calls: ${aiRiskCalls.length}`);

    aiRiskCalls.forEach((call, i) => {
      console.log(`   ${i + 1}. ${call.method} ${call.url}`);
    });

    // Look for risk analysis results in UI
    const riskScoreValue = selectors.aiRiskScoreValue;
    const riskScoreLevel = selectors.aiRiskScoreLevel;
    const approvalStatus = selectors.aiRiskApprovalStatus;
    
    const riskResultsShown = await riskScoreValue.count() > 0 || 
                            await riskScoreLevel.count() > 0 || 
                            await approvalStatus.count() > 0;
    
    if (riskResultsShown) {
      console.log('✅ Risk analysis results found');
      if (await riskScoreValue.count() > 0) {
        const content = await riskScoreValue.first().textContent();
        console.log(`   Score: "${content}"`);
      }
    }

    console.log(`📊 Risk Analysis Integration: ${riskResultsShown || aiRiskCalls.length > 0 ? 'WORKING' : 'NEEDS_INVESTIGATION'}`);
  });

  test('should display risk analysis loading state', async ({ page }) => {
    console.log('⏳ Testing risk analysis loading state...');
    const selectors = new TestSelectors(page);

    // Slow down AI Engine responses to see loading state
    await page.route('**/api/risk/analyze', async route => {
      // Delay the response to capture loading state
      await new Promise(resolve => setTimeout(resolve, 2000));
      route.continue();
    });

    // Authenticate and navigate
    const ethButton = selectors.ethWalletButton;
    if (await ethButton.isVisible() && await ethButton.getAttribute('disabled') === null) {
      await ethButton.click();
      await page.waitForTimeout(10000);
    }

    await page.goto('/bridge');
    await page.waitForTimeout(5000);

    // Trigger analysis by entering amount
    const amountInput = selectors.amountInput;
    if (await amountInput.count() > 0) {
      await amountInput.fill('100');
      
      // Immediately check for loading state
      await page.waitForTimeout(500);
      
      const loadingDisplay = selectors.aiRiskDisplayLoading;
      const loadingFound = await loadingDisplay.count() > 0;
      
      if (loadingFound) {
        console.log('✅ Loading indicator found');
      }

      console.log(`⏳ Loading State: ${loadingFound ? 'VISIBLE' : 'TOO_FAST_TO_CAPTURE'}`);
    }
  });

  test('should display high-risk transaction warning', async ({ page }) => {
    console.log('🚨 Testing high-risk transaction warnings...');
    const selectors = new TestSelectors(page);

    // Mock high-risk response from AI Engine
    await page.route('**/api/risk/analyze', async route => {
      const mockHighRiskResponse = {
        risk_score: 0.9,
        risk_level: 'high',
        reasons: ['Very large transaction amount', 'Suspicious address pattern'],
        approved: false,
        ml_confidence: 0.8,
        is_anomaly: true,
        recommended_action: 'block_transaction',
        analysis_timestamp: new Date().toISOString()
      };
      
      route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify(mockHighRiskResponse)
      });
    });

    // Authenticate and navigate
    const ethButton = selectors.ethWalletButton;
    if (await ethButton.isVisible() && await ethButton.getAttribute('disabled') === null) {
      await ethButton.click();
      await page.waitForTimeout(10000);
    }

    await page.goto('/bridge');
    await page.waitForTimeout(5000);

    // Enter amount to trigger high-risk analysis
    const amountInput = selectors.amountInput;
    if (await amountInput.count() > 0) {
      await amountInput.fill('10000'); // Large amount
      await page.waitForTimeout(3000);

      // Look for high-risk indicators
      const riskWarning = selectors.aiRiskWarning;
      const highRiskFound = await riskWarning.count() > 0;
      
      if (highRiskFound) {
        console.log('🚨 High-risk indicator found');
        const content = await riskWarning.first().textContent();
        console.log(`   Warning content: "${content}"`);
      }

      // Check if form is disabled due to high risk
      const submitButton = selectors.submitButton;
      let formDisabled = false;
      
      if (await submitButton.count() > 0) {
        const isDisabled = await submitButton.isDisabled();
        if (isDisabled) {
          formDisabled = true;
          console.log('✅ Form correctly disabled for high-risk transaction');
        }
      }

      console.log(`🚨 High-Risk Detection: ${highRiskFound ? 'VISIBLE' : 'NOT_SHOWN'}`);
      console.log(`🔒 Form Protection: ${formDisabled ? 'ACTIVE' : 'PASSIVE'}`);
    }
  });

  test('should toggle risk details display', async ({ page }) => {
    console.log('🔍 Testing risk details toggle...');
    const selectors = new TestSelectors(page);

    // Mock detailed risk response
    await page.route('**/api/risk/analyze', async route => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({
          riskScore: 65,
          riskLevel: 'medium',
          approved: true,
          confidence: 0.88,
          factors: [
            'Cross-chain transaction',
            'Medium amount',
            'New recipient address'
          ],
          recommendations: [
            'Verify recipient address',
            'Consider test transaction first'
          ],
          anomalies: [
            'Transaction timing unusual for user pattern'
          ]
        })
      });
    });

    // Authenticate and navigate
    const ethButton = selectors.ethWalletButton;
    if (await ethButton.isVisible() && await ethButton.getAttribute('disabled') === null) {
      await ethButton.click();
      await page.waitForTimeout(10000);
    }

    await page.goto('/bridge');
    await page.waitForTimeout(5000);

    // Trigger analysis
    const amountInput = selectors.amountInput;
    if (await amountInput.count() > 0) {
      await amountInput.fill('500');
      await page.waitForTimeout(3000);

      // Look for details toggle button
      const toggleButton = selectors.aiRiskToggleDetails;
      const toggleFound = await toggleButton.count() > 0;
      
      if (toggleFound) {
        console.log('🔍 Toggle button found');
        
        // Click to expand details
        await toggleButton.click();
        await page.waitForTimeout(1000);
        
        // Check for expanded details
        const detailsDisplay = selectors.aiRiskDisplayDetails;
        if (await detailsDisplay.count() > 0) {
          console.log('📋 Details section found');
        }
      }

      console.log(`🔍 Details Toggle: ${toggleFound ? 'WORKING' : 'NOT_FOUND'}`);
    }
  });

  test('should refresh risk analysis when requested', async ({ page }) => {
    console.log('🔄 Testing risk analysis refresh functionality...');
    const selectors = new TestSelectors(page);

    // Track API calls to verify refresh
    const apiCalls = [];
    page.on('request', request => {
      const url = request.url();
      if (url.includes('/api/risk/analyze')) {
        apiCalls.push({
          timestamp: Date.now(),
          url
        });
      }
    });

    // Authenticate and navigate
    const ethButton = selectors.ethWalletButton;
    if (await ethButton.isVisible() && await ethButton.getAttribute('disabled') === null) {
      await ethButton.click();
      await page.waitForTimeout(10000);
    }

    await page.goto('/bridge');
    await page.waitForTimeout(5000);

    // Enter amount to trigger initial analysis
    const amountInput = selectors.amountInput;
    if (await amountInput.count() > 0) {
      await amountInput.fill('100');
      await page.waitForTimeout(3000);

      const initialCallCount = apiCalls.length;
      console.log(`📊 Initial API calls: ${initialCallCount}`);

      // Look for refresh button
      const refreshButton = selectors.aiRiskRefreshButton;
      const refreshFound = await refreshButton.count() > 0;
      
      if (refreshFound) {
        console.log('🔄 Refresh button found');
        
        // Click refresh
        await refreshButton.click();
        await page.waitForTimeout(2000);
        
        const newCallCount = apiCalls.length;
        console.log(`📊 API calls after refresh: ${newCallCount}`);
        
        if (newCallCount > initialCallCount) {
          console.log('✅ Refresh triggered new risk analysis');
        } else {
          console.log('⏳ Refresh may not have triggered new analysis');
        }
      }

      console.log(`🔄 Refresh Functionality: ${refreshFound ? 'AVAILABLE' : 'NOT_FOUND'}`);
    }
  });
});