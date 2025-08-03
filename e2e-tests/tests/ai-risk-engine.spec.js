import { test, expect } from '@playwright/test';
import { SERVICE_URLS, RISK_ANALYSIS, DEFAULT_USER_ID } from '../utils/constants.js';
import { getBackendUrl } from '../utils/page-evaluate-utils.js';

test.describe('AI Risk Engine Integration Tests', () => {
  const AI_ENGINE_URL = process.env.AI_ENGINE_URL || getBackendUrl('aiEngine');
  
  test.beforeEach(async ({ page }) => {
    console.log('🤖 Setting up AI Risk Engine tests...');
    console.log(`   AI Engine URL: ${AI_ENGINE_URL}`);
  });

  test('should verify AI Engine health status', async ({ page }) => {
    console.log('🏥 Testing AI Engine health endpoint...');

    const response = await page.request.get(`${AI_ENGINE_URL}/health`);
    expect(response.ok()).toBeTruthy();
    
    const healthData = await response.json();
    console.log('📊 Health Status Response:', JSON.stringify(healthData, null, 2));

    // Verify required health fields
    expect(healthData.status).toBeDefined();
    expect(healthData.service).toBe('kembridge-ai-engine');
    expect(healthData.version).toBeDefined();
    expect(healthData.database_status).toBeDefined();
    expect(healthData.ml_models_status).toBeDefined();
    
    console.log(`✅ AI Engine Status: ${healthData.status}`);
    console.log(`✅ Database Status: ${healthData.database_status}`);
    console.log(`✅ ML Models Status: ${healthData.ml_models_status}`);
    console.log(`✅ Blacklist Loaded: ${healthData.blacklist_loaded}`);
  });

  test('should analyze transaction risk with valid data', async ({ page }) => {
    console.log('📊 Testing risk analysis with valid transaction data...');

    const riskRequest = {
      user_id: DEFAULT_USER_ID,
      transaction_id: 'test-tx-001',
      amount_in: 100.0,
      source_chain: 'ethereum',
      destination_chain: 'near',
      source_token: 'ETH',
      destination_token: 'NEAR',
      user_address: '0x742d35Cc6Eba4C34aCe21Db51B0B87a9e1234567'
    };

    console.log('📝 Risk Analysis Request:', JSON.stringify(riskRequest, null, 2));

    const response = await page.request.post(`${AI_ENGINE_URL}/api/risk/analyze`, {
      headers: { 'Content-Type': 'application/json' },
      data: riskRequest
    });

    expect(response.ok()).toBeTruthy();
    const riskData = await response.json();
    
    console.log('📊 Risk Analysis Response:', JSON.stringify(riskData, null, 2));

    // Verify response structure
    expect(riskData.risk_score).toBeDefined();
    expect(typeof riskData.risk_score).toBe('number');
    expect(riskData.risk_score).toBeGreaterThanOrEqual(0);
    expect(riskData.risk_score).toBeLessThanOrEqual(1);
    
    expect(riskData.risk_level).toBeDefined();
    expect(['low', 'medium', 'high']).toContain(riskData.risk_level);
    
    expect(riskData.reasons).toBeDefined();
    expect(Array.isArray(riskData.reasons)).toBeTruthy();
    
    expect(typeof riskData.approved).toBe('boolean');
    expect(riskData.recommended_action).toBeDefined();
    expect(riskData.analysis_timestamp).toBeDefined();

    console.log(`✅ Risk Score: ${riskData.risk_score} (${riskData.risk_level})`);
    console.log(`✅ Approved: ${riskData.approved}`);
    console.log(`✅ Risk Factors: ${riskData.reasons.length} identified`);
    console.log(`✅ Recommended Action: ${riskData.recommended_action}`);
  });

  test('should analyze high-risk transaction', async ({ page }) => {
    console.log('🚨 Testing high-risk transaction analysis...');

    const highRiskRequest = {
      user_id: DEFAULT_USER_ID,
      transaction_id: 'test-high-risk-001',
      amount_in: 10000.0, // Large amount to trigger high risk
      source_chain: 'ethereum',
      destination_chain: 'near',
      source_token: 'ETH',
      destination_token: 'NEAR',
      user_address: '0x742d35Cc6Eba4C34aCe21Db51B0B87a9e1234567'
    };

    const response = await page.request.post(`${AI_ENGINE_URL}/api/risk/analyze`, {
      headers: { 'Content-Type': 'application/json' },
      data: highRiskRequest
    });

    expect(response.ok()).toBeTruthy();
    const riskData = await response.json();
    
    console.log('🚨 High-Risk Analysis Response:', JSON.stringify(riskData, null, 2));

    // High amount should trigger higher risk score
    expect(riskData.risk_score).toBeGreaterThan(0.3);
    
    // Should contain risk factors related to amount
    const hasAmountRisk = riskData.reasons.some(reason => 
      reason.toLowerCase().includes('amount') || 
      reason.toLowerCase().includes('large')
    );
    expect(hasAmountRisk).toBeTruthy();

    console.log(`✅ High-Risk Score: ${riskData.risk_score} (${riskData.risk_level})`);
    console.log(`✅ Amount-related risk factor found: ${hasAmountRisk}`);
  });

  test('should get user risk profile', async ({ page }) => {
    console.log('👤 Testing user risk profile endpoint...');

    const response = await page.request.get(`${AI_ENGINE_URL}/api/risk/profile/${DEFAULT_USER_ID}`);
    expect(response.ok()).toBeTruthy();
    
    const profileData = await response.json();
    console.log('👤 User Profile Response:', JSON.stringify(profileData, null, 2));

    // Verify profile structure
    expect(profileData.user_id).toBe(DEFAULT_USER_ID);
    expect(profileData.overall_risk_level).toBeDefined();
    expect(['low', 'medium', 'high']).toContain(profileData.overall_risk_level);
    
    expect(typeof profileData.transaction_count).toBe('number');
    expect(typeof profileData.avg_risk_score).toBe('number');
    expect(typeof profileData.high_risk_transactions).toBe('number');
    expect(profileData.last_analysis_date).toBeDefined();

    console.log(`✅ User ID: ${profileData.user_id}`);
    console.log(`✅ Overall Risk Level: ${profileData.overall_risk_level}`);
    console.log(`✅ Transaction Count: ${profileData.transaction_count}`);
    console.log(`✅ Average Risk Score: ${profileData.avg_risk_score}`);
  });

  test('should check address blacklist', async ({ page }) => {
    console.log('🚫 Testing address blacklist functionality...');

    const testAddress = '0x742d35Cc6Eba4C34aCe21Db51B0B87a9e1234567';
    const testChain = 'ethereum';

    const response = await page.request.post(`${AI_ENGINE_URL}/api/risk/blacklist/check?address=${testAddress}&chain=${testChain}`, {
      headers: { 'Content-Type': 'application/json' }
    });

    expect(response.ok()).toBeTruthy();
    const blacklistData = await response.json();
    
    console.log('🚫 Blacklist Check Response:', JSON.stringify(blacklistData, null, 2));

    // Verify blacklist response structure
    expect(blacklistData.address).toBe(testAddress);
    expect(blacklistData.chain).toBe(testChain);
    expect(typeof blacklistData.is_blacklisted).toBe('boolean');
    expect(typeof blacklistData.risk_score_increase).toBe('number');
    expect(blacklistData.timestamp).toBeDefined();

    console.log(`✅ Address: ${blacklistData.address}`);
    console.log(`✅ Chain: ${blacklistData.chain}`);
    console.log(`✅ Is Blacklisted: ${blacklistData.is_blacklisted}`);
    console.log(`✅ Risk Score Increase: ${blacklistData.risk_score_increase}`);
  });

  test('should get blacklist statistics', async ({ page }) => {
    console.log('📈 Testing blacklist statistics endpoint...');

    const response = await page.request.get(`${AI_ENGINE_URL}/api/risk/blacklist/stats`);
    expect(response.ok()).toBeTruthy();
    
    const statsData = await response.json();
    console.log('📈 Blacklist Stats Response:', JSON.stringify(statsData, null, 2));

    // Verify stats structure (fields may vary based on implementation)
    expect(statsData).toBeDefined();
    expect(typeof statsData).toBe('object');

    console.log('✅ Blacklist statistics retrieved successfully');
  });

  test('should handle invalid risk analysis requests', async ({ page }) => {
    console.log('❌ Testing invalid risk analysis requests...');

    // Test 1: Missing required fields
    const invalidRequest1 = {
      user_id: DEFAULT_USER_ID,
      // Missing amount_in, source_chain, etc.
    };

    const response1 = await page.request.post(`${AI_ENGINE_URL}/api/risk/analyze`, {
      headers: { 'Content-Type': 'application/json' },
      data: invalidRequest1
    });

    expect(response1.status()).toBe(422); // Validation error
    console.log('✅ Test 1: Missing fields correctly rejected');

    // Test 2: Invalid amount
    const invalidRequest2 = {
      user_id: DEFAULT_USER_ID,
      amount_in: -100.0, // Negative amount
      source_chain: 'ethereum',
      destination_chain: 'near',
      source_token: 'ETH',
      destination_token: 'NEAR'
    };

    const response2 = await page.request.post(`${AI_ENGINE_URL}/api/risk/analyze`, {
      headers: { 'Content-Type': 'application/json' },
      data: invalidRequest2
    });

    expect(response2.status()).toBe(422); // Validation error
    console.log('✅ Test 2: Negative amount correctly rejected');

    // Test 3: Invalid user profile request
    const response3 = await page.request.get(`${AI_ENGINE_URL}/api/risk/profile/`);
    expect(response3.status()).toBe(404); // Not found
    console.log('✅ Test 3: Empty user ID correctly rejected');
  });

  test('should test AI Engine performance', async ({ page }) => {
    console.log('⚡ Testing AI Engine performance...');

    const performanceMetrics = {
      healthCheckTime: 0,
      riskAnalysisTime: 0,
      profileFetchTime: 0,
      blacklistCheckTime: 0
    };

    // Health check performance
    const healthStart = Date.now();
    const healthResponse = await page.request.get(`${AI_ENGINE_URL}/health`);
    expect(healthResponse.ok()).toBeTruthy();
    performanceMetrics.healthCheckTime = Date.now() - healthStart;

    // Risk analysis performance
    const riskStart = Date.now();
    const riskResponse = await page.request.post(`${AI_ENGINE_URL}/api/risk/analyze`, {
      headers: { 'Content-Type': 'application/json' },
      data: {
        user_id: DEFAULT_USER_ID,
        amount_in: 100.0,
        source_chain: 'ethereum',
        destination_chain: 'near',
        source_token: 'ETH',
        destination_token: 'NEAR'
      }
    });
    expect(riskResponse.ok()).toBeTruthy();
    performanceMetrics.riskAnalysisTime = Date.now() - riskStart;

    // Profile fetch performance
    const profileStart = Date.now();
    const profileResponse = await page.request.get(`${AI_ENGINE_URL}/api/risk/profile/${DEFAULT_USER_ID}`);
    expect(profileResponse.ok()).toBeTruthy();
    performanceMetrics.profileFetchTime = Date.now() - profileStart;

    // Blacklist check performance
    const blacklistStart = Date.now();
    const blacklistResponse = await page.request.post(`${AI_ENGINE_URL}/api/risk/blacklist/check?address=0x123&chain=ethereum`);
    expect(blacklistResponse.ok()).toBeTruthy();
    performanceMetrics.blacklistCheckTime = Date.now() - blacklistStart;

    console.log('📊 Performance Metrics:');
    console.log(`   Health Check: ${performanceMetrics.healthCheckTime}ms`);
    console.log(`   Risk Analysis: ${performanceMetrics.riskAnalysisTime}ms`);
    console.log(`   Profile Fetch: ${performanceMetrics.profileFetchTime}ms`);
    console.log(`   Blacklist Check: ${performanceMetrics.blacklistCheckTime}ms`);

    // Performance assertions (based on plan requirements: < 2 seconds for AI analysis)
    expect(performanceMetrics.healthCheckTime).toBeLessThan(1000); // 1 second
    expect(performanceMetrics.riskAnalysisTime).toBeLessThan(2000); // 2 seconds
    expect(performanceMetrics.profileFetchTime).toBeLessThan(1000); // 1 second
    expect(performanceMetrics.blacklistCheckTime).toBeLessThan(1000); // 1 second

    console.log('✅ AI Engine Performance: MEETS REQUIREMENTS');
  });
});