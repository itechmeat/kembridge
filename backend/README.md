# KEMBridge Database Schema

## Overview

KEMBridge uses PostgreSQL 18 Beta 1 with advanced features including post-quantum cryptography, AI risk analysis, and comprehensive audit logging. The database is designed for high-performance cross-chain operations with enterprise-grade security and monitoring.

## Database Architecture

```mermaid
erDiagram
    users {
        uuid id PK "UUIDv7 primary key"
        varchar username UK "Unique username"
        jsonb profile_data "User profile information"
        transaction_timestamp created_at
        transaction_timestamp updated_at
        boolean is_active
        jsonb risk_profile "AI risk assessment data"
        int profile_completeness "Generated column"
        varchar risk_category "Generated column"
        timestamp last_login_at
        varchar account_status
    }

    user_auth_methods {
        uuid id PK "UUIDv7 primary key"
        uuid user_id FK
        varchar auth_type "web3_wallet, oauth_*"
        varchar chain_type "ethereum, near, polygon"
        varchar wallet_address
        varchar oauth_provider "google, github, discord"
        varchar oauth_subject_id
        varchar oauth_token_hash "SHA-256 hash"
        text public_key
        jsonb signature_params
        jsonb oauth_metadata
        transaction_timestamp first_used_at
        transaction_timestamp last_used_at
        boolean is_verified
        boolean is_primary
        varchar security_level
        timestamp expires_at
        varchar auth_category "Generated column"
        int auth_strength "Generated column"
    }

    user_sessions {
        uuid id PK "UUIDv7 primary key"
        uuid user_id FK
        uuid auth_method_id FK
        varchar jwt_token_hash UK "SHA-256 hash"
        int jwt_token_version
        varchar oauth_refresh_token_hash UK
        varchar oauth_access_token_hash
        jsonb session_data
        inet ip_address
        text user_agent
        varchar device_fingerprint
        jsonb geolocation
        transaction_timestamp created_at
        timestamp expires_at
        transaction_timestamp last_activity_at
        boolean is_active
        timestamp revoked_at
        varchar revoked_reason
        varchar security_level
        jsonb risk_flags
        int session_duration_minutes "Generated column"
        varchar session_status "Generated column"
        varchar risk_level "Generated column"
    }

    quantum_keys {
        quantum_key_id id PK "UUIDv7 primary key"
        uuid user_id FK
        varchar algorithm "ml-kem-1024, dilithium-5"
        varchar key_type "key_encapsulation, digital_signature"
        bytea public_key
        bytea encrypted_private_key
        varchar encryption_algorithm
        bytea encryption_iv
        bytea encryption_salt
        jsonb key_derivation_params
        jsonb security_metadata
        transaction_timestamp created_at
        timestamp expires_at
        timestamp rotated_at
        boolean is_active
        boolean is_compromised
        varchar validation_status
        uuid previous_key_id FK
        varchar rotation_reason
        int rotation_generation
        varchar hsm_key_id
        varchar hsm_provider
        varchar key_status "Generated column"
        varchar key_strength "Generated column"
        varchar usage_category "Generated column"
    }

    transactions {
        uuid id PK "UUIDv7 primary key"
        uuid user_id FK
        varchar source_chain
        varchar destination_chain
        varchar source_token
        varchar destination_token
        decimal amount_in
        decimal amount_out
        decimal exchange_rate
        decimal bridge_fee_amount
        decimal network_fee_amount
        decimal quantum_protection_fee
        varchar source_tx_hash
        varchar destination_tx_hash
        varchar bridge_tx_hash
        varchar status
        jsonb status_history
        quantum_key_id quantum_key_id FK
        bytea encrypted_payload
        jsonb encryption_metadata
        risk_score risk_score
        jsonb risk_factors
        varchar ai_analysis_version
        transaction_timestamp created_at
        transaction_timestamp updated_at
        timestamp confirmed_at
        timestamp completed_at
        timestamp expires_at
        varchar oneinch_order_id
        varchar oneinch_quote_id
        jsonb fusion_metadata
        jsonb near_chain_signature
        varchar near_account_id
        decimal transaction_value_usd "Generated column"
        int processing_time_minutes "Generated column"
        varchar transaction_category "Generated column"
        varchar bridge_direction "Generated column"
        varchar risk_category "Generated column"
    }

    audit_logs {
        uuid id PK "UUIDv7 primary key"
        uuid user_id FK
        uuid transaction_id FK
        uuid session_id FK
        uuid quantum_key_id FK
        varchar event_type
        varchar event_category
        varchar event_subcategory
        jsonb event_data
        inet ip_address
        text user_agent
        uuid request_id
        uuid correlation_id
        varchar api_endpoint
        varchar http_method
        int response_status
        int response_time_ms
        transaction_timestamp created_at
        varchar severity
        varchar retention_category
        boolean is_sensitive
        boolean is_pii
        varchar data_source
        varchar processing_status
        jsonb threat_indicators
        jsonb investigation_tags
        int event_severity_score "Generated column"
        varchar event_classification "Generated column"
        varchar response_category "Generated column"
        varchar processing_time_category "Generated column"
    }

    pg_audit_config {
        varchar setting_name PK
        varchar setting_value
        varchar description
        timestamp updated_at
    }

    users ||--o{ user_auth_methods : "has many"
    users ||--o{ user_sessions : "has many"
    users ||--o{ quantum_keys : "has many"
    users ||--o{ transactions : "has many"
    users ||--o{ audit_logs : "has many"
    
    user_auth_methods ||--o{ user_sessions : "has many"
    user_auth_methods ||--o{ audit_logs : "references"
    
    quantum_keys ||--o{ transactions : "encrypts"
    quantum_keys ||--o{ audit_logs : "references"
    quantum_keys ||--o{ quantum_keys : "rotation chain"
    
    transactions ||--o{ audit_logs : "has many"
    user_sessions ||--o{ audit_logs : "has many"
```

