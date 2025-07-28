#!/bin/bash

# KEMBridge NEAR Contract Build Script
# This script compiles the NEAR contract for deployment

set -e

echo "🔧 Building KEMBridge NEAR contract..."

# Check if cargo is installed
if ! command -v cargo &> /dev/null; then
    echo "❌ Cargo is not installed. Please install Rust first."
    exit 1
fi

# Check if wasm32-unknown-unknown target is installed
if ! rustup target list --installed | grep -q "wasm32-unknown-unknown"; then
    echo "📦 Installing wasm32-unknown-unknown target..."
    rustup target add wasm32-unknown-unknown
fi

# Build the contract with cargo near build for NEAR VM compatibility
echo "🏗️  Compiling contract with cargo near build..."
cargo near build non-reproducible-wasm --no-abi

# Copy to out directory for compatibility with existing scripts
mkdir -p out
cp target/near/kembridge_near_contract.wasm out/

# Check if the file was created
if [ -f "target/near/kembridge_near_contract.wasm" ]; then
    echo "✅ Contract built successfully!"
    echo "📄 Primary output: target/near/kembridge_near_contract.wasm"
    echo "📄 Backup output: out/kembridge_near_contract.wasm"
    echo "📊 File size: $(du -h target/near/kembridge_near_contract.wasm | cut -f1)"
else
    echo "❌ Build failed - WASM file not found"
    exit 1
fi

echo "🎉 Build complete!"