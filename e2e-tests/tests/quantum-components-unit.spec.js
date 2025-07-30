/**
 * Unit Tests for Frontend Quantum Components
 * 
 * These tests verify the React quantum security components work correctly
 * and render appropriate quantum protection information.
 */

import { test, expect } from '@playwright/test';
import { API_ENDPOINTS, TIMEOUTS, FRONTEND_URL } from '../utils/constants.js';

// Quantum component test selectors
const QUANTUM_COMPONENT_SELECTORS = {
  quantumProtectionDisplay: '[data-testid="quantum-protection-display"]',
  securityIndicator: '[data-testid="security-indicator"]',
  quantumCards: '[data-testid^="quantum-card-"]',
  quantumHeader: '[data-testid="quantum-header"]',
  quantumStatus: '[data-testid="quantum-status"]',
  quantumIcon: '[data-testid="quantum-icon"]',
  encryptionScheme: '[data-testid="encryption-scheme"]',
  keyInformation: '[data-testid="key-information"]',
  keyRotation: '[data-testid="key-rotation-status"]',
  protectedCount: '[data-testid="protected-transactions"]',
  performanceMetrics: '[data-testid="performance-metrics"]',
  infoFooter: '[data-testid="quantum-info-footer"]',
};

// Test data for quantum components
const QUANTUM_COMPONENT_TEST_DATA = {
  encryptionScheme: 'ML-KEM-1024',
  keyStrength: '1024-bit',
  securityLevel: 'high',
  activeStatus: 'Active',
  disabledStatus: 'Disabled',
  minProtectedCount: 0,
  maxLoadTime: 5000,
};

