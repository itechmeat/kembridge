/**
 * API monitoring and testing helpers
 */
import { API_ENDPOINTS, TIMEOUTS } from './constants.js';
import { TEST_URLS } from './test-constants';

/**
 * Set up API monitoring for a page
 * @param {import('@playwright/test').Page} page 
 * @returns {Object} Monitoring utilities
 */
export function setupApiMonitoring(page) {
  const apiCalls = [];
  const networkErrors = [];
  const consoleMessages = [];
  
  // Monitor API requests
  page.on('request', request => {
    const url = request.url();
    if (url.includes('/api/v1/') || Object.values(TEST_URLS.BACKEND).some(backendUrl => url.includes(backendUrl))) {
      apiCalls.push({
        url,
        method: request.method(),
        timestamp: Date.now()
      });
    }
  });
  
  // Monitor network failures
  page.on('requestfailed', request => {
    networkErrors.push({
      url: request.url(),
      failure: request.failure(),
      timestamp: Date.now()
    });
  });
  
  // Monitor console messages
  page.on('console', msg => {
    consoleMessages.push({
      type: msg.type(),
      text: msg.text(),
      timestamp: Date.now()
    });
  });
  
  return {
    getApiCalls: () => [...apiCalls],
    getNetworkErrors: () => [...networkErrors],
    getConsoleMessages: () => [...consoleMessages],
    
    // Filtered getters
    getAuthCalls: () => apiCalls.filter(call => call.url.includes('/auth/')),
    getBridgeCalls: () => apiCalls.filter(call => call.url.includes('/bridge/')),
    getRiskCalls: () => apiCalls.filter(call => 
      call.url.includes('/risk/') || 
      call.url.includes('/analysis/') || 
      call.url.includes('/security/')
    ),
    getNonceCalls: () => apiCalls.filter(call => call.url.includes('/auth/nonce')),
    getVerifyCalls: () => apiCalls.filter(call => call.url.includes('/auth/verify')),
    getQuoteCalls: () => apiCalls.filter(call => call.url.includes('/quote')),
    getSwapCalls: () => apiCalls.filter(call => call.url.includes('/swap')),
    
    // Console message filters
    getErrorMessages: () => consoleMessages.filter(msg => msg.type === 'error'),
    getAuthMessages: () => consoleMessages.filter(msg => 
      msg.text.includes('authentication') || 
      msg.text.includes('Auth') ||
      msg.text.includes('signature') ||
      msg.text.includes('nonce')
    ),
    
    // Summary methods
    logApiSummary: () => {
      console.log('📊 API Call Summary:');
      console.log(`   Total calls: ${apiCalls.length}`);
      console.log(`   Auth calls: ${apiCalls.filter(call => call.url.includes('/auth/')).length}`);
      console.log(`   Bridge calls: ${apiCalls.filter(call => call.url.includes('/bridge/')).length}`);
      console.log(`   Network errors: ${networkErrors.length}`);
      console.log(`   Console errors: ${consoleMessages.filter(msg => msg.type === 'error').length}`);
    },
    
    logDetailedCalls: (filter = null) => {
      const callsToLog = filter ? apiCalls.filter(filter) : apiCalls;
      callsToLog.forEach((call, i) => {
        console.log(`   ${i + 1}. ${call.method} ${call.url}`);
      });
    }
  };
}

/**
 * Check health of all backend services
 * @param {import('@playwright/test').APIRequestContext} request 
 * @returns {Promise<Object>} Health check results
 */
export async function checkServicesHealth(request) {
  const services = [
    { name: 'Gateway', url: `${API_ENDPOINTS.BASE}/health` },
    { name: '1inch', url: `${TEST_URLS.BACKEND.ONEINCH}/health` },
    { name: 'Blockchain', url: `${TEST_URLS.BACKEND.BLOCKCHAIN}/health` },
    { name: 'Crypto', url: `${TEST_URLS.BACKEND.CRYPTO}/health` },
    { name: 'Auth', url: `${TEST_URLS.BACKEND.AUTH}/health` }
  ];
  
  const results = {};
  
  for (const service of services) {
    try {
      const response = await request.get(service.url);
      results[service.name] = {
        healthy: response.ok(),
        status: response.status(),
        url: service.url
      };
    } catch (error) {
      results[service.name] = {
        healthy: false,
        error: error.message,
        url: service.url
      };
    }
  }
  
  return results;
}

/**
 * Test nonce generation for authentication
 * @param {import('@playwright/test').APIRequestContext} request 
 * @param {string} walletAddress 
 * @param {'near'|'ethereum'} chainType 
 * @returns {Promise<Object>} Nonce test result
 */
export async function testNonceGeneration(request, walletAddress, chainType) {
  try {
    const response = await request.get(`${API_ENDPOINTS.BASE}${API_ENDPOINTS.AUTH_NONCE}`, {
      params: {
        wallet_address: walletAddress,
        chain_type: chainType
      }
    });
    
    if (!response.ok()) {
      return {
        success: false,
        status: response.status(),
        error: 'HTTP error'
      };
    }
    
    const data = await response.json();
    
    return {
      success: data.success === true,
      nonce: data.data?.nonce,
      message: data.data?.message,
      validFormat: data.data?.nonce ? /^[a-f0-9]{64}$/.test(data.data.nonce) : false,
      containsExpectedContent: data.data?.message ? [
        'KEMBridge Authentication',
        walletAddress,
        chainType
      ].every(content => data.data.message.includes(content)) : false
    };
  } catch (error) {
    return {
      success: false,
      error: error.message
    };
  }
}

/**
 * Wait for specific API calls to complete
 * @param {Object} monitoring - Monitoring object from setupApiMonitoring
 * @param {Array<string>} expectedCalls - Array of URL patterns to wait for
 * @param {number} timeout - Timeout in milliseconds
 * @returns {Promise<boolean>} Success status
 */
export async function waitForApiCalls(monitoring, expectedCalls, timeout = TIMEOUTS.LONG) {
  const startTime = Date.now();
  
  while (Date.now() - startTime < timeout) {
    const apiCalls = monitoring.getApiCalls();
    const foundCalls = expectedCalls.filter(pattern => 
      apiCalls.some(call => call.url.includes(pattern))
    );
    
    if (foundCalls.length === expectedCalls.length) {
      return true;
    }
    
    await new Promise(resolve => setTimeout(resolve, TIMEOUTS.SHORT));
  }
  
  return false;
}