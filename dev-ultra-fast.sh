#!/bin/bash
# СВЕРХБЫСТРЫЙ режим разработки KEMBridge
# Запускает только PostgreSQL/Redis в Docker, backend/frontend нативно

set -e

echo "🚀 СВЕРХБЫСТРЫЙ режим разработки KEMBridge"
echo "=========================================="

# Остановить все Docker контейнеры
echo "🛑 Останавливаем Docker контейнеры..."
docker-compose down 2>/dev/null || true

# Запустить только БД сервисы
echo "🗄️ Запускаем только PostgreSQL и Redis..."
docker-compose up -d postgres redis

# Ждем готовности БД
echo "⏳ Ждем готовности базы данных..."
sleep 5

# Проверяем установку bacon (для instant feedback)
if ! command -v bacon &> /dev/null; then
    echo "📦 Устанавливаем bacon для instant feedback..."
    cargo install bacon
fi

# Проверяем установку cargo-watch
if ! command -v cargo-watch &> /dev/null; then
    echo "📦 Устанавливаем cargo-watch..."
    cargo install cargo-watch
fi

# Запускаем backend нативно с cargo-watch
echo "⚡ Запускаем backend нативно с hot reload..."
cd backend

# Устанавливаем переменные окружения
export DATABASE_URL="postgresql://postgres:dev_password@localhost:5432/kembridge_dev"
export REDIS_URL="redis://:dev_redis_password@localhost:6379"
export JWT_SECRET="hackathon-super-secret-key-change-in-production"
export AI_ENGINE_URL="http://localhost:4003"
export RUST_LOG="debug"
export RUST_BACKTRACE="1"

# Запускаем миграции
echo "🔧 Применяем миграции базы данных..."
sqlx migrate run || echo "⚠️ Миграции не удались, но продолжаем..."

echo ""
echo "🎯 КОМАНДЫ ДЛЯ СВЕРХБЫСТРОЙ РАЗРАБОТКИ:"
echo "======================================"
echo ""
echo "В отдельных терминалах запустите:"
echo ""
echo "1. 🦀 Backend (instant check):"
echo "   cd backend && bacon check"
echo ""
echo "2. 🦀 Backend (run server):"
echo "   cd backend && cargo run --bin kembridge-backend"
echo ""
echo "3. ⚛️ Frontend (hot reload):"
echo "   cd frontend && pnpm run dev"
echo ""
echo "4. 🤖 AI Engine:"
echo "   cd ai-engine && python main.py"
echo ""
echo "💡 ПРЕИМУЩЕСТВА:"
echo "- Компиляция: секунды вместо минут"
echo "- cargo check: мгновенная проверка ошибок"
echo "- bacon: live feedback при изменениях"
echo "- Нативная скорость без Docker overhead"
echo ""
echo "📊 СТАТУС СЕРВИСОВ:"
docker-compose ps

echo ""
echo "✅ Готово! Теперь разработка будет молниеносной ⚡"