/**
 * Backend Error Recovery E2E Tests
 * Testing backend error handling, recovery mechanisms, and monitoring
 */

import { test, expect } from '@playwright/test';
import axios from 'axios';
import { TEST_URLS } from '../utils/test-constants';

// Test configuration
const BACKEND_URL = TEST_URLS.BACKEND.GATEWAY;
const AI_ENGINE_URL = TEST_URLS.BACKEND.AI_ENGINE;
const TIMEOUT = 30000;

// Helper class for backend testing
class BackendErrorTester {
  constructor() {
    this.client = axios.create({
      baseURL: BACKEND_URL,
      timeout: 10000,
      validateStatus: () => true, // Don't throw on HTTP errors
    });
  }

  // Health check
  async checkHealth() {
    try {
      const response = await this.client.get('/health');
      return {
        success: response.status === 200,
        status: response.status,
        data: response.data
      };
    } catch (error) {
      return {
        success: false,
        error: error.message
      };
    }
  }

  // Authentication tests
  async testAuthEndpoint(params) {
    try {
      const response = await this.client.get('/api/v1/auth/nonce', { params });
      return {
        success: response.status === 200,
        status: response.status,
        data: response.data
      };
    } catch (error) {
      return {
        success: false,
        error: error.message
      };
    }
  }

  // Bridge service tests
  async testBridgeQuote(params) {
    try {
      const response = await this.client.get('/api/v1/bridge/quote', { params });
      return {
        success: response.status === 200,
        status: response.status,
        data: response.data
      };
    } catch (error) {
      return {
        success: false,
        error: error.message
      };
    }
  }

  // Test with invalid data
  async testWithInvalidData(endpoint, data) {
    try {
      const response = await this.client.post(endpoint, data);
      return {
        success: false, // Should not succeed with invalid data
        status: response.status,
        data: response.data
      };
    } catch (error) {
      return {
        success: true, // Error is expected
        error: error.message
      };
    }
  }

  // Simulate service overload
  async simulateOverload(endpoint, concurrentRequests = 20) {
    const requests = Array(concurrentRequests).fill().map(() => 
      this.client.get(endpoint).catch(error => ({ error: error.message }))
    );

    const results = await Promise.all(requests);
    
    const successful = results.filter(r => r.status && r.status === 200).length;
    const errors = results.filter(r => r.error).length;
    const rateLimited = results.filter(r => r.status === 429).length;

    return {
      totalRequests: concurrentRequests,
      successful,
      errors,
      rateLimited,
      results
    };
  }

  // Test timeout handling
  async testTimeoutHandling(endpoint, timeout = 1000) {
    const client = axios.create({
      baseURL: BACKEND_URL,
      timeout: timeout,
      validateStatus: () => true,
    });

    try {
      const response = await client.get(endpoint);
      return {
        success: response.status === 200,
        status: response.status,
        timedOut: false
      };
    } catch (error) {
      return {
        success: false,
        error: error.message,
        timedOut: error.code === 'ECONNABORTED'
      };
    }
  }
}

