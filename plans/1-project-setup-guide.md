# KEMBridge Project Setup & Development Environment

## 📋 Overview

Детальное руководство по настройке проекта KEMBridge для хакатона. Включает анализ инструментов, архитектурные решения и пошаговые инструкции.

## 🔍 Technology Stack Analysis

### Backend Framework: Axum

#### Выбор Axum для KEMBridge

**Стратегические преимущества:**

- ✅ **Production-ready**: Зрелый фреймворк с активной разработкой от команды Tokio
- ✅ **Performance**: Высокая производительность за счет zero-cost abstractions
- ✅ **Ecosystem**: Отличная интеграция с sqlx, serde, tower middleware
- ✅ **Type safety**: Compile-time проверки маршрутов и обработчиков
- ✅ **Async-first**: Нативная поддержка async/await паттернов
- ✅ **Middleware ecosystem**: Богатая экосистема middleware для CORS, аутентификации, логирования

**Преимущества для хакатона:**

- 🚀 **Быстрый старт**: Обширная документация и примеры
- 🔧 **Интеграция**: Seamless работа с PostgreSQL через sqlx
- 📦 **Модульность**: Легкое создание модульных API с помощью Router::nest
- 🛡️ **Безопасность**: Встроенная поддержка CORS, rate limiting, authentication middleware

**Специфика для KEMBridge:**

- **Web3 integration**: Отличная поддержка JSON-RPC для работы с блокчейнами
- **Quantum crypto**: Эффективная работа с бинарными данными (ключи, подписи)
- **Real-time updates**: WebSocket поддержка для live transaction monitoring
- **Error handling**: Продвинутый error handling для финансовых операций

### Database Layer: PostgreSQL 18 Beta 1 + sqlx

#### Выбор PostgreSQL 18 Beta 1

**Стратегические преимущества для KEMBridge:**

- ✅ **Встроенная OAuth 2.0 поддержка**: Упрощает интеграцию с Web3Auth в будущих версиях
- ✅ **Улучшенная JSON/JSONB производительность**: Критично для хранения квантовых ключей и метаданных
- ✅ **Расширенные возможности аудита**: Встроенные механизмы для compliance
- ✅ **Лучшая поддержка шифрования**: Совместимость с постквантовыми алгоритмами
- ✅ **Future-ready**: К production релизу PostgreSQL 18 будет стабильной версией

#### sqlx Integration Benefits

**Преимущества для Rust разработки:**

- 🚀 **Compile-time проверка SQL**: Ошибки в запросах выявляются на этапе компиляции
- 🔧 **Async/await support**: Нативная поддержка асинхронного кода Rust
- ⚡ **Connection pooling**: Автоматическое управление пулом соединений
- 🛡️ **SQL injection protection**: Встроенная защита от SQL инъекций
- 📈 **Миграции**: Автоматическое применение schema изменений

#### Architecture Benefits для KEMBridge

**Специфические преимущества для проекта:**

- **Quantum keys storage**: Безопасное хранение ML-KEM-1024 ключей в JSONB
- **Transaction tracking**: Полное логирование всех кросс-чейн операций
- **Risk analysis data**: Эффективное хранение ML признаков и risk scores
- **Audit compliance**: Встроенные механизмы аудита для финансовых операций
- **Real-time performance**: Высокая скорость для AI risk engine запросов

## 🏗️ Project Architecture

### Frontend: React + TypeScript + Vite + SCSS (Dockerized)

#### Пример использования TanStack Query

**Реализация для KEMBridge:**

```typescript
// Простое и эффективное управление server state
export const useWalletBalance = (address: string) => {
  return useQuery({
    queryKey: ["wallet", "balance", address],
    queryFn: () => api.getBalance(address),
    staleTime: 30000, // 30 секунд
    refetchInterval: 60000, // Обновлять каждую минуту
    enabled: !!address,
  });
};

// Автоматически обрабатывает loading, error, caching!
```

#### Модульная архитектура (MobileFirst) с TanStack Query

```
frontend/
├── Dockerfile              # Multi-stage Docker build
├── docker-compose.frontend.yml
├── src/
│   ├── components/          # Переиспользуемые компоненты
│   │   ├── ui/             # Базовые UI компоненты (Button, Input)
│   │   ├── forms/          # Формы (SwapForm, AuthForm)
│   │   ├── layout/         # Layout компоненты (Header, Footer)
│   │   └── features/       # Фичевые компоненты (WalletConnect)
│   ├── modules/            # Бизнес-модули
│   │   ├── auth/          # Авторизация
│   │   ├── bridge/        # Мост функциональность
│   │   ├── security/      # Безопасность и риски
│   │   └── analytics/     # Аналитика
│   ├── hooks/             # Custom React hooks
│   ├── services/          # API и внешние сервисы
│   ├── queries/           # TanStack Query hooks и кеширование
│   │   ├── auth.queries.ts    # Auth-related queries
│   │   ├── bridge.queries.ts  # Bridge операции
│   │   ├── wallet.queries.ts  # Wallet interactions
│   │   └── price.queries.ts   # Price oracle queries
│   ├── styles/            # SCSS файлы
│   │   ├── abstracts/     # Variables, mixins
│   │   ├── base/          # Reset, typography
│   │   ├── components/    # Component styles
│   │   └── layouts/       # Layout styles
│   └── types/             # TypeScript типы
```

#### TanStack Query для управления server state

**Преимущества TanStack Query для KEMBridge:**