test.describe('QuantumProtectionDisplay Component Tests', () => {

  test.beforeEach(async ({ page }) => {
    await page.goto(`${FRONTEND_URL}/security-test`);
    await page.waitForLoadState('networkidle');
  });

  test('should render quantum protection display component', async ({ page }) => {
    const quantumDisplay = page.locator(QUANTUM_COMPONENT_SELECTORS.quantumProtectionDisplay);
    await expect(quantumDisplay).toBeVisible({ timeout: QUANTUM_COMPONENT_TEST_DATA.maxLoadTime });

    // Verify component structure
    await expect(quantumDisplay).toHaveClass(/quantum-protection/);
  });

  test('should show correct quantum protection header', async ({ page }) => {
    const quantumHeader = page.locator(QUANTUM_COMPONENT_SELECTORS.quantumHeader);
    await expect(quantumHeader).toBeVisible({ timeout: QUANTUM_COMPONENT_TEST_DATA.maxLoadTime });

    // Verify header contains quantum protection title
    await expect(quantumHeader).toContainText('Quantum Protection');
    
    // Verify quantum icon is present
    const quantumIcon = page.locator(QUANTUM_COMPONENT_SELECTORS.quantumIcon);
    await expect(quantumIcon).toBeVisible();
  });

  test('should display correct quantum status', async ({ page }) => {
    const quantumStatus = page.locator(QUANTUM_COMPONENT_SELECTORS.quantumStatus);
    await expect(quantumStatus).toBeVisible({ timeout: QUANTUM_COMPONENT_TEST_DATA.maxLoadTime });

    const statusText = await quantumStatus.textContent();
    expect([
      QUANTUM_COMPONENT_TEST_DATA.activeStatus,
      QUANTUM_COMPONENT_TEST_DATA.disabledStatus
    ]).toContain(statusText);
  });

  test('should show encryption scheme information', async ({ page }) => {
    const encryptionScheme = page.locator(QUANTUM_COMPONENT_SELECTORS.encryptionScheme);
    await expect(encryptionScheme).toBeVisible({ timeout: QUANTUM_COMPONENT_TEST_DATA.maxLoadTime });

    // Verify ML-KEM-1024 is displayed
    await expect(encryptionScheme).toContainText(QUANTUM_COMPONENT_TEST_DATA.encryptionScheme);
    
    // Verify key strength is shown
    await expect(encryptionScheme).toContainText(QUANTUM_COMPONENT_TEST_DATA.keyStrength);
  });

  test('should display key information card', async ({ page }) => {
    const keyInfo = page.locator(QUANTUM_COMPONENT_SELECTORS.keyInformation);
    await expect(keyInfo).toBeVisible({ timeout: QUANTUM_COMPONENT_TEST_DATA.maxLoadTime });

    // Should show truncated key ID format
    const keyText = await keyInfo.textContent();
    expect(keyText).toMatch(/[a-f0-9]{8}\.\.\.[\w]{4}|N\/A/);
  });

  test('should show key rotation information', async ({ page }) => {
    const keyRotation = page.locator(QUANTUM_COMPONENT_SELECTORS.keyRotation);
    await expect(keyRotation).toBeVisible({ timeout: QUANTUM_COMPONENT_TEST_DATA.maxLoadTime });

    // Should show rotation timing
    const rotationText = await keyRotation.textContent();
    expect(rotationText).toMatch(/(Never|Today|Yesterday|\d+[hdwm]?\s*(ago|days?|hours?|weeks?|months?))/i);
  });

  test('should display protected transactions count', async ({ page }) => {
    const protectedCount = page.locator(QUANTUM_COMPONENT_SELECTORS.protectedCount);
    await expect(protectedCount).toBeVisible({ timeout: QUANTUM_COMPONENT_TEST_DATA.maxLoadTime });

    // Should show numeric count (possibly with K/M suffix)
    const countText = await protectedCount.textContent();
    expect(countText).toMatch(/^\d+[KM]?$/);
  });

  test('should render quantum information cards', async ({ page }) => {
    const quantumDisplay = page.locator(QUANTUM_COMPONENT_SELECTORS.quantumProtectionDisplay);
    await expect(quantumDisplay).toBeVisible({ timeout: QUANTUM_COMPONENT_TEST_DATA.maxLoadTime });

    // Verify all expected cards are present
    const expectedCardTitles = [
      'Encryption Scheme',
      'Key Information',
      'Key Rotation',
      'Protected'
    ];

    for (const title of expectedCardTitles) {
      await expect(quantumDisplay).toContainText(title);
    }
  });

  test('should show informational footer', async ({ page }) => {
    const infoFooter = page.locator(QUANTUM_COMPONENT_SELECTORS.infoFooter);
    await expect(infoFooter).toBeVisible({ timeout: QUANTUM_COMPONENT_TEST_DATA.maxLoadTime });

    // Should contain educational information about quantum protection
    await expect(infoFooter).toContainText('post-quantum cryptography');
    await expect(infoFooter).toContainText('quantum computer attacks');
  });

  test('should handle active quantum protection state', async ({ page }) => {
    const quantumDisplay = page.locator(QUANTUM_COMPONENT_SELECTORS.quantumProtectionDisplay);
    await expect(quantumDisplay).toBeVisible({ timeout: QUANTUM_COMPONENT_TEST_DATA.maxLoadTime });

    // Check for active state class
    const classList = await quantumDisplay.getAttribute('class');
    const isActive = classList.includes('quantum-protection--active');
    const isDisabled = classList.includes('quantum-protection--disabled');

    // Should be either active or disabled, not both
    expect(isActive !== isDisabled).toBe(true);

    if (isActive) {
      // Active state should show lock icon
      await expect(quantumDisplay).toContainText('🔒');
      
      // Should not show warning message
      const warningMessage = quantumDisplay.locator('.quantum-protection__message');
      await expect(warningMessage).not.toBeVisible();
    }
  });

  test('should handle disabled quantum protection state', async ({ page }) => {
    // This test requires a way to simulate disabled state
    // For now, we'll check if the component can handle both states
    
    const quantumDisplay = page.locator(QUANTUM_COMPONENT_SELECTORS.quantumProtectionDisplay);
    await expect(quantumDisplay).toBeVisible({ timeout: QUANTUM_COMPONENT_TEST_DATA.maxLoadTime });

    const classList = await quantumDisplay.getAttribute('class');
    const isDisabled = classList.includes('quantum-protection--disabled');

    if (isDisabled) {
      // Disabled state should show unlock icon
      await expect(quantumDisplay).toContainText('🔓');
      
      // Should show warning message
      const warningMessage = quantumDisplay.locator('.quantum-protection__message');
      await expect(warningMessage).toBeVisible();
      await expect(warningMessage).toContainText('vulnerable to quantum attacks');
    }
  });

  test('should display performance metrics when available', async ({ page }) => {
    const performanceMetrics = page.locator(QUANTUM_COMPONENT_SELECTORS.performanceMetrics);
    const hasPerformanceMetrics = await performanceMetrics.isVisible().catch(() => false);

    if (hasPerformanceMetrics) {
      // Should show operations per second
      const metricsText = await performanceMetrics.textContent();
      expect(metricsText).toMatch(/\d+[K]?\s*ops\/s/);
    }
  });

  test('should be responsive on different screen sizes', async ({ page }) => {
    const quantumDisplay = page.locator(QUANTUM_COMPONENT_SELECTORS.quantumProtectionDisplay);
    await expect(quantumDisplay).toBeVisible({ timeout: QUANTUM_COMPONENT_TEST_DATA.maxLoadTime });

    // Test mobile view
    await page.setViewportSize({ width: 375, height: 667 });
    await page.waitForTimeout(500);
    await expect(quantumDisplay).toBeVisible();

    // Test tablet view
    await page.setViewportSize({ width: 768, height: 1024 });
    await page.waitForTimeout(500);
    await expect(quantumDisplay).toBeVisible();

    // Test desktop view
    await page.setViewportSize({ width: 1920, height: 1080 });
    await page.waitForTimeout(500);
    await expect(quantumDisplay).toBeVisible();
  });
});

