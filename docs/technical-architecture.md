# KEMBridge Technical Architecture

## Техническая архитектура и диаграммы

Данный документ содержит детальные архитектурные схемы для KEMBridge — квантово-устойчивого кросс-чейн моста. Представленные диаграммы отражают архитектуру для хакатон версии проекта (2-3 недели разработки).

## 🎯 Хакатон версия - упрощенная архитектура

**Авторизация:** Только Web3 кошельки (MetaMask, NEAR Wallet, WalletConnect)
**Блокчейны:** Ethereum ↔ NEAR Protocol
**База данных:** PostgreSQL 18 Beta 1 (готова для будущих версий)
**Криптография:** Базовый ML-KEM-1024 для демонстрации

### Выбор архитектуры для хакатона

Для демонстрации на хакатоне выбрана **упрощенная архитектура** с минимальным количеством сервисов. После хакатона планируется миграция к **полноценной микросервисной архитектуре** с Event Bus и доменным разделением.

## Архитектура микросервисов для хакатона

```mermaid
graph TB
    %% Клиентский слой
    subgraph "Client Layer"
        UI[Web App<br/>React/Next.js]
        WALLET[Wallet Integration<br/>MetaMask/NEAR Wallet]
    end

    %% API Gateway с упрощенной аутентификацией
    subgraph "API Gateway"
        GATEWAY[API Gateway<br/>Rust/Axum]
        AUTH[Auth Service<br/>Web3 Wallet Only]
    end

    %% Core Services для хакатона
    subgraph "Core Services"
        CRYPTO[Quantum Crypto<br/>ML-KEM-1024<br/>Rust]
        BRIDGE[Bridge Service<br/>ETH ↔ NEAR<br/>Rust]
        AI[AI Risk Engine<br/>Python/FastAPI]
        ORACLE[Price Oracle<br/>Chainlink Integration]
    end

    %% Blockchain Integrations
    subgraph "Blockchain Layer"
        ETH[Ethereum<br/>Sepolia Testnet]
        NEAR[NEAR Protocol<br/>Testnet]
        ONEINCH[1inch Fusion+<br/>Integration]
    end

    %% Data Layer (готова для будущих версий)
    subgraph "Data Layer"
        PG[PostgreSQL 18 Beta 1<br/>Future-Ready Schema]
        REDIS[Redis<br/>Cache & Sessions]
        IPFS[IPFS<br/>Transaction Logs]
    end

    %% Monitoring
    subgraph "Monitoring"
        METRICS[Prometheus<br/>Metrics]
        LOGS[Grafana<br/>Dashboards]
    end

    %% Connections
    UI --> GATEWAY
    WALLET --> GATEWAY
    GATEWAY --> AUTH
    AUTH --> PG
    GATEWAY --> CRYPTO
    GATEWAY --> BRIDGE
    GATEWAY --> AI
    
    BRIDGE --> ETH
    BRIDGE --> NEAR
    BRIDGE --> ONEINCH
    
    CRYPTO --> PG
    BRIDGE --> PG
    AI --> PG
    ORACLE --> PG
    
    GATEWAY --> REDIS
    
    BRIDGE --> IPFS
    
    %% Monitoring connections
    GATEWAY -.-> METRICS
    CRYPTO -.-> METRICS
    BRIDGE -.-> METRICS
    AI -.-> METRICS
    
    METRICS --> LOGS
```

## Последовательность операций кросс-чейн свопа

```mermaid
sequenceDiagram
    autonumber
    participant User as Пользователь
    participant UI as Web App
    participant GW as API Gateway
    participant Auth as Auth Service
    participant AI as AI Risk Engine
    participant QC as Quantum Crypto
    participant Bridge as Bridge Service
    participant ETH as Ethereum
    participant NEAR as NEAR Protocol
    participant DB as PostgreSQL 18
    participant Oracle as Price Oracle

    User->>UI: Запрос свопа 1 ETH → NEAR
    UI->>GW: POST /swap {from: ETH, to: NEAR, amount: 1}
    GW->>Auth: Проверка JWT токена
    Auth-->>GW: ✓ Авторизован
    
    GW->>Oracle: Получить курс ETH/NEAR
    Oracle-->>GW: Курс: 1 ETH = 2,500 NEAR
    
    GW->>AI: Анализ риска операции
    AI->>DB: Получить историю пользователя
    DB-->>AI: История транзакций
    AI-->>GW: Риск: НИЗКИЙ (score: 0.15)
    
    GW->>QC: Генерация квантовых ключей
    QC-->>GW: {publicKey, encryptedPrivateKey}
    
    GW->>Bridge: Инициализация свопа
    Bridge->>DB: Создать запись транзакции
    DB-->>Bridge: TX_ID: 0x1234...
    
    Bridge->>ETH: Блокировка 1 ETH
    ETH-->>Bridge: ETH заблокирован (tx_hash)
    
    Bridge->>NEAR: Минт 2,500 NEAR
    NEAR-->>Bridge: NEAR заминчен (tx_hash)
    
    Bridge->>DB: Обновить статус: COMPLETED
    Bridge->>UI: Уведомление о завершении
    UI->>User: ✓ Своп завершен успешно
```

