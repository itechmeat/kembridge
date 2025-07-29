#!/bin/bash
# Start KEMBridge with external cache configuration

set -e

echo "🔧 Initializing external cache system..."

# Check if .env.cache exists
if [ ! -f .env.cache ]; then
    echo "❌ .env.cache not found. Create it first with your cache paths."
    exit 1
fi

# Read variables from .env.cache (without export)
eval $(cat .env.cache | grep -v '^#' | xargs -I {} echo "export {}")

# Create cache directories
echo "📁 Creating cache directories..."
mkdir -p "$DEV_CACHE_BASE_PATH"
mkdir -p "$CARGO_CACHE_PATH"
mkdir -p "$RUST_TARGET_PATH"
mkdir -p "$NODE_MODULES_PATH"
mkdir -p "$PNPM_STORE_PATH"

echo "✅ Cache directories created at $DEV_CACHE_BASE_PATH"

echo "📊 Using cache paths:"
echo "  CARGO: $CARGO_CACHE_PATH"
echo "  RUST: $RUST_TARGET_PATH"
echo "  NODE: $NODE_MODULES_PATH"
echo "  PNPM: $PNPM_STORE_PATH"

echo ""
echo "🚀 Starting Docker Compose with external caches..."

# Create symlinks for frontend caches
if [ ! -d "$NODE_MODULES_PATH" ]; then
    mkdir -p "$NODE_MODULES_PATH"
fi
if [ ! -d "$PNPM_STORE_PATH" ]; then
    mkdir -p "$PNPM_STORE_PATH"
fi

# Copy existing caches to external location if they exist
if [ -d "./frontend/node_modules" ] && [ ! "$(ls -A $NODE_MODULES_PATH)" ]; then
    echo "📦 Copying existing node_modules to external cache..."
    cp -r ./frontend/node_modules/* "$NODE_MODULES_PATH/" 2>/dev/null || true
fi

# Note: pnpm-store will be populated by the frontend container during first build

# Start with exported environment variables
export CARGO_CACHE_PATH
export RUST_TARGET_PATH
export NODE_MODULES_PATH
export PNPM_STORE_PATH

echo "📝 Using environment variables for cache paths"

# Start docker-compose
docker-compose "$@"