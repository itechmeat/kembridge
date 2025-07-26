-- User sessions with PostgreSQL 18 advanced session management
-- Enhanced JWT management with OAuth integration and security features

CREATE TABLE user_sessions (
    -- PostgreSQL 18: UUIDv7 for session ID with timestamp ordering
    id UUID PRIMARY KEY DEFAULT generate_uuidv7(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    auth_method_id UUID NOT NULL REFERENCES user_auth_methods(id) ON DELETE CASCADE,
    
    -- JWT token management with PostgreSQL 18 enhanced security
    jwt_token_hash VARCHAR(255) NOT NULL,
    jwt_token_version INTEGER DEFAULT 1,
    
    -- PostgreSQL 18: OAuth refresh token support
    oauth_refresh_token_hash VARCHAR(255),
    oauth_access_token_hash VARCHAR(255),
    
    -- Session metadata with PostgreSQL 18 JSONB optimization
    session_data JSONB DEFAULT '{}',
    
    -- Security context and forensics
    ip_address INET,
    user_agent TEXT,
    device_fingerprint VARCHAR(255),
    geolocation JSONB,
    
    -- PostgreSQL 18: Temporal session lifecycle
    created_at transaction_timestamp,
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    last_activity_at transaction_timestamp,
    
    -- Session status and security
    is_active BOOLEAN DEFAULT true,
    revoked_at TIMESTAMP WITH TIME ZONE,
    revoked_reason VARCHAR(255),
    
    -- PostgreSQL 18: Security classification
    security_level VARCHAR(20) DEFAULT 'standard',
    risk_flags JSONB DEFAULT '[]',
    
    -- PostgreSQL 18: Virtual generated columns for session analysis
    session_duration_minutes INTEGER GENERATED ALWAYS AS (
        CASE 
            WHEN is_active = false AND revoked_at IS NOT NULL THEN
                EXTRACT(EPOCH FROM (revoked_at - created_at)) / 60
            WHEN is_active = true THEN
                EXTRACT(EPOCH FROM (last_activity_at - created_at)) / 60
            ELSE 0
        END
    ) STORED,
    
    session_status VARCHAR(20) GENERATED ALWAYS AS (
        CASE 
            WHEN NOT is_active AND revoked_reason IS NOT NULL THEN 'REVOKED'
            WHEN is_active THEN 'ACTIVE'
            ELSE 'UNKNOWN'
        END
    ) STORED,
    
    -- Risk assessment based on flags
    risk_level VARCHAR(10) GENERATED ALWAYS AS (
        CASE 
            WHEN jsonb_array_length(risk_flags) > 3 THEN 'HIGH'
            WHEN jsonb_array_length(risk_flags) > 1 THEN 'MEDIUM'
            ELSE 'LOW'
        END
    ) STORED,
    
    -- =====================================================
    -- PostgreSQL 18: Enhanced Validation Constraints
    -- =====================================================
    
    -- Session lifecycle validation
    CONSTRAINT sessions_lifecycle_valid CHECK (
        expires_at > created_at AND
        (revoked_at IS NULL OR revoked_at >= created_at) AND
        last_activity_at >= created_at
    ),
    
    -- JWT token hash validation (SHA-256)
    CONSTRAINT sessions_jwt_token_format CHECK (
        char_length(jwt_token_hash) = 64 AND
        jwt_token_hash ~ '^[a-fA-F0-9]+$'
    ),
    
    -- OAuth token hash validation
    CONSTRAINT sessions_oauth_tokens_format CHECK (
        (oauth_refresh_token_hash IS NULL OR 
         (char_length(oauth_refresh_token_hash) = 64 AND oauth_refresh_token_hash ~ '^[a-fA-F0-9]+$')) AND
        (oauth_access_token_hash IS NULL OR 
         (char_length(oauth_access_token_hash) = 64 AND oauth_access_token_hash ~ '^[a-fA-F0-9]+$'))
    ),
    
    -- Security level validation
    CONSTRAINT sessions_security_level_valid CHECK (
        security_level IN ('basic', 'standard', 'high', 'enterprise', 'quantum')
    ),
    
    -- PostgreSQL 18: Enhanced JSONB validation
    CONSTRAINT sessions_data_structure CHECK (
        jsonb_typeof(session_data) = 'object'
    ),
    
    CONSTRAINT sessions_geolocation_structure CHECK (
        geolocation IS NULL OR (
            jsonb_typeof(geolocation) = 'object' AND
            geolocation ? 'country' AND
            (NOT geolocation ? 'latitude' OR 
             (geolocation->>'latitude')::NUMERIC BETWEEN -90 AND 90) AND
            (NOT geolocation ? 'longitude' OR 
             (geolocation->>'longitude')::NUMERIC BETWEEN -180 AND 180)
        )
    ),
    
    CONSTRAINT sessions_risk_flags_array CHECK (
        jsonb_typeof(risk_flags) = 'array'
    ),
    
    -- Unique JWT token constraint
    CONSTRAINT sessions_unique_jwt_token UNIQUE (jwt_token_hash),
    CONSTRAINT sessions_unique_oauth_refresh UNIQUE (oauth_refresh_token_hash)
);

-- =====================================================
-- PostgreSQL 18: Advanced Indexing with Skip Scan
-- =====================================================

-- Primary session lookup with skip scan optimization
CREATE INDEX idx_sessions_user_active_skip ON user_sessions(user_id, is_active, expires_at);

-- JWT token lookup (most frequent operation)
CREATE INDEX idx_sessions_jwt_lookup ON user_sessions(jwt_token_hash) 
    WHERE is_active = true;

-- OAuth token management
CREATE INDEX idx_sessions_oauth_refresh ON user_sessions(oauth_refresh_token_hash)
    WHERE oauth_refresh_token_hash IS NOT NULL AND is_active = true;

-- PostgreSQL 18: Session security monitoring
CREATE INDEX idx_sessions_security_monitoring ON user_sessions(security_level, risk_level, created_at)
    WHERE risk_level IN ('HIGH', 'MEDIUM');

-- Session cleanup and maintenance
CREATE INDEX idx_sessions_cleanup_expired ON user_sessions(expires_at, is_active)
    WHERE is_active = true;

CREATE INDEX idx_sessions_cleanup_idle ON user_sessions(last_activity_at, is_active)
    WHERE is_active = true;

-- Forensics and security analysis
CREATE INDEX idx_sessions_forensics_ip ON user_sessions(ip_address, created_at, user_id);
CREATE INDEX idx_sessions_forensics_device ON user_sessions(device_fingerprint, user_id)
    WHERE device_fingerprint IS NOT NULL;

-- PostgreSQL 18: Enhanced JSONB indexes
CREATE INDEX idx_sessions_data_gin ON user_sessions USING GIN (session_data);
CREATE INDEX idx_sessions_geolocation_gin ON user_sessions USING GIN (geolocation);
CREATE INDEX idx_sessions_risk_flags_gin ON user_sessions USING GIN (risk_flags);

-- Virtual column indexes for analysis
CREATE INDEX idx_sessions_duration_analysis ON user_sessions(session_duration_minutes, security_level)
    WHERE session_duration_minutes > 60;

CREATE INDEX idx_sessions_status_monitoring ON user_sessions(session_status, risk_level, created_at);

-- =====================================================
-- PostgreSQL 18: Advanced Session Management Functions
-- =====================================================

-- Function to create secure session with comprehensive validation
CREATE OR REPLACE FUNCTION create_secure_session(
    p_user_id UUID,
    p_auth_method_id UUID,
    p_jwt_token_hash VARCHAR(255),
    p_session_duration_hours INTEGER DEFAULT 24,
    p_ip_address INET DEFAULT NULL,
    p_user_agent TEXT DEFAULT NULL,
    p_device_fingerprint VARCHAR(255) DEFAULT NULL,
    p_geolocation JSONB DEFAULT NULL,
    p_security_level VARCHAR(20) DEFAULT 'standard'
)
RETURNS UUID AS $$
DECLARE
    session_id UUID;
    session_expires_at TIMESTAMP WITH TIME ZONE;
    suspicious_activity BOOLEAN := false;
    risk_flags_array JSONB := '[]';
BEGIN
    -- Calculate session expiration
    session_expires_at := NOW() + (p_session_duration_hours || ' hours')::INTERVAL;
    
    -- Analyze for suspicious activity
    IF p_ip_address IS NOT NULL THEN
        -- Check for multiple recent sessions from same IP
        IF EXISTS(
            SELECT 1 FROM user_sessions 
            WHERE ip_address = p_ip_address 
              AND created_at > NOW() - INTERVAL '1 hour'
              AND user_id != p_user_id
            LIMIT 3
        ) THEN
            risk_flags_array := risk_flags_array || '"multiple_users_same_ip"';
            suspicious_activity := true;
        END IF;
    END IF;
    
    -- Check for device fingerprint anomalies
    IF p_device_fingerprint IS NOT NULL THEN
        IF NOT EXISTS(
            SELECT 1 FROM user_sessions 
            WHERE user_id = p_user_id 
              AND device_fingerprint = p_device_fingerprint
              AND created_at > NOW() - INTERVAL '30 days'
        ) THEN
            risk_flags_array := risk_flags_array || '"new_device"';
        END IF;
    END IF;
    
    -- Check geolocation anomalies
    IF p_geolocation IS NOT NULL AND p_geolocation ? 'country' THEN
        IF NOT EXISTS(
            SELECT 1 FROM user_sessions 
            WHERE user_id = p_user_id 
              AND geolocation->>'country' = p_geolocation->>'country'
              AND created_at > NOW() - INTERVAL '90 days'
        ) THEN
            risk_flags_array := risk_flags_array || '"new_location"';
        END IF;
    END IF;
    
    -- Create session with risk assessment
    INSERT INTO user_sessions (
        user_id, auth_method_id, jwt_token_hash,
        expires_at, ip_address, user_agent, device_fingerprint,
        geolocation, security_level, risk_flags,
        session_data
    ) VALUES (
        p_user_id, p_auth_method_id, p_jwt_token_hash,
        session_expires_at, p_ip_address, p_user_agent, p_device_fingerprint,
        p_geolocation, p_security_level, risk_flags_array,
        jsonb_build_object(
            'creation_method', 'secure_function',
            'suspicious_activity', suspicious_activity,
            'browser_features', COALESCE(p_user_agent, 'unknown')
        )
    ) RETURNING id INTO session_id;
    
    -- Log session creation with risk assessment
    INSERT INTO audit_logs (
        user_id, session_id, event_type, event_category,
        event_data, ip_address, user_agent, severity, is_sensitive
    ) VALUES (
        p_user_id, session_id, 'session_created', 'auth',
        jsonb_build_object(
            'session_id', session_id,
            'security_level', p_security_level,
            'duration_hours', p_session_duration_hours,
            'risk_flags', risk_flags_array,
            'suspicious_activity', suspicious_activity
        ),
        p_ip_address, p_user_agent,
        CASE WHEN suspicious_activity THEN 'warning' ELSE 'info' END,
        true
    );
    
    RETURN session_id;
END;
$$ LANGUAGE plpgsql SECURITY DEFINER;

-- Function to revoke session with comprehensive audit
CREATE OR REPLACE FUNCTION revoke_session(
    p_session_id UUID,
    p_reason VARCHAR(255) DEFAULT 'manual_revocation',
    p_revoked_by_user_id UUID DEFAULT NULL
)
RETURNS BOOLEAN AS $$
DECLARE
    session_user_id UUID;
    session_exists BOOLEAN;
BEGIN
    -- Get session information
    SELECT user_id, is_active
    INTO session_user_id, session_exists
    FROM user_sessions 
    WHERE id = p_session_id;
    
    IF NOT FOUND OR NOT session_exists THEN
        RETURN false;
    END IF;
    
    -- Revoke the session
    UPDATE user_sessions 
    SET 
        is_active = false,
        revoked_at = NOW(),
        revoked_reason = p_reason
    WHERE id = p_session_id;
    
    -- Log session revocation
    INSERT INTO audit_logs (
        user_id, session_id, event_type, event_category,
        event_data, severity, is_sensitive
    ) VALUES (
        session_user_id, p_session_id, 'session_revoked', 'security',
        jsonb_build_object(
            'reason', p_reason,
            'revoked_by_user_id', p_revoked_by_user_id,
            'session_id', p_session_id
        ),
        'warning', true
    );
    
    RETURN true;
END;
$$ LANGUAGE plpgsql SECURITY DEFINER;

-- PostgreSQL 18: Automated cleanup function
CREATE OR REPLACE FUNCTION cleanup_expired_sessions()
RETURNS TABLE(cleaned_count INTEGER, idle_count INTEGER) AS $$
DECLARE
    expired_count INTEGER;
    idle_session_count INTEGER;
BEGIN
    -- Clean up expired sessions
    UPDATE user_sessions 
    SET 
        is_active = false,
        revoked_at = NOW(),
        revoked_reason = 'automatic_expiry'
    WHERE expires_at < NOW() AND is_active = true;
    
    GET DIAGNOSTICS expired_count = ROW_COUNT;
    
    -- Clean up idle sessions (inactive for over 2 hours)
    UPDATE user_sessions 
    SET 
        is_active = false,
        revoked_at = NOW(),
        revoked_reason = 'idle_timeout'
    WHERE last_activity_at < NOW() - INTERVAL '2 hours' 
      AND is_active = true;
    
    GET DIAGNOSTICS idle_session_count = ROW_COUNT;
    
    -- Log cleanup activity
    IF expired_count > 0 OR idle_session_count > 0 THEN
        INSERT INTO audit_logs (
            event_type, event_category, event_data, severity
        ) VALUES (
            'session_cleanup',
            'system',
            jsonb_build_object(
                'expired_sessions', expired_count,
                'idle_sessions', idle_session_count,
                'cleanup_timestamp', NOW()
            ),
            'info'
        );
    END IF;
    
    RETURN QUERY SELECT expired_count, idle_session_count;
END;
$$ LANGUAGE plpgsql;

-- =====================================================
-- PostgreSQL 18: Enhanced Triggers
-- =====================================================

-- Function to update session activity
CREATE OR REPLACE FUNCTION update_session_activity()
RETURNS TRIGGER AS $$
BEGIN
    NEW.last_activity_at = NOW();
    
    -- Detect suspicious activity patterns
    IF OLD.ip_address IS DISTINCT FROM NEW.ip_address AND NEW.ip_address IS NOT NULL THEN
        -- IP address changed during session - potential security issue
        NEW.risk_flags = NEW.risk_flags || '"ip_address_changed"';
        
        INSERT INTO audit_logs (
            user_id, session_id, event_type, event_category,
            event_data, severity, is_sensitive
        ) VALUES (
            NEW.user_id, NEW.id, 'session_ip_changed', 'security',
            jsonb_build_object(
                'old_ip', OLD.ip_address,
                'new_ip', NEW.ip_address,
                'session_id', NEW.id
            ),
            'warning', true
        );
    END IF;
    
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Trigger for session activity tracking
CREATE TRIGGER sessions_update_activity 
    BEFORE UPDATE ON user_sessions 
    FOR EACH ROW 
    EXECUTE FUNCTION update_session_activity();

-- =====================================================
-- PostgreSQL 18: Statistics and Performance Optimization
-- =====================================================

-- Configure enhanced statistics
ALTER TABLE user_sessions ALTER COLUMN security_level SET STATISTICS 1000;
ALTER TABLE user_sessions ALTER COLUMN risk_flags SET STATISTICS 500;
ALTER TABLE user_sessions ALTER COLUMN ip_address SET STATISTICS 1000;

-- Extended statistics for correlated columns
CREATE STATISTICS sessions_security_correlation (dependencies, ndistinct)
    ON security_level, risk_level, session_status FROM user_sessions;

CREATE STATISTICS sessions_activity_correlation (dependencies)
    ON last_activity_at, session_duration_minutes, is_active FROM user_sessions;

-- =====================================================
-- PostgreSQL 18: Table Configuration
-- =====================================================

ALTER TABLE user_sessions SET (
    autovacuum_vacuum_scale_factor = 0.05,
    autovacuum_analyze_scale_factor = 0.02,
    autovacuum_vacuum_cost_delay = 5,
    fillfactor = 80  -- Higher update frequency
);

-- Comments for documentation
COMMENT ON TABLE user_sessions IS 'Advanced session management with PostgreSQL 18 features, OAuth integration, and security monitoring';
COMMENT ON COLUMN user_sessions.session_duration_minutes IS 'Virtual column calculating session duration in minutes';
COMMENT ON COLUMN user_sessions.session_status IS 'Virtual column showing current session status';
COMMENT ON COLUMN user_sessions.risk_level IS 'Virtual column assessing session risk based on flags';
COMMENT ON FUNCTION create_secure_session(UUID, UUID, VARCHAR, INTEGER, INET, TEXT, VARCHAR, JSONB, VARCHAR) IS 'Creates secure session with comprehensive risk assessment and audit logging';
COMMENT ON FUNCTION cleanup_expired_sessions() IS 'PostgreSQL 18 automated session cleanup with detailed reporting';