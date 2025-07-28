-- 012_rate_limiting_complete.sql - Complete rate limiting system
-- Created for H7: Rate Limiting Implementation

-- Add missing indexes and constraints to existing table
CREATE INDEX IF NOT EXISTS idx_rate_limit_violations_endpoint_created 
    ON rate_limit_violations(endpoint_class, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_rate_limit_violations_user_id_created 
    ON rate_limit_violations(user_id, created_at DESC) 
    WHERE user_id IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_rate_limit_violations_ip_created 
    ON rate_limit_violations(ip_address, created_at DESC);

-- Add unique constraint (simplified version without COALESCE)
-- We'll handle this in the application logic instead

-- Rate limiting hourly statistics table
CREATE TABLE IF NOT EXISTS rate_limit_hourly_stats (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    -- Time bucket (truncated to hour)
    hour_bucket TIMESTAMPTZ NOT NULL,
    endpoint_class TEXT NOT NULL,
    
    -- Aggregated statistics
    total_requests BIGINT NOT NULL DEFAULT 0,
    blocked_requests BIGINT NOT NULL DEFAULT 0,
    unique_ips INTEGER NOT NULL DEFAULT 0,
    unique_users INTEGER NOT NULL DEFAULT 0,
    
    -- Performance metrics
    avg_requests_per_ip DECIMAL(10,2),
    max_requests_per_ip INTEGER,
    avg_requests_per_user DECIMAL(10,2),
    max_requests_per_user INTEGER,
    
    -- Calculated fields
    violation_rate DECIMAL(5,4) GENERATED ALWAYS AS (
        CASE 
            WHEN total_requests > 0 THEN blocked_requests::decimal / total_requests::decimal
            ELSE 0 
        END
    ) STORED,
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Unique constraint to prevent duplicates
    UNIQUE(hour_bucket, endpoint_class)
);

-- Rate limiting configuration table
CREATE TABLE IF NOT EXISTS rate_limit_config (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    -- Configuration context
    endpoint_class TEXT NOT NULL,
    user_tier TEXT NOT NULL DEFAULT 'free',
    
    -- Rate limits
    request_limit INTEGER NOT NULL,
    window_seconds INTEGER NOT NULL,
    
    -- Configuration metadata
    description TEXT,
    is_active BOOLEAN NOT NULL DEFAULT true,
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Constraints
    CHECK (request_limit > 0),
    CHECK (window_seconds > 0),
    CHECK (user_tier IN ('free', 'premium', 'admin')),
    
    -- Unique constraint for endpoint_class + user_tier combination
    UNIQUE(endpoint_class, user_tier)
);

-- Default rate limiting configurations
INSERT INTO rate_limit_config (endpoint_class, user_tier, request_limit, window_seconds, description) VALUES
    ('health', 'free', 1000, 60, 'Health check endpoints - high limit'),
    ('auth', 'free', 10, 60, 'Authentication endpoints - prevent brute force'),
    ('auth', 'premium', 20, 60, 'Authentication endpoints - premium users'),
    ('bridge', 'free', 5, 300, 'Bridge operations - strict for unauthenticated'),
    ('bridge', 'premium', 50, 60, 'Bridge operations - standard users'),
    ('bridge', 'admin', 200, 60, 'Bridge operations - premium users'),
    ('quantum', 'free', 2, 60, 'Quantum operations - basic limit'),
    ('quantum', 'premium', 20, 60, 'Quantum operations - authenticated users'),
    ('quantum', 'admin', 100, 60, 'Quantum operations - premium users'),
    ('user', 'premium', 60, 60, 'User management - standard'),
    ('user', 'admin', 120, 60, 'User management - premium'),
    ('admin', 'admin', 30, 60, 'Admin endpoints - admin only'),
    ('docs', 'free', 200, 60, 'Documentation - high limit'),
    ('websocket', 'free', 10, 300, 'WebSocket connections - connection-based'),
    ('general', 'free', 30, 60, 'General endpoints - unauthenticated'),
    ('general', 'premium', 100, 60, 'General endpoints - authenticated'),
    ('general', 'admin', 500, 60, 'General endpoints - premium')
ON CONFLICT (endpoint_class, user_tier) DO NOTHING;