#!/bin/bash

# Script for setting up shared Cargo caches for all microservices
# Creates centralized caches in user folder

set -e

echo "🚀 Setting up shared Cargo caches for KEMBridge microservices..."

# Create directory structure for shared caches
CACHE_BASE="${HOME}/.cache/kembridge"
echo "📁 Creating cache directories in: $CACHE_BASE"

mkdir -p "$CACHE_BASE/shared-cargo-registry"
mkdir -p "$CACHE_BASE/shared-cargo-git" 
mkdir -p "$CACHE_BASE/shared-cargo-target"

# Create separate caches for different build modes
mkdir -p "$CACHE_BASE/docker-cargo-registry"
mkdir -p "$CACHE_BASE/docker-cargo-git"
mkdir -p "$CACHE_BASE/docker-cargo-target"

# Create symbolic links if needed
if [ ! -d "$HOME/.cargo/registry" ]; then
    echo "🔗 Creating link to global Cargo registry"
    mkdir -p "$HOME/.cargo"
    ln -sf "$CACHE_BASE/shared-cargo-registry" "$HOME/.cargo/registry"
fi

if [ ! -d "$HOME/.cargo/git" ]; then
    echo "🔗 Creating link to global Cargo git"
    ln -sf "$CACHE_BASE/shared-cargo-git" "$HOME/.cargo/git"
fi

# Set access permissions
chmod -R 755 "$CACHE_BASE"

# Display cache size information
echo ""
echo "📊 Current cache sizes:"
du -sh "$CACHE_BASE"/* 2>/dev/null || echo "Caches empty (first run)"

echo ""
echo "✅ Setup completed!"
echo ""
echo "🎯 Shared cache advantages:"
echo "   • All microservices use one dependency registry"
echo "   • No repeated downloading of identical crates"
echo "   • Build acceleration by 60-80% after first time"
echo "   • Disk space savings"
echo ""
echo "🔧 Usage:"
echo "   docker-compose -f docker-compose.microservices.ultra.yml up --build"
echo ""

# Export variables for current session
export CARGO_HOME="$HOME/.cargo"
export CARGO_TARGET_DIR="$CACHE_BASE/shared-cargo-target"

echo "📋 Configured environment variables:"
echo "   CARGO_HOME=$CARGO_HOME"
echo "   CARGO_TARGET_DIR=$CARGO_TARGET_DIR"