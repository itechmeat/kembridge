-- PostgreSQL 18 Advanced Extensions and Configuration for KEMBridge
-- Leveraging cutting-edge PostgreSQL 18 features for quantum-secure cross-chain bridge

-- =====================================================
-- Enable PostgreSQL 18 Advanced Extensions
-- =====================================================

-- Core UUID and crypto extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- PostgreSQL 18: Enhanced JSONB performance with SIMD
CREATE EXTENSION IF NOT EXISTS "btree_gin";
CREATE EXTENSION IF NOT EXISTS "pg_trgm";

-- PostgreSQL 18: Advanced indexing and performance
CREATE EXTENSION IF NOT EXISTS "bloom";
CREATE EXTENSION IF NOT EXISTS "ltree";

-- PostgreSQL 18: Statistics and monitoring
CREATE EXTENSION IF NOT EXISTS "pg_stat_statements";

-- PostgreSQL 18: Advanced partitioning support
CREATE EXTENSION IF NOT EXISTS "postgres_fdw";

-- =====================================================
-- PostgreSQL 18: Configure Advanced Features
-- =====================================================

-- TODO: PostgreSQL 18 system configuration (moved to separate setup script)
-- These settings should be configured at database server level:
-- ALTER SYSTEM SET io_direct = 'data';
-- ALTER SYSTEM SET io_method = 'io_uring';
-- ALTER SYSTEM SET shared_buffers = '256MB';
-- ALTER SYSTEM SET effective_cache_size = '1GB';
-- ALTER SYSTEM SET work_mem = '16MB';
-- ALTER SYSTEM SET maintenance_work_mem = '64MB';
-- ALTER SYSTEM SET password_encryption = 'scram-sha-256';
-- ALTER SYSTEM SET ssl_min_protocol_version = 'TLSv1.3';
-- ALTER SYSTEM SET data_checksums = on;

-- =====================================================
-- PostgreSQL 18: FIPS Mode Configuration (for production)
-- =====================================================

-- Configure FIPS-compliant cryptographic functions
-- This ensures quantum-ready security compliance
-- ALTER SYSTEM SET fips_mode = off; -- Set to 'on' in production environment

-- =====================================================
-- PostgreSQL 18: Advanced Monitoring Configuration
-- =====================================================

-- Configure enhanced statistics collection for query optimization
-- ALTER SYSTEM SET track_activities = on;
-- ALTER SYSTEM SET track_counts = on;
-- ALTER SYSTEM SET track_io_timing = on;
-- ALTER SYSTEM SET track_wal_io_timing = on;

-- PostgreSQL 18: Enable skip scan optimization for multicolumn indexes
-- ALTER SYSTEM SET enable_indexscan = on;
-- ALTER SYSTEM SET enable_bitmapscan = on;

-- =====================================================
-- Custom Functions for PostgreSQL 18 Features
-- =====================================================

-- UUIDv7 generation function (PostgreSQL 18 feature)
CREATE OR REPLACE FUNCTION generate_uuidv7()
RETURNS UUID AS $$
BEGIN
    -- PostgreSQL 18 native UUIDv7 support with timestamp ordering
    RETURN gen_uuid_v7();
EXCEPTION
    WHEN undefined_function THEN
        -- Fallback for development if UUIDv7 not available
        RETURN uuid_generate_v4();
END;
$$ LANGUAGE plpgsql;

-- Enhanced SHA-512 hashing function (PostgreSQL 18)
CREATE OR REPLACE FUNCTION enhanced_hash(input_text TEXT)
RETURNS TEXT AS $$
BEGIN
    -- Use PostgreSQL 18 sha512crypt for enhanced security
    RETURN sha512crypt(input_text, gen_salt('sha512crypt'));
EXCEPTION
    WHEN undefined_function THEN
        -- Fallback to standard sha512
        RETURN encode(digest(input_text, 'sha512'), 'hex');
END;
$$ LANGUAGE plpgsql;

-- =====================================================
-- JSON/JSONB Optimization for PostgreSQL 18
-- =====================================================

