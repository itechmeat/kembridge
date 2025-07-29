# KEMBridge Backend

Rust 1.88+ backend for KEMBridge quantum-secure cross-chain bridge with PostgreSQL 18 Beta 1, Web3 authentication, and enterprise-grade security features.

## Quick Start

### Using Makefile (Recommended)

```bash
# Start all services with hot reload
make dev

# Start in background mode
make dev-detached

# Check system health
curl http://localhost:4000/health

# View API documentation
open http://localhost:4000/docs
```

### Manual Docker Commands

```bash
# Build and start all services
docker-compose up --build

# Start in background
docker-compose up -d

# Rebuild without cache
docker-compose build --no-cache && docker-compose down && docker-compose up -d

# Start specific service
docker-compose up -d postgres redis backend
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

## Development Setup

### Prerequisites

- **Docker & Docker Compose** (required)
- **Rust 1.88+** (with edition 2024 support)
- **Make** (for convenience commands)

### Development Commands

#### Primary Commands (Use Makefile)

```bash
# Start all services with hot reload
make dev

# Start in background mode
make dev-detached

# View logs from all services
make logs

# View logs from specific service
make logs-backend
make logs-frontend
make logs-postgres

# Run health checks
make health

# Database migrations
make migrate

# Access service shells
make shell-backend
make shell-frontend

# Complete cleanup (removes all data)
make clean
```

#### Docker Compose Commands

```bash
# Build and start all services
docker-compose up --build

# Start specific service
docker-compose up -d postgres redis

# View logs for specific service
docker-compose logs -f backend
docker-compose logs -f postgres

# Restart specific service
docker-compose restart backend

# Stop all services
docker-compose down

# Clean rebuild (removes cached layers)
docker-compose build --no-cache
docker-compose down
docker-compose up -d
```

#### Backend Commands (within container)

```bash
# Access backend container shell
make shell-backend
# OR: docker-compose exec backend bash

# Run specific binary
cargo run --bin kembridge-backend

# Run integration tests
cargo run --bin test_api_integration
cargo run --bin test_auth_system
cargo run --bin test_auth_http

# Check code compilation
cargo check

# Database migrations (from within container)
sqlx migrate run
```

### Database & Cache Access

```bash
# Connect to PostgreSQL
docker-compose exec postgres psql -U postgres -d kembridge_dev

# Connect to Redis
docker-compose exec redis redis-cli -a dev_redis_password

# Check database connection
docker-compose exec postgres pg_isready -U postgres
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

## Testing

**📋 Complete testing documentation:** [tests/README.md](tests/README.md)

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

## Configuration

### Environment Variables

The project uses environment variables defined in `docker-compose.yml`. For local development, these are automatically configured:

```env
# Database
DATABASE_URL=postgresql://postgres:dev_password@postgres:5432/kembridge_dev
REDIS_URL=redis://:dev_redis_password@redis:6379

# Authentication
JWT_SECRET=hackathon-super-secret-key-change-in-production

# API Services
ONEINCH_API_KEY=your_api_key_here
ETHEREUM_RPC_URL=https://rpc.sepolia.org
NEAR_RPC_URL=https://rpc.testnet.near.org
AI_ENGINE_URL=http://ai-engine:8000

# Server Configuration
PORT=4000
HOST=0.0.0.0
ENVIRONMENT=development
ENABLE_SWAGGER_UI=true
CORS_ALLOWED_ORIGINS=http://localhost:4001,http://localhost:4100

# Blockchain
ETHEREUM_CHAIN_ID=11155111  # Sepolia testnet
```

### Docker Services Configuration

| Service    | Container Port | Host Port | Purpose                   |
|------------|----------------|-----------|---------------------------|
| backend    | 4000           | 4000      | Rust API server          |
| frontend   | 3000           | 4001      | React development server |
| grafana    | 3000           | 4002      | Monitoring dashboards    |
| ai-engine  | 8000           | 4003      | Python FastAPI service   |
| prometheus | 9090           | 4004      | Metrics collection       |
| postgres   | 5432           | 5432      | PostgreSQL database      |
| redis      | 6379           | 6379      | Cache and sessions       |

## Troubleshooting

### Health Checks

```bash
# Check all services status
docker-compose ps

# Health check via Makefile
make health

# Manual health checks
curl http://localhost:4000/health        # Backend API
curl http://localhost:4003/health        # AI Engine
open http://localhost:4002               # Grafana (admin:admin)
open http://localhost:4004               # Prometheus
```

### Viewing Logs

```bash
# All services logs
make logs

# Specific service logs
docker-compose logs -f backend
docker-compose logs -f postgres
docker-compose logs -f redis
docker-compose logs -f ai-engine

# Backend with timestamp
docker-compose logs -f --timestamps backend
```

### Common Issues & Solutions

#### Backend Not Starting

```bash
# Check compilation errors
docker-compose logs backend

# Access container for debugging
docker-compose exec backend bash
cargo check

# Rebuild backend service
docker-compose build backend
docker-compose restart backend
```

#### Database Connection Issues

```bash
# Check PostgreSQL health
docker-compose exec postgres pg_isready -U postgres

# Check database logs
docker-compose logs postgres

# Test connection from backend
docker-compose exec backend bash
psql postgresql://postgres:dev_password@postgres:5432/kembridge_dev
```

#### Redis Connection Issues

```bash
# Check Redis health
docker-compose exec redis redis-cli -a dev_redis_password ping

# Check Redis logs
docker-compose logs redis

# Clear Redis cache if needed
docker-compose exec redis redis-cli -a dev_redis_password FLUSHALL
```

#### Port Conflicts

```bash
# Check what's using ports
lsof -i :4000  # Backend
lsof -i :4001  # Frontend
lsof -i :5432  # PostgreSQL
lsof -i :6379  # Redis

# Kill conflicting processes
sudo lsof -ti:4000 | xargs kill -9
```

#### Complete Reset

```bash
# Nuclear option - removes all data
make clean

# Manual cleanup
docker-compose down -v --remove-orphans
docker system prune -f
docker volume prune -f
make dev
```

### Performance Monitoring

#### Database Performance

```sql
-- Monitor query performance
SELECT query, calls, total_time, mean_time
FROM pg_stat_statements
ORDER BY total_time DESC LIMIT 10;

-- Check index usage
SELECT schemaname, tablename, attname, n_distinct
FROM pg_stats
WHERE tablename IN ('users', 'transactions', 'audit_logs');

-- Check active connections
SELECT count(*), state FROM pg_stat_activity GROUP BY state;
```

#### Service Monitoring

```bash
# View Prometheus metrics
open http://localhost:4004/metrics

# View Grafana dashboards
open http://localhost:4002  # admin:admin

# Backend metrics endpoint
curl http://localhost:4000/metrics
```

#### Docker Resource Usage

```bash
# Monitor container resources
docker stats

# Check disk usage
docker system df

# Container logs size
docker-compose logs --tail=0 backend | wc -l
```

## Current Implementation Status

### ✅ Completed (Phase 1, 2.1, 2.2 & 2.3)

- PostgreSQL 18 database with advanced features
- Rust backend with Axum 0.8.4 framework
- Web3 authentication (Ethereum signature verification)
- JWT session management with middleware
- Advanced Auth extractors (AuthUser, OptionalAuth, AdminAuth, PremiumAuth)
- Logout & refresh token functionality
- Redis nonce management
- **User Management System:**
  - GET/PUT/DELETE `/api/v1/user/profile` endpoints
  - Multi-wallet support (add/remove/set primary)
  - Soft delete functionality
  - User statistics and profile data
  - UserService integration with AppState
- Comprehensive audit logging
- API documentation with Swagger UI

### ⏸️ In Progress (Phase 2.3 - Final Tasks)

- Automatic user creation on first authentication
- Input validation for user data
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
