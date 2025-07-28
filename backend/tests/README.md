# KEMBridge Backend Tests

This directory contains integration tests for verifying KEMBridge backend functionality.

## Available Tests

### 1. API Integration Test (`test_api_integration.rs`)

Tests integration with external APIs:

- 1inch API for price quotes
- Ethereum RPC for blockchain interaction
- Real endpoint verification

```bash
cargo run --bin test_api_integration
```

### 2. Authentication System Test (`test_auth_system.rs`)

Basic authentication system testing:

- JWT token generation and verification
- User tier determination based on wallet address
- Auth middleware logic simulation
- Public endpoint routing verification

```bash
cargo run --bin test_auth_system
```

### 3. HTTP Authentication Test (`test_auth_http.rs`)

Full HTTP integration test for authentication:

- Testing with real server (requires server to be running)
- Protected endpoints verification
- Role-based access control testing
- JWT token validation through HTTP requests
- Admin vs regular user access testing

```bash
cargo run --bin test_auth_http
```

### 4. Authentication Integration Test (`test_auth_integration.rs`)

Comprehensive integration test for authentication updates:

- Tests authentication components with REAL DATA ONLY
- No mocks, stubs, or fallbacks - all real validation
- JWT authentication core functionality
- User tier determination logic
- Token validation and claims extraction
- Full authentication flow integration

```bash
cargo run --bin test_auth_integration
```

### 5. Rate Limiting Test (`test_rate_limiting.rs`)

Unit tests for rate limiting functionality:

- Tests rate limiting constants validation
- Rate limiting configuration logic verification
- Time window calculations
- Mock implementation for real integration tests

```bash
cargo test test_rate_limiting
```

### 6. Rate Limiting HTTP Integration Test (`test_rate_limiting.sh`)

Shell script for HTTP integration testing of rate limiting endpoints:

- Tests rate limiting monitoring endpoints
- Verifies authentication requirements for admin endpoints
- Checks API documentation accessibility
- Real HTTP requests to running server

```bash
# Requires running backend server
./tests/test_rate_limiting.sh
```

## Prerequisites

### Test Dependencies

| Test                       | PostgreSQL | Backend Server | External APIs | Notes                    |
| -------------------------- | ---------- | -------------- | ------------- | ------------------------ |
| `test_api_integration.rs`  | ❌         | ❌             | ✅            | Requires 1inch API key   |
| `test_auth_system.rs`      | ❌         | ❌             | ❌            | Standalone unit tests    |
| `test_auth_http.rs`        | ✅         | ✅             | ❌            | Full HTTP integration    |
| `test_auth_integration.rs` | ❌         | ❌             | ❌            | Component integration    |
| `test_rate_limiting.rs`    | ❌         | ❌             | ❌            | Unit tests for constants |
| `test_rate_limiting.sh`    | ✅         | ✅             | ❌            | HTTP integration script  |

### General Requirements:

- **PostgreSQL**: Required for HTTP tests (via Docker)
- **Backend Server**: Required for HTTP tests only
- **External APIs**: Required for API integration tests only

### Starting Server for HTTP Tests

```bash
# From project root
make dev

# Or
docker-compose up -d
```

### Environment Variables

For API tests, create a `.env` file:

```env
# 1inch API
ONEINCH_API_KEY=your_api_key_here

# Ethereum RPC
ETHEREUM_RPC_URL=https://rpc.sepolia.org

# JWT Secret (for HTTP tests)
JWT_SECRET=hackathon-super-secret-key-change-in-production
```

## Manual Testing Examples

### Authentication Flow

```bash
# 1. Generate nonce
curl -X GET "http://localhost:4000/api/v1/auth/nonce?wallet_address=0x742d35Cc6635C0532925a3b8D400a69ee0f44AD2&chain_type=ethereum"

# 2. Verify signature (sign message in MetaMask first)
curl -X POST http://localhost:4000/api/v1/auth/verify-wallet \
  -H "Content-Type: application/json" \
  -d '{"wallet_address": "0x742d35Cc6635C0532925a3b8D400a69ee0f44AD2", "chain_type": "ethereum", "message": "MESSAGE_FROM_STEP_1", "signature": "0xYOUR_SIGNATURE", "nonce": "NONCE_FROM_STEP_1"}'

# 3. Use JWT token for protected endpoints
curl -X GET http://localhost:4000/api/v1/user/profile \
  -H "Authorization: Bearer YOUR_JWT_TOKEN"

# 4. Test logout
curl -X POST http://localhost:4000/api/v1/auth/logout \
  -H "Authorization: Bearer YOUR_JWT_TOKEN"
```

### User Management Endpoints

