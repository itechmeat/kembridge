# Constants Migration Guide

## Overview

This document describes the migration from hardcoded localhost URLs to centralized constants in the e2e-tests suite.

## New Constants Structure

All URL constants are now centralized in `/utils/test-constants.ts`:

```typescript
export const TEST_URLS = {
  // Frontend URLs
  FRONTEND: {
    LOCAL_DEV: "http://localhost:4100",
    STAGING: "https://staging.kembridge.io",
    PRODUCTION: "https://kembridge.io",
  },

  // Backend service URLs
  BACKEND: {
    GATEWAY: "http://localhost:4000",
    ONEINCH: "http://localhost:4001",
    BLOCKCHAIN: "http://localhost:4002",
    CRYPTO: "http://localhost:4003",
    AUTH: "http://localhost:4004",
    AI_ENGINE: "http://localhost:4005",
  },

  // WebSocket endpoints
  WEBSOCKET: {
    GATEWAY: "ws://localhost:4000/ws",
    FRONTEND: "ws://localhost:4100/ws",
  },
};
```

## Usage Examples

### Before Migration
```javascript
// ❌ Old way - hardcoded URLs
const ws = new WebSocket('ws://localhost:4000/ws');
await page.goto('http://localhost:4100/');
const response = await request.get('http://localhost:4005/health');
```

### After Migration
```javascript
// ✅ New way - using constants
import { TEST_URLS } from '../utils/test-constants';

const ws = new WebSocket(TEST_URLS.WEBSOCKET.GATEWAY);
await page.goto(TEST_URLS.FRONTEND.LOCAL_DEV);
const response = await request.get(`${TEST_URLS.BACKEND.AI_ENGINE}/health`);
```

## Environment Variable Support

The constants support environment variable overrides:

```bash
# Override base URL for testing
export TEST_BASE_URL="https://staging.kembridge.io"

# Override specific service URLs
export AI_ENGINE_URL="https://ai-staging.kembridge.io"
export BACKEND_URL="https://api-staging.kembridge.io"
```

## Utility Functions

The `TEST_UTILS` object provides helpful utility functions:

```typescript
// Get current base URL (respects environment variables)
const baseUrl = TEST_UTILS.getBaseUrl();

// Build full URL from path
const fullUrl = TEST_UTILS.buildUrl('/bridge');

// Get API endpoint URL
const apiUrl = TEST_UTILS.getApiUrl('/auth/nonce');

// Check if feature is enabled
if (TEST_UTILS.isFeatureEnabled('PERFORMANCE_TESTING')) {
  // Run performance tests
}
```

## Migration Checklist

### ✅ Completed Files
- [x] `test-constants.ts` - Added centralized URL constants
- [x] `enhanced-auth.spec.ts` - Updated to use new constants
- [x] `enhanced-error-handling.spec.ts` - Updated to use new constants
- [x] `enhanced-performance.spec.ts` - Created with new constants
- [x] `websocket-backend-test.spec.js` - Updated WebSocket URLs
- [x] `ai-risk-display.spec.js` - Updated AI Engine URL
- [x] `websocket-advanced.spec.js` - Updated frontend URLs
- [x] `api-helpers.js` - Updated service health check URLs
- [x] `constants.js` - Updated to use TEST_URLS constants
- [x] `playwright.config.js` - Added environment variable support

### 🔄 Files That May Need Updates
- [ ] `websocket-direct.spec.ts`
- [ ] `websocket-performance.spec.ts`
- [ ] `websocket-utils.ts`
- [ ] `ai-risk-engine.spec.js`
- [ ] `websocket-security.spec.ts`
- [ ] `websocket-integration.spec.js`
- [ ] `backend-error-recovery.spec.js`
- [ ] `crypto-service-api.spec.ts`
- [ ] `simple-websocket-test.spec.ts`
- [ ] `websocket-frontend-backend-integration.spec.js`
- [ ] `websocket-comprehensive.spec.ts`
- [ ] `gateway-crypto-proxy.spec.ts`

## Benefits of Migration

1. **Centralized Configuration**: All URLs in one place
2. **Environment Flexibility**: Easy switching between dev/staging/prod
3. **Maintainability**: Single source of truth for URLs
4. **Type Safety**: TypeScript constants with proper typing
5. **Consistency**: Standardized URL usage across all tests
6. **Feature Flags**: Conditional test execution based on environment

## Best Practices

1. **Always import constants**: Never hardcode URLs in test files
2. **Use utility functions**: Leverage `TEST_UTILS` for common operations
3. **Respect environment variables**: Allow runtime URL overrides
4. **Document changes**: Update this guide when adding new constants
5. **Test across environments**: Verify tests work with different URL configurations

## Troubleshooting

### Common Issues

1. **Import errors**: Make sure to import from the correct path
   ```typescript
   import { TEST_URLS } from '../utils/test-constants';
   ```

2. **Missing constants**: Check if the URL constant exists in `TEST_URLS`

3. **Environment variables not working**: Verify the variable name matches the expected format

4. **WebSocket connection failures**: Ensure WebSocket URLs use `ws://` or `wss://` protocol

### Debug Commands

```bash
# Check current environment variables
echo $TEST_BASE_URL
echo $AI_ENGINE_URL
echo $BACKEND_URL

# Run tests with debug output
DEBUG=true npm run test:e2e

# Run tests with custom URLs
TEST_BASE_URL=https://staging.kembridge.io npm run test:e2e
```

## Future Improvements

1. **Service Discovery**: Automatic service URL detection
2. **Health Checks**: Pre-test service availability validation
3. **Load Balancing**: Support for multiple service instances
4. **SSL/TLS**: Automatic protocol switching for secure environments
5. **Monitoring**: URL performance and availability tracking