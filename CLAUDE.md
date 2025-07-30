# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

KEMBridge is a quantum-secure cross-chain bridge built for the NEAR hackathon. It enables secure asset transfers between Ethereum and NEAR Protocol using post-quantum cryptography. The project is containerized with Docker and uses a simplified microservices architecture.

## Core Architecture

- **Backend**: Rust workspace with Axum web framework, organized into domain-specific crates:

  - `kembridge-auth`: Web3 wallet authentication and JWT
  - `kembridge-crypto`: Post-quantum cryptography (ML-KEM, Dilithium)
  - `kembridge-bridge`: Cross-chain bridge orchestration
  - `kembridge-database`: PostgreSQL models and connections
  - `kembridge-blockchain`: Ethereum and NEAR protocol adapters

- **Frontend**: React 19 with TypeScript, Vite dev server, and SCSS styling
- **AI Engine**: Python FastAPI service for risk analysis using scikit-learn
- **Database**: PostgreSQL 18 Beta 1 with OAuth 2.0 support
- **Cache**: Redis 8.0.3 for sessions and temporary data
- **Monitoring**: Prometheus metrics with Grafana dashboards

## Development Commands

### Primary Commands (Use Makefile)

```bash
# Start all services with hot reload
make dev

# Start in background mode
make dev-detached

# View logs from all services
make logs

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

### Docker Compose Commands

```bash
# Build and start all services
docker-compose up --build

# Start specific service
docker-compose up -d postgres

# View logs for specific service
docker-compose logs -f backend

# Restart specific service
docker-compose restart ai-engine
```

### Frontend Commands (within container)

```bash
# Development server
pnpm run dev

# Build for production
pnpm run build

# Lint code
pnpm run lint
```

### Backend Commands (within container)

```bash
# Run specific binary
cargo run --bin kembridge-backend

# Run integration tests
cargo run --bin test_api_integration
cargo run --bin test_auth_system
cargo run --bin test_auth_http

# Check code
cargo check

# Database migrations
sqlx migrate run
```

## Service Endpoints

- Frontend (Docker): http://localhost:4010
- Backend API: http://localhost:4000
- AI Engine: http://localhost:4005
- Grafana: http://localhost:4002 (admin:admin)
- Prometheus: http://localhost:4004

## Frontend Testing Guidelines

**IMPORTANT**: Frontend port allocation strategy:

- **Docker Frontend URL**: http://localhost:4010 (both dev and prod Docker containers)
- **Local Frontend URL**: http://localhost:4100 (reserved for local development without Docker)
- **Testing**: Use http://localhost:4010 for Docker-based testing, http://localhost:4100 for local testing

**Port Allocation:**
- **4010**: Frontend in Docker (development and production containers)
- **4100**: Reserved for local frontend development without Docker (pnpm run dev)

This ensures clean separation between Docker and local development environments.

**CRITICAL**: Claude Code MUST NEVER start the frontend development server automatically:
- **FORBIDDEN**: Running `pnpm run dev`, `npm start`, or any frontend development commands
- **REASON**: User manages frontend development process manually
- **ALLOWED**: Only reading frontend files and making edits when requested

## Key Technologies

- **Rust 1.88+**: Latest stable with let chains support
- **React 19**: Latest with concurrent features
- **PostgreSQL 18 Beta 1**: OAuth 2.0 integration ready
- **Post-quantum crypto**: ML-KEM-1024, Dilithium-5, SPHINCS+
- **Web3 integration**: MetaMask, NEAR Wallet, WalletConnect
- **NEAR Protocol**: Chain Signatures, 1Click API integration
- **1inch Fusion+**: Atomic swap integration

## Critical Version Requirements

**MANDATORY**: These exact versions MUST be used throughout the project:

- **Rust**: Version 1.88.0 (no exceptions - required for latest language features)
- **Axum**: Version 0.8.4 (no exceptions - required for modern async patterns)
- **Edition**: 2024 (using latest Rust 1.88.0 features like let chains, naked functions)

**DO NOT downgrade these versions under any circumstances.** If dependency conflicts arise:

1. Research proper solutions online first
2. Use version overrides in Cargo.toml if needed
3. Only ask for permission to downgrade if no other solution exists

**CRITICAL**: NEVER change edition = "2024" to any other version. If compilation errors occur due to edition 2024, find alternative solutions or use nightly Rust toolchain.

## Architecture Patterns

### Rust Backend Structure

- Workspace-based multi-crate organization
- Domain-driven crate separation
- Shared dependencies in workspace.dependencies
- Axum for HTTP handling with tower middleware
- SQLx for type-safe database operations
- JWT-based authentication for Web3 wallets

### Frontend Structure

- Component-based architecture with feature modules
- SCSS with modular structure (abstracts, base, components, layouts)
- React Query for server state management
- Wallet integration through RainbowKit and NEAR Wallet Selector

### Database Schema

- PostgreSQL with JSONB for flexible metadata storage
- Quantum key storage with encryption
- Audit logging for all operations
- Risk analysis data for AI/ML features

## Environment Variables

The project uses environment variables defined in docker-compose.yml:

- Database connections (PostgreSQL, Redis)
- Blockchain RPC URLs (Ethereum, NEAR)
- JWT secrets and CORS origins
- Development-specific settings

## Testing Strategy

- Backend: Cargo test with workspace support
- Frontend: Component testing with development tools
- Integration: Docker-based test environment
- Health checks: Automated endpoint verification

### E2E Testing Guidelines

**CRITICAL**: E2E тесты должны запускаться ТОЛЬКО в Chromium браузере:

- **НИКОГДА не добавлять** кроссбраузерные тесты (Firefox, Safari, WebKit)
- **НИКОГДА не добавлять** мобильные браузеры в конфигурацию
- **ВСЕГДА использовать** только `chromium` проект в `playwright.config.js`
- **ПРИЧИНА**: Экономия времени разработки и фокус на основном браузере

```javascript
// ПРАВИЛЬНО - только Chromium
projects: [
  {
    name: 'chromium',
    use: { ...devices['Desktop Chrome'] }
  }
],