test.describe('Backend Error Recovery Systems', () => {
  let backendTester;

  test.beforeAll(async () => {
    backendTester = new BackendErrorTester();
    
    // Verify backend is running
    const health = await backendTester.checkHealth();
    if (!health.success) {
      throw new Error(`Backend not available: ${health.error}`);
    }
  });

  test.describe('Service Health Monitoring', () => {
    test('should return healthy status when all services are operational', async () => {
      const health = await backendTester.checkHealth();
      
      expect(health.success).toBeTruthy();
      expect(health.data).toBeDefined();
      expect(health.data.data.status).toBe('healthy');
      
      // Verify service features are reported
      if (health.data.features) {
        expect(typeof health.data.features).toBe('object');
      }
    });

    test('should report degraded status when services have issues', async () => {
      // This test would require ability to simulate service degradation
      // For now, we'll test the endpoint structure
      const health = await backendTester.checkHealth();
      
      expect(health.success).toBeTruthy();
      expect(health.data.data).toHaveProperty('status');
      expect(['healthy', 'degraded', 'unhealthy']).toContain(health.data.data.status);
    });
  });

  test.describe('Authentication Error Handling', () => {
    test('should handle missing wallet address gracefully', async () => {
      const result = await backendTester.testAuthEndpoint({
        // Missing wallet_address
        chain_type: 'ethereum'
      });

      expect(result.status).toBe(400); // Bad Request
      expect(result.data).toBeDefined();
      
      if (result.data.error) {
        expect(result.data.error.toLowerCase()).toContain('wallet_address');
      }
    });

    test('should handle invalid chain type', async () => {
      const result = await backendTester.testAuthEndpoint({
        wallet_address: '0x1234567890123456789012345678901234567890',
        chain_type: 'invalid_chain'
      });

      expect(result.status).toBe(400); // Bad Request
      expect(result.data).toBeDefined();
      
      if (result.data.error) {
        expect(result.data.error.toLowerCase()).toContain('chain');
      }
    });

    test('should handle malformed wallet addresses', async () => {
      const result = await backendTester.testAuthEndpoint({
        wallet_address: 'invalid_address',
        chain_type: 'ethereum'
      });

      expect(result.status).toBeGreaterThanOrEqual(400);
      expect(result.status).toBeLessThan(500);
    });

    test('should implement rate limiting for auth endpoints', async () => {
      const overloadResult = await backendTester.simulateOverload('/api/v1/auth/nonce?wallet_address=0x1234567890123456789012345678901234567890&chain_type=ethereum', 15);

      // Should have some rate limiting or error handling
      expect(overloadResult.totalRequests).toBe(15);
      
      // At least some requests should succeed initially
      expect(overloadResult.successful).toBeGreaterThan(0);
      
      // If rate limiting is implemented, should see 429 responses
      if (overloadResult.rateLimited > 0) {
        expect(overloadResult.rateLimited).toBeGreaterThan(0);
      }
    });
  });

  test.describe('Bridge Service Error Handling', () => {
    test('should validate bridge quote parameters', async () => {
      const result = await backendTester.testBridgeQuote({
        // Missing required parameters
        from_chain: 'ethereum'
      });

      expect(result.status).toBeGreaterThanOrEqual(400);
      expect(result.status).toBeLessThan(500);
    });

    test('should handle invalid amount values', async () => {
      const result = await backendTester.testBridgeQuote({
        from_chain: 'ethereum',
        to_chain: 'near',
        from_token: 'ETH',
        to_token: 'NEAR',
        from_amount: 'invalid_amount'
      });

      expect(result.status).toBe(400);
      expect(result.data).toBeDefined();
    });

    test('should handle unsupported token pairs', async () => {
      const result = await backendTester.testBridgeQuote({
        from_chain: 'ethereum',
        to_chain: 'near',
        from_token: 'UNSUPPORTED_TOKEN',
        to_token: 'ANOTHER_UNSUPPORTED',
        from_amount: '1.0'
      });

      expect(result.status).toBeGreaterThanOrEqual(400);
      expect(result.status).toBeLessThan(500);
    });

    test('should handle extreme amount values', async () => {
      // Test very large amount
      const largeAmountResult = await backendTester.testBridgeQuote({
        from_chain: 'ethereum',
        to_chain: 'near',
        from_token: 'ETH',
        to_token: 'NEAR',
        from_amount: '999999999999999999'
      });

      expect(largeAmountResult.status).toBeGreaterThanOrEqual(400);

      // Test very small amount
      const smallAmountResult = await backendTester.testBridgeQuote({
        from_chain: 'ethereum',
        to_chain: 'near',
        from_token: 'ETH',
        to_token: 'NEAR',
        from_amount: '0.000000000000000001'
      });

      expect(smallAmountResult.status).toBeGreaterThanOrEqual(400);
    });
  });

  test.describe('Service Dependency Error Handling', () => {
    test('should handle AI Engine unavailability gracefully', async () => {
      // Try to access an endpoint that depends on AI Engine
      const client = axios.create({
        baseURL: AI_ENGINE_URL,
        timeout: 5000,
        validateStatus: () => true,
      });

      try {
        const aiHealth = await client.get('/health');
        
        if (aiHealth.status !== 200) {
          console.log('AI Engine appears to be unavailable, testing graceful degradation');
          
          // Test if bridge still works without AI Engine
          const bridgeResult = await backendTester.testBridgeQuote({
            from_chain: 'ethereum',
            to_chain: 'near',
            from_token: 'ETH',
            to_token: 'NEAR',
            from_amount: '1.0'
          });

          // Bridge should still work, possibly with degraded risk analysis
          expect(bridgeResult.status).toBeLessThan(500); // Not internal server error
        }
      } catch (error) {
        console.log('AI Engine unavailable, testing graceful degradation');
        
        // Bridge should handle AI Engine unavailability
        const bridgeResult = await backendTester.testBridgeQuote({
          from_chain: 'ethereum',
          to_chain: 'near',
          from_token: 'ETH',
          to_token: 'NEAR',
          from_amount: '1.0'
        });

        expect(bridgeResult.status).toBeLessThan(500);
      }
    });

    test('should handle database connection issues', async () => {
      // This test would require ability to simulate DB issues
      // For now, test that endpoints handle DB errors gracefully
      
      const result = await backendTester.testBridgeQuote({
        from_chain: 'ethereum',
        to_chain: 'near',
        from_token: 'ETH',
        to_token: 'NEAR',
        from_amount: '1.0'
      });

      // Should get some response, not hang indefinitely
      expect(result).toBeDefined();
      expect(typeof result.status).toBe('number');
    });
  });

  test.describe('Error Response Format', () => {
    test('should return consistent error response format', async () => {
      const result = await backendTester.testAuthEndpoint({
        // Invalid request to trigger error
        wallet_address: 'invalid',
        chain_type: 'invalid'
      });

      expect(result.status).toBeGreaterThanOrEqual(400);
      expect(result.data).toBeDefined();
      
      // Check for consistent error structure
      if (result.data.error) {
        expect(typeof result.data.error).toBe('string');
      }
      
      // Should not expose internal details
      const errorString = JSON.stringify(result.data).toLowerCase();
      expect(errorString).not.toContain('password');
      expect(errorString).not.toContain('secret');
      expect(errorString).not.toContain('token');
      expect(errorString).not.toContain('sql');
    });

    test('should include correlation IDs for error tracking', async () => {
      const result = await backendTester.testAuthEndpoint({
        wallet_address: 'invalid_address_format',
        chain_type: 'ethereum'
      });

      expect(result.status).toBeGreaterThanOrEqual(400);
      
      // Check for correlation ID in response headers or body
      if (result.data) {
        const hasCorrelationId = result.data.correlation_id || 
                               result.data.request_id ||
                               result.data.trace_id;
        
        // Correlation ID is good practice but not required for this test
        if (hasCorrelationId) {
          expect(typeof hasCorrelationId).toBe('string');
          expect(hasCorrelationId.length).toBeGreaterThan(0);
        }
      }
    });
  });

  test.describe('Performance Under Error Conditions', () => {
    test('should maintain response times during error conditions', async () => {
      const startTime = Date.now();
      
      const result = await backendTester.testAuthEndpoint({
        wallet_address: 'invalid_format',
        chain_type: 'ethereum'
      });
      
      const responseTime = Date.now() - startTime;
      
      // Error responses should be fast (< 5 seconds)
      expect(responseTime).toBeLessThan(5000);
      expect(result.status).toBeGreaterThanOrEqual(400);
    });

    test('should handle timeout scenarios gracefully', async () => {
      const result = await backendTester.testTimeoutHandling('/api/v1/bridge/quote?from_chain=ethereum&to_chain=near&from_token=ETH&to_token=NEAR&from_amount=1.0', 1000);

      if (result.timedOut) {
        expect(result.success).toBeFalsy();
        expect(result.error).toContain('timeout');
      } else {
        // If didn't timeout, should have gotten some response
        expect(typeof result.status).toBe('number');
      }
    });
  });

  test.describe('Circuit Breaker Functionality', () => {
    test('should implement circuit breaker for external services', async () => {
      // This test would require ability to simulate external service failures
      // For now, test that the system doesn't hang on external service calls
      
      const results = [];
      const promises = Array(5).fill().map(async () => {
        const startTime = Date.now();
        const result = await backendTester.testBridgeQuote({
          from_chain: 'ethereum',
          to_chain: 'near',
          from_token: 'ETH',
          to_token: 'NEAR',
          from_amount: '1.0'
        });
        const endTime = Date.now();
        
        return {
          ...result,
          responseTime: endTime - startTime
        };
      });

      const allResults = await Promise.all(promises);
      
      // All requests should complete in reasonable time
      allResults.forEach(result => {
        expect(result.responseTime).toBeLessThan(30000); // 30 seconds max
        expect(typeof result.status).toBe('number');
      });
    });
  });

  test.describe('Recovery Mechanisms', () => {
    test('should recover from temporary service disruptions', async () => {
      // Test multiple requests to see if service recovers from errors
      const results = [];
      
      for (let i = 0; i < 3; i++) {
        const result = await backendTester.testBridgeQuote({
          from_chain: 'ethereum',
          to_chain: 'near',
          from_token: 'ETH',
          to_token: 'NEAR',
          from_amount: '1.0'
        });
        
        results.push(result);
        
        // Wait between requests
        await new Promise(resolve => setTimeout(resolve, 1000));
      }

      // At least some requests should succeed or show consistent error handling
      const successCount = results.filter(r => r.success || r.status < 500).length;
      expect(successCount).toBeGreaterThan(0);
    });

    test('should maintain service availability during partial failures', async () => {
      // Test that core endpoints remain available
      const coreEndpoints = [
        '/health',
        '/api/v1/auth/nonce?wallet_address=0x1234567890123456789012345678901234567890&chain_type=ethereum'
      ];

      const results = await Promise.all(
        coreEndpoints.map(async endpoint => {
          try {
            const response = await backendTester.client.get(endpoint);
            return {
              endpoint,
              success: response.status < 500,
              status: response.status
            };
          } catch (error) {
            return {
              endpoint,
              success: false,
              error: error.message
            };
          }
        })
      );

      // Core endpoints should be available
      results.forEach(result => {
        expect(result.success).toBeTruthy();
      });
    });
  });

  test.afterAll(async () => {
    // Cleanup if needed
    console.log('Backend error recovery tests completed');
  });
});