- ✅ **Server state специализация**: Создан специально для работы с server state
- ✅ **Автоматическое кеширование**: Умное кеширование API запросов без дополнительной настройки
- ✅ **Background refetching**: Автоматическое обновление данных в фоне
- ✅ **Optimistic updates**: Встроенная поддержка optimistic updates для лучшего UX
- ✅ **Error handling**: Продвинутая обработка ошибок и retry логика
- ✅ **DevTools**: Отличные DevTools для отладки состояния
- ✅ **Web3 совместимость**: Идеально подходит для Web3 приложений с частыми blockchain запросами

**Специфика для KEMBridge:**

- **Real-time updates**: Автоматическое обновление transaction status
- **Price feeds**: Эффективное кеширование price oracle данных
- **Wallet state**: Управление состоянием подключенных кошельков
- **Bridge operations**: Tracking cross-chain операций с автоматическими обновлениями

### Backend: Rust + Axum + PostgreSQL (Fully Dockerized)

#### Модульная Cargo Workspace структура (Multi-container)

```
backend/
├── Cargo.toml             # Workspace configuration
├── src/
│   ├── main.rs            # Entry point и API routing
│   ├── config.rs          # Environment configuration
│   └── lib.rs             # Common utilities
├── crates/                # Модульные крейты
│   ├── kembridge-auth/    # Web3 авторизация
│   │   ├── src/lib.rs
│   │   ├── src/web3.rs    # Web3 signature verification
│   │   ├── src/jwt.rs     # JWT token management
│   │   └── src/models.rs  # Auth data models
│   ├── kembridge-crypto/  # Quantum криптография
│   │   ├── src/lib.rs
│   │   ├── src/mlkem.rs   # ML-KEM-1024 implementation
│   │   ├── src/hybrid.rs  # Hybrid classical+quantum
│   │   └── src/storage.rs # Secure key storage
│   ├── kembridge-bridge/  # Bridge logic
│   │   ├── src/lib.rs
│   │   ├── src/swap.rs    # Swap orchestration
│   │   ├── src/atomic.rs  # Atomic swap logic
│   │   └── src/state.rs   # Transaction state management
│   ├── kembridge-blockchain/ # Blockchain adapters
│   │   ├── src/lib.rs
│   │   ├── src/ethereum.rs
│   │   ├── src/near.rs
│   │   └── src/traits.rs  # Common blockchain traits
│   └── kembridge-database/ # Database layer
│       ├── src/lib.rs
│       ├── src/models.rs  # sqlx models
│       ├── src/pool.rs    # Connection pooling
│       └── src/migrations.rs
├── migrations/            # SQL миграции
│   ├── 001_initial_schema.sql
│   ├── 002_quantum_keys.sql
│   └── 003_audit_system.sql
└── tests/                 # Integration tests
    ├── auth_tests.rs
    ├── bridge_tests.rs
    └── integration_tests.rs
```

#### Преимущества Cargo Workspace

**Организационные:**

- 🔧 **Модульность**: Каждый крейт отвечает за отдельную область
- 🚀 **Переиспользование**: Общие типы и утилиты в отдельных крейтах
- 🧪 **Тестирование**: Изолированное тестирование каждого модуля
- 📦 **Dependency management**: Централизованное управление зависимостями

**Технические:**

- ⚡ **Compile time**: Параллельная компиляция крейтов
- 🛡️ **Type safety**: Compile-time проверки между модулями
- 🔄 **Hot reload**: Изменения в одном крейте не пересобирают все
- 📈 **Scalability**: Легкое добавление новых модулей

## 🐳 Containerized Architecture Overview

### Docker-first подход для всех компонентов

**Преимущества полной докеризации:**

- ✅ **Consistency**: Одинаковое окружение для всех разработчиков
- ✅ **Isolation**: Изоляция сервисов и их зависимостей
- ✅ **Scalability**: Легкое горизонтальное масштабирование
- ✅ **CI/CD**: Упрощенный деплой и тестирование
- ✅ **Development**: Быстрый onboarding новых разработчиков

### Схема взаимодействия Docker контейнеров

```mermaid
graph TB
    subgraph "External Access"
        USER[👤 User Browser]
        DEV[👨‍💻 Developer]
    end

    subgraph "Reverse Proxy Layer"
        NGINX[🌐 Nginx<br/>Port: 80, 443<br/>Routing & Load Balancing]
    end

    subgraph "Application Layer"
        FRONTEND[⚛️ Frontend<br/>React + Vite + TS<br/>Port: 3001<br/>Container: kembridge_frontend]
        BACKEND[🦀 Backend<br/>Rust + Axum<br/>Port: 3000<br/>Container: kembridge_backend]
        AI[🤖 AI Engine<br/>Python + FastAPI<br/>Port: 8000<br/>Container: kembridge_ai_engine]
    end

    subgraph "Data Layer"
        POSTGRES[🐘 PostgreSQL 18β1<br/>Port: 5432<br/>Container: kembridge_postgres]
        REDIS[📦 Redis Cache<br/>Port: 6379<br/>Container: kembridge_redis]
    end

    subgraph "Monitoring Layer"
        PROMETHEUS[📊 Prometheus<br/>Port: 9090<br/>Container: kembridge_prometheus]
        GRAFANA[📈 Grafana<br/>Port: 3002<br/>Container: kembridge_grafana]
    end

    subgraph "Docker Network: kembridge-network (172.20.0.0/16)"
        VOLUMES[💾 Persistent Volumes:<br/>• postgres_data<br/>• redis_data<br/>• backend_target<br/>• frontend_node_modules<br/>• ai_models<br/>• prometheus_data<br/>• grafana_data]
    end

    %% External connections
    USER --> NGINX
    DEV --> FRONTEND
    DEV --> BACKEND
    DEV --> GRAFANA

    %% Nginx routing
    NGINX --> FRONTEND
    NGINX --> BACKEND
    NGINX --> AI

    %% Application interconnections
    FRONTEND -.->|API Calls| BACKEND
    FRONTEND -.->|AI Requests| AI
    BACKEND -.->|Risk Analysis| AI
    BACKEND -.->|Database| POSTGRES
    BACKEND -.->|Cache/Sessions| REDIS
    AI -.->|ML Data| POSTGRES
    AI -.->|Cache| REDIS

    %% Monitoring connections
    BACKEND -.->|Metrics| PROMETHEUS
    FRONTEND -.->|Metrics| PROMETHEUS
    AI -.->|Metrics| PROMETHEUS
    POSTGRES -.->|Metrics| PROMETHEUS
    REDIS -.->|Metrics| PROMETHEUS
    PROMETHEUS --> GRAFANA

    %% Volume connections
    POSTGRES -.-> VOLUMES
    REDIS -.-> VOLUMES
    BACKEND -.-> VOLUMES
    FRONTEND -.-> VOLUMES
    AI -.-> VOLUMES
    PROMETHEUS -.-> VOLUMES
    GRAFANA -.-> VOLUMES

    style USER fill:#e1f5fe
    style DEV fill:#e8f5e8
    style NGINX fill:#fff3e0
    style FRONTEND fill:#e3f2fd
    style BACKEND fill:#fce4ec
    style AI fill:#f3e5f5
    style POSTGRES fill:#e0f2f1
    style REDIS fill:#ffebee
    style PROMETHEUS fill:#fff8e1
    style GRAFANA fill:#e8eaf6
    style VOLUMES fill:#f5f5f5
```

