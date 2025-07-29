# KEMBridge

## Cross-Chain Intelligence Meets Quantum Security

KEMBridge is an autonomous cross-chain bridge that enables secure asset transfers between different blockchains using post-quantum cryptography for protection against future quantum attacks. The project combines Near Protocol technologies (Chain Signatures, Shade Agents, 1Click API) with 1inch (Fusion+) to create a fully automated and AI-powered bridge.

## Quick Start

### Prerequisites

- **Docker** (v28.3.1 or later)
- **Docker Compose** (included with Docker Desktop)
- **Git**
- **Rust** 1.86.0 (automatically managed via `rust-toolchain.toml`)

### Setup

```bash
# Clone the repository
git clone <kembridge-repo>
cd kembridge-mono

# ⚠️ IMPORTANT: Configure API keys first
cp .env.example .env
# Edit .env with your API keys (see setup section below)

# Then start the application
make dev
```

📋 **For detailed API configuration see "API Configuration" section below**

### Rust Version Management

This project uses **Rust 1.86.0** for NEAR blockchain compatibility. The version is automatically managed via `rust-toolchain.toml` files:

- **Global Rust**: You can use any version globally (e.g., 1.88.0)
- **Project Rust**: Automatically switches to 1.86.0 in project directories
- **Why 1.86.0**: Ensures compatibility with NEAR VM (1.87+ has WebAssembly ABI changes)

📋 **See [RUST_VERSION_POLICY.md](RUST_VERSION_POLICY.md) for details**

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

## API Configuration

Before running KEMBridge, you need to configure API keys for external services:

### 1. MetaMask/Infura (Ethereum RPC)

