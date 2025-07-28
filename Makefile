.PHONY: dev prod build clean logs test health test-all test-backend test-frontend test-near test-http test-unit test-integration

dev:
	docker-compose up --build

dev-detached:
	docker-compose up -d --build

prod:
	docker-compose -f docker-compose.yml -f docker-compose.prod.yml up -d --build

build:
	docker-compose build

clean:
	docker-compose down -v --rmi all

logs:
	docker-compose logs -f

test:
	docker-compose -f docker-compose.test.yml up --abort-on-container-exit

migrate:
	docker-compose exec backend sqlx migrate run

shell-backend:
	docker-compose exec backend bash

shell-frontend:
	docker-compose exec frontend sh

health:
	@echo "Checking all services..."
	@curl -f http://localhost:4000/health || echo "Backend: FAIL"
	@curl -f http://localhost:4003/health || echo "AI Engine: FAIL"
	@curl -f http://localhost:4001 || echo "Frontend: FAIL"

# ===== TEST COMMANDS =====

test-all: test-unit test-integration test-near
	@echo ""
	@echo "🎉 All test suites completed!"
	@echo "Note: HTTP tests require running server (make dev)"

test-backend: test-unit test-integration
	@echo ""
	@echo "✅ All backend tests completed!"

test-unit:
	@echo "🧪 Running Backend Unit Tests..."
	@echo "================================"
	@cd backend && cargo test --lib
	@cd backend && cargo test --test test_rate_limiting

test-integration:
	@echo ""
	@echo "🔗 Running Backend Integration Tests..."
	@echo "======================================"
	@cd backend && cargo run --bin test_auth_system
	@cd backend && cargo run --bin test_auth_integration
	@cd backend && cargo run --bin test_api_integration
	@cd backend && cargo test --test test_fusion_plus_integration

test-http:
	@echo ""
	@echo "🌐 Running HTTP Integration Tests..."
	@echo "===================================="
	@echo "⚠️  Requires running server: make dev"
	@cd backend && cargo run --bin test_auth_http
	@cd backend && ./tests/test_rate_limiting.sh

test-near:
	@echo ""
	@echo "🔗 Running NEAR Contract Tests..."
	@echo "================================="
	@cd near-contracts && make test

test-frontend:
	@echo ""
	@echo "⚛️  Running Frontend Tests..."
	@echo "============================="
	@docker-compose exec frontend pnpm test || echo "❌ Frontend tests require running container"

test-contracts:
	@echo ""
	@echo "📜 Running Smart Contract Tests..."
	@echo "=================================="
	@cd contracts && npm test || echo "❌ Contract tests require npm install"

# Help command
help:
	@echo "KEMBridge Project Commands:"
	@echo ""
	@echo "📦 DEVELOPMENT:"
	@echo "  dev            - Start all services with hot reload"
	@echo "  dev-detached   - Start services in background"
	@echo "  build          - Build all Docker images"
	@echo "  clean          - Remove all containers and volumes"
	@echo "  logs           - View logs from all services"
	@echo "  health         - Check all service health"
	@echo ""
	@echo "🗄️  DATABASE:"
	@echo "  migrate        - Run database migrations"
	@echo ""
	@echo "🔧 DEBUGGING:"
	@echo "  shell-backend  - Access backend container shell"
	@echo "  shell-frontend - Access frontend container shell"
	@echo ""
	@echo "🧪 TESTING:"
	@echo "  test-all       - Run all test suites (unit, integration, near)"
	@echo "  test-backend   - Run all backend tests"
	@echo "  test-unit      - Run backend unit tests only"
	@echo "  test-integration - Run backend integration tests"
	@echo "  test-http      - Run HTTP integration tests (requires server)"
	@echo "  test-near      - Run NEAR contract tests"
	@echo "  test-frontend  - Run frontend tests"
	@echo "  test-contracts - Run Ethereum smart contract tests"
	@echo ""
	@echo "🚀 PRODUCTION:"
	@echo "  prod           - Start in production mode"
	@echo ""
	@echo "📚 OTHER:"
	@echo "  help           - Show this help message"