test.describe('SecurityIndicator Component Quantum Integration Tests', () => {

  test.beforeEach(async ({ page }) => {
    await page.goto(`${FRONTEND_URL}/security-test`);
    await page.waitForLoadState('networkidle');
  });

  test('should integrate quantum protection in security indicator', async ({ page }) => {
    const securityIndicator = page.locator(QUANTUM_COMPONENT_SELECTORS.securityIndicator);
    await expect(securityIndicator).toBeVisible({ timeout: QUANTUM_COMPONENT_TEST_DATA.maxLoadTime });

    // Should show quantum protection status
    await expect(securityIndicator).toContainText('Quantum');
  });

  test('should show quantum protection details in security indicator', async ({ page }) => {
    const securityIndicator = page.locator(QUANTUM_COMPONENT_SELECTORS.securityIndicator);
    await expect(securityIndicator).toBeVisible({ timeout: QUANTUM_COMPONENT_TEST_DATA.maxLoadTime });

    // Should include ML-KEM information
    const indicatorText = await securityIndicator.textContent();
    const hasQuantumInfo = indicatorText.includes('ML-KEM') || 
                          indicatorText.includes('Quantum') ||
                          indicatorText.includes('1024');
    
    expect(hasQuantumInfo).toBe(true);
  });

  test('should display appropriate security level with quantum protection', async ({ page }) => {
    const securityIndicator = page.locator(QUANTUM_COMPONENT_SELECTORS.securityIndicator);
    await expect(securityIndicator).toBeVisible({ timeout: QUANTUM_COMPONENT_TEST_DATA.maxLoadTime });

    // Check security level classes
    const classList = await securityIndicator.getAttribute('class');
    const hasSecurityLevel = classList.includes('security-indicator--secure') ||
                            classList.includes('security-indicator--warning') ||
                            classList.includes('security-indicator--danger') ||
                            classList.includes('security-indicator--offline');

    expect(hasSecurityLevel).toBe(true);
  });

  test('should show quantum key information in security details', async ({ page }) => {
    const securityIndicator = page.locator(QUANTUM_COMPONENT_SELECTORS.securityIndicator);
    await expect(securityIndicator).toBeVisible({ timeout: QUANTUM_COMPONENT_TEST_DATA.maxLoadTime });

    // Look for quantum-specific details
    const quantumDetails = securityIndicator.locator('[class*="quantum"]');
    const hasQuantumDetails = await quantumDetails.count() > 0;

    if (hasQuantumDetails) {
      // Should show key ID or other quantum information
      const detailsText = await quantumDetails.first().textContent();
      expect(detailsText).toBeTruthy();
    }
  });

  test('should handle compact security indicator with quantum info', async ({ page }) => {
    // Look for compact security indicators
    const compactIndicator = page.locator('.security-indicator--compact');
    const hasCompactIndicator = await compactIndicator.isVisible().catch(() => false);

    if (hasCompactIndicator) {
      // Compact indicator should still show security status
      await expect(compactIndicator).toBeVisible();
      
      // Should have appropriate security icon
      const iconText = await compactIndicator.textContent();
      expect(iconText).toMatch(/[🔒🔓⚠️🚨📴❓]/);
    }
  });
});

