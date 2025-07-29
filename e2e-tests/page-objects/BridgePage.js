/**
 * Bridge Page Object Model
 * Encapsulates all bridge-related interactions and validations
 */
import { 
  getBridgeFormElements, 
  fillAmount, 
  findElementWithFallback, 
  waitForTokensLoaded,
  checkSecurityIndicators,
  getErrorMessages
} from '../utils/element-helpers.js';
import { SELECTORS, TIMEOUTS, TEST_DATA } from '../utils/constants.js';

export class BridgePage {
  constructor(page, monitoring) {
    this.page = page;
    this.monitoring = monitoring;
  }
  
  /**
   * Navigate to bridge page
   */
  async navigate() {
    const result = await findElementWithFallback(this.page, SELECTORS.SWAP_NAV_BUTTON);
    
    if (result.element) {
      await result.element.click();
      await this.page.waitForTimeout(TIMEOUTS.MEDIUM);
      return true;
    }
    
    // Fallback to direct navigation
    await this.page.goto('/bridge');
    await this.page.waitForTimeout(TIMEOUTS.LONG);
    return true;
  }
  
  /**
   * Check if bridge form is accessible (not requiring authentication)
   */
  async isFormAccessible() {
    const authRequired = await this.page.locator(SELECTORS.AUTH_REQUIRED)
      .isVisible()
      .catch(() => false);
    return !authRequired;
  }
  
  /**
   * Wait for form to become accessible
   */
  async waitForFormAccessible(timeout = TIMEOUTS.LONG) {
    const startTime = Date.now();
    
    while (Date.now() - startTime < timeout) {
      if (await this.isFormAccessible()) {
        return true;
      }
      await this.page.waitForTimeout(TIMEOUTS.SHORT);
    }
    
    return false;
  }
  
  /**
   * Get all form elements
   */
  async getFormElements() {
    return await getBridgeFormElements(this.page);
  }
  
  /**
   * Wait for tokens to load
   */
  async waitForTokensLoaded(timeout = TIMEOUTS.LONG) {
    return await waitForTokensLoaded(this.page, this.monitoring, timeout);
  }
  
  /**
   * Enter amount in the form
   */
  async enterAmount(amount) {
    return await fillAmount(this.page, amount);
  }
  
  /**
   * Click direction switch to reverse bridge direction
   */
  async switchDirection() {
    const elements = await this.getFormElements();
    
    if (!elements.directionSwitch) {
      console.log('❌ Direction switch button not found');
      return false;
    }
    
    // Get initial state
    const initialState = await this.getBridgeDirection();
    
    console.log('🔄 Clicking direction switch...');
    await elements.directionSwitch.click();
    await this.page.waitForTimeout(TIMEOUTS.SHORT);
    
    // Check if direction changed
    const newState = await this.getBridgeDirection();
    const changed = initialState.from !== newState.from || initialState.to !== newState.to;
    
    if (changed) {
      console.log('✅ Bridge direction switched successfully');
      console.log(`   From: ${initialState.from} -> ${newState.from}`);
      console.log(`   To: ${initialState.to} -> ${newState.to}`);
    } else {
      console.log('⚠️ Direction switch clicked but no change detected');
    }
    
    return changed;
  }
  
  /**
   * Get current bridge direction
   */
  async getBridgeDirection() {
    const sections = await this.page.locator('.swap-form__section').all();
    
    if (sections.length >= 2) {
      const fromText = await sections[0].textContent();
      const toText = await sections[1].textContent();
      return {
        from: fromText?.trim() || 'Unknown',
        to: toText?.trim() || 'Unknown'
      };
    }
    
    return { from: 'Unknown', to: 'Unknown' };
  }
  
  /**
   * Click token selector
   */
  async clickTokenSelector(index = 0) {
    const elements = await this.getFormElements();
    
    if (elements.tokenSelectorCount <= index) {
      console.log(`❌ Token selector ${index} not available (only ${elements.tokenSelectorCount} found)`);
      return false;
    }
    
    const selector = index === 0 ? elements.tokenSelectors.from : elements.tokenSelectors.to;
    
    if (!selector) {
      console.log(`❌ Token selector ${index} not found`);
      return false;
    }
    
    try {
      await selector.click();
      await this.page.waitForTimeout(TIMEOUTS.SHORT);
      
      // Check for dropdown
      const dropdown = this.page.locator('.token-dropdown, .token-modal, .dropdown-menu');
      const dropdownVisible = await dropdown.isVisible().catch(() => false);
      
      if (dropdownVisible) {
        console.log(`✅ Token selector ${index} opened dropdown`);
        // Close dropdown
        await this.page.click('body');
        await this.page.waitForTimeout(TIMEOUTS.SHORT / 2);
      }
      
      return true;
    } catch (error) {
      console.log(`❌ Failed to click token selector ${index}: ${error.message}`);
      return false;
    }
  }
  