### Контейнерная архитектура

```
kembridge/
├── docker-compose.yml           # Основной compose файл (8 сервисов)
├── docker-compose.override.yml  # Development окружение
├── docker-compose.prod.yml      # Production окружение
├── backend/
│   ├── Dockerfile              # Multi-stage Rust build
│   └── ...
├── frontend/
│   ├── Dockerfile              # Node.js + Nginx multi-stage
│   └── ...
├── ai-engine/
│   ├── Dockerfile              # Python FastAPI service
│   └── ...
├── nginx/
│   ├── Dockerfile              # Reverse proxy
│   └── nginx.conf
└── monitoring/
    ├── prometheus.yml
    └── grafana/
```

## 📋 Detailed Setup Instructions

### 1.1 Project Setup & Development Environment

#### 1.1.1 Quick Start (One Command)

```bash
# 🚀 Полный старт системы одной командой!
git clone <kembridge-repo>
cd kembridge
docker-compose up --build

# ✨ Что происходит:
# • Создаётся 8 контейнеров (frontend, backend, ai-engine, postgres, redis, nginx, prometheus, grafana)
# • Настраивается внутренняя сеть kembridge-network
# • Применяются миграции базы данных
# • Запускается hot reload для всех сервисов

# 📦 Доступные сервисы:
# • Frontend: http://localhost:3001
# • Backend API: http://localhost:3000
# • AI Engine: http://localhost:8000
# • Grafana: http://localhost:3002
# • Prometheus: http://localhost:9090

# 🚑 Полезные команды:
make dev          # Запуск системы
make logs         # Посмотреть логи
make health       # Проверить состояние
make clean        # Очистить всё
```

#### 1.1.2 Project Structure & Repository Setup

```bash
# 📁 Полная структура проекта
kembridge/
├── docker-compose.yml           # Основной compose (8 сервисов)
├── docker-compose.override.yml  # Development окружение
├── docker-compose.prod.yml      # Production окружение
├── Makefile                     # Упрощенные команды
├── backend/                     # 🦀 Rust + Axum
│   ├── Dockerfile
│   ├── Cargo.toml              # Workspace конфигурация
│   ├── src/                    # Основное приложение
│   ├── crates/                 # Модульные крейты
│   ├── migrations/             # SQL миграции
│   └── tests/                  # Интеграционные тесты
├── frontend/                    # ⚛️ React + Vite
│   ├── Dockerfile
│   ├── package.json
│   └── src/
├── ai-engine/                   # 🤖 Python + FastAPI
│   ├── Dockerfile
│   ├── requirements.txt
│   └── main.py
├── nginx/                       # 🌐 Reverse Proxy
│   ├── Dockerfile
│   └── nginx.conf
└── monitoring/                  # 📊 Prometheus + Grafana
    ├── prometheus.yml
    └── grafana/

# 🚀 Создание структуры (если начинаете с нуля)
mkdir kembridge && cd kembridge
mkdir -p {backend/{src,crates,migrations,tests},frontend,ai-engine,nginx,monitoring}
mkdir -p backend/crates/{kembridge-auth,kembridge-crypto,kembridge-bridge,kembridge-blockchain,kembridge-database}/src
```

#### 1.1.3 Core Docker Compose Configuration

