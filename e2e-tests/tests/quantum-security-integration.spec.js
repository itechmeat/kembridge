/**
 * E2E Tests for Quantum Security Integration (Phase 8.1.2)
 * 
 * These tests verify the quantum cryptography integration works correctly
 * in the browser and provides proper security indicators to users.
 */

import { test, expect } from '@playwright/test';
import { setupApiMonitoring, checkServicesHealth } from '../utils/api-helpers.js';
import { findElementWithFallback, checkSecurityIndicators } from '../utils/element-helpers.js';
import { API_ENDPOINTS, TIMEOUTS, FRONTEND_URL } from '../utils/constants.js';

// Test configuration constants
const QUANTUM_SECURITY_SELECTORS = {
  securityIndicator: '[data-testid="security-indicator"]',
  quantumProtectionDisplay: '[data-testid="quantum-protection-display"]',
  quantumStatus: '[data-testid="quantum-status"]',
  quantumKeyId: '[data-testid="quantum-key-id"]',
  encryptionScheme: '[data-testid="encryption-scheme"]',
  keyRotationStatus: '[data-testid="key-rotation-status"]',
  protectedTransactions: '[data-testid="protected-transactions"]',
  quantumIcon: '[data-testid="quantum-icon"]',
  securityLevel: '[data-testid="security-level"]',
};

const QUANTUM_TEST_DATA = {
  expectedEncryptionScheme: 'ML-KEM-1024',
  expectedSecurityLevel: 'Quantum Protected',
  minimumKeyIdLength: 8,
  maxLoadingTime: 5000,
};