1. **Go to [MetaMask Developer Dashboard](https://developer.metamask.io/)**
2. **Create a new project** called "KEMBridge"
3. **Get your Project ID** from the project settings
4. **Enable Sepolia testnet** in network settings
5. **Configure API security:**
   - Check "Require API Key Secret for all requests"
   - Copy the generated API Secret
6. **Update your .env file:**
   ```env
   ETHEREUM_RPC_URL=https://sepolia.infura.io/v3/YOUR_PROJECT_ID
   INFURA_API_SECRET=YOUR_API_SECRET_FROM_DASHBOARD
   INFURA_API_KEY=YOUR_PROJECT_ID
   ```

### 2. 1inch API (For Fusion+ Integration)

1. **Go to [1inch Developer Portal](https://portal.1inch.dev/)**
2. **Create a new application**
3. **Copy the API key**
4. **Update your .env file:**
   ```env
   ONEINCH_API_KEY=YOUR_1INCH_API_KEY
   ```

### 3. WalletConnect (Optional - for frontend)

1. **Go to [WalletConnect Cloud](https://cloud.walletconnect.com/)**
2. **Create a new project**
3. **Copy the Project ID**
4. **Update your .env file:**
   ```env
   VITE_WALLET_CONNECT_PROJECT_ID=YOUR_WALLET_CONNECT_PROJECT_ID
   ```

### 4. External Cache Configuration (Optional)

For development performance optimization, you can configure external cache storage:

```bash
# Create cache configuration file (will be ignored by git)
cp .env.cache.example .env.cache

# Edit .env.cache with your external storage path
# Example for external SSD:
DEV_CACHE_BASE_PATH=/Volumes/external-ssd/kembridge-cache
```

**Benefits of external caches:**
- Save **3GB+** of space on main disk (node_modules, Rust artifacts)
- Faster development builds with persistent caches
- One-command startup with `make dev-cache-detached`

**Cache commands:**
```bash
# Start with external caches (auto-creates .env.cache if needed)
make dev-cache-detached

# Clean external cache contents
make clean-cache

# Use regular Docker volumes (default)
make dev
```

### 5. Test Your Configuration

After configuring API keys, test the connection:

```bash
# Start the backend for testing
make dev

# In another terminal, test Ethereum RPC
curl http://localhost:4000/health

# Check the logs for successful API connections
docker-compose logs backend | grep -i "ethereum\|infura"
```

### Environment Files

- **Root `.env`** - Used by Docker Compose for all services
- **Backend `.env`** - Used for local backend development without Docker
- Both files have corresponding `.env.example` templates

## 🔥 Hot Reload Development Mode

KEMBridge includes a **powerful hot reload development mode** for rapid microservices development with cargo-watch and Docker volume mounts.

### Quick Start with Hot Reload

```bash
# Start all 5 microservices with hot reload in background
make microservices-hot-dev

# Or start with logs visible
docker-compose -f docker-compose.microservices.dev.yml up --build
```

### 🚀 Hot Reload Performance

**Tested hot reload timings for each service:**

| Service | Hot Reload Time | Status |
|---------|----------------|---------|
| **1inch** | ~10 seconds | ✅ Optimized |
| **Auth** | ~10 seconds | ✅ Optimized |
| **Gateway** | ~10 seconds | ✅ Optimized |
| **Blockchain** | ~15 seconds | ✅ Good |
| **Crypto** | ~33 seconds | ✅ Acceptable |

**Average: 15.6 seconds** - excellent for Rust microservices!

### Hot Reload Features

✅ **cargo-watch** - Automatic Rust compilation on file changes  
✅ **Volume mounts** - Source code changes reflected instantly  
✅ **Incremental builds** - Fast compilation with cargo caches  
✅ **Isolated caches** - Shared cargo registry + isolated target dirs  
✅ **Circuit breaker** - Gateway with resilience patterns  

### Available Hot Reload Commands

```bash
# Start microservices hot reload mode
make microservices-hot-dev

# Start in background (recommended for development)
make microservices-hot-detached

# View hot reload logs
make microservices-logs

# Stop hot reload mode
make microservices-down

# Health check all microservices
make microservices-health

# Individual service endpoints for testing
curl http://localhost:4001/health  # 1inch Service
curl http://localhost:4002/health  # Blockchain Service  
curl http://localhost:4003/health  # Crypto Service
curl http://localhost:4004/health  # Auth Service
curl http://localhost:4000/health  # Gateway Service
```

### Hot Reload Architecture

**Services included:**
- **Gateway** (Port 4000) - API Gateway with circuit breaker
- **1inch Service** (Port 4001) - DEX integration 
- **Blockchain Service** (Port 4002) - Ethereum/NEAR adapters
- **Crypto Service** (Port 4003) - Post-quantum cryptography
- **Auth Service** (Port 4004) - Web3 authentication

**Infrastructure:**
- **PostgreSQL** - Shared database
- **Redis** - Shared cache  
- **Cargo cache** - Shared Rust dependencies for faster builds

### Testing Your Changes

1. **Make code changes** in any service:
   ```bash
   # Example: Edit crypto service
   vim services/kembridge-crypto-service/src/main.rs
   ```

2. **Watch the logs** for automatic rebuild:
   ```bash
   docker-compose -f docker-compose.microservices.dev.yml logs -f crypto-service
   ```

3. **Test immediately** when rebuild completes:
   ```bash
   curl http://localhost:4003/health
   ```

### Optimized Build System

**Parallel builds** available with isolated caches:
```bash
# Ultra-fast parallel build (experimental)
docker-compose -f docker-compose.microservices.ultra.yml up --build
```

**Cache strategy:**
- Shared cargo registry: `~/.cache/kembridge/shared-cargo-registry`
- Isolated target dirs: `~/.cache/kembridge/{service}-target`
- No cache conflicts between parallel builds

### Development Commands

```bash
# Remove all unused Docker objects
docker system prune -f

# Clean docker
docker system prune -a --volumes -f

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

# Start all services in background mode with build
docker-compose build --no-cache && docker-compose down && docker-compose up -d

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