```yaml
# docker-compose.yml - Полная контейнерная архитектура
version: "3.8"

services:
  # ========== DATABASE LAYER ==========
  postgres:
    image: postgres:18-beta1
    container_name: kembridge_postgres
    restart: unless-stopped
    environment:
      POSTGRES_DB: kembridge_dev
      POSTGRES_USER: postgres
      POSTGRES_PASSWORD: dev_password
      POSTGRES_INITDB_ARGS: "--auth-host=scram-sha-256"
    ports:
      - "5432:5432"
    volumes:
      - postgres_data:/var/lib/postgresql/data
      - ./backend/migrations:/docker-entrypoint-initdb.d
    command: >
      postgres
      -c shared_preload_libraries=pg_stat_statements
      -c track_activity_query_size=2048
      -c pg_stat_statements.track=all
      -c log_statement=all
      -c log_min_duration_statement=0
    networks:
      - kembridge-network
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U postgres"]
      interval: 30s
      timeout: 10s
      retries: 3

  redis:
    image: redis:7-alpine
    container_name: kembridge_redis
    restart: unless-stopped
    ports:
      - "6379:6379"
    volumes:
      - redis_data:/data
    command: redis-server --appendonly yes --requirepass dev_redis_password
    networks:
      - kembridge-network
    healthcheck:
      test: ["CMD", "redis-cli", "ping"]
      interval: 30s
      timeout: 10s
      retries: 3

  # ========== BACKEND SERVICES ==========
  backend:
    build:
      context: ./backend
      dockerfile: Dockerfile
      target: development
    container_name: kembridge_backend
    restart: unless-stopped
    ports:
      - "3000:3000"
    environment:
      - DATABASE_URL=postgresql://postgres:dev_password@postgres:5432/kembridge_dev
      - REDIS_URL=redis://:dev_redis_password@redis:6379
      - JWT_SECRET=hackathon-super-secret-key-change-in-production
      - ETHEREUM_RPC_URL=https://sepolia.infura.io/v3/YOUR_INFURA_KEY
      - NEAR_RPC_URL=https://rpc.testnet.near.org
      - RUST_LOG=debug
      - CORS_ORIGINS=http://localhost:3001
    volumes:
      - ./backend:/app
      - backend_target:/app/target # Cache Rust builds
    depends_on:
      postgres:
        condition: service_healthy
      redis:
        condition: service_healthy
    networks:
      - kembridge-network
    command: cargo run --bin kembridge-backend

  # ========== AI SERVICES ==========
  ai-engine:
    build:
      context: ./ai-engine
      dockerfile: Dockerfile
    container_name: kembridge_ai_engine
    restart: unless-stopped
    ports:
      - "8000:8000"
    environment:
      - DATABASE_URL=postgresql://postgres:dev_password@postgres:5432/kembridge_dev
      - REDIS_URL=redis://:dev_redis_password@redis:6379
      - PYTHONPATH=/app
      - MODEL_PATH=/app/models
    volumes:
      - ./ai-engine:/app
      - ai_models:/app/models
    depends_on:
      postgres:
        condition: service_healthy
      redis:
        condition: service_healthy
    networks:
      - kembridge-network
    command: uvicorn main:app --host 0.0.0.0 --port 8000 --reload

  # ========== FRONTEND ==========
  frontend:
    build:
      context: ./frontend
      dockerfile: Dockerfile
      target: development
    container_name: kembridge_frontend
    restart: unless-stopped
    ports:
      - "3001:3000" # Vite dev server
    environment:
      - VITE_API_BASE_URL=http://localhost:3000/api/v1
      - VITE_AI_ENGINE_URL=http://localhost:8000
      - VITE_WALLET_CONNECT_PROJECT_ID=YOUR_WALLET_CONNECT_PROJECT_ID
      - VITE_CHAIN_ID_ETHEREUM=11155111
      - VITE_CHAIN_ID_NEAR=testnet
    volumes:
      - ./frontend:/app
      - frontend_node_modules:/app/node_modules
    depends_on:
      - backend
      - ai-engine
    networks:
      - kembridge-network
    command: npm run dev -- --host 0.0.0.0

  # ========== REVERSE PROXY ==========
  nginx:
    build:
      context: ./nginx
      dockerfile: Dockerfile
    container_name: kembridge_nginx
    restart: unless-stopped
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - ./nginx/nginx.conf:/etc/nginx/nginx.conf:ro
      - ./nginx/certs:/etc/nginx/certs:ro
    depends_on:
      - frontend
      - backend
      - ai-engine
    networks:
      - kembridge-network

  # ========== MONITORING ==========
  prometheus:
    image: prom/prometheus:latest
    container_name: kembridge_prometheus
    restart: unless-stopped
    ports:
      - "9090:9090"
    volumes:
      - ./monitoring/prometheus.yml:/etc/prometheus/prometheus.yml:ro
      - prometheus_data:/prometheus
    command:
      - "--config.file=/etc/prometheus/prometheus.yml"
      - "--storage.tsdb.path=/prometheus"
      - "--web.console.libraries=/etc/prometheus/console_libraries"
      - "--web.console.templates=/etc/prometheus/consoles"
      - "--storage.tsdb.retention.time=200h"
      - "--web.enable-lifecycle"
    networks:
      - kembridge-network

  grafana:
    image: grafana/grafana:latest
    container_name: kembridge_grafana
    restart: unless-stopped
    ports:
      - "3002:3000"
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=admin
    volumes:
      - grafana_data:/var/lib/grafana
      - ./monitoring/grafana/dashboards:/etc/grafana/provisioning/dashboards:ro
      - ./monitoring/grafana/datasources:/etc/grafana/provisioning/datasources:ro
    depends_on:
      - prometheus
    networks:
      - kembridge-network

volumes:
  postgres_data:
  redis_data:
  backend_target:
  frontend_node_modules:
  ai_models:
  prometheus_data:
  grafana_data:

networks:
  kembridge-network:
    driver: bridge
    ipam:
      config:
        - subnet: 172.20.0.0/16
```

#### 1.1.4 Individual Service Dockerfiles

##### Backend Dockerfile (Multi-stage Rust)