test.describe('Quantum Security Integration Tests', () => {
  
  test.beforeEach(async ({ page }) => {
    // Navigate to SecurityTestPage where quantum components are visible
    await page.goto(`${FRONTEND_URL}/security-test`);
    await page.waitForLoadState('networkidle');
    
    // Wait for the security components to initialize
    await page.waitForTimeout(2000);
  });

  test('should display quantum protection status correctly', async ({ page }) => {
    // Check if quantum protection display is visible
    const quantumDisplay = page.locator(QUANTUM_SECURITY_SELECTORS.quantumProtectionDisplay);
    await expect(quantumDisplay).toBeVisible({ timeout: QUANTUM_TEST_DATA.maxLoadingTime });

    // Verify quantum protection is active
    const quantumStatus = page.locator(QUANTUM_SECURITY_SELECTORS.quantumStatus);
    await expect(quantumStatus).toContainText('Active');

    // Verify encryption scheme is ML-KEM-1024
    const encryptionScheme = page.locator(QUANTUM_SECURITY_SELECTORS.encryptionScheme);
    await expect(encryptionScheme).toContainText(QUANTUM_TEST_DATA.expectedEncryptionScheme);

    // Verify quantum icon is displayed
    const quantumIcon = page.locator(QUANTUM_SECURITY_SELECTORS.quantumIcon);
    await expect(quantumIcon).toBeVisible();
  });

  test('should show quantum key information', async ({ page }) => {
    // Wait for quantum key ID to be displayed
    const keyIdElement = page.locator(QUANTUM_SECURITY_SELECTORS.quantumKeyId);
    await expect(keyIdElement).toBeVisible({ timeout: QUANTUM_TEST_DATA.maxLoadingTime });

    // Verify key ID format (should be truncated UUID format)
    const keyIdText = await keyIdElement.textContent();
    expect(keyIdText).toBeTruthy();
    expect(keyIdText.length).toBeGreaterThanOrEqual(QUANTUM_TEST_DATA.minimumKeyIdLength);
    
    // Should contain ellipsis for truncated UUID
    expect(keyIdText).toMatch(/^[a-f0-9]{8}\.\.\..*$/);
  });

  test('should display key rotation status', async ({ page }) => {
    // Check key rotation status element within quantum protection display
    const quantumDisplay = page.locator(QUANTUM_SECURITY_SELECTORS.quantumProtectionDisplay);
    const rotationStatus = quantumDisplay.locator(QUANTUM_SECURITY_SELECTORS.keyRotationStatus);
    await expect(rotationStatus).toBeVisible({ timeout: QUANTUM_TEST_DATA.maxLoadingTime });

    // Verify rotation status shows appropriate information
    const rotationText = await rotationStatus.textContent();
    expect(rotationText).toBeTruthy();
    
    // Should show either a time-based status or "Never" for new installations
    expect(rotationText).toMatch(/(Never|Today|Yesterday|\d+\s+(days?|weeks?|months?)\s+ago)/);
  });

  test('should show protected transactions count', async ({ page }) => {
    const protectedTxElement = page.locator(QUANTUM_SECURITY_SELECTORS.protectedTransactions);
    await expect(protectedTxElement).toBeVisible({ timeout: QUANTUM_TEST_DATA.maxLoadingTime });

    // Verify protected transactions count is displayed
    const txCountText = await protectedTxElement.textContent();
    expect(txCountText).toBeTruthy();
    
    // Should be a number (possibly with K/M suffix for large numbers)
    expect(txCountText).toMatch(/^\d+(\.\d+)?[KM]?$/);
  });

  test('should integrate with security indicator component', async ({ page }) => {
    // Check main security indicator (first one)
    const securityIndicator = page.locator(QUANTUM_SECURITY_SELECTORS.securityIndicator).first();
    await expect(securityIndicator).toBeVisible({ timeout: QUANTUM_TEST_DATA.maxLoadingTime });

    // Verify security level shows quantum protection
    const securityLevel = page.locator(QUANTUM_SECURITY_SELECTORS.securityLevel);
    await expect(securityLevel).toContainText(QUANTUM_TEST_DATA.expectedSecurityLevel);

    // Verify quantum protection details are shown
    await expect(securityIndicator).toContainText('ML-KEM-1024');
    await expect(securityIndicator).toContainText('Quantum Protection');
  });

  test('should show quantum protection animations when active', async ({ page }) => {
    // Wait for quantum protection display
    const quantumDisplay = page.locator(QUANTUM_SECURITY_SELECTORS.quantumProtectionDisplay);
    await expect(quantumDisplay).toBeVisible({ timeout: QUANTUM_TEST_DATA.maxLoadingTime });

    // Check if active quantum protection has animation classes
    const classList = await quantumDisplay.getAttribute('class');
    expect(classList).toContain('quantum-protection--active');

    // Verify quantum border animation is present
    const computedStyle = await quantumDisplay.evaluate(el => {
      return window.getComputedStyle(el).animation;
    });
    expect(computedStyle).toContain('quantum-border');
  });

  test('should handle quantum protection state changes', async ({ page }) => {
    // Initial state - should be active
    const quantumStatus = page.locator(QUANTUM_SECURITY_SELECTORS.quantumStatus);
    await expect(quantumStatus).toContainText('Active');

    // Monitor for any state changes during page interaction
    let statusChanges = [];
    page.on('console', msg => {
      if (msg.text().includes('quantum') || msg.text().includes('security')) {
        statusChanges.push(msg.text());
      }
    });

    // Interact with the page to trigger potential state changes
    await page.click('[data-testid="swap-form"]', { timeout: 5000 }).catch(() => {
      // Swap form might not be present, that's OK
    });

    // Wait a bit for any state changes
    await page.waitForTimeout(2000);

    // Quantum protection should remain active
    await expect(quantumStatus).toContainText('Active');
  });

  test('should display correct quantum security metrics', async ({ page }) => {
    // Check all quantum metrics are displayed correctly
    const quantumDisplay = page.locator(QUANTUM_SECURITY_SELECTORS.quantumProtectionDisplay);
    await expect(quantumDisplay).toBeVisible({ timeout: QUANTUM_TEST_DATA.maxLoadingTime });

    // Verify all required quantum information cards are present
    const expectedCards = [
      'Encryption Scheme',
      'Key Information', 
      'Key Rotation',
      'Protected'
    ];

    for (const cardTitle of expectedCards) {
      await expect(quantumDisplay).toContainText(cardTitle);
    }

    // Verify security strength indicator
    await expect(quantumDisplay).toContainText('1024-bit');
    
    // Verify informational text about quantum protection
    await expect(quantumDisplay).toContainText('post-quantum cryptography');
    await expect(quantumDisplay).toContainText('quantum computer attacks');
  });

  test('should show performance indicators for quantum operations', async ({ page }) => {
    // Look for performance-related quantum indicators
    const quantumDisplay = page.locator(QUANTUM_SECURITY_SELECTORS.quantumProtectionDisplay);
    await expect(quantumDisplay).toBeVisible({ timeout: QUANTUM_TEST_DATA.maxLoadingTime });

    // Check if performance card is present (optional feature)
    const performanceCard = quantumDisplay.locator('[data-testid="quantum-performance"]');
    const hasPerformanceCard = await performanceCard.isVisible().catch(() => false);

    if (hasPerformanceCard) {
      // Verify performance metrics format
      const performanceText = await performanceCard.textContent();
      expect(performanceText).toMatch(/\d+[K]?\s+ops\/s/); // Should show operations per second
    }
  });

  test('should maintain quantum protection during navigation', async ({ page }) => {
    // Initial quantum protection check
    const quantumStatus = page.locator(QUANTUM_SECURITY_SELECTORS.quantumStatus);
    await expect(quantumStatus).toContainText('Active');

    // Navigate to different sections of the app (if they exist)
    const navigationLinks = [
      '[data-testid="bridge-tab"]',
      '[data-testid="history-tab"]',
      '[data-testid="settings-tab"]'
    ];

    for (const linkSelector of navigationLinks) {
      const link = page.locator(linkSelector);
      const linkExists = await link.isVisible().catch(() => false);
      
      if (linkExists) {
        await link.click();
        await page.waitForTimeout(1000);
        
        // Quantum protection should remain active
        await expect(quantumStatus).toContainText('Active', { timeout: 3000 });
      }
    }
  });

  test('should validate quantum protection accessibility', async ({ page }) => {
    // Check accessibility attributes on quantum components
    const quantumDisplay = page.locator(QUANTUM_SECURITY_SELECTORS.quantumProtectionDisplay);
    await expect(quantumDisplay).toBeVisible({ timeout: QUANTUM_TEST_DATA.maxLoadingTime });

    // Verify ARIA labels and roles
    const ariaLabel = await quantumDisplay.getAttribute('aria-label');
    if (ariaLabel) {
      expect(ariaLabel).toContain('quantum');
    }

    // Check that quantum status is announced to screen readers
    const quantumStatus = page.locator(QUANTUM_SECURITY_SELECTORS.quantumStatus);
    const statusRole = await quantumStatus.getAttribute('role');
    if (statusRole) {
      expect(['status', 'alert', 'region']).toContain(statusRole);
    }

    // Verify color contrast for security indicators
    const securityLevel = page.locator(QUANTUM_SECURITY_SELECTORS.securityLevel);
    const computedStyle = await securityLevel.evaluate(el => {
      const styles = window.getComputedStyle(el);
      return {
        color: styles.color,
        backgroundColor: styles.backgroundColor
      };
    });

    // Basic color validation (colors should be defined)
    expect(computedStyle.color).not.toBe('');
    expect(computedStyle.backgroundColor).not.toBe('');
  });

  test('should handle quantum security errors gracefully', async ({ page }) => {
    // Monitor console for quantum-related errors
    const errors = [];
    page.on('console', msg => {
      if (msg.type() === 'error' && 
          (msg.text().includes('quantum') || msg.text().includes('crypto'))) {
        errors.push(msg.text());
      }
    });

    // Wait for quantum components to load
    const quantumDisplay = page.locator(QUANTUM_SECURITY_SELECTORS.quantumProtectionDisplay);
    await expect(quantumDisplay).toBeVisible({ timeout: QUANTUM_TEST_DATA.maxLoadingTime });

    // Should not have critical quantum-related errors
    expect(errors.length).toBe(0);

    // If quantum protection fails, should show appropriate fallback
    const quantumStatus = page.locator(QUANTUM_SECURITY_SELECTORS.quantumStatus);
    const statusText = await quantumStatus.textContent();
    
    // Status should be either Active or show a proper error state (not undefined/null)
    expect(statusText).toBeTruthy();
    expect(statusText).not.toBe('undefined');
    expect(statusText).not.toBe('null');
  });
});