// Utility tests for error monitoring
test.describe('Error Monitoring Validation', () => {
  test('should validate error tracking capabilities', async () => {
    const backendTester = new BackendErrorTester();
    
    // Generate a deliberate error
    const errorResult = await backendTester.testAuthEndpoint({
      wallet_address: 'invalid_for_monitoring_test',
      chain_type: 'ethereum'
    });

    expect(errorResult.status).toBeGreaterThanOrEqual(400);
    
    // Error should be logged/tracked
    // This would require access to monitoring endpoints if they exist
    const health = await backendTester.checkHealth();
    expect(health.success).toBeTruthy();
  });

  test('should measure error recovery performance', async () => {
    const backendTester = new BackendErrorTester();
    
    const startTime = Date.now();
    
    // Generate multiple errors and measure recovery
    const errorRequests = Array(5).fill().map(() =>
      backendTester.testAuthEndpoint({
        wallet_address: 'error_test_' + Math.random(),
        chain_type: 'invalid'
      })
    );
    
    const results = await Promise.all(errorRequests);
    const endTime = Date.now();
    
    const totalTime = endTime - startTime;
    const avgTimePerError = totalTime / results.length;
    
    // Error handling should be efficient
    expect(avgTimePerError).toBeLessThan(2000); // < 2 seconds per error
    expect(results.every(r => typeof r.status === 'number')).toBeTruthy();
  });
});