/**
 * DOM element interaction helpers
 */
import { SELECTORS, TIMEOUTS } from './constants.js';

/**
 * Find element using multiple selectors
 * @param {import('@playwright/test').Page} page 
 * @param {string|Array<string>} selectors 
 * @returns {Promise<{element: import('@playwright/test').Locator|null, selectorUsed: string|null}>}
 */
export async function findElementWithFallback(page, selectors) {
  const selectorArray = Array.isArray(selectors) ? selectors : [selectors];
  
  for (const selector of selectorArray) {
    const element = page.locator(selector);
    const count = await element.count();
    
    if (count > 0) {
      return {
        element: element.first(),
        selectorUsed: selector
      };
    }
  }
  
  return {
    element: null,
    selectorUsed: null
  };
}

/**
 * Wait for bridge form to be accessible (not showing auth required)
 * @param {import('@playwright/test').Page} page 
 * @param {number} timeout 
 * @returns {Promise<boolean>} Form is accessible
 */
export async function waitForBridgeFormAccessible(page, timeout = TIMEOUTS.LONG) {
  const startTime = Date.now();
  
  while (Date.now() - startTime < timeout) {
    const authRequired = await page.locator(SELECTORS.AUTH_REQUIRED).isVisible().catch(() => false);
    
    if (!authRequired) {
      return true;
    }
    
    await page.waitForTimeout(TIMEOUTS.SHORT);
  }
  
  return false;
}

/**
 * Navigate to bridge page using multiple selector attempts
 * @param {import('@playwright/test').Page} page 
 * @returns {Promise<boolean>} Navigation success
 */
export async function navigateToBridge(page) {
  console.log('🌉 Navigating to bridge page...');
  
  // Try navigation button first
  const navResult = await findElementWithFallback(page, SELECTORS.SWAP_NAV_BUTTON);
  
  if (navResult.element) {
    console.log(`✅ Using navigation button: ${navResult.selectorUsed}`);
    await navResult.element.click();
    await page.waitForTimeout(TIMEOUTS.MEDIUM);
    return true;
  }
  
  // Fallback to direct navigation
  console.log('🔄 Fallback: Direct navigation to /bridge');
  await page.goto('/bridge');
  await page.waitForTimeout(TIMEOUTS.LONG);
  return true;
}

/**
 * Get bridge form elements
 * @param {import('@playwright/test').Page} page 
 * @returns {Promise<Object>} Form elements
 */
export async function getBridgeFormElements(page) {
  const elements = {};
  
  // Token selectors
  const tokenSelectors = page.locator(SELECTORS.TOKEN_SELECTOR);
  elements.tokenSelectorCount = await tokenSelectors.count();
  elements.tokenSelectors = {
    from: elements.tokenSelectorCount > 0 ? tokenSelectors.first() : null,
    to: elements.tokenSelectorCount > 1 ? tokenSelectors.last() : null
  };
  
  // Amount input
  const amountResult = await findElementWithFallback(page, SELECTORS.AMOUNT_INPUT);
  elements.amountInput = amountResult.element;
  elements.amountInputSelector = amountResult.selectorUsed;
  
  // Submit button
  const submitResult = await findElementWithFallback(page, SELECTORS.SUBMIT_BUTTON);
  elements.submitButton = submitResult.element;
  elements.submitButtonSelector = submitResult.selectorUsed;
  
  // Direction switch
  const directionResult = await findElementWithFallback(page, SELECTORS.DIRECTION_SWITCH);
  elements.directionSwitch = directionResult.element;
  elements.directionSwitchSelector = directionResult.selectorUsed;
  
  return elements;
}

/**
 * Fill amount input with validation
 * @param {import('@playwright/test').Page} page 
 * @param {string} amount 
 * @returns {Promise<boolean>} Success status
 */
export async function fillAmount(page, amount) {
  const amountResult = await findElementWithFallback(page, SELECTORS.AMOUNT_INPUT);
  
  if (!amountResult.element) {
    console.log('❌ Amount input not found');
    return false;
  }
  
  console.log(`💰 Entering amount: ${amount}`);
  await amountResult.element.fill(amount);
  await page.waitForTimeout(TIMEOUTS.SHORT);
  
  // Verify the value was set
  const actualValue = await amountResult.element.inputValue();
  const success = actualValue === amount;
  
  if (success) {
    console.log(`✅ Amount entered successfully: ${actualValue}`);
  } else {
    console.log(`❌ Amount entry failed. Expected: ${amount}, Got: ${actualValue}`);
  }
  
  return success;
}

/**
 * Check for error elements on the page
 * @param {import('@playwright/test').Page} page 
 * @returns {Promise<Array>} Array of error messages
 */
export async function getErrorMessages(page) {
  const errorElements = page.locator(SELECTORS.ERROR_ELEMENTS);
  const errorCount = await errorElements.count();
  const errors = [];
  
  for (let i = 0; i < Math.min(errorCount, 5); i++) {
    const errorText = await errorElements.nth(i).textContent();
    if (errorText && errorText.trim()) {
      errors.push(errorText.trim());
    }
  }
  
  return errors;
}

/**
 * Check for security indicators on the page
 * @param {import('@playwright/test').Page} page 
 * @returns {Promise<Object>} Security indicators found
 */
export async function checkSecurityIndicators(page) {
  const results = {
    quantumFound: false,
    riskFound: false,
    foundIndicators: []
  };
  
  // Check quantum security indicators
  for (const indicator of SELECTORS.SECURITY_INDICATORS) {
    const elements = page.locator(indicator);
    const count = await elements.count();
    
    if (count > 0) {
      results.quantumFound = true;
      const text = await elements.first().textContent();
      results.foundIndicators.push({
        type: 'quantum',
        selector: indicator,
        text: text?.trim()
      });
    }
  }
  
  // Check risk analysis indicators
  for (const indicator of SELECTORS.RISK_INDICATORS) {
    const elements = page.locator(indicator);
    const count = await elements.count();
    
    if (count > 0) {
      results.riskFound = true;
      const text = await elements.first().textContent();
      results.foundIndicators.push({
        type: 'risk',
        selector: indicator,
        text: text?.trim()
      });
    }
  }
  
  return results;
}

/**
 * Wait for tokens to load by checking API calls and UI elements
 * @param {import('@playwright/test').Page} page 
 * @param {Object} monitoring - API monitoring object
 * @param {number} timeout 
 * @returns {Promise<boolean>} Tokens loaded successfully
 */
export async function waitForTokensLoaded(page, monitoring, timeout = TIMEOUTS.LONG) {
  const startTime = Date.now();
  
  while (Date.now() - startTime < timeout) {
    // Check if tokens API was called
    const tokenCalls = monitoring.getBridgeCalls().filter(call => 
      call.url.includes('/bridge/tokens') || call.url.includes('/tokens')
    );
    
    if (tokenCalls.length > 0) {
      console.log('✅ Bridge tokens API called');
      
      // Wait a bit more for UI to update
      await page.waitForTimeout(TIMEOUTS.MEDIUM);
      
      // Check if token selectors are present
      const tokenSelectors = page.locator(SELECTORS.TOKEN_SELECTOR);
      const count = await tokenSelectors.count();
      
      if (count >= 2) {
        console.log(`✅ Token selectors loaded: ${count} found`);
        return true;
      }
    }
    
    await page.waitForTimeout(TIMEOUTS.SHORT);
  }
  
  console.log('❌ Tokens failed to load within timeout');
  return false;
}