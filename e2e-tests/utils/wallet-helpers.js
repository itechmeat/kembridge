/**
 * Wallet-related helper functions for testing
 */
import { installMockWallet } from '@johanneskares/wallet-mock';
import { privateKeyToAccount } from 'viem/accounts';
import { http } from 'viem';
import { mainnet, sepolia } from 'viem/chains';
import { TEST_DATA, TIMEOUTS, SELECTORS } from './constants.js';

/**
 * Check if mock wallet is available
 * @param {import('@playwright/test').Page} page - Playwright page object
 * @returns {Promise<boolean>} True if mock wallet is available
 */
export async function isMockWalletAvailable(page) {
  try {
    const result = await page.evaluate(() => {
      const info = {
        hasEthereum: typeof window.ethereum !== 'undefined',
        ethereumKeys: window.ethereum ? Object.keys(window.ethereum) : [],
        hasProviders: window.ethereum && window.ethereum.providers,
        providersCount: window.ethereum && window.ethereum.providers ? window.ethereum.providers.length : 0,
        hasEvmProviders: typeof window.evmProviders !== 'undefined',
        evmProvidersKeys: window.evmProviders ? Object.keys(window.evmProviders) : [],
        isMockWallet: window.ethereum && window.ethereum.isMockWallet,
        isMetaMask: window.ethereum && window.ethereum.isMetaMask
      };
      
      // Check for EIP-6963 providers (modern way)
      if (window.ethereum && window.ethereum.providers) {
        const hasMockWallet = window.ethereum.providers.some(provider => 
          provider.info && provider.info.name === 'Mock Wallet'
        );
        if (hasMockWallet) return { available: true, info };
      }
      
      // Check for direct ethereum provider
      if (window.ethereum && (window.ethereum.isMockWallet || window.ethereum.isMetaMask)) {
        return { available: true, info };
      }
      
      // Check for EIP-6963 events
      if (window.evmProviders && Object.keys(window.evmProviders).length > 0) {
        return { available: true, info };
      }
      
      return { available: false, info };
    });
    
    console.log(`🔍 Mock Ethereum Provider Available: ${result.available ? '✅' : '❌'}`);
    if (!result.available) {
      console.log('🔍 Debug info:', JSON.stringify(result.info, null, 2));
    }
    return result.available;
  } catch (error) {
    console.warn('⚠️ Error checking mock wallet availability:', error.message);
    return false;
  }
}

/**
 * Setup mock wallet for testing
 * @param {import('@playwright/test').Page} page - Playwright page object
 */
export async function setupMockWallet(page) {
  try {
    console.log('🦊 Setting up mock wallet...');
    
    const account = privateKeyToAccount(TEST_DATA.PRIVATE_KEY);
    
    // Add context to page before installing mock wallet
    await page.addInitScript(() => {
      // Ensure window.ethereum is available for injection
      if (!window.ethereum) {
        window.ethereum = {};
      }
    });
    
    await installMockWallet({
      page,
      account,
      defaultChain: mainnet,
      transports: { 
        [mainnet.id]: http(),
        [sepolia.id]: http()
      },
    });
    
    // Reload page to ensure mock wallet is properly injected
    await page.reload();
    await page.waitForTimeout(TIMEOUTS.SHORT);
    
    // Wait for mock wallet to be available
    await waitForMockWallet(page);
    
    console.log('✅ Mock wallet setup complete');
  } catch (error) {
    console.warn('⚠️ Mock wallet setup failed:', error.message);
    console.error(error);
    // Don't throw - let tests continue without mock wallet
  }
}

/**
 * Wait for mock wallet to become available
 * @param {import('@playwright/test').Page} page - Playwright page object
 */
export async function waitForMockWallet(page) {
  const maxAttempts = 10;
  const delay = 500;
  
  for (let i = 0; i < maxAttempts; i++) {
    const isAvailable = await isMockWalletAvailable(page);
    if (isAvailable) {
      console.log('✅ Mock wallet is now available');
      return true;
    }
    
    console.log(`⏳ Waiting for mock wallet... (${i + 1}/${maxAttempts})`);
    await page.waitForTimeout(delay);
  }
  
  console.warn('⚠️ Mock wallet did not become available within timeout');
  return false;
}

/**
 * Authenticate with Ethereum wallet
 * @param {import('@playwright/test').Page} page 
 * @returns {Promise<boolean>} Success status
 */
export async function authenticateEthereumWallet(page) {
  const ethButton = page.locator(SELECTORS.ETH_WALLET_BUTTON);
  
  const isVisible = await ethButton.isVisible();
  if (!isVisible) {
    console.log('❌ Ethereum wallet button not visible');
    return false;
  }
  
  const isEnabled = await ethButton.getAttribute('disabled') === null;
  if (!isEnabled) {
    console.log('❌ Ethereum wallet button is disabled');
    return false;
  }
  
  console.log('🔐 Authenticating with Ethereum wallet...');
  await ethButton.click();
  await page.waitForTimeout(TIMEOUTS.AUTH_FLOW);
  console.log('✅ Ethereum authentication completed');
  
  return true;
}

/**
 * Check wallet connection status
 * @param {import('@playwright/test').Page} page 
 * @param {'ethereum'|'near'} walletType 
 * @returns {Promise<{connected: boolean, buttonText: string}>}
 */
export async function getWalletStatus(page, walletType = 'ethereum') {
  const selector = walletType === 'ethereum' 
    ? SELECTORS.ETH_WALLET_BUTTON 
    : SELECTORS.NEAR_WALLET_BUTTON;
    
  const button = page.locator(selector);
  const isVisible = await button.isVisible();
  
  if (!isVisible) {
    return { connected: false, buttonText: 'NOT_FOUND' };
  }
  
  const buttonText = await button.textContent();
  const connected = !buttonText.includes('Connect') && !buttonText.includes('Wallet');
  
  return { connected, buttonText };
}

/**
 * Wait for authentication to complete by monitoring API calls
 * @param {import('@playwright/test').Page} page 
 * @param {Array} apiCallsArray - Array to store API calls
 * @returns {Promise<{success: boolean, nonceCalled: boolean, verifyCalled: boolean}>}
 */
export async function waitForAuthenticationComplete(page, apiCallsArray) {
  const startTime = Date.now();
  const timeout = TIMEOUTS.AUTH_FLOW;
  
  while (Date.now() - startTime < timeout) {
    const nonceCalls = apiCallsArray.filter(call => call.url.includes('/auth/nonce'));
    const verifyCalls = apiCallsArray.filter(call => call.url.includes('/auth/verify'));
    
    if (nonceCalls.length > 0 && verifyCalls.length > 0) {
      return {
        success: true,
        nonceCalled: true,
        verifyCalled: true
      };
    }
    
    await page.waitForTimeout(TIMEOUTS.SHORT);
  }
  
  const nonceCalled = apiCallsArray.some(call => call.url.includes('/auth/nonce'));
  const verifyCalled = apiCallsArray.some(call => call.url.includes('/auth/verify'));
  
  return {
    success: false,
    nonceCalled,
    verifyCalled
  };
}