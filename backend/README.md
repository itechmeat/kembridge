# KEMBridge Backend

Rust 1.88+ backend for KEMBridge quantum-secure cross-chain bridge with PostgreSQL 18 Beta 1, Web3 authentication, and enterprise-grade security features.

## Quick Start

```bash
# Start all services
make dev

# Check system health
curl http://localhost:4000/health

# View API documentation
open http://localhost:4000/docs
```

## Architecture

- **Framework**: Axum 0.8.4 with tower middleware
- **Database**: PostgreSQL 18 Beta 1 with advanced features
- **Cache**: Redis 8.0.3 for sessions and nonce management
- **Authentication**: Web3 wallet signature verification (Ethereum/NEAR)
- **Cryptography**: Post-quantum ML-KEM-1024, Dilithium-5
- **Monitoring**: Prometheus metrics with Grafana dashboards

## Core Features

### Database Schema

- **UUIDv7**: Timestamp-ordered UUIDs for optimal performance
- **Virtual Generated Columns**: Real-time calculated analytics fields
- **Enhanced JSONB**: SIMD-optimized JSON operations
- **Post-Quantum Crypto**: ML-KEM-1024 and Dilithium-5 key storage
- **Comprehensive Audit**: Every operation logged with threat detection

### Web3 Authentication

- **Multi-chain Support**: Ethereum (secp256k1) and NEAR (ed25519)
- **Nonce Management**: Redis-based with TTL expiration
- **JWT Tokens**: Web3-specific claims and session management
- **Signature Verification**: Real cryptographic validation

### Security

- **Risk Assessment**: AI-powered transaction risk scoring
- **Audit Logging**: Comprehensive operation tracking
- **OAuth 2.0**: Enterprise-grade authentication ready
- **HSM Support**: Hardware Security Module integration

## Service Endpoints

| Service     | Port | URL                   |
| ----------- | ---- | --------------------- |
| Backend API | 4000 | http://localhost:4000 |
| Frontend    | 4001 | http://localhost:4001 |
| Grafana     | 4002 | http://localhost:4002 |
| AI Engine   | 4003 | http://localhost:4003 |
| Prometheus  | 4004 | http://localhost:4004 |

## Development

### Prerequisites

- Docker & Docker Compose
- Rust 1.88+ (with nightly toolchain for edition 2024)
- PostgreSQL 18 Beta 1

### Commands

```bash
# Development mode with hot reload
make dev

# Check compilation
cargo check --package kembridge-auth

# Database migrations
make migrate

# View logs
make logs

# Cleanup
make clean
```

### Database Connection

```bash
# Connect to PostgreSQL
docker-compose exec postgres psql -U postgres -d kembridge_dev

# Check Redis
docker-compose exec redis redis-cli
```

## Testing Authentication & JWT

### 1. Generate Nonce

```bash
curl -X GET "http://localhost:4000/api/v1/auth/nonce?wallet_address=0x742d35Cc6635C0532925a3b8D400a69ee0f44AD2&chain_type=ethereum"
```

### 2. Verify Signature (returns JWT token)

```bash
# Sign the message from step 1 in MetaMask, then:
curl -X POST http://localhost:4000/api/v1/auth/verify-wallet \
  -H "Content-Type: application/json" \
  -d '{
    "wallet_address": "0x742d35Cc6635C0532925a3b8D400a69ee0f44AD2",
    "chain_type": "ethereum",
    "message": "MESSAGE_FROM_STEP_1",
    "signature": "0xYOUR_METAMASK_SIGNATURE",
    "nonce": "NONCE_FROM_STEP_1"
  }'
```

### 3. Test JWT Middleware

