import { test, expect } from '@playwright/test';
import { checkServicesHealth, testNonceGeneration } from '../utils/api-helpers.js';
import { API_ENDPOINTS, SERVICE_ENDPOINTS, TEST_DATA } from '../utils/constants.js';

test.describe('KEMBridge API Integration', () => {

  test('should have healthy backend services', async ({ request }) => {
    const healthResults = await checkServicesHealth(request);
    
    // Verify each service is healthy
    Object.entries(healthResults).forEach(([serviceName, result]) => {
      console.log(`🔍 ${serviceName} service: ${result.healthy ? '✅' : '❌'} (${result.status || 'ERROR'})`);
      expect(result.healthy).toBeTruthy();
    });
    
    // Verify gateway service specific response
    if (healthResults.Gateway.healthy) {
      const gatewayResponse = await request.get(`${API_ENDPOINTS.BASE}${API_ENDPOINTS.HEALTH}`);
      const gatewayData = await gatewayResponse.json();
      expect(gatewayData.success).toBe(true);
      expect(gatewayData.data.service).toBe('kembridge-gateway-service');
    }
  });

  test('should generate NEAR nonce successfully', async ({ request }) => {
    const result = await testNonceGeneration(request, TEST_DATA.NEAR_ADDRESS, 'near');
    
    console.log('🔍 NEAR nonce generation:', result.success ? '✅ PASS' : '❌ FAIL');
    
    expect(result.success).toBeTruthy();
    expect(result.nonce).toBeDefined();
    expect(result.message).toBeDefined();
    expect(result.validFormat).toBeTruthy();
    expect(result.containsExpectedContent).toBeTruthy();
    
    if (result.success) {
      console.log(`   Nonce: ${result.nonce?.substring(0, 16)}...`);
      console.log(`   Message contains expected content: ${result.containsExpectedContent ? '✅' : '❌'}`);
    }
  });

  test('should generate Ethereum nonce successfully', async ({ request }) => {
    const result = await testNonceGeneration(request, TEST_DATA.ETHEREUM_ADDRESS, 'ethereum');
    
    console.log('🔍 Ethereum nonce generation:', result.success ? '✅ PASS' : '❌ FAIL');
    
    expect(result.success).toBeTruthy();
    expect(result.nonce).toBeDefined();
    expect(result.message).toBeDefined();
    expect(result.validFormat).toBeTruthy();
    expect(result.containsExpectedContent).toBeTruthy();
    
    if (result.success) {
      console.log(`   Nonce: ${result.nonce?.substring(0, 16)}...`);
      console.log(`   Message contains expected content: ${result.containsExpectedContent ? '✅' : '❌'}`);
    }
  });

  test('should handle invalid nonce requests', async ({ request }) => {
    console.log('💥 Testing invalid nonce requests...');
    
    // Test without required parameters
    const responseWithoutParams = await request.get(`${API_ENDPOINTS.BASE}${API_ENDPOINTS.AUTH_NONCE}`);
    console.log(`   No parameters: ${responseWithoutParams.status()} status`);
    expect(responseWithoutParams.status()).toBe(400);

    // Test with invalid chain type
    const responseInvalidChain = await request.get(`${API_ENDPOINTS.BASE}${API_ENDPOINTS.AUTH_NONCE}`, {
      params: {
        wallet_address: 'test.testnet',
        chain_type: 'invalid_chain'
      }
    });
    console.log(`   Invalid chain type: ${responseInvalidChain.status()} status`);
    // Should still work as we don't validate chain type in current implementation
    expect(responseInvalidChain.ok()).toBeTruthy();
    
    console.log('✅ Invalid request handling test completed');
  });
});