## Key Features

### PostgreSQL 18 Advanced Features

- **UUIDv7**: Timestamp-ordered UUIDs for optimal B-tree performance
- **Virtual Generated Columns**: Real-time calculated fields for analytics
- **Enhanced JSONB**: SIMD optimization for high-performance JSON operations
- **Skip Scan Indexing**: Optimized multicolumn B-tree index performance
- **Table Partitioning**: Monthly partitions for audit_logs high-volume data
- **Extended Statistics**: Correlated column statistics for query optimization

### Security Features

- **Post-Quantum Cryptography**: ML-KEM-1024 and Dilithium-5 support
- **Comprehensive Audit**: Every operation logged with threat detection
- **Risk Assessment**: AI-powered risk scoring with virtual columns
- **OAuth 2.0 Integration**: Enterprise-grade authentication
- **HSM Support**: Hardware Security Module integration ready

### Performance Optimizations

- **GIN Indexes**: Optimized JSONB queries
- **Partial Indexes**: Condition-based indexing for active records
- **Statistics Collection**: Enhanced statistics for complex queries
- **Connection Pooling**: Optimized for high-concurrency operations

## Installation & Setup

### Prerequisites

- Docker & Docker Compose
- PostgreSQL 18 Beta 1
- VS Code with PostgreSQL extensions

### Quick Start

```bash
# Start the database
make dev

# Or with Docker Compose
docker-compose up -d postgres

# Wait for database to be ready
docker-compose exec postgres pg_isready -U postgres

# Check database version
docker-compose exec postgres psql -U postgres -d kembridge_dev -c "SELECT version();"
```

### Manual Migration Execution

```bash
# Run all migrations manually
docker-compose exec postgres psql -U postgres -d kembridge_dev -f /docker-entrypoint-initdb.d/001_postgresql18_extensions_config.sql
docker-compose exec postgres psql -U postgres -d kembridge_dev -f /docker-entrypoint-initdb.d/002_users_table_postgresql18.sql
docker-compose exec postgres psql -U postgres -d kembridge_dev -f /docker-entrypoint-initdb.d/003_auth_methods_oauth_postgresql18.sql
docker-compose exec postgres psql -U postgres -d kembridge_dev -f /docker-entrypoint-initdb.d/004_user_sessions_advanced_postgresql18.sql
docker-compose exec postgres psql -U postgres -d kembridge_dev -f /docker-entrypoint-initdb.d/005_quantum_keys_postgresql18.sql
docker-compose exec postgres psql -U postgres -d kembridge_dev -f /docker-entrypoint-initdb.d/006_transactions_advanced_postgresql18.sql
docker-compose exec postgres psql -U postgres -d kembridge_dev -f /docker-entrypoint-initdb.d/007_audit_logs_comprehensive_postgresql18.sql
```

## Testing with VS Code

### Recommended Extensions

1. **PostgreSQL** by Chris Kolkman
2. **SQLTools** by Matheus Teixeira
3. **Thunder Client** for API testing

### VS Code PostgreSQL Extension Setup