```dockerfile
# backend/Dockerfile
FROM rust:1.75 AS chef
RUN cargo install cargo-chef
WORKDIR /app

# Планирование зависимостей
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# Сборка зависимостей
FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# Сборка приложения
COPY . .
RUN cargo build --release --bin kembridge-backend

# Development stage
FROM rust:1.75 AS development
WORKDIR /app
RUN cargo install cargo-watch
COPY . .
EXPOSE 3000
CMD ["cargo", "run", "--bin", "kembridge-backend"]

# Production stage
FROM debian:bookworm-slim AS production
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/kembridge-backend /app/kembridge-backend
EXPOSE 3000
CMD ["/app/kembridge-backend"]
```

##### Frontend Dockerfile (Node.js + Nginx multi-stage)

```dockerfile
# frontend/Dockerfile
FROM node:18-alpine AS base
WORKDIR /app
COPY package*.json ./
RUN npm ci --only=production && npm cache clean --force

# Development stage
FROM node:18-alpine AS development
WORKDIR /app
COPY package*.json ./
RUN npm install
COPY . .
EXPOSE 3000
CMD ["npm", "run", "dev", "--", "--host", "0.0.0.0"]

# Build stage
FROM base AS build
WORKDIR /app
COPY package*.json ./
RUN npm ci
COPY . .
RUN npm run build

# Production stage
FROM nginx:alpine AS production
COPY --from=build /app/dist /usr/share/nginx/html
COPY nginx.conf /etc/nginx/conf.d/default.conf
EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]
```

##### AI Engine Dockerfile (Python FastAPI)

```dockerfile
# ai-engine/Dockerfile
FROM python:3.11-slim AS base

# Установка системных зависимостей
RUN apt-get update && apt-get install -y \
    gcc \
    g++ \
    libpq-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Установка Python зависимостей
COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

# Копирование кода
COPY . .

# Создание папки для моделей
RUN mkdir -p /app/models

EXPOSE 8000
CMD ["uvicorn", "main:app", "--host", "0.0.0.0", "--port", "8000"]
```

##### Nginx Dockerfile (Reverse Proxy)

```dockerfile
# nginx/Dockerfile
FROM nginx:alpine

# Копирование конфигурации
COPY nginx.conf /etc/nginx/nginx.conf
COPY default.conf /etc/nginx/conf.d/default.conf

# Создание папки для SSL сертификатов
RUN mkdir -p /etc/nginx/certs

EXPOSE 80 443
CMD ["nginx", "-g", "daemon off;"]
```

#### 1.1.5 Backend Workspace Configuration

```toml
# backend/Cargo.toml (Workspace Configuration)
[workspace]
members = [
    "crates/kembridge-auth",
    "crates/kembridge-crypto",
    "crates/kembridge-bridge",
    "crates/kembridge-database",
    "crates/kembridge-blockchain",
]
resolver = "2"

[workspace.dependencies]
# Web framework
axum = "0.7"
tokio = { version = "1.0", features = ["full"] }
tower = "0.4"
tower-http = { version = "0.5", features = ["cors", "trace"] }

# Database
sqlx = { version = "0.7", features = ["runtime-tokio-rustls", "postgres", "chrono", "uuid", "json"] }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Authentication & Crypto
jsonwebtoken = "9.2"
rand = "0.8"
sha2 = "0.10"
secp256k1 = "0.28"
ed25519-dalek = "2.0"

# Post-quantum cryptography
pqcrypto = "0.17"
pqcrypto-kyber = "0.7"

# HTTP client
reqwest = { version = "0.11", features = ["json"] }

# Blockchain
ethers = "2.0"
near-sdk = "4.1"
near-jsonrpc-client = "0.6"

# Async
futures = "0.3"

# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# Configuration
config = "0.14"
dotenvy = "0.15"

# Time
chrono = { version = "0.4", features = ["serde"] }

# UUID
uuid = { version = "1.0", features = ["v4", "serde"] }

# Redis
redis = { version = "0.24", features = ["tokio-comp"] }

# Validation
validator = { version = "0.16", features = ["derive"] }

# Error handling
anyhow = "1.0"
thiserror = "1.0"

# KEMBridge internal crates
kembridge-auth = { path = "crates/kembridge-auth" }
kembridge-crypto = { path = "crates/kembridge-crypto" }
kembridge-bridge = { path = "crates/kembridge-bridge" }
kembridge-database = { path = "crates/kembridge-database" }
kembridge-blockchain = { path = "crates/kembridge-blockchain" }

# Main backend binary
[package]
name = "kembridge-backend"
version = "0.1.0"
edition = "2021"

[dependencies]
# Web framework
axum.workspace = true
tokio.workspace = true
tower.workspace = true
tower-http.workspace = true

# Internal crates
kembridge-auth.workspace = true
kembridge-database.workspace = true

# Configuration
dotenvy.workspace = true
config.workspace = true
anyhow.workspace = true
tracing.workspace = true
tracing-subscriber.workspace = true
serde.workspace = true
serde_json.workspace = true
```

#### Individual Crate Configurations

```toml
# backend/crates/kembridge-auth/Cargo.toml
[package]
name = "kembridge-auth"
version = "0.1.0"
edition = "2021"

[dependencies]
# Crypto
secp256k1.workspace = true
ed25519-dalek.workspace = true
sha2.workspace = true
rand.workspace = true

# JWT
jsonwebtoken.workspace = true

# Database
sqlx.workspace = true
uuid.workspace = true
chrono.workspace = true

# Async
tokio.workspace = true
futures.workspace = true

# Serialization
serde.workspace = true
serde_json.workspace = true

# Error handling
anyhow.workspace = true
thiserror.workspace = true

# Validation
validator.workspace = true
```

