/**
 * WebSocket Authentication Messages E2E Tests
 * Specifically tests that AuthSuccess and AuthFailed messages are sent correctly
 */

import { test, expect } from '@playwright/test';
import { createTestJWT, createExpiredTestJWT, createInvalidTestJWT } from '../utils/jwt-helper';
import { WebSocketTestUtils } from '../utils/websocket-utils';
import { TEST_URLS } from '../utils/test-constants';

test.describe('WebSocket Authentication Messages', () => {
  test('should send AuthSuccess message with valid JWT token', async ({ page }) => {
    const validToken = createTestJWT({ sub: 'test_user_123' });
    const wsUtils = new WebSocketTestUtils(page);
    
    const authResult = await wsUtils.testAuthentication({
      token: validToken,
      expectedSuccess: true
    });

    console.log('Debug - Auth messages received:', authResult.authMessages);
    console.log('Debug - Auth success message:', authResult.authSuccessMessage);
    console.log('Debug - Errors:', authResult.errors);
    
    expect(authResult.authenticated).toBe(true);
    expect(authResult.authSuccessMessage).toBeTruthy();
    expect(authResult.authSuccessMessage.type).toBe('AuthSuccess');
    expect(authResult.authSuccessMessage.user_id).toBeDefined();
    expect(authResult.authSuccessMessage.user_id).toBe('test_user_123');
    expect(authResult.errors).toHaveLength(0);
    
    console.log('✅ AuthSuccess message sent correctly:', authResult.authSuccessMessage);
  });

  test('should send AuthFailed message with invalid JWT token', async ({ page }) => {
    const invalidToken = createInvalidTestJWT();
    const wsUtils = new WebSocketTestUtils(page);
    
    const authResult = await wsUtils.testAuthentication({
      token: invalidToken,
      expectedSuccess: false
    });

    console.log('Debug - Auth messages received:', authResult.authMessages);
    console.log('Debug - Auth failed message:', authResult.authFailedMessage);
    console.log('Debug - Errors:', authResult.errors);
    
    expect(authResult.authenticated).toBe(false);
    expect(authResult.authFailedMessage).toBeTruthy();
    expect(authResult.authFailedMessage.type).toBe('AuthFailed');
    expect(authResult.authFailedMessage.error).toBeDefined();
    expect(authResult.errors).toHaveLength(0);
    
    console.log('✅ AuthFailed message sent correctly:', authResult.authFailedMessage);
  });

  test('should send AuthFailed message with expired JWT token', async ({ page }) => {
    const expiredToken = createExpiredTestJWT({ sub: 'test_user_456' });
    const wsUtils = new WebSocketTestUtils(page);
    
    const authResult = await wsUtils.testAuthentication({
      token: expiredToken,
      expectedSuccess: false
    });

    expect(authResult.authenticated).toBe(false);
    expect(authResult.authFailedMessage).toBeTruthy();
    expect(authResult.authFailedMessage.type).toBe('AuthFailed');
    expect(authResult.authFailedMessage.error).toBeDefined();
    
    // Check if error message mentions expiration or invalid token
    const errorMessage = authResult.authFailedMessage.error.toLowerCase();
    const errorContainsExpired = errorMessage.includes('expired') || errorMessage.includes('invalid');
    expect(errorContainsExpired).toBe(true);
    expect(authResult.errors).toHaveLength(0);
    
    console.log('✅ AuthFailed message for expired token sent correctly:', authResult.authFailedMessage);
  });

  test('should send AuthFailed message with missing user ID in token', async ({ page }) => {
    const tokenWithoutUserId = createTestJWT({ sub: '' }); // Empty user ID
    const wsUtils = new WebSocketTestUtils(page);
    
    const authResult = await wsUtils.testAuthentication({
      token: tokenWithoutUserId,
      expectedSuccess: false
    });

    expect(authResult.authenticated).toBe(false);
    expect(authResult.authFailedMessage).toBeTruthy();
    expect(authResult.authFailedMessage.type).toBe('AuthFailed');
    expect(authResult.authFailedMessage.error).toBeDefined();
    
    // Check if error message mentions user ID issue or invalid token
    const errorMessage = authResult.authFailedMessage.error.toLowerCase();
    const errorContainsUserIdIssue = errorMessage.includes('user') || errorMessage.includes('invalid');
    expect(errorContainsUserIdIssue).toBe(true);
    expect(authResult.errors).toHaveLength(0);
    
    console.log('✅ AuthFailed message for missing user ID sent correctly:', authResult.authFailedMessage);
  });
});