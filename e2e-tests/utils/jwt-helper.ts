/**
 * JWT Helper for E2E Tests
 * Generates valid JWT tokens for testing WebSocket authentication
 */

import * as crypto from 'crypto';

interface JWTClaims {
  sub: string; // user_id
  exp: number; // expiration timestamp
  iat: number; // issued at timestamp
  iss?: string; // issuer
  aud?: string; // audience
  wallet_address?: string;
  user_tier?: string;
}

/**
 * Create a valid JWT token for testing
 * Uses the same secret as the server: "default_secret_key_for_development"
 */
export function createTestJWT(claims: Partial<JWTClaims> = {}): string {
  const secret = 'dev_super_secret_jwt_key_for_development_only';
  const now = Math.floor(Date.now() / 1000);
  
  const defaultClaims: JWTClaims = {
    sub: 'test_user_123',
    exp: now + 3600, // expires in 1 hour
    iat: now,
    iss: 'kembridge-test',
    aud: 'kembridge-gateway',
    wallet_address: '0x742d35Cc6634C0532925a3b8D4C9db96C4b4d8bE',
    user_tier: 'standard',
    ...claims
  };

  // Create JWT header
  const header = {
    alg: 'HS256',
    typ: 'JWT'
  };

  // Base64URL encode header and payload
  const encodedHeader = base64UrlEncode(JSON.stringify(header));
  const encodedPayload = base64UrlEncode(JSON.stringify(defaultClaims));
  
  // Create signature
  const data = `${encodedHeader}.${encodedPayload}`;
  const signature = crypto
    .createHmac('sha256', secret)
    .update(data)
    .digest('base64')
    .replace(/\+/g, '-')
    .replace(/\//g, '_')
    .replace(/=/g, '');

  return `${data}.${signature}`;
}

/**
 * Create an expired JWT token for testing
 */
export function createExpiredTestJWT(claims: Partial<JWTClaims> = {}): string {
  const now = Math.floor(Date.now() / 1000);
  return createTestJWT({
    ...claims,
    exp: now - 3600, // expired 1 hour ago
    iat: now - 7200  // issued 2 hours ago
  });
}

/**
 * Create an invalid JWT token for testing
 */
export function createInvalidTestJWT(): string {
  return 'invalid.jwt.token';
}

/**
 * Base64URL encode function
 */
function base64UrlEncode(str: string): string {
  return Buffer.from(str)
    .toString('base64')
    .replace(/\+/g, '-')
    .replace(/\//g, '_')
    .replace(/=/g, '');
}

/**
 * Decode JWT token for debugging (without verification)
 */
export function decodeJWT(token: string): { header: any; payload: any; signature: string } | null {
  try {
    const parts = token.split('.');
    if (parts.length !== 3) {
      return null;
    }

    const header = JSON.parse(Buffer.from(parts[0], 'base64').toString());
    const payload = JSON.parse(Buffer.from(parts[1], 'base64').toString());
    
    return {
      header,
      payload,
      signature: parts[2]
    };
  } catch (error) {
    return null;
  }
}