```bash
# Get user profile
curl -X GET http://localhost:4000/api/v1/user/profile \
  -H "Authorization: Bearer YOUR_JWT_TOKEN"

# Update user profile
curl -X PUT http://localhost:4000/api/v1/user/profile \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"username": "new_username", "profile_data": {"email": "user@example.com"}}'

# Get user wallets
curl -X GET http://localhost:4000/api/v1/user/wallets \
  -H "Authorization: Bearer YOUR_JWT_TOKEN"

# Add new wallet
curl -X POST http://localhost:4000/api/v1/user/wallets \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"wallet_address": "0x...", "chain_type": "ethereum", "signature": "0x...", "message": "verification message"}'
```

### Database & Redis Verification

```sql
-- Check user data
SELECT u.id, u.created_at, uam.wallet_address, us.expires_at
FROM users u
JOIN user_auth_methods uam ON u.id = uam.user_id
JOIN user_sessions us ON u.id = us.user_id
ORDER BY u.created_at DESC LIMIT 1;
```

```bash
# Check Redis nonce storage
docker-compose exec redis redis-cli -a dev_redis_password KEYS "kembridge:auth:nonce:*"
```

## Interpreting Results

### Successful Tests

- ✅ Green checkmarks indicate successful verifications
- HTTP 200 OK for successful requests
- HTTP 401/403 for proper authorization handling

### Errors

- ❌ Red X marks indicate problems
- HTTP 500 codes may indicate server issues
- Timeout errors may indicate network problems

## Test Selection Guide

### When to Use Each Test

| Scenario                 | Recommended Test                                   | Reason                                  |
| ------------------------ | -------------------------------------------------- | --------------------------------------- |
| **Unit Testing**         | `test_auth_system.rs`                              | Fast, no dependencies, basic validation |
| **HTTP Integration**     | `test_auth_http.rs`                                | Full server testing, real endpoints     |
| **Component Validation** | `test_auth_integration.rs`                         | Deep authentication logic testing       |
| **API Integration**      | `test_api_integration.rs`                          | External service connections            |
| **Development**          | `test_auth_system.rs`                              | Quick feedback during development       |
| **CI/CD Pipeline**       | `test_auth_system.rs` + `test_auth_integration.rs` | No external dependencies                |
| **Production Testing**   | `test_auth_http.rs`                                | Full system verification                |

### Test Execution Order

For comprehensive testing, run in this order:

1. `test_auth_system.rs` - Basic functionality
2. `test_auth_integration.rs` - Component integration
3. `test_auth_http.rs` - HTTP integration (requires server)
4. `test_api_integration.rs` - External APIs (requires keys)

## Common Issues

1. **Server not running**: Ensure `make dev` is executed
2. **Database unavailable**: Check Docker containers
3. **API keys**: Ensure environment variables are configured
4. **JWT secret**: Verify secret matches between test and Docker environment
5. **Port conflicts**: Ensure port 4000 is available for HTTP tests

## Expected Outputs

### API Integration Test

```
🚀 KEMBridge API Integration Test
==================================

🔗 Testing Ethereum RPC Connection...
   ✅ Connected! Latest block: 0x85f4ea
   ✅ Network: Sepolia testnet ✓

📱 Testing 1inch API Connection...
   ✅ Found 139 liquidity sources
   ✅ API key validation successful

✅ All API integrations tested successfully!
```

### Authentication System Test

```
🔐 Testing KEMBridge Authentication System
==========================================

1. Testing JWT Manager...
✅ JWT token generated and verified
✅ User tier determination working

2. Testing Auth Middleware Logic...
✅ Route protection logic validated

✅ Authentication system tests completed!
```

### HTTP Authentication Test

```
🌐 Testing KEMBridge Authentication HTTP Integration
==================================================

✅ Server connectivity verified
✅ Protected endpoints return 401 without auth
✅ Admin endpoints return 200 with admin token
✅ Admin endpoints return 403 with regular token

✅ HTTP Authentication tests completed!
```

### Authentication Integration Test

```
🔐 KEMBridge Authentication Integration Test
============================================
Testing all authentication updates with REAL DATA ONLY
❌ NO MOCKS, NO STUBS, NO FALLBACKS

1️⃣ Testing JWT Authentication Core...
   ✅ Real JWT token generated: eyJ0eXAiOiJKV1QiLCJh...
   ✅ JWT token verified successfully
   ✅ JWT correctly rejected invalid token

2️⃣ Testing User Tier Determination...
   ✅ 0x000admin_wallet -> Admin (Admin prefix 0x000)
   ✅ admin_wallet_123 -> Admin (Admin prefix 'admin')
   ✅ Premium wallets correctly identified
   ✅ Regular wallets correctly identified

3️⃣ Testing Token Validation and Claims...
   ✅ Admin user token validated successfully
   ✅ Premium user token validated successfully
   ✅ Regular user token validated successfully

4️⃣ Testing Authentication Flow Integration...
   ✅ Full authentication flow simulation completed
   ✅ Admin access granted for quantum operations
   ✅ System-level access granted

✅ All authentication integration tests PASSED!
🛡️  All components use real data - no mocks detected
```
