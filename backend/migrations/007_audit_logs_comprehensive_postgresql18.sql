-- Simple audit logs for Phase 1.3 Basic API Gateway
-- This will be replaced with comprehensive version in Phase 2.x

CREATE TABLE audit_logs (
    -- Primary key
    id UUID PRIMARY KEY DEFAULT generate_uuidv7(),
    
    -- Event information
    event_type VARCHAR(100) NOT NULL,
    event_category VARCHAR(50) NOT NULL DEFAULT 'system',
    event_data JSONB DEFAULT '{}',
    
    -- Context
    user_id UUID,
    session_id UUID,
    ip_address INET,
    user_agent TEXT,
    
    -- Timestamps
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    
    -- Severity
    severity VARCHAR(20) DEFAULT 'info' CHECK (
        severity IN ('debug', 'info', 'warning', 'error', 'critical')
    ),
    
    -- Source
    data_source VARCHAR(50) DEFAULT 'application'
);

-- Basic indexes for Phase 1.3
CREATE INDEX idx_audit_logs_event_category ON audit_logs(event_category);
CREATE INDEX idx_audit_logs_created_at ON audit_logs(created_at);
CREATE INDEX idx_audit_logs_severity ON audit_logs(severity);
CREATE INDEX idx_audit_logs_user_id ON audit_logs(user_id) WHERE user_id IS NOT NULL;

-- Simple audit log creation function for Phase 1.3
CREATE OR REPLACE FUNCTION create_audit_log(
    p_user_id UUID,
    p_session_id UUID,
    p_quantum_key_id UUID,
    p_event_type VARCHAR,
    p_event_category VARCHAR,
    p_event_data JSONB,
    p_ip_address INET,
    p_user_agent TEXT,
    p_api_endpoint VARCHAR,
    p_response_status INTEGER,
    p_response_time_ms INTEGER,
    p_severity VARCHAR,
    p_is_sensitive BOOLEAN,
    p_transaction_id UUID,
    p_data_source VARCHAR
)
RETURNS UUID AS $$
DECLARE
    log_id UUID;
BEGIN
    INSERT INTO audit_logs (
        user_id, session_id, event_type, event_category, event_data,
        ip_address, user_agent, severity, data_source, created_at
    ) VALUES (
        p_user_id, p_session_id, p_event_type, 
        COALESCE(p_event_category, 'system'),
        COALESCE(p_event_data, '{}'),
        p_ip_address, p_user_agent,
        COALESCE(p_severity, 'info'),
        COALESCE(p_data_source, 'application'),
        NOW()
    ) RETURNING id INTO log_id;
    
    RETURN log_id;
END;
$$ LANGUAGE plpgsql;

-- Comments
COMMENT ON TABLE audit_logs IS 'Simple audit logging for Phase 1.3 - will be enhanced in Phase 2.x';
COMMENT ON FUNCTION create_audit_log IS 'Simple audit log creation function for Phase 1.3';