## Архитектура безопасности

```mermaid
graph TB
    subgraph "Security Layers"
        subgraph "Quantum Layer"
            MLKEM[ML-KEM-1024<br/>Key Encapsulation]
            DILI[Dilithium-5<br/>Digital Signatures]
            HYBRID[Hybrid Crypto<br/>Classical + Quantum]
        end
        
        subgraph "AI Security"
            ANOMALY[Anomaly Detection<br/>Real-time Analysis]
            RISK[Risk Scoring<br/>ML Models]
            ADAPTIVE[Adaptive Protection<br/>Dynamic Thresholds]
        end
        
        subgraph "Traditional Security"
            OAUTH[OAuth 2.0<br/>PostgreSQL 18]
            JWT[JWT Tokens<br/>Session Management]
            RBAC[Role-Based Access<br/>Permissions]
        end
    end
    
    subgraph "Threat Detection"
        MONITOR[Transaction Monitor<br/>Real-time Analysis]
        THREAT[Threat Intelligence<br/>Known Attack Patterns]
        ALERT[Alert System<br/>Automated Response]
    end
    
    %% Connections
    MLKEM --> HYBRID
    DILI --> HYBRID
    HYBRID --> MONITOR
    
    ANOMALY --> RISK
    RISK --> ADAPTIVE
    ADAPTIVE --> ALERT
    
    OAUTH --> JWT
    JWT --> RBAC
    RBAC --> MONITOR
    
    MONITOR --> THREAT
    THREAT --> ALERT
```

## База данных - PostgreSQL 18 Beta 1

```mermaid
erDiagram
    users {
        uuid id PK
        string wallet_address
        string email
        jsonb oauth_profile
        timestamp created_at
        timestamp updated_at
    }
    
    transactions {
        uuid id PK
        uuid user_id FK
        string from_chain
        string to_chain
        string from_token
        string to_token
        decimal amount_from
        decimal amount_to
        string status
        jsonb quantum_keys
        jsonb risk_analysis
        timestamp created_at
        timestamp completed_at
    }
    
    risk_profiles {
        uuid id PK
        uuid user_id FK
        decimal risk_score
        jsonb behavior_data
        jsonb ml_features
        timestamp updated_at
    }
    
    audit_logs {
        uuid id PK
        uuid transaction_id FK
        string event_type
        jsonb event_data
        timestamp created_at
    }
    
    quantum_keys {
        uuid id PK
        uuid transaction_id FK
        text public_key
        text encrypted_private_key
        string algorithm
        timestamp created_at
    }
    
    users ||--o{ transactions : "creates"
    users ||--o{ risk_profiles : "has"
    transactions ||--o{ audit_logs : "generates"
    transactions ||--o{ quantum_keys : "uses"
```

## Интеграция с Near Protocol

```mermaid
graph LR
    subgraph "KEMBridge"
        BRIDGE[Bridge Service]
        SHADE[Shade Agent<br/>AI Security]
        QUANTUM[Quantum Crypto<br/>ML-KEM]
    end
    
    subgraph "NEAR Protocol"
        CHAIN_SIG[Chain Signatures<br/>Cross-chain Control]
        CLICK_API[1Click API<br/>Simplified UX]
        CONTRACT[Smart Contract<br/>Bridge Logic]
    end
    
    subgraph "External Chains"
        ETH[Ethereum<br/>ERC-20 Tokens]
        POLYGON[Polygon<br/>Future Support]
    end
    
    subgraph "1inch Integration"
        FUSION[Fusion+<br/>Atomic Swaps]
        ROUTING[Route Optimization<br/>Best Prices]
    end
    
    %% Connections
    BRIDGE --> CHAIN_SIG
    BRIDGE --> CLICK_API
    SHADE --> CONTRACT
    QUANTUM --> CONTRACT
    
    CHAIN_SIG --> ETH
    CHAIN_SIG --> POLYGON
    
    CLICK_API --> FUSION
    FUSION --> ROUTING
    
    CONTRACT --> FUSION
```

## Будущая микросервисная архитектура (Post-Hackathon)

После успешной демонстрации на хакатоне планируется миграция к полноценной микросервисной архитектуре:

