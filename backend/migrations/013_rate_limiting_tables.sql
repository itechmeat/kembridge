-- 011_rate_limiting_simple.sql - Simple rate limiting table for testing
-- Created for H7: Rate Limiting Implementation

-- Rate limit violations tracking table
CREATE TABLE IF NOT EXISTS rate_limit_violations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    -- Rate limiting context
    endpoint_class TEXT NOT NULL,
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    ip_address INET NOT NULL,
    
    -- Violation details
    limit_exceeded INTEGER NOT NULL,
    current_requests INTEGER NOT NULL,
    violation_count BIGINT NOT NULL DEFAULT 1,
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_violation TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Test a function
CREATE OR REPLACE FUNCTION test_function() RETURNS INTEGER AS $$
BEGIN
    RETURN 42;
END;
$$ LANGUAGE plpgsql;