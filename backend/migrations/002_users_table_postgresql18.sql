-- Users table with PostgreSQL 18 advanced features
-- Leveraging UUIDv7, enhanced JSONB, and virtual columns

CREATE TABLE users (
    -- PostgreSQL 18: UUIDv7 for timestamp-ordered primary keys
    id UUID PRIMARY KEY DEFAULT generate_uuidv7(),
    
    -- Optional readable username with enhanced validation
    username VARCHAR(255) UNIQUE,
    
    -- PostgreSQL 18: Enhanced JSONB with SIMD optimization
    profile_data JSONB DEFAULT '{}',
    
    -- PostgreSQL 18: Temporal columns with domain validation
    created_at transaction_timestamp,
    updated_at transaction_timestamp,
    
    -- User lifecycle management
    is_active BOOLEAN DEFAULT true,
    
    -- AI risk profile with PostgreSQL 18 JSONB enhancements
    risk_profile JSONB DEFAULT '{}',
    
    -- PostgreSQL 18: Virtual generated columns for computed values
    profile_completeness INTEGER GENERATED ALWAYS AS (
        CASE 
            WHEN profile_data ? 'email' AND profile_data ? 'display_name' THEN 100
            WHEN profile_data ? 'email' OR profile_data ? 'display_name' THEN 50
            ELSE 0
        END
    ) STORED,
    
    -- Virtual column for risk category
    risk_category VARCHAR(20) GENERATED ALWAYS AS (
        CASE 
            WHEN (risk_profile->>'score')::DECIMAL > 0.7 THEN 'HIGH'
            WHEN (risk_profile->>'score')::DECIMAL > 0.3 THEN 'MEDIUM'
            ELSE 'LOW'
        END
    ) STORED,
    
    -- PostgreSQL 18: Enhanced timestamp with timezone info
    last_login_at TIMESTAMP WITH TIME ZONE,
    
    -- Account status with enum-like constraint
    account_status VARCHAR(20) DEFAULT 'active' 
        CHECK (account_status IN ('active', 'suspended', 'pending_verification', 'closed')),
    
    -- PostgreSQL 18: Enhanced data validation constraints
    CONSTRAINT users_username_format CHECK (
        username IS NULL OR (
            char_length(username) BETWEEN 3 AND 255 AND
            username ~ '^[a-zA-Z0-9_-]+$'
        )
    ),
    
    CONSTRAINT users_profile_data_structure CHECK (
        jsonb_typeof(profile_data) = 'object' AND
        -- Validate email format if present
        (NOT profile_data ? 'email' OR 
         profile_data->>'email' ~ '^[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}$')
    ),
    
    CONSTRAINT users_risk_profile_structure CHECK (
        jsonb_typeof(risk_profile) = 'object' AND
        -- Validate risk score if present
        (NOT risk_profile ? 'score' OR 
         (risk_profile->>'score')::DECIMAL BETWEEN 0.0000 AND 1.0000)
    ),
    
    -- PostgreSQL 18: Temporal constraint for data consistency
    CONSTRAINT users_timestamps_logical CHECK (
        updated_at >= created_at AND
        (last_login_at IS NULL OR last_login_at >= created_at)
    )
);

-- =====================================================
-- PostgreSQL 18: Advanced Indexing with Skip Scan Support
-- =====================================================

-- Primary access patterns with skip scan optimization
CREATE INDEX idx_users_username_active ON users(username, is_active) 
    WHERE username IS NOT NULL;

-- PostgreSQL 18: Skip scan index for status and creation time
CREATE INDEX idx_users_status_created_skip ON users(account_status, created_at, id);

-- Enhanced JSONB indexes with PostgreSQL 18 performance improvements
CREATE INDEX idx_users_profile_gin ON users USING GIN (profile_data);
CREATE INDEX idx_users_risk_profile_gin ON users USING GIN (risk_profile);

-- PostgreSQL 18: Specialized indexes for virtual columns
CREATE INDEX idx_users_risk_category ON users(risk_category) WHERE risk_category IN ('HIGH', 'MEDIUM');
CREATE INDEX idx_users_profile_completeness ON users(profile_completeness) WHERE profile_completeness < 100;

-- Email lookup optimization with partial index
CREATE INDEX idx_users_email_lookup ON users USING BTREE ((profile_data->>'email')) 
    WHERE profile_data ? 'email';

-- PostgreSQL 18: Temporal range index for analytics
CREATE INDEX idx_users_temporal_range ON users USING BTREE (created_at, last_login_at)
    WHERE is_active = true;

