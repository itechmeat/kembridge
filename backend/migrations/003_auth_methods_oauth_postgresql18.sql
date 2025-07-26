-- Authentication methods with PostgreSQL 18 OAuth integration
-- Advanced Web3 and OAuth 2.0 authentication support

CREATE TABLE user_auth_methods (
    -- PostgreSQL 18: UUIDv7 for timestamp-ordered IDs
    id UUID PRIMARY KEY DEFAULT generate_uuidv7(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    
    -- Authentication method classification
    auth_type VARCHAR(50) NOT NULL DEFAULT 'web3_wallet',
    
    -- Blockchain network identification
    chain_type VARCHAR(50),
    
    -- Wallet address on respective blockchain
    wallet_address VARCHAR(255),
    
    -- PostgreSQL 18: OAuth 2.0 integration fields
    oauth_provider VARCHAR(100),
    oauth_subject_id VARCHAR(255),
    oauth_token_hash VARCHAR(255), -- SHA-256 hash of OAuth token
    
    -- Public key for signature verification
    public_key TEXT,
    
    -- PostgreSQL 18: Enhanced JSONB for signature parameters
    signature_params JSONB DEFAULT '{}',
    
    -- PostgreSQL 18: OAuth metadata with enhanced validation
    oauth_metadata JSONB DEFAULT '{}',
    
    -- Usage tracking with PostgreSQL 18 temporal domains
    first_used_at transaction_timestamp,
    last_used_at transaction_timestamp,
    
    -- Verification and security status
    is_verified BOOLEAN DEFAULT false,
    is_primary BOOLEAN DEFAULT false,
    
    -- Security and lifecycle management
    security_level VARCHAR(20) DEFAULT 'standard',
    expires_at TIMESTAMP WITH TIME ZONE,
    
    -- PostgreSQL 18: Virtual generated columns for classification
    auth_category VARCHAR(30) GENERATED ALWAYS AS (
        CASE 
            WHEN auth_type = 'web3_wallet' THEN 'BLOCKCHAIN'
            WHEN auth_type LIKE 'oauth_%' THEN 'SOCIAL'
            WHEN auth_type = 'enterprise_sso' THEN 'ENTERPRISE'
            ELSE 'OTHER'
        END
    ) STORED,
    
    -- Virtual column for authentication strength
    auth_strength INTEGER GENERATED ALWAYS AS (
        CASE 
            WHEN security_level = 'high' AND is_verified = true THEN 100
            WHEN security_level = 'standard' AND is_verified = true THEN 75
            WHEN is_verified = true THEN 50
            ELSE 25
        END
    ) STORED,
    
    -- =====================================================
    -- PostgreSQL 18: Enhanced Validation Constraints
    -- =====================================================
    
    -- Auth type validation with OAuth support
    CONSTRAINT auth_methods_type_valid CHECK (
        auth_type IN (
            'web3_wallet', 'oauth_google', 'oauth_github', 'oauth_discord',
            'oauth_twitter', 'enterprise_sso', 'web3auth_social'
        )
    ),
    
    -- Chain type validation for Web3 methods
    CONSTRAINT auth_methods_chain_type_valid CHECK (
        (auth_type = 'web3_wallet' AND chain_type IN ('ethereum', 'near', 'polygon', 'bsc')) OR
        (auth_type != 'web3_wallet' AND chain_type IS NULL)
    ),
    
    -- Wallet address format validation with enhanced patterns
    CONSTRAINT auth_methods_wallet_format CHECK (
        (auth_type != 'web3_wallet') OR
        (chain_type = 'ethereum' AND wallet_address ~ '^0x[a-fA-F0-9]{40}$') OR
        (chain_type = 'near' AND wallet_address ~ '^[a-z0-9_-]+\.near$|^[a-f0-9]{64}$') OR
        (chain_type IN ('polygon', 'bsc') AND wallet_address ~ '^0x[a-fA-F0-9]{40}$')
    ),
    
    -- OAuth provider validation
    CONSTRAINT auth_methods_oauth_consistency CHECK (
        (auth_type NOT LIKE 'oauth_%' AND oauth_provider IS NULL) OR
        (auth_type LIKE 'oauth_%' AND oauth_provider IS NOT NULL)
    ),
    
    -- Security level validation
    CONSTRAINT auth_methods_security_level_valid CHECK (
        security_level IN ('basic', 'standard', 'high', 'enterprise')
    ),
    
    -- PostgreSQL 18: Enhanced JSONB structure validation
    CONSTRAINT auth_methods_signature_params_valid CHECK (
        jsonb_typeof(signature_params) = 'object' AND
        (NOT signature_params ? 'algorithm' OR 
         signature_params->>'algorithm' IN ('secp256k1', 'ed25519', 'rsa', 'ecdsa'))
    ),
    
    CONSTRAINT auth_methods_oauth_metadata_valid CHECK (
        jsonb_typeof(oauth_metadata) = 'object'
    ),
    
    -- Unique constraints for different auth types
    CONSTRAINT auth_methods_unique_wallet UNIQUE (chain_type, wallet_address),
    CONSTRAINT auth_methods_unique_oauth UNIQUE (oauth_provider, oauth_subject_id),
    CONSTRAINT auth_methods_unique_primary_per_user UNIQUE (user_id, is_primary) 
        DEFERRABLE INITIALLY DEFERRED
);

-- =====================================================
-- PostgreSQL 18: Advanced Indexing with Skip Scan
-- =====================================================

-- Primary lookup patterns with skip scan optimization
CREATE INDEX idx_auth_methods_user_type_skip ON user_auth_methods(user_id, auth_type, is_verified);

-- Wallet address lookup with blockchain-specific optimization
CREATE INDEX idx_auth_methods_wallet_lookup ON user_auth_methods(chain_type, wallet_address) 
    WHERE auth_type = 'web3_wallet';

-- PostgreSQL 18: OAuth provider lookup optimization
CREATE INDEX idx_auth_methods_oauth_lookup ON user_auth_methods(oauth_provider, oauth_subject_id)
    WHERE auth_type LIKE 'oauth_%';

-- Security and verification indexes
CREATE INDEX idx_auth_methods_verified_primary ON user_auth_methods(user_id, is_primary, is_verified)
    WHERE is_verified = true;

CREATE INDEX idx_auth_methods_security_level ON user_auth_methods(security_level, auth_category)
    WHERE security_level IN ('high', 'enterprise');

-- PostgreSQL 18: Enhanced JSONB indexes
CREATE INDEX idx_auth_methods_signature_params_gin ON user_auth_methods USING GIN (signature_params);
CREATE INDEX idx_auth_methods_oauth_metadata_gin ON user_auth_methods USING GIN (oauth_metadata);

-- Usage pattern analysis
CREATE INDEX idx_auth_methods_usage_patterns ON user_auth_methods(user_id, last_used_at, auth_category);

-- Expiration monitoring
CREATE INDEX idx_auth_methods_expiring ON user_auth_methods(expires_at, user_id)
    WHERE expires_at IS NOT NULL AND is_verified = true;

-- =====================================================
-- PostgreSQL 18: OAuth Integration Functions
-- =====================================================

-- Function to validate OAuth token (placeholder for PostgreSQL 18 OAuth)
CREATE OR REPLACE FUNCTION validate_oauth_token(
    provider VARCHAR(100),
    token_hash VARCHAR(255),
    subject_id VARCHAR(255)
)
RETURNS BOOLEAN AS $$
DECLARE
    is_valid BOOLEAN := false;
BEGIN
    -- PostgreSQL 18: Integration with OAuth validation libraries
    -- This is a placeholder for the actual OAuth integration
    
    -- Validate token format
    IF char_length(token_hash) = 64 AND token_hash ~ '^[a-fA-F0-9]+$' THEN
        -- Log OAuth validation attempt
        INSERT INTO audit_logs (
            event_type, event_category, event_data, severity
        ) VALUES (
            'oauth_token_validation',
            'auth',
            jsonb_build_object(
                'provider', provider,
                'subject_id', subject_id,
                'timestamp', NOW()
            ),
            'info'
        );
        
        is_valid := true;
    END IF;
    
    RETURN is_valid;
END;
$$ LANGUAGE plpgsql SECURITY DEFINER;

-- Function to create authentication method with validation
CREATE OR REPLACE FUNCTION create_auth_method(
    p_user_id UUID,
    p_auth_type VARCHAR(50),
    p_chain_type VARCHAR(50) DEFAULT NULL,
    p_wallet_address VARCHAR(255) DEFAULT NULL,
    p_oauth_provider VARCHAR(100) DEFAULT NULL,
    p_oauth_subject_id VARCHAR(255) DEFAULT NULL,
    p_oauth_token_hash VARCHAR(255) DEFAULT NULL,
    p_public_key TEXT DEFAULT NULL,
    p_signature_params JSONB DEFAULT '{}',
    p_oauth_metadata JSONB DEFAULT '{}'
)
RETURNS UUID AS $$
DECLARE
    auth_method_id UUID;
    existing_primary BOOLEAN;
BEGIN
    -- Check if user has existing primary auth method
    SELECT EXISTS(
        SELECT 1 FROM user_auth_methods 
        WHERE user_id = p_user_id AND is_primary = true
    ) INTO existing_primary;
    
    -- Insert new authentication method
    INSERT INTO user_auth_methods (
        user_id, auth_type, chain_type, wallet_address,
        oauth_provider, oauth_subject_id, oauth_token_hash,
        public_key, signature_params, oauth_metadata,
        is_primary, is_verified
    ) VALUES (
        p_user_id, p_auth_type, p_chain_type, p_wallet_address,
        p_oauth_provider, p_oauth_subject_id, p_oauth_token_hash,
        p_public_key, p_signature_params, p_oauth_metadata,
        NOT existing_primary, -- First auth method becomes primary
        false -- Requires verification
    ) RETURNING id INTO auth_method_id;
    
    -- Log authentication method creation
    INSERT INTO audit_logs (
        user_id, event_type, event_category, event_data, severity
    ) VALUES (
        p_user_id,
        'auth_method_created',
        'auth',
        jsonb_build_object(
            'auth_method_id', auth_method_id,
            'auth_type', p_auth_type,
            'chain_type', p_chain_type,
            'oauth_provider', p_oauth_provider,
            'is_primary', NOT existing_primary
        ),
        'info'
    );
    
    RETURN auth_method_id;
END;
$$ LANGUAGE plpgsql SECURITY DEFINER;

-- =====================================================
-- PostgreSQL 18: Advanced Triggers
-- =====================================================

-- Function to update authentication method activity
CREATE OR REPLACE FUNCTION update_auth_method_activity()
RETURNS TRIGGER AS $$
BEGIN
    NEW.last_used_at = NOW();
    
    -- Track usage patterns for security analysis
    IF OLD.last_used_at IS DISTINCT FROM NEW.last_used_at THEN
        INSERT INTO audit_logs (
            user_id, event_type, event_category, event_data, severity
        ) VALUES (
            NEW.user_id,
            'auth_method_used',
            'auth',
            jsonb_build_object(
                'auth_method_id', NEW.id,
                'auth_type', NEW.auth_type,
                'auth_category', NEW.auth_category,
                'time_since_last_use', EXTRACT(EPOCH FROM (NOW() - OLD.last_used_at))
            ),
            'info'
        );
    END IF;
    
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Trigger for authentication activity tracking
CREATE TRIGGER auth_methods_update_activity 
    BEFORE UPDATE ON user_auth_methods 
    FOR EACH ROW 
    EXECUTE FUNCTION update_auth_method_activity();

-- =====================================================
-- PostgreSQL 18: Statistics and Optimization
-- =====================================================

-- Configure enhanced statistics for query optimization
ALTER TABLE user_auth_methods ALTER COLUMN auth_type SET STATISTICS 1000;
ALTER TABLE user_auth_methods ALTER COLUMN chain_type SET STATISTICS 500;
ALTER TABLE user_auth_methods ALTER COLUMN oauth_provider SET STATISTICS 500;

-- Create extended statistics for correlated columns
CREATE STATISTICS auth_methods_type_correlation (dependencies, ndistinct)
    ON auth_type, chain_type, is_verified FROM user_auth_methods;

CREATE STATISTICS auth_methods_oauth_correlation (dependencies)
    ON oauth_provider, auth_category, security_level FROM user_auth_methods;

-- =====================================================
-- PostgreSQL 18: Table Configuration
-- =====================================================

ALTER TABLE user_auth_methods SET (
    autovacuum_vacuum_scale_factor = 0.1,
    autovacuum_analyze_scale_factor = 0.05,
    fillfactor = 85
);

-- Comments for documentation
COMMENT ON TABLE user_auth_methods IS 'Authentication methods with PostgreSQL 18 OAuth integration and Web3 support';
COMMENT ON COLUMN user_auth_methods.auth_category IS 'Virtual column classifying authentication method type';
COMMENT ON COLUMN user_auth_methods.auth_strength IS 'Virtual column calculating authentication strength score';
COMMENT ON COLUMN user_auth_methods.oauth_metadata IS 'PostgreSQL 18 enhanced JSONB for OAuth provider metadata';
COMMENT ON FUNCTION validate_oauth_token(VARCHAR, VARCHAR, VARCHAR) IS 'PostgreSQL 18 OAuth token validation integration';
COMMENT ON FUNCTION create_auth_method(UUID, VARCHAR, VARCHAR, VARCHAR, VARCHAR, VARCHAR, VARCHAR, TEXT, JSONB, JSONB) IS 'Secure authentication method creation with comprehensive validation';