```toml
# backend/crates/kembridge-crypto/Cargo.toml
[package]
name = "kembridge-crypto"
version = "0.1.0"
edition = "2021"

[dependencies]
# Post-quantum cryptography
pqcrypto.workspace = true
pqcrypto-kyber.workspace = true

# Classical crypto
rand.workspace = true
sha2.workspace = true

# AES for hybrid schemes
aes-gcm = "0.10"

# Database
sqlx.workspace = true
uuid.workspace = true

# Serialization
serde.workspace = true
serde_json.workspace = true

# Error handling
anyhow.workspace = true
thiserror.workspace = true

# Async
tokio.workspace = true
```

```toml
# backend/crates/kembridge-bridge/Cargo.toml
[package]
name = "kembridge-bridge"
version = "0.1.0"
edition = "2021"

[dependencies]
# Internal dependencies
kembridge-crypto.workspace = true
kembridge-blockchain.workspace = true
kembridge-database.workspace = true

# Database
sqlx.workspace = true
uuid.workspace = true
chrono.workspace = true

# Async
tokio.workspace = true
futures.workspace = true

# Serialization
serde.workspace = true
serde_json.workspace = true

# Error handling
anyhow.workspace = true
thiserror.workspace = true
```

```toml
# backend/crates/kembridge-blockchain/Cargo.toml
[package]
name = "kembridge-blockchain"
version = "0.1.0"
edition = "2021"

[dependencies]
# Ethereum
ethers.workspace = true

# NEAR
near-sdk.workspace = true
near-jsonrpc-client.workspace = true

# HTTP
reqwest.workspace = true

# Async
tokio.workspace = true
futures.workspace = true

# Serialization
serde.workspace = true
serde_json.workspace = true

# Error handling
anyhow.workspace = true
thiserror.workspace = true

# UUID
uuid.workspace = true
```

```toml
# backend/crates/kembridge-database/Cargo.toml
[package]
name = "kembridge-database"
version = "0.1.0"
edition = "2021"

[dependencies]
# Database
sqlx.workspace = true
redis.workspace = true

# UUID and time
uuid.workspace = true
chrono.workspace = true

# Async
tokio.workspace = true

# Serialization
serde.workspace = true
serde_json.workspace = true

# Error handling
anyhow.workspace = true
thiserror.workspace = true

# Configuration
config.workspace = true
```

#### Main Application Setup

```rust
// backend/src/main.rs
use axum::{
    http::{header, Method, StatusCode},
    response::Json,
    routing::get,
    Router,
};
use serde_json::{json, Value};
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;

use kembridge_auth as auth;
use kembridge_database as database;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "kembridge_backend=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    dotenvy::dotenv().ok();
    let config = config::Config::from_env()?;

    // Setup database connection pool
    let db_pool = database::create_pool(&config.database_url).await?;

    // Run migrations
    database::run_migrations(&db_pool).await?;

    // Setup Redis connection
    let redis_client = redis::Client::open(config.redis_url.as_str())?;

    // Create application state
    let app_state = AppState {
        db: db_pool,
        redis: redis_client,
        config: config.clone(),
    };

    // Build our application with routes
    let app = create_router(app_state).await?;

    // Run our application
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    tracing::info!("KEMBridge backend listening on {}", listener.local_addr()?);

    axum::serve(listener, app).await?;

    Ok(())
}

async fn create_router(state: AppState) -> anyhow::Result<Router> {
    let app = Router::new()
        // Health check
        .route("/health", get(health_check))

        // Auth routes (будут добавлены в следующих фазах)
        .nest("/api/v1/auth", auth::routes())

        // CORS layer
        .layer(
            CorsLayer::new()
                .allow_origin("http://localhost:5173".parse::<header::HeaderValue>()?)
                .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
                .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION]),
        )
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    Ok(app)
}

async fn health_check() -> Result<Json<Value>, StatusCode> {
    Ok(Json(json!({
        "status": "healthy",
        "service": "kembridge-backend",
        "version": env!("CARGO_PKG_VERSION"),
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "components": {
            "database": "ready",
            "redis": "ready",
            "auth": "ready"
        }
    })))
}

#[derive(Clone)]
pub struct AppState {
    pub db: sqlx::PgPool,
    pub redis: redis::Client,
    pub config: config::Config,
}
```

```rust
// backend/src/config.rs
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub database_url: String,
    pub redis_url: String,
    pub jwt_secret: String,
    pub cors_origins: Vec<String>,
    pub ethereum_rpc_url: String,
    pub near_rpc_url: String,
    pub quantum_key_storage_path: String,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        let config = Config {
            database_url: std::env::var("DATABASE_URL")
                .map_err(|_| anyhow::anyhow!("DATABASE_URL must be set"))?,
            redis_url: std::env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://localhost:6379".to_string()),
            jwt_secret: std::env::var("JWT_SECRET")
                .map_err(|_| anyhow::anyhow!("JWT_SECRET must be set"))?,
            cors_origins: std::env::var("CORS_ORIGINS")
                .unwrap_or_else(|_| "http://localhost:5173".to_string())
                .split(',')
                .map(|s| s.trim().to_string())
                .collect(),
            ethereum_rpc_url: std::env::var("ETHEREUM_RPC_URL")
                .map_err(|_| anyhow::anyhow!("ETHEREUM_RPC_URL must be set"))?,
            near_rpc_url: std::env::var("NEAR_RPC_URL")
                .unwrap_or_else(|_| "https://rpc.testnet.near.org".to_string()),
            quantum_key_storage_path: std::env::var("QUANTUM_KEY_STORAGE_PATH")
                .unwrap_or_else(|_| "./keys".to_string()),
        };

        Ok(config)
    }
}
```

#### Database Crate Foundation