// НЕПРАВИЛЬНО - НЕ добавлять другие браузеры
// firefox, webkit, mobile-chrome, mobile-safari и т.д.
```

## Deployment Notes

This is a hackathon version with simplified architecture. Production deployment will migrate to full microservices with event bus (NATS/Kafka) and domain separation.

## Security Considerations

- Post-quantum cryptography implementation
- Web3 wallet-only authentication
- AI-powered risk analysis
- Comprehensive audit logging
- Real-time transaction monitoring

## Critical Development Rules

### ABSOLUTE PROHIBITIONS - NO EXCEPTIONS

**1. NEVER USE MOCK DATA AS FALLBACK IN PRODUCTION PATHS**
- FORBIDDEN: Any fallback to mock/fake data when real APIs are unavailable
- REASON: This destroys user trust and can cause financial losses
- ACCEPTABLE: Mock data ONLY during development with clear TODO comments for removal
- REQUIRED: Proper error handling instead of fake data

**2. NO FAKE RESPONSES TO USERS**
- If external service is unavailable, return proper error messages
- Users must know when service is degraded, not receive fake data
- Better to fail gracefully than provide misleading information

**3. MOCK DATA USAGE RULES**
- ONLY allowed during initial development for compilation
- MUST have TODO comments explaining removal plan
- MUST be removed before any user-facing deployment
- NEVER as a fallback mechanism for production

### Constants Management

**MANDATORY**: Use global constants in `/src/constants.rs` for any values that might be variable:
- API endpoints and URLs
- Timeout values and retry counts
- Default configurations
- Error messages and status codes
- Numeric thresholds and limits
- String patterns and prefixes

**Examples requiring constants:**
```rust
// BAD - hardcoded values
let timeout = 30;
let api_url = "https://api.example.com";
let max_retries = 3;

// GOOD - using constants
let timeout = HTTP_CLIENT_TIMEOUT;
let api_url = EXTERNAL_API_BASE_URL;
let max_retries = DEFAULT_MAX_RETRIES;
```

## Language Guidelines

**IMPORTANT**: All project files (code, configuration, documentation except MD files) MUST contain ONLY English language content, including:

- Code comments
- Variable and function names
- Configuration files
- Error messages
- Log outputs
- README files and technical documentation

**Exceptions**:

- Markdown (.md) files: Can be written in the language requested by the user
- Chat communication: Should be conducted in the user's preferred language

This ensures code maintainability and international collaboration while respecting user communication preferences.

## Git Workflow Guidelines

**IMPORTANT**: Claude Code MUST NOT create git commits automatically.

- **FORBIDDEN**: Using `git commit`, `git add .`, or any other git commands that modify repository state
- **ALLOWED**: Reading git status, logs, and other read-only git operations for informational purposes
- **REASON**: The user manages git workflow manually to maintain full control over version history

When work is completed:
1. Report what files were changed and summarize the work done
2. Allow the user to review changes and create commits manually
3. Never attempt to automatically stage or commit changes

## Maintenance & Cleanup

### Regular Cleanup Commands

**IMPORTANT**: The project can accumulate large build artifacts and caches. Perform regular cleanup to free disk space:

```bash
# Clean local project build artifacts (run from project root)
cargo clean

# Clean Cargo global caches (WARNING: will re-download dependencies)
cargo install cargo-cache
cargo cache --autoclean

# Alternative manual cleanup of Cargo caches
rm -rf ~/.cargo/registry/cache
rm -rf ~/.cargo/git/db

# Clean Docker build artifacts if using Docker development
docker system prune -f
docker volume prune -f

# Clean frontend node_modules if needed
rm -rf frontend/node_modules
cd frontend && pnpm install
```

**Typical Space Usage:**
- Local project build artifacts: 1-3 GB (`cargo clean` to remove)
- Cargo registry cache: 1-5 GB (shared across all Rust projects)
- Docker cache: 1-10 GB (`docker system prune` to clean)
- Frontend node_modules: 200-500 MB (`rm -rf node_modules` to clean)

**Recommendation:** Run `cargo clean` after major dependency changes or when disk space is low.