```mermaid
%% KEMBridge — микросервисная архитектура (post-hackathon)
graph TD
  %% ───────────── Client ─────────────
  subgraph "Client Layer"
    direction TB
    WEBUI["Web App<br/>(React / Next.js)"]
    WALLET["Wallet Provider<br/>(MetaMask / NEAR Wallet)"]
  end

  %% ───────────── Edge ─────────────
  subgraph "Edge"
    APIGW["API Gateway<br/>(Rust / Axum)"]
  end

  %% ───────────── Auth ─────────────
  subgraph "Auth Domain"
    AUTH["Auth Service<br/>Web3 + OAuth"]
    JWT["Session Service<br/>(JWT / Refresh)"]
  end

  %% ───────────── Swap / Bridge ─────────────
  subgraph "Swap Domain"
    ORCH["Swap Orchestrator"]
    QC["Quantum Crypto Svc<br/>(Kyber KEM)"]
    RISK["AI Risk Engine"]
  end

  %% ───────────── Chain Adapters ─────────────
  subgraph "Chain Adapters"
    ETH_ADPT["Ethereum Adapter"]
    NEAR_ADPT["NEAR Adapter<br/>(Chain Signatures)"]
    FUSION_ADPT["1inch Fusion Router"]
  end

  %% ───────────── External Networks ─────────────
  ETH_NET(("Ethereum"))
  NEAR_NET(("NEAR Protocol"))
  ONEINCH(("1inch Fusion"))

  %% ───────────── Data Layer ─────────────
  subgraph "Data Layer"
    PG["PostgreSQL 18"]
    REDIS["Redis Cache"]
    IPFS["IPFS / Logs"]
  end

  %% ───────────── Infra & Observability ─────────────
  subgraph "Infrastructure"
    BUS["NATS / Kafka<br/>Event Bus"]
    PROM["Prometheus"]
    GRAF["Grafana"]
  end

  %% ───────────── Connections ─────────────
  WEBUI -->|REST / WS| APIGW
  WALLET -->|Wallet RPC| APIGW

  APIGW --> AUTH
  APIGW --> ORCH

  AUTH --> JWT
  AUTH --> PG
  AUTH -. Publishes .-> BUS

  ORCH --> QC
  ORCH --> RISK
  ORCH --> ETH_ADPT
  ORCH --> NEAR_ADPT
  ORCH --> FUSION_ADPT
  ORCH --> PG
  ORCH -. Publishes .-> BUS

  QC --> PG
  RISK --> REDIS
  RISK --> PG
  RISK -. Publishes .-> BUS

  ETH_ADPT -- tx --> ETH_NET
  NEAR_ADPT -- tx --> NEAR_NET
  FUSION_ADPT -- swap --> ONEINCH

  ORCH --> IPFS
  AUTH --> IPFS

  APIGW -.-> PROM
  ORCH -.-> PROM
  RISK -.-> PROM
  ETH_ADPT -.-> PROM
  PROM --> GRAF

  BUS --> PROM
```

### Преимущества микросервисной архитектуры:

**🚀 Масштабируемость:**
- Независимое масштабирование каждого домена
- Event Bus для асинхронной обработки
- Горизонтальное масштабирование адаптеров

**🔧 Гибкость разработки:**
- Четкие границы доменов (Auth, Swap, Chain IO)
- Простое добавление новых блокчейнов
- Независимые команды разработки

**🛡️ Надежность:**
- Изоляция сбоев
- Circuit breakers между сервисами
- Распределенный мониторинг

## Развертывание для хакатона

```mermaid
graph TB
    subgraph "Development Environment"
        DEV[Local Development<br/>Docker Compose]
        TEST[Unit Tests<br/>Integration Tests]
    end
    
    subgraph "Hackathon Demo"
        DEMO[Demo Environment<br/>AWS/Digital Ocean]
        FRONTEND[React Frontend<br/>Vercel/Netlify]
        BACKEND[Rust Backend<br/>Docker Containers]
    end
    
    subgraph "Blockchain Testnets"
        ETH_TEST[Ethereum Sepolia<br/>Testnet]
        NEAR_TEST[NEAR Testnet<br/>Testnet]
    end
    
    subgraph "External Services"
        CHAINLINK[Chainlink<br/>Price Feeds]
        IPFS_NODE[IPFS Node<br/>Pinata/Fleek]
    end
    
    %% Connections
    DEV --> TEST
    TEST --> DEMO
    
    DEMO --> FRONTEND
    DEMO --> BACKEND
    
    BACKEND --> ETH_TEST
    BACKEND --> NEAR_TEST
    BACKEND --> CHAINLINK
    BACKEND --> IPFS_NODE
```

## Особенности PostgreSQL 18 Beta 1

В хакатон версии KEMBridge планируется использовать **PostgreSQL 18 Beta 1** с нативной поддержкой **OAuth 2.0**. Это стратегическое решение основано на том, что к моменту production релиза KEMBridge PostgreSQL 18 будет иметь стабильную версию.

**Преимущества PostgreSQL 18 для KEMBridge:**

1. **Встроенная OAuth 2.0 поддержка** - упрощает интеграцию с Web3Auth и другими OAuth провайдерами
2. **Улучшенная производительность JSON/JSONB** - критично для хранения квантовых ключей и метаданных транзакций
3. **Расширенные возможности аудита** - встроенные механизмы для compliance и безопасности
4. **Лучшая поддержка шифрования** - совместимость с постквантовыми алгоритмами

**Применение в KEMBridge:**
- Аутентификация пользователей через OAuth 2.0
- Хранение зашифрованных квантовых ключей
- Журналирование всех операций для аудита
- Интеграция с AI/ML системами для анализа рисков