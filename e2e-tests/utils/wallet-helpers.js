/**
 * Wallet-related helper functions for testing
 */
import { installMockWallet } from '@johanneskares/wallet-mock';
import { privateKeyToAccount } from 'viem/accounts';
import { http } from 'viem';
import { sepolia } from 'viem/chains';
import { TEST_DATA, TIMEOUTS, SELECTORS } from './constants.js';

/**
 * Install mock wallet for Ethereum testing
 * @param {import('@playwright/test').Page} page 
 */
export async function setupMockWallet(page) {
  await installMockWallet({
    page,
    account: privateKeyToAccount(TEST_DATA.PRIVATE_KEY),
    defaultChain: sepolia,
    transports: { [sepolia.id]: http() },
  });
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