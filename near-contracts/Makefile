# KEMBridge NEAR Contract Makefile

.PHONY: build deploy test clean setup docs

# Build the contract
build:
	@echo "🔧 Building KEMBridge contract..."
	@scripts/build.sh

# Deploy the contract
deploy:
	@echo "🚀 Deploying KEMBridge contract..."
	@scripts/deploy-kembridge.sh

# Run comprehensive contract tests
test:
	@echo "🧪 Running KEMBridge contract tests..."
	@scripts/test-contract.sh

# Run unit tests only
unit-test:
	@echo "🔬 Running unit tests..."
	@cargo test

# Clean build artifacts
clean:
	@echo "🧹 Cleaning build artifacts..."
	@cargo clean
	@rm -rf out/

# Setup development environment
setup:
	@echo "⚙️  Setting up development environment..."
	@rustup target add wasm32-unknown-unknown
	@echo "✅ Setup complete!"

# Show documentation
docs:
	@echo "📚 KEMBridge NEAR Contract Documentation:"
	@echo ""
	@cat docs/GUIDE.md

# Development workflow: build -> deploy -> test
dev: build deploy test

# Quick check: unit tests + contract tests
check: unit-test test

# Help
help:
	@echo "KEMBridge NEAR Contract Commands:"
	@echo "  build      - Build the contract"
	@echo "  deploy     - Deploy to testnet"
	@echo "  test       - Run integration tests"
	@echo "  unit-test  - Run unit tests only"
	@echo "  dev        - Full development cycle"
	@echo "  check      - Run all tests"
	@echo "  clean      - Clean build artifacts"
	@echo "  setup      - Setup development environment"
	@echo "  docs       - Show documentation" 