// Additional integration tests for specific quantum workflows
test.describe('Quantum Workflow Integration', () => {
  
  test('should maintain quantum protection during mock transaction flow', async ({ page }) => {
    // Try to navigate to bridge page, but fallback to security-test if it fails
    try {
      await page.goto(`${FRONTEND_URL}/bridge`, { timeout: 10000 });
      await page.waitForLoadState('networkidle', { timeout: 10000 });
    } catch (error) {
      console.log('Bridge page not available, using security-test page for quantum verification');
      await page.goto(`${FRONTEND_URL}/security-test`);
      await page.waitForLoadState('networkidle');
    }

    // Initial quantum protection verification
    const quantumStatus = page.locator(QUANTUM_SECURITY_SELECTORS.quantumStatus);
    await expect(quantumStatus).toContainText('Active', { timeout: QUANTUM_TEST_DATA.maxLoadingTime });

    // Try to interact with swap form if present
    const swapForm = page.locator('[data-testid="swap-form"]');
    const swapFormExists = await swapForm.isVisible().catch(() => false);

    if (swapFormExists) {
      // Enter mock transaction data
      const fromAmountInput = page.locator('[data-testid="from-amount-input"]');
      const fromAmountExists = await fromAmountInput.isVisible().catch(() => false);
      
      if (fromAmountExists) {
        await fromAmountInput.fill('1.0');
        await page.waitForTimeout(1000);
        
        // Quantum protection should remain active during input
        await expect(quantumStatus).toContainText('Active');
      }
    } else {
      // If no swap form, just verify quantum protection remains active
      console.log('No swap form found, verifying quantum protection stability');
      await page.waitForTimeout(2000);
      await expect(quantumStatus).toContainText('Active');
    }
  });

  test('should show quantum protection status in security components', async ({ page }) => {
    await page.goto(`${FRONTEND_URL}/security-test`);
    await page.waitForLoadState('networkidle');

    // Look for security indicator with quantum information
    const securityIndicator = page.locator(QUANTUM_SECURITY_SELECTORS.securityIndicator);
    const indicatorExists = await securityIndicator.isVisible().catch(() => false);

    if (indicatorExists) {
      // Should show quantum protection details
      await expect(securityIndicator).toContainText('Quantum', { timeout: QUANTUM_TEST_DATA.maxLoadingTime });
      
      // Should show ML-KEM encryption scheme
      await expect(securityIndicator).toContainText('ML-KEM', { timeout: 3000 });
    }
  });
});