  /**
   * Submit bridge form (click submit button)
   */
  async submitForm() {
    const elements = await this.getFormElements();
    
    if (!elements.submitButton) {
      console.log('❌ Submit button not found');
      return false;
    }
    
    const isEnabled = await elements.submitButton.getAttribute('disabled') === null;
    const buttonText = await elements.submitButton.textContent();
    
    console.log(`🎯 Submit button found: "${buttonText}" (enabled: ${isEnabled})`);
    
    if (!isEnabled) {
      console.log('⏳ Submit button is disabled');
      return false;
    }
    
    try {
      await elements.submitButton.click();
      await this.page.waitForTimeout(TIMEOUTS.MEDIUM);
      console.log('✅ Submit button clicked successfully');
      return true;
    } catch (error) {
      console.log(`❌ Failed to click submit button: ${error.message}`);
      return false;
    }
  }
  
  /**
   * Perform complete bridge transaction flow
   */
  async performTransactionFlow(amount = TEST_DATA.TEST_AMOUNTS.SMALL) {
    console.log('🚀 Starting complete bridge transaction flow...');
    
    const results = {
      formAccessible: false,
      tokensLoaded: false,
      amountEntered: false,
      formSubmitted: false,
      apiCalls: {
        tokens: 0,
        quote: 0,
        swap: 0
      }
    };
    
    // Step 1: Check form accessibility
    results.formAccessible = await this.isFormAccessible();
    if (!results.formAccessible) {
      console.log('❌ Bridge form not accessible');
      return results;
    }
    console.log('✅ Bridge form is accessible');
    
    // Step 2: Wait for tokens to load
    results.tokensLoaded = await this.waitForTokensLoaded();
    if (!results.tokensLoaded) {
      console.log('❌ Tokens failed to load');
      return results;
    }
    console.log('✅ Tokens loaded successfully');
    
    // Step 3: Enter amount
    results.amountEntered = await this.enterAmount(amount);
    if (!results.amountEntered) {
      console.log('❌ Failed to enter amount');
      return results;
    }
    console.log(`✅ Amount entered: ${amount}`);
    
    // Wait for quote generation
    await this.page.waitForTimeout(TIMEOUTS.MEDIUM);
    
    // Step 4: Submit form
    results.formSubmitted = await this.submitForm();
    if (results.formSubmitted) {
      console.log('✅ Form submitted successfully');
    } else {
      console.log('⏳ Form submission skipped (button disabled or not found)');
    }
    
    // Collect API call statistics
    const apiCalls = this.monitoring.getApiCalls();
    results.apiCalls.tokens = apiCalls.filter(call => call.url.includes('/tokens')).length;
    results.apiCalls.quote = apiCalls.filter(call => call.url.includes('/quote')).length;
    results.apiCalls.swap = apiCalls.filter(call => call.url.includes('/swap')).length;
    
    console.log('📊 Transaction flow completed');
    console.log(`   Form accessible: ${results.formAccessible ? '✅' : '❌'}`);
    console.log(`   Tokens loaded: ${results.tokensLoaded ? '✅' : '❌'}`);
    console.log(`   Amount entered: ${results.amountEntered ? '✅' : '❌'}`);
    console.log(`   Form submitted: ${results.formSubmitted ? '✅' : '⏳'}`);
    console.log(`   API calls: ${results.apiCalls.tokens} tokens, ${results.apiCalls.quote} quote, ${results.apiCalls.swap} swap`);
    
    return results;
  }
  
  /**
   * Check security features
   */
  async checkSecurityFeatures() {
    return await checkSecurityIndicators(this.page);
  }
  
  /**
   * Get error messages
   */
  async getErrors() {
    return await getErrorMessages(this.page);
  }
  
  /**
   * Test invalid input handling
   */
  async testInvalidInputs() {
    const results = [];
    
    for (const invalidInput of TEST_DATA.TEST_AMOUNTS.INVALID) {
      console.log(`🧪 Testing invalid input: "${invalidInput}"`);
      
      const inputSuccess = await this.enterAmount(invalidInput);
      await this.page.waitForTimeout(TIMEOUTS.SHORT);
      
      const errors = await this.getErrors();
      
      results.push({
        input: invalidInput,
        inputAccepted: inputSuccess,
        errorsShown: errors.length > 0,
        errorMessages: errors
      });
      
      if (errors.length > 0) {
        console.log(`✅ Invalid input "${invalidInput}" - error shown: ${errors[0]}`);
      } else {
        console.log(`⏳ Invalid input "${invalidInput}" - no error message`);
      }
    }
    
    return results;
  }
}