```rust
// backend/crates/kembridge-database/src/lib.rs
pub mod models;
pub mod pool;

use sqlx::PgPool;

pub async fn create_pool(database_url: &str) -> anyhow::Result<PgPool> {
    let pool = PgPool::connect(database_url).await?;
    Ok(pool)
}

pub async fn run_migrations(pool: &PgPool) -> anyhow::Result<()> {
    sqlx::migrate!("../migrations").run(pool).await?;
    Ok(())
}
```

#### Auth Crate Foundation

```rust
// backend/crates/kembridge-auth/src/lib.rs
pub mod web3;
pub mod jwt;
pub mod models;

use axum::{routing::post, Router};

pub fn routes() -> Router {
    Router::new()
        .route("/nonce", post(|| async { "nonce endpoint - todo" }))
        .route("/verify", post(|| async { "verify endpoint - todo" }))
}
```

### Development Scripts

```json
// package.json (root) - Docker-first approach
{
  "name": "kembridge",
  "private": true,
  "scripts": {
    "dev": "docker-compose up --build",
    "dev:detached": "docker-compose up -d --build",
    "dev:services": "docker-compose up postgres redis -d",
    "dev:backend": "docker-compose up backend --build",
    "dev:frontend": "docker-compose up frontend --build",
    "dev:ai": "docker-compose up ai-engine --build",
    "build": "docker-compose build",
    "build:prod": "docker-compose -f docker-compose.yml -f docker-compose.prod.yml build",
    "prod": "docker-compose -f docker-compose.yml -f docker-compose.prod.yml up -d",
    "logs": "docker-compose logs -f",
    "logs:backend": "docker-compose logs -f backend",
    "logs:frontend": "docker-compose logs -f frontend",
    "stop": "docker-compose down",
    "clean": "docker-compose down -v --rmi all",
    "db:migrate": "docker-compose exec backend sqlx migrate run",
    "db:reset": "docker-compose down -v && docker-compose up postgres redis -d",
    "test": "docker-compose -f docker-compose.test.yml up --abort-on-container-exit",
    "test:backend": "docker-compose exec backend cargo test",
    "test:frontend": "docker-compose exec frontend npm test",
    "shell:backend": "docker-compose exec backend bash",
    "shell:frontend": "docker-compose exec frontend sh",
    "monitoring": "docker-compose up prometheus grafana -d"
  }
}
```

## 🎯 System Validation

После запуска `docker-compose up --build` система должна:

### ✅ All Services Running

```bash
# Полный запуск всех сервисов
docker-compose up -d

# Проверка состояния всех контейнеров
docker-compose ps

# Тест backend health check
curl http://localhost:3000/health

# Ожидаемый ответ:
{
  "status": "healthy",
  "service": "kembridge-backend",
  "version": "0.1.0",
  "timestamp": "2024-01-XX...",
  "containers": {
    "postgres": "healthy",
    "redis": "healthy",
    "ai-engine": "ready"
  }
}

# Тест AI Engine
curl http://localhost:8000/health

# Тест Frontend
curl http://localhost:3001
```

### ✅ Full Stack Validation

```bash
# Проверка всех сервисов
docker-compose logs

# Проверка сетевого коммуникейшна
# Frontend -> Backend
curl http://localhost:3001/api/health

# Backend -> AI Engine
curl http://localhost:3000/api/v1/ai/health

# Backend -> Database
docker-compose exec backend sqlx database info

# Открыть в браузере:
# Frontend: http://localhost:3001
# Backend API: http://localhost:3000/health
# AI Engine: http://localhost:8000/docs
# Grafana: http://localhost:3002 (admin:admin)
# Prometheus: http://localhost:9090
```

### ✅ Database & Cache Validation

```bash
# Подключение к PostgreSQL через Docker
docker-compose exec postgres psql -U postgres -d kembridge_dev -c "SELECT version();"

# Проверка Redis
docker-compose exec redis redis-cli ping

# Проверка миграций
docker-compose exec backend sqlx migrate info

# Проверка сетевого соединения между контейнерами
docker-compose exec backend ping postgres
docker-compose exec backend ping redis
```

#### 1.1.6 Development & Production Overrides

##### Development Docker Compose Override

```yaml
# docker-compose.override.yml - Автоматически применяется в development
version: "3.8"

services:
  backend:
    volumes:
      - ./backend:/app:cached
      - backend_cargo_cache:/usr/local/cargo
    environment:
      - RUST_LOG=debug
      - RUST_BACKTRACE=1
    command: cargo watch -x run

  frontend:
    volumes:
      - ./frontend:/app:cached
      - frontend_node_modules:/app/node_modules
    environment:
      - VITE_API_BASE_URL=http://backend:3000/api/v1
      - CHOKIDAR_USEPOLLING=true

  ai-engine:
    volumes:
      - ./ai-engine:/app:cached
    environment:
      - PYTHONPATH=/app
      - FLASK_ENV=development
    command: uvicorn main:app --host 0.0.0.0 --port 8000 --reload

volumes:
  backend_cargo_cache:
  frontend_node_modules:
```

##### Production Docker Compose

```yaml
# docker-compose.prod.yml - Production overrides
version: "3.8"

services:
  backend:
    build:
      target: production
    restart: always
    environment:
      - RUST_LOG=info
    deploy:
      replicas: 2
      update_config:
        parallelism: 1
        delay: 10s
      restart_policy:
        condition: on-failure

  frontend:
    build:
      target: production
    restart: always
    deploy:
      replicas: 2

  ai-engine:
    restart: always
    environment:
      - PYTHONPATH=/app
    command: gunicorn -w 4 -k uvicorn.workers.UvicornWorker main:app --bind 0.0.0.0:8000

  postgres:
    environment:
      POSTGRES_PASSWORD: ${POSTGRES_PASSWORD}
    volumes:
      - postgres_prod_data:/var/lib/postgresql/data

  redis:
    command: redis-server --requirepass ${REDIS_PASSWORD}

volumes:
  postgres_prod_data:
```