```bash
# Test protected endpoint without token (should fail)
curl -X POST http://localhost:4000/api/v1/auth/refresh

# Test with invalid token (should fail)
curl -X POST http://localhost:4000/api/v1/auth/refresh \
  -H "Authorization: Bearer invalid_token"

# Test with valid token (use JWT from step 2)
curl -X POST http://localhost:4000/api/v1/auth/refresh \
  -H "Authorization: Bearer YOUR_JWT_TOKEN"
```

### 4. Test Logout

```bash
# Logout (invalidates session in database)
curl -X POST http://localhost:4000/api/v1/auth/logout \
  -H "Authorization: Bearer YOUR_JWT_TOKEN"
```

### 5. Verify Database & Redis

```sql
-- Check created user and auth data
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

## Database Schema

### Core Tables

- **users**: User profiles with AI risk assessment
- **user_auth_methods**: Web3 wallet and OAuth authentication
- **user_sessions**: JWT session management with security monitoring
- **quantum_keys**: Post-quantum cryptographic key storage
- **transactions**: Cross-chain bridge operations with encryption
- **audit_logs**: Comprehensive operation logging (partitioned)

### Advanced Features

- **Generated Columns**: Automatic analytics calculations
- **GIN Indexes**: Optimized JSONB queries
- **Partitioning**: Monthly audit log partitions
- **Extended Statistics**: Query optimization

## Migration Files

| File                                            | Purpose                        |
| ----------------------------------------------- | ------------------------------ |
| `001_postgresql18_extensions_config.sql`        | Extensions and configuration   |
| `002_users_table_postgresql18.sql`              | User management with analytics |
| `003_auth_methods_oauth_postgresql18.sql`       | Authentication methods         |
| `004_user_sessions_advanced_postgresql18.sql`   | Session management             |
| `005_quantum_keys_postgresql18.sql`             | Post-quantum cryptography      |
| `006_transactions_advanced_postgresql18.sql`    | Cross-chain transactions       |
| `007_audit_logs_comprehensive_postgresql18.sql` | Audit logging                  |

## Environment Variables

```env
DATABASE_URL=postgresql://postgres:dev_password@localhost:5432/kembridge_dev
REDIS_URL=redis://localhost:6379
JWT_SECRET=your-secret-key
PORT=4000
ENVIRONMENT=development
```

## Troubleshooting

### Common Issues

```bash
# Check services status
docker-compose ps

# View specific logs
docker-compose logs backend
docker-compose logs postgres
docker-compose logs redis

# Database health
docker-compose exec postgres pg_isready -U postgres

# Check compilation
cargo check --package kembridge-auth
```

### Performance

```sql
-- Monitor query performance
SELECT query, calls, total_time, mean_time
FROM pg_stat_statements
ORDER BY total_time DESC LIMIT 10;

-- Check index usage
SELECT schemaname, tablename, attname, n_distinct
FROM pg_stats
WHERE tablename IN ('users', 'transactions', 'audit_logs');
```

## Current Implementation Status

### ✅ Completed (Phase 1, 2.1 & 2.2)

- PostgreSQL 18 database with advanced features
- Rust backend with Axum 0.8.4 framework
- Web3 authentication (Ethereum signature verification)
- JWT session management with middleware
- Advanced Auth extractors (AuthUser, OptionalAuth, AdminAuth, PremiumAuth)
- Logout & refresh token functionality
- Redis nonce management
- Comprehensive audit logging
- API documentation with Swagger UI

### ⏸️ In Progress (Phase 2.3)

- User management endpoints
- NEAR signature verification (postponed to Phase 4.2)

### 🔄 Next Steps

- Quantum cryptography module (Phase 3)
- Blockchain adapters (Phase 4)
- AI risk engine integration (Phase 5)

## Security Considerations

- All sensitive data encrypted at rest
- Web3 wallet signature verification
- Comprehensive audit trail
- Post-quantum cryptography ready
- Rate limiting and CORS protection
- SQL injection prevention with SQLx

---

**Documentation**: Full API docs available at `/docs` endpoint  
**Support**: Check logs and health endpoints for debugging