1. Install the PostgreSQL extension from the marketplace
2. Open Command Palette (`Ctrl+Shift+P` / `Cmd+Shift+P`)
3. Run `PostgreSQL: New Query`
4. Configure connection:

```json
{
  "host": "localhost",
  "port": 5432,
  "database": "kembridge_dev",
  "username": "postgres",
  "password": "dev_password",
  "ssl": false
}
```

### Example Queries for Testing

#### 1. Test UUIDv7 Generation

```sql
-- Test UUIDv7 generation and ordering
SELECT 
    generate_uuidv7() as uuid1,
    generate_uuidv7() as uuid2,
    generate_uuidv7() as uuid3;

-- Verify timestamp ordering
WITH test_uuids AS (
    SELECT generate_uuidv7() as id, NOW() as created
    FROM generate_series(1, 10)
)
SELECT id, created,
       EXTRACT(EPOCH FROM created) as timestamp_seconds
FROM test_uuids
ORDER BY id;
```

#### 2. Test User Creation with Virtual Columns

```sql
-- Create a test user with profile data
INSERT INTO users (username, profile_data, risk_profile) VALUES 
('test_user_1', 
 '{"email": "test@example.com", "display_name": "Test User", "country": "US"}',
 '{"score": 0.15, "factors": ["new_user"], "last_assessment": "2024-07-12T08:00:00Z"}');

-- View the generated columns in action
SELECT 
    id,
    username,
    profile_data,
    profile_completeness, -- Generated column
    risk_category,        -- Generated column
    risk_profile
FROM users 
WHERE username = 'test_user_1';
```

#### 3. Test Authentication Methods

```sql
-- Create Web3 authentication method
INSERT INTO user_auth_methods (
    user_id, 
    auth_type, 
    chain_type, 
    wallet_address,
    is_verified,
    security_level
) VALUES (
    (SELECT id FROM users WHERE username = 'test_user_1'),
    'web3_wallet',
    'ethereum',
    '0x742d35Cc6634C0532925a3b8D4FF88ac73B2b7F4',
    true,
    'high'
);

-- View virtual columns
SELECT 
    auth_type,
    chain_type,
    wallet_address,
    auth_category,    -- Generated column
    auth_strength,    -- Generated column
    security_level
FROM user_auth_methods 
WHERE user_id = (SELECT id FROM users WHERE username = 'test_user_1');
```

#### 4. Test Quantum Key Generation

```sql
-- Generate a quantum key
SELECT generate_quantum_key(
    (SELECT id FROM users WHERE username = 'test_user_1'),
    'ml-kem-1024',
    'key_encapsulation',
    decode('48656c6c6f20576f726c64', 'hex'), -- Mock public key
    decode('53656372657420507269766174652041', 'hex'), -- Mock encrypted private key
    '{"kdf": "pbkdf2", "iterations": 100000}',
    365,
    null
) as quantum_key_id;

-- View quantum keys with virtual columns
SELECT 
    id,
    algorithm,
    key_type,
    key_status,      -- Generated column
    key_strength,    -- Generated column
    usage_category,  -- Generated column
    validation_status,
    created_at
FROM quantum_keys 
WHERE user_id = (SELECT id FROM users WHERE username = 'test_user_1');
```

#### 5. Test Transaction with Risk Analysis

```sql
-- Create a bridge transaction
SELECT create_bridge_transaction(
    (SELECT id FROM users WHERE username = 'test_user_1'),
    'ethereum',
    'near',
    'ETH',
    'NEAR',
    1.5, -- 1.5 ETH
    6.0, -- Expected 6.0 NEAR
    (SELECT id FROM quantum_keys WHERE user_id = (SELECT id FROM users WHERE username = 'test_user_1') LIMIT 1),
    24,
    'quote_12345'
) as transaction_id;

-- View transaction with all virtual columns
SELECT 
    id,
    source_chain,
    destination_chain,
    amount_in,
    amount_out,
    transaction_value_usd,    -- Generated column
    transaction_category,     -- Generated column
    bridge_direction,         -- Generated column
    risk_score,
    risk_category,           -- Generated column
    status,
    created_at
FROM transactions 
WHERE user_id = (SELECT id FROM users WHERE username = 'test_user_1');
```

#### 6. Test Audit Logging

```sql
-- View recent audit events
SELECT 
    event_type,
    event_category,
    event_classification,        -- Generated column
    event_severity_score,        -- Generated column
    response_category,           -- Generated column
    processing_time_category,    -- Generated column
    ip_address,
    created_at,
    event_data
FROM audit_logs 
WHERE user_id = (SELECT id FROM users WHERE username = 'test_user_1')
ORDER BY created_at DESC
LIMIT 10;
```