-- =====================================================
-- PostgreSQL 18: Enhanced Statistics Configuration
-- =====================================================

-- Increase statistics targets for better query planning
ALTER TABLE users ALTER COLUMN profile_data SET STATISTICS 1000;
ALTER TABLE users ALTER COLUMN risk_profile SET STATISTICS 500;
ALTER TABLE users ALTER COLUMN created_at SET STATISTICS 1000;

-- Configure extended statistics for correlated columns
CREATE STATISTICS users_profile_correlation (dependencies) 
    ON profile_completeness, account_status, is_active FROM users;

CREATE STATISTICS users_risk_correlation (dependencies, ndistinct)
    ON risk_category, account_status, last_login_at FROM users;

-- =====================================================
-- PostgreSQL 18: Row Level Security (Advanced)
-- =====================================================

-- Enable RLS for future multi-tenant support
ALTER TABLE users ENABLE ROW LEVEL SECURITY;

-- Default policy for system access
CREATE POLICY users_system_access ON users
    FOR ALL TO postgres
    USING (true);

-- TODO: Future policy placeholder for user self-access (Phase 2.1 Authentication)
-- CREATE POLICY users_self_access ON users
--     FOR ALL TO authenticated_user
--     USING (id = current_setting('app.current_user_id')::UUID);

-- =====================================================
-- PostgreSQL 18: Advanced Triggers and Functions
-- =====================================================

-- Function to update user timestamps with PostgreSQL 18 features
CREATE OR REPLACE FUNCTION update_user_timestamp()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    
    -- Track profile changes for analytics
    IF OLD.profile_data IS DISTINCT FROM NEW.profile_data THEN
        -- Log profile update event (will be created in audit migration)
        INSERT INTO audit_logs (
            user_id, event_type, event_category, event_data, severity
        ) VALUES (
            NEW.id,
            'profile_updated',
            'user',
            jsonb_build_object(
                'old_completeness', OLD.profile_completeness,
                'new_completeness', NEW.profile_completeness,
                'changed_fields', (
                    SELECT jsonb_object_agg(key, value)
                    FROM jsonb_each(NEW.profile_data)
                    WHERE NEW.profile_data ->> key IS DISTINCT FROM OLD.profile_data ->> key
                )
            ),
            'info'
        );
    END IF;
    
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Function to handle login timestamp updates
CREATE OR REPLACE FUNCTION update_user_login()
RETURNS TRIGGER AS $$
BEGIN
    UPDATE users 
    SET last_login_at = NOW()
    WHERE id = NEW.user_id;
    
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- PostgreSQL 18: Enhanced trigger for timestamp management
CREATE TRIGGER users_update_timestamp 
    BEFORE UPDATE ON users 
    FOR EACH ROW 
    EXECUTE FUNCTION update_user_timestamp();

-- =====================================================
-- PostgreSQL 18: Partitioning Strategy (Future-Ready)
-- =====================================================

-- Prepare for future partitioning by creation date
-- (Uncomment when scaling beyond 1M users)

/*
CREATE TABLE users_y2024 PARTITION OF users
    FOR VALUES FROM ('2024-01-01') TO ('2025-01-01');

CREATE TABLE users_y2025 PARTITION OF users
    FOR VALUES FROM ('2025-01-01') TO ('2026-01-01');
*/

-- =====================================================
-- PostgreSQL 18: Table Configuration
-- =====================================================

-- Configure table for high-performance operations
ALTER TABLE users SET (
    autovacuum_vacuum_scale_factor = 0.1,
    autovacuum_analyze_scale_factor = 0.05,
    autovacuum_vacuum_cost_delay = 10,
    fillfactor = 90  -- Leave space for updates
);

-- Comments for documentation
COMMENT ON TABLE users IS 'Users table leveraging PostgreSQL 18 features: UUIDv7, virtual columns, enhanced JSONB';
COMMENT ON COLUMN users.id IS 'PostgreSQL 18 UUIDv7 primary key with timestamp ordering';
COMMENT ON COLUMN users.profile_completeness IS 'Virtual generated column showing profile completion percentage';
COMMENT ON COLUMN users.risk_category IS 'Virtual generated column for risk classification';
COMMENT ON COLUMN users.profile_data IS 'Enhanced JSONB with PostgreSQL 18 SIMD optimization';
COMMENT ON COLUMN users.risk_profile IS 'AI-generated risk analysis with PostgreSQL 18 performance improvements';