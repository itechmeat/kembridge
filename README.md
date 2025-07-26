# KEMBridge

## Cross-Chain Intelligence Meets Quantum Security

KEMBridge is an autonomous cross-chain bridge that enables secure asset transfers between different blockchains using post-quantum cryptography for protection against future quantum attacks. The project combines Near Protocol technologies (Chain Signatures, Shade Agents, 1Click API) with 1inch (Fusion+) to create a fully automated and AI-powered bridge.

## Quick Start

### Prerequisites

- **Docker** (v28.3.1 or later)
- **Docker Compose** (included with Docker Desktop)
- **Git**

### One-Command Setup

```bash
# Clone the repository
git clone <kembridge-repo>
cd kembridge-mono

# Start all services
docker-compose up --build
```

This single command will:

- Build and start 8 containerized services
- Set up PostgreSQL 18 Beta 1 database
- Configure Redis cache
- Launch Rust backend with hot reload
- Start React frontend with Vite
- Initialize AI engine with FastAPI
- Set up monitoring with Prometheus & Grafana
- Configure Nginx reverse proxy

### Service Endpoints

After successful startup, you can access:

| Service         | URL                   | Description                         |
| --------------- | --------------------- | ----------------------------------- |
| **Frontend**    | http://localhost:4001 | React app with quantum bridge UI    |
| **Backend API** | http://localhost:4000 | Rust API with health endpoint       |
| **AI Engine**   | http://localhost:4003 | Python ML risk analysis             |
| **Grafana**     | http://localhost:4002 | Monitoring dashboards (admin:admin) |
| **Prometheus**  | http://localhost:4004 | Metrics collection                  |

### Verify Installation

1. **Check all services are running:**

   ```bash
   docker-compose ps
   ```

   All 8 containers should show "Up" status.

2. **Test health endpoints:**

   ```bash
   curl http://localhost:4000/health
   curl http://localhost:4003/health
   ```

3. **View frontend:**
   Open http://localhost:4001 in your browser.

### Expected Results

✅ **Backend Health Response:**

```json
{
  "status": "healthy",
  "service": "kembridge-backend",
  "version": "0.1.0",
  "timestamp": "2025-07-11T...",
  "components": {
    "database": "ready",
    "redis": "ready",
    "auth": "ready"
  }
}
```

✅ **AI Engine Health Response:**

```json
{
  "status": "healthy",
  "service": "kembridge-ai-engine",
  "version": "0.1.0"
}
```

✅ **Frontend:** Welcome page with "KEMBridge - Cross-Chain Intelligence Meets Quantum Security"

### Development Commands

```bash
# View logs from all services
docker-compose logs -f

# View logs from specific service
docker-compose logs -f backend

# Access backend shell
docker-compose exec backend bash

# Access frontend shell
docker-compose exec frontend sh

# Stop all services (data preserved in volumes)
docker-compose down

# WARNING: Remove ALL data (database, cache, volumes)
docker-compose down -v

# WARNING: Complete cleanup (data + images)
docker-compose down -v --rmi all

# Run health checks
make health
```

### Background Mode (Detached)

To run all containers in the background so you can close the terminal:

```bash
# Start all services in background mode
docker-compose up -d --build

# Check status of all services
docker-compose ps

# View logs without blocking terminal
docker-compose logs

# Follow logs in real-time (Ctrl+C to exit)
docker-compose logs -f

# Follow logs for specific service
docker-compose logs -f backend

# Stop all background services (data preserved in volumes)
docker-compose down
```

⚠️ **Important**: `docker-compose down` only stops containers but **preserves all data** in volumes (PostgreSQL DB, Redis cache, build caches). For complete cleanup use `docker-compose down -v`.

**Advantages of background mode:**

- Terminal remains available for other commands
- Services continue running when terminal is closed
- Easy to manage multiple services
- Perfect for long-running development sessions

### Service Management

```bash
# Start specific service
docker-compose up -d postgres

# Restart specific service
docker-compose restart backend

# Stop specific service
docker-compose stop frontend

# View resource usage
docker-compose top

# Scale services (if supported)
docker-compose up -d --scale ai-engine=2
```

### Monitoring and Observability

KEMBridge includes comprehensive monitoring with **Prometheus** and **Grafana**:

#### Accessing Monitoring Dashboards

```bash
# Open Grafana in your browser
open http://localhost:4002

# Default credentials:
# Username: admin
# Password: admin
```

#### Available Dashboards

1. **KEMBridge Overview** - Service uptime and health status
2. **Backend Metrics** - API response times, error rates
3. **AI Engine Performance** - ML model execution times
4. **Database Metrics** - PostgreSQL connections and queries
5. **Infrastructure** - Container resource usage

#### Prometheus Metrics

Access raw metrics at: http://localhost:4004

**Key metrics monitored:**

- Service uptime (`up`)
- HTTP request duration (`http_request_duration_seconds`)
- Database connection pools (`db_connections_active`)
- Memory and CPU usage per container
- Cross-chain transaction success rates

#### Custom Metrics Setup

To add metrics to your service:

1. **Backend (Rust):** Add `prometheus` crate and expose `/metrics` endpoint
2. **AI Engine (Python):** Use `prometheus_client` library
3. **Frontend (Node.js):** Add `prom-client` for client-side metrics

#### Alerting (Optional)

Set up alerts in Grafana for:

- Service downtime > 1 minute
- High error rates > 5%
- Database connection failures
- Memory usage > 80%

### Using the System

Currently available endpoints:

1. **Backend API:**

   - `GET /health` - Service health check
   - `POST /api/v1/auth/nonce` - Authentication nonce (placeholder)
   - `POST /api/v1/auth/verify` - Wallet verification (placeholder)

2. **AI Engine:**

   - `GET /health` - Service health check
   - `POST /api/risk/analyze` - Risk analysis for transactions

3. **Monitoring:**
   - **Grafana:** http://localhost:4002 (admin:admin)
   - **Prometheus:** http://localhost:4004

### Architecture

The system runs 8 Docker containers:

- **kembridge_postgres** - PostgreSQL 18 Beta 1 with quantum key storage
- **kembridge_redis** - Redis 8.0.3 for caching and sessions
- **kembridge_backend** - Rust + Axum API with hot reload
- **kembridge_ai_engine** - Python + FastAPI for ML risk analysis
- **kembridge_frontend** - React 19 + Vite + TypeScript with hot reload
- **kembridge_nginx** - Reverse proxy for routing
- **kembridge_prometheus** - Metrics collection
- **kembridge_grafana** - Monitoring dashboards

### Technology Stack (Latest Versions - July 2025)

- **Docker:** v28.3.1
- **PostgreSQL:** 18beta1 (OAuth 2.0 support, enhanced JSON performance)
- **Redis:** 8.0.3-alpine
- **Rust:** 1.88.0 (latest stable with let chains)
- **Node.js:** 22.17.0 (LTS 'Jod' with built-in WebSocket support)
- **pnpm:** 10.12.1 (faster than npm, shared node_modules)
- **Python:** 3.12.5 (compatible with pandas)
- **Nginx:** 1.28.0 (latest stable)

### Troubleshooting

**Common Issues:**

1. **Port conflicts:** Ensure ports 4000, 4001, 4002, 4003, 4004, 5432, 6379 are free
2. **Docker memory:** Ensure Docker has at least 4GB RAM allocated
3. **Build failures:** Run `docker-compose down -v && docker-compose up --build`
4. **Node-gyp/bufferutil errors:** Fixed by installing build dependencies in Alpine and using WS_NO_BUFFER_UTIL=1

**Get help:**

```bash
# Check container status
docker-compose ps

# View specific service logs
docker-compose logs [service-name]

# Restart specific service
docker-compose restart [service-name]
```

### Next Steps

This is Phase 1.1 of the KEMBridge development plan. Next phases will add:

- Database schema and migrations
- Web3 authentication
- Post-quantum cryptography implementation
- Blockchain adapters for Ethereum and NEAR
- AI risk analysis models

## Key Features

- **Post-Quantum Security**: Uses NIST-standardized algorithms (Kyber, Dilithium, SPHINCS+)
- **AI-Powered Security**: Autonomous agents monitor transactions and detect threats in real-time
- **Simplified UX**: One-click cross-chain swaps through Near 1Click API
- **Atomic Swaps**: Integration with 1inch Fusion+ ensures atomic operations
- **Autonomous Agents**: Shade Agents provide decentralized security management in TEE

## Why Quantum Security Matters

Quantum computers pose a real threat to current blockchain security. KEMBridge provides future-proof protection using cryptographic algorithms that remain secure even against quantum attacks, positioning itself as essential infrastructure for the post-quantum era.