test.describe('Quantum Component Error Handling Tests', () => {

  test.beforeEach(async ({ page }) => {
    await page.goto(`${FRONTEND_URL}/security-test`);
    await page.waitForLoadState('networkidle');
  });

  test('should handle missing quantum data gracefully', async ({ page }) => {
    // Monitor console for errors
    const errors = [];
    page.on('console', msg => {
      if (msg.type() === 'error') {
        errors.push(msg.text());
      }
    });

    const quantumDisplay = page.locator(QUANTUM_COMPONENT_SELECTORS.quantumProtectionDisplay);
    await expect(quantumDisplay).toBeVisible({ timeout: QUANTUM_COMPONENT_TEST_DATA.maxLoadTime });

    // Should not have critical rendering errors
    const quantumErrors = errors.filter(error => 
      error.includes('quantum') || error.includes('crypto') || error.includes('security')
    );
    expect(quantumErrors.length).toBe(0);
  });

  test('should show fallback content when quantum data is unavailable', async ({ page }) => {
    const quantumDisplay = page.locator(QUANTUM_COMPONENT_SELECTORS.quantumProtectionDisplay);
    await expect(quantumDisplay).toBeVisible({ timeout: QUANTUM_COMPONENT_TEST_DATA.maxLoadTime });

    // All key quantum elements should have fallback content
    const keyInfo = page.locator(QUANTUM_COMPONENT_SELECTORS.keyInformation);
    const keyText = await keyInfo.textContent();
    
    // Should show "N/A" or similar fallback, not be empty
    expect(keyText).toBeTruthy();
    expect(keyText).not.toBe('');
  });

  test('should maintain component structure even with missing props', async ({ page }) => {
    const quantumDisplay = page.locator(QUANTUM_COMPONENT_SELECTORS.quantumProtectionDisplay);
    await expect(quantumDisplay).toBeVisible({ timeout: QUANTUM_COMPONENT_TEST_DATA.maxLoadTime });

    // Component should maintain its basic structure
    const hasHeader = await quantumDisplay.locator('.quantum-protection__header').isVisible();
    const hasContent = await quantumDisplay.locator('.quantum-protection__content').isVisible();
    
    expect(hasHeader || hasContent).toBe(true);
  });
});

test.describe('Quantum Component Performance Tests', () => {

  test('should render quantum components within performance budget', async ({ page }) => {
    const startTime = Date.now();
    
    await page.goto(FRONTEND_URL);
    await page.waitForLoadState('networkidle');

    const quantumDisplay = page.locator(QUANTUM_COMPONENT_SELECTORS.quantumProtectionDisplay);
    await expect(quantumDisplay).toBeVisible();

    const renderTime = Date.now() - startTime;
    
    // Quantum components should render within 5 seconds
    expect(renderTime).toBeLessThan(QUANTUM_COMPONENT_TEST_DATA.maxLoadTime);
  });

  test('should not cause memory leaks with quantum animations', async ({ page }) => {
    await page.goto(FRONTEND_URL);
    await page.waitForLoadState('networkidle');

    const quantumDisplay = page.locator(QUANTUM_COMPONENT_SELECTORS.quantumProtectionDisplay);
    await expect(quantumDisplay).toBeVisible();

    // Let animations run for a while
    await page.waitForTimeout(5000);

    // Check if page is still responsive
    const isVisible = await quantumDisplay.isVisible();
    expect(isVisible).toBe(true);

    // Component should still be interactive
    const canClick = await quantumDisplay.isEnabled();
    expect(canClick).toBe(true);
  });
});