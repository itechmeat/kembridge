.PHONY: dev dev-detached prod down clean logs health health-quick build migrate test test-e2e test-install

# 🔥 Development with HOT RELOAD (микросервисы + cargo-watch)
dev:
	@echo "🔥 Starting KEMBridge microservices with HOT RELOAD..."
	@echo "Services: Gateway + 1inch + Blockchain + Crypto + Auth + Infrastructure"
	docker-compose up --build

# Development in background
dev-detached:
	@echo "🔥 Starting microservices with HOT RELOAD (background)..."
	docker-compose up -d --build

# 🚀 Production mode
prod:
	@echo "🚀 Starting KEMBridge in PRODUCTION mode..."
	docker-compose -f docker-compose.prod.yml up -d --build

# Stop all services
down:
	@echo "🛑 Stopping all services..."
	docker-compose down

# Clean everything (containers, volumes, images)
clean:
	@echo "🧹 Cleaning all Docker resources..."
	docker-compose down -v --rmi all
	docker system prune -f

# View logs from all services
logs:
	@echo "📋 Viewing logs from all services..."
	docker-compose logs -f

# Build all images
build:
	@echo "🔨 Building all Docker images..."
	docker-compose build

# Database migrations
migrate:
	@echo "🗄️ Running database migrations..."
	docker-compose exec gateway sqlx migrate run

# Health check for all microservices
health:
	@echo "🔍 Checking microservices health..."
	@echo ""
	@echo "🟦 KEMBridge Microservices:"
	@curl -s -f --max-time 3 http://localhost:4000/health > /dev/null && echo "✅ Gateway (4000): HEALTHY" || echo "❌ Gateway: DOWN"
	@curl -s -f --max-time 3 http://localhost:4001/health > /dev/null && echo "✅ 1inch Service (4001): HEALTHY" || echo "❌ 1inch: DOWN"
	@curl -s -f --max-time 3 http://localhost:4002/health > /dev/null && echo "✅ Blockchain Service (4002): HEALTHY" || echo "❌ Blockchain: DOWN"
	@curl -s -f --max-time 3 http://localhost:4003/health > /dev/null && echo "✅ Crypto Service (4003): HEALTHY" || echo "❌ Crypto: DOWN"
	@curl -s -f --max-time 3 http://localhost:4004/health > /dev/null && echo "✅ Auth Service (4004): HEALTHY" || echo "❌ Auth: DOWN"
	@echo ""
	@echo "🟦 Supporting Services:"
	@curl -s -f --max-time 3 http://localhost:4010 > /dev/null && echo "✅ Frontend (4010): HEALTHY" || echo "❌ Frontend: DOWN"
	@curl -s -f --max-time 3 http://localhost:4005/health > /dev/null && echo "✅ AI Engine (4005): HEALTHY" || echo "❌ AI Engine: DOWN"
	@echo ""
	@echo "🟦 Infrastructure:"
	@timeout 3 bash -c "</dev/tcp/localhost/5432" 2>/dev/null && echo "✅ PostgreSQL (5432): HEALTHY" || echo "❌ PostgreSQL: DOWN"
	@docker exec kembridge_redis_micro redis-cli -a dev_redis_password ping > /dev/null 2>&1 && echo "✅ Redis: HEALTHY" || echo "❌ Redis: DOWN"

# Quick health check for critical services only
health-quick:
	@echo "⚡ Quick health check..."
	@curl -s -f --max-time 2 http://localhost:4000/health > /dev/null && echo "✅ Gateway: OK" || echo "❌ Gateway: DOWN"
	@curl -s -f --max-time 2 http://localhost:4010 > /dev/null && echo "✅ Frontend: OK" || echo "❌ Frontend: DOWN"

# E2E Testing with Playwright
test-install:
	@echo "📦 Installing E2E test dependencies..."
	cd e2e-tests && npm install && npm run install-browsers

test-e2e:
	@echo "🧪 Running E2E tests..."
	@echo "Prerequisites: Services should be running (make dev)"
	cd e2e-tests && npm test

test-e2e-ui:
	@echo "🧪 Running E2E tests with UI..."
	cd e2e-tests && npm run test:ui

test:
	@echo "🧪 Running all tests..."
	@echo "1️⃣ Health check..."
	@make health-quick
	@echo ""
	@echo "2️⃣ E2E tests..."
	@make test-e2e

# Development utilities
shell-gateway:
	docker-compose exec gateway-service sh

shell-postgres:
	docker-compose exec postgres psql -U postgres -d kembridge_dev

shell-redis:
	docker-compose exec redis redis-cli -a dev_redis_password

# Show help
help:
	@echo "KEMBridge Microservices Commands:"
	@echo ""
	@echo "📦 MAIN COMMANDS:"
	@echo "  dev            - 🔥 Start with HOT RELOAD (recommended for development)"
	@echo "  dev-detached   - 🔥 Start with HOT RELOAD in background"
	@echo "  prod           - 🚀 Start in production mode"
	@echo "  down           - 🛑 Stop all services"
	@echo "  clean          - 🧹 Clean all Docker resources"
	@echo ""
	@echo "🔧 UTILITIES:"
	@echo "  build          - 🔨 Build all Docker images"
	@echo "  logs           - 📋 View logs from all services"
	@echo "  health         - 🔍 Check health of all services"
	@echo "  health-quick   - ⚡ Quick check of critical services"
	@echo "  migrate        - 🗄️ Run database migrations"
	@echo ""
	@echo "🧪 TESTING:"
	@echo "  test-install   - 📦 Install E2E test dependencies"
	@echo "  test-e2e       - 🧪 Run E2E tests (headless)"
	@echo "  test-e2e-ui    - 🧪 Run E2E tests with UI"
	@echo "  test           - 🧪 Run all tests (health + E2E)"
	@echo ""
	@echo "🌐 ENDPOINTS:"
	@echo "  Gateway:       http://localhost:4000"
	@echo "  1inch Service: http://localhost:4001"
	@echo "  Blockchain:    http://localhost:4002"
	@echo "  Crypto:        http://localhost:4003"
	@echo "  Auth:          http://localhost:4004"
	@echo "  Frontend:      http://localhost:4010  (Docker)"
	@echo "  AI Engine:     http://localhost:4005"
	@echo ""
	@echo "📝 NOTE: Port 4100 reserved for local frontend development without Docker"
	@echo ""
	@echo "💡 Start development: make dev"