#### 7. Test Advanced JSONB Queries

```sql
-- Search users by email using GIN index
SELECT username, profile_data->>'email' as email
FROM users 
WHERE profile_data @> '{"email": "test@example.com"}';

-- Complex risk factor analysis
SELECT 
    username,
    risk_profile->>'score' as risk_score,
    jsonb_array_elements_text(risk_profile->'factors') as risk_factor
FROM users 
WHERE (risk_profile->>'score')::numeric > 0.1;

-- Search audit logs for security events
SELECT 
    event_type,
    event_data->>'reason' as reason,
    severity,
    created_at
FROM audit_logs 
WHERE event_data @> '{"event_classification": "SECURITY_INCIDENT"}'
ORDER BY created_at DESC;
```

#### 8. Test Performance with EXPLAIN

```sql
-- Analyze query performance
EXPLAIN (ANALYZE, BUFFERS, FORMAT JSON) 
SELECT u.username, t.amount_in, t.transaction_category
FROM users u
JOIN transactions t ON u.id = t.user_id
WHERE u.risk_category = 'MEDIUM' 
  AND t.transaction_category = 'LARGE'
ORDER BY t.created_at DESC;
```

### Performance Testing

```sql
-- Check index usage
SELECT schemaname, tablename, attname, n_distinct, correlation
FROM pg_stats 
WHERE tablename IN ('users', 'transactions', 'audit_logs')
ORDER BY tablename, attname;

-- Monitor query performance
SELECT query, calls, total_time, mean_time, rows
FROM pg_stat_statements 
WHERE query LIKE '%users%' OR query LIKE '%transactions%'
ORDER BY total_time DESC
LIMIT 10;
```

## Database Configuration

### Environment Variables

```env
DATABASE_URL=postgresql://postgres:dev_password@postgres:5432/kembridge_dev
POSTGRES_DB=kembridge_dev
POSTGRES_USER=postgres
POSTGRES_PASSWORD=dev_password
```

### Advanced PostgreSQL 18 Configuration

The database is configured with:

- **Enhanced I/O**: io_uring for async operations
- **FIPS Compliance**: Ready for production security requirements
- **Extended Statistics**: Optimized for complex analytical queries
- **Partitioning**: Automated monthly partitions for audit logs
- **Connection Pooling**: Optimized for high-concurrency applications

## Migration Files

| File | Description |
|------|-------------|
| `001_postgresql18_extensions_config.sql` | Extensions, domains, and core configuration |
| `002_users_table_postgresql18.sql` | Users table with profile and risk analytics |
| `003_auth_methods_oauth_postgresql18.sql` | Web3 and OAuth authentication methods |
| `004_user_sessions_advanced_postgresql18.sql` | JWT session management with security monitoring |
| `005_quantum_keys_postgresql18.sql` | Post-quantum cryptographic key storage |
| `006_transactions_advanced_postgresql18.sql` | Cross-chain transaction management |
| `007_audit_logs_comprehensive_postgresql18.sql` | Comprehensive audit logging with partitioning |

## Troubleshooting

### Common Issues

1. **Connection refused**: Ensure PostgreSQL container is running
2. **Permission denied**: Check database credentials in docker-compose.yml
3. **Migration errors**: Run migrations manually in correct order
4. **Performance issues**: Check `pg_stat_statements` for slow queries

### Health Checks

```bash
# Check container health
docker-compose exec postgres pg_isready -U postgres

# Check database tables
docker-compose exec postgres psql -U postgres -d kembridge_dev -c "\dt"

# Check database size
docker-compose exec postgres psql -U postgres -d kembridge_dev -c "
SELECT 
    schemaname,
    tablename,
    pg_size_pretty(pg_total_relation_size(schemaname||'.'||tablename)) as size
FROM pg_tables 
WHERE schemaname = 'public'
ORDER BY pg_total_relation_size(schemaname||'.'||tablename) DESC;"
```

## Security Considerations

- All sensitive data is encrypted at rest
- Audit logs track every database operation
- Risk scoring prevents malicious transactions
- OAuth tokens are hashed with SHA-256
- Quantum keys support HSM integration
- Row-level security policies implemented where applicable

This database schema provides a solid foundation for KEMBridge's quantum-secure cross-chain bridge with enterprise-grade security, performance, and monitoring capabilities.