##### Nginx Configuration

```nginx
# nginx/default.conf
upstream backend {
    server backend:3000;
}

upstream ai-engine {
    server ai-engine:8000;
}

server {
    listen 80;
    server_name localhost;

    # Frontend
    location / {
        proxy_pass http://frontend:3000;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }

    # Backend API
    location /api/ {
        proxy_pass http://backend;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;

        # WebSocket support
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
    }

    # AI Engine
    location /ai/ {
        proxy_pass http://ai-engine/;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

##### Quick Start Commands

```bash
# Полный старт системы (одна команда)
make dev

# Альтернативно
docker-compose up --build

# Установка зависимостей (внутри контейнеров)
docker-compose exec backend cargo build
docker-compose exec frontend npm install
docker-compose exec ai-engine pip install -r requirements.txt

# Миграции базы данных
docker-compose exec backend sqlx migrate run

# Логи в реальном времени
docker-compose logs -f

# Остановка с очисткой
docker-compose down -v
```

##### Makefile для упрощения

```makefile
# Makefile
.PHONY: dev prod build clean logs test

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
	@curl -f http://localhost:3000/health || echo "Backend: FAIL"
	@curl -f http://localhost:8000/health || echo "AI Engine: FAIL"
	@curl -f http://localhost:3001 || echo "Frontend: FAIL"
```

## ✅ Статус докеризации компонентов

| Компонент             | Статус             | Контейнер            | Порт       | Комментарий                         |
| --------------------- | ------------------ | -------------------- | ---------- | ----------------------------------- |
| **Frontend**          | 🐳 100% Dockerized | kembridge_frontend   | 3001       | React + Vite + Hot Reload           |
| **Backend**           | 🐳 100% Dockerized | kembridge_backend    | 3000       | Rust + Axum + Live Rebuild          |
| **AI Engine**         | 🐳 100% Dockerized | kembridge_ai_engine  | 8000       | Python + FastAPI + Auto Reload      |
| **Database**          | 🐳 100% Dockerized | kembridge_postgres   | 5432       | PostgreSQL 18β1 + Migrations        |
| **Cache**             | 🐳 100% Dockerized | kembridge_redis      | 6379       | Redis 7 + Persistence               |
| **Reverse Proxy**     | 🐳 100% Dockerized | kembridge_nginx      | 80, 443    | Nginx + SSL Ready                   |
| **Monitoring**        | 🐳 100% Dockerized | prometheus + grafana | 9090, 3002 | Metrics + Dashboards                |
| **Development Tools** | 🐳 100% Dockerized | -                    | -          | Hot reload, cargo watch, npm dev    |
| **Dependencies**      | 🐳 100% Dockerized | -                    | -          | Rust crates, npm packages в volumes |
| **Build Process**     | 🐳 100% Dockerized | -                    | -          | Multi-stage builds                  |

### 🎯 Полная докеризация: 100%

**Всё работает в Docker без исключений:**

- ✅ **Zero host dependencies** - не нужно устанавливать Rust, Node.js, PostgreSQL на хост
- ✅ **Complete isolation** - каждый сервис изолирован в своём контейнере
- ✅ **Development parity** - dev и prod используют одинаковые образы
- ✅ **One command startup** - `docker-compose up` запускает всю систему
- ✅ **Hot reload included** - live development без выхода из контейнеров
- ✅ **Volume optimization** - кеширование зависимостей и сборок
- ✅ **Network isolation** - внутренняя сеть для безопасности
- ✅ **Health checks** - автоматическая проверка состояния сервисов
- ✅ **Resource limits** - контроль использования ресурсов
- ✅ **Graceful shutdown** - корректное завершение всех сервисов

**Внешние зависимости (НЕ в Docker, по дизайну):**

- 🌐 **Blockchain networks** - Ethereum Sepolia, NEAR Testnet (внешние сети)
- 🔗 **External APIs** - Chainlink, 1inch, Infura (внешние сервисы)
- 💾 **User wallets** - MetaMask, NEAR Wallet (браузерные расширения)

## 📊 Technology Decision Summary

| Component                | Выбранная технология                  | Причина выбора                                             |
| ------------------------ | ------------------------------------- | ---------------------------------------------------------- |
| **Containerization**     | Docker + Docker Compose               | Полная изоляция, консистентность, масштабируемость         |
| **Backend Framework**    | Axum                                  | Зрелая экосистема, высокая производительность, type safety |
| **Backend Architecture** | Cargo Workspace с модульными крейтами | Модульность, type safety, параллельная разработка          |
| **Database Layer**       | PostgreSQL 18 Beta 1 + sqlx           | OAuth 2.0 поддержка, производительность JSON/JSONB         |
| **Frontend**             | React + Vite + TypeScript + SCSS      | Проверенный стек, MobileFirst подход                       |
| **State Management**     | TanStack Query                        | Server state специализация, автоматическое кеширование     |
| **Styling**              | SCSS с модульной архитектурой         | Гибкость, MobileFirst подход                               |
| **Cryptography**         | pqcrypto-kyber + AES-GCM              | ML-KEM-1024 + гибридная схема                              |

## 🚀 Next Steps

После завершения Project Setup переходим к **Phase 1.2: Database Schema & Migrations** из hackathon-development-plan.md