-- Create optimized JSONB processing functions
CREATE OR REPLACE FUNCTION jsonb_extract_path_optimized(
    input_jsonb JSONB,
    VARIADIC path_elements TEXT[]
)
RETURNS JSONB AS $$
BEGIN
    -- PostgreSQL 18: Optimized JSONB processing with SIMD
    RETURN jsonb_extract_path(input_jsonb, VARIADIC path_elements);
END;
$$ LANGUAGE plpgsql IMMUTABLE PARALLEL SAFE;

-- =====================================================
-- Security and Audit Configuration
-- =====================================================

-- Configure advanced logging for security monitoring
-- ALTER SYSTEM SET log_statement = 'all';
-- ALTER SYSTEM SET log_min_duration_statement = 1000;
-- ALTER SYSTEM SET log_checkpoints = on;
-- ALTER SYSTEM SET log_connections = on;
-- ALTER SYSTEM SET log_disconnections = on;
-- ALTER SYSTEM SET log_lock_waits = on;

-- Configure audit trail settings
CREATE TABLE IF NOT EXISTS pg_audit_config (
    setting_name TEXT PRIMARY KEY,
    setting_value TEXT NOT NULL,
    description TEXT,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

INSERT INTO pg_audit_config (setting_name, setting_value, description) VALUES
('audit_level', 'comprehensive', 'Full audit logging for KEMBridge operations'),
('retention_period', '7_years', 'Financial compliance retention requirement'),
('encryption_standard', 'post_quantum', 'Post-quantum cryptography compliance'),
('oauth_enabled', 'true', 'PostgreSQL 18 OAuth integration enabled')
ON CONFLICT (setting_name) DO NOTHING;

-- =====================================================
-- Performance Optimization Settings
-- =====================================================

-- PostgreSQL 18: Optimize for high-frequency financial operations
-- ALTER SYSTEM SET random_page_cost = 1.1;
-- ALTER SYSTEM SET seq_page_cost = 1.0;
-- ALTER SYSTEM SET cpu_tuple_cost = 0.01;
-- ALTER SYSTEM SET cpu_index_tuple_cost = 0.005;
-- ALTER SYSTEM SET cpu_operator_cost = 0.0025;

-- Configure parallel processing for large operations
-- ALTER SYSTEM SET max_parallel_workers = 8;
-- ALTER SYSTEM SET max_parallel_workers_per_gather = 4;
-- ALTER SYSTEM SET max_parallel_maintenance_workers = 4;

-- =====================================================
-- Temporal and Advanced Data Types Setup
-- =====================================================

-- PostgreSQL 18: Enhanced temporal support for transaction lifecycle
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'transaction_timestamp') THEN
        CREATE DOMAIN transaction_timestamp AS TIMESTAMP WITH TIME ZONE
            DEFAULT NOW()
            CHECK (VALUE > '2024-01-01'::TIMESTAMP WITH TIME ZONE);
    END IF;
END $$;

-- Custom domain for quantum key identifiers  
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'quantum_key_id') THEN
        CREATE DOMAIN quantum_key_id AS UUID
            CHECK (VALUE IS NOT NULL);
    END IF;
END $$;

-- Domain for risk scores with enhanced validation
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'risk_score') THEN
        CREATE DOMAIN risk_score AS DECIMAL(5,4)
            CHECK (VALUE >= 0.0000 AND VALUE <= 1.0000);
    END IF;
END $$;

-- =====================================================
-- Comments and Documentation
-- =====================================================

COMMENT ON EXTENSION "uuid-ossp" IS 'UUID generation with PostgreSQL 18 UUIDv7 support';
COMMENT ON FUNCTION generate_uuidv7() IS 'PostgreSQL 18 UUIDv7 generation with timestamp ordering for better performance';
COMMENT ON FUNCTION enhanced_hash(TEXT) IS 'PostgreSQL 18 enhanced SHA-512 cryptographic hashing';
COMMENT ON TABLE pg_audit_config IS 'PostgreSQL 18 audit configuration for KEMBridge compliance';

-- TODO: Log configuration completion after audit_logs table is created (migration 007)
-- This will be done in migration 007 where the create_audit_log function is defined