#!/bin/bash
# ULTRA-FAST KEMBridge development mode
# Runs only PostgreSQL/Redis in Docker, frontend natively

set -e

echo "🚀 ULTRA-FAST KEMBridge development mode"
echo "=========================================="

# Stop all Docker containers
echo "🛑 Stopping Docker containers..."
docker-compose down 2>/dev/null || true

# Start only DB services
echo "🗄️ Starting only PostgreSQL and Redis..."
docker-compose up -d postgres redis

# Wait for DB readiness
echo "⏳ Waiting for database readiness..."
sleep 5

# Check bacon installation (for instant feedback)
if ! command -v bacon &> /dev/null; then
    echo "📦 Installing bacon for instant feedback..."
    cargo install bacon
fi

# Check cargo-watch installation
if ! command -v cargo-watch &> /dev/null; then
    echo "📦 Installing cargo-watch..."
    cargo install cargo-watch
fi

# Backend removed - using microservices architecture

echo ""
echo "🎯 ULTRA-FAST DEVELOPMENT COMMANDS:"
echo "======================================"
echo ""
echo "Run in separate terminals:"
echo ""
echo "1. 🦀 Microservices (use make dev):"
echo "   make dev"
echo ""
echo "3. ⚛️ Frontend (hot reload):"
echo "   cd frontend && pnpm run dev"
echo ""
echo "4. 🤖 AI Engine:"
echo "   cd ai-engine && python main.py"
echo ""
echo "💡 ADVANTAGES:"
echo "- Compilation: seconds instead of minutes"
echo "- cargo check: instant error checking"
echo "- bacon: live feedback on changes"
echo "- Native speed without Docker overhead"
echo ""
echo "📊 SERVICES STATUS:"
docker-compose ps

echo ""
echo "✅ Done! Now development will be lightning fast ⚡"