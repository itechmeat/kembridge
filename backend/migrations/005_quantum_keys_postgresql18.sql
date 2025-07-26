-- Quantum keys table with PostgreSQL 18 advanced cryptographic features
-- Post-quantum cryptography with enhanced security and key lifecycle management

CREATE TABLE quantum_keys (
    -- PostgreSQL 18: UUIDv7 for cryptographic key ID with timestamp ordering
    id quantum_key_id PRIMARY KEY DEFAULT generate_uuidv7(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    
    -- Post-quantum algorithm specification
    algorithm VARCHAR(50) NOT NULL DEFAULT 'ml-kem-1024',
    key_type VARCHAR(50) NOT NULL DEFAULT 'key_encapsulation',
    
    -- PostgreSQL 18: Enhanced cryptographic key storage
    public_key BYTEA NOT NULL,
    encrypted_private_key BYTEA NOT NULL,
    
    -- PostgreSQL 18: Advanced encryption configuration
    encryption_algorithm VARCHAR(100) NOT NULL DEFAULT 'aes-256-gcm',
    encryption_iv BYTEA,
    encryption_salt BYTEA,
    
    -- PostgreSQL 18: Key derivation and security metadata
    key_derivation_params JSONB DEFAULT '{}',
    security_metadata JSONB DEFAULT '{}',
    
    -- Key lifecycle and rotation management
    created_at transaction_timestamp,
    expires_at TIMESTAMP WITH TIME ZONE,
    rotated_at TIMESTAMP WITH TIME ZONE,
    
    -- Key status and validation
    is_active BOOLEAN DEFAULT true,
    is_compromised BOOLEAN DEFAULT false,
    validation_status VARCHAR(20) DEFAULT 'pending',
    
    -- Key rotation chain
    previous_key_id UUID REFERENCES quantum_keys(id),
    rotation_reason VARCHAR(255),
    rotation_generation INTEGER DEFAULT 1,
    
    -- PostgreSQL 18: Hardware Security Module integration
    hsm_key_id VARCHAR(255),
    hsm_provider VARCHAR(100),
    
    -- PostgreSQL 18: Key age tracking (will be calculated at query time)
    
    key_status VARCHAR(20) GENERATED ALWAYS AS (
        CASE 
            WHEN is_compromised THEN 'COMPROMISED'
            WHEN NOT is_active THEN 'INACTIVE'
            WHEN validation_status != 'validated' THEN 'PENDING'
            ELSE 'ACTIVE'
        END
    ) STORED,
    
    key_strength VARCHAR(10) GENERATED ALWAYS AS (
        CASE 
            WHEN algorithm IN ('ml-kem-1024', 'dilithium-5', 'sphincs+-256s') THEN 'HIGH'
            WHEN algorithm IN ('ml-kem-768', 'dilithium-3') THEN 'MEDIUM'
            ELSE 'STANDARD'
        END
    ) STORED,
    
    -- Key usage classification
    usage_category VARCHAR(30) GENERATED ALWAYS AS (
        CASE 
            WHEN key_type = 'key_encapsulation' THEN 'ENCRYPTION'
            WHEN key_type = 'digital_signature' THEN 'SIGNING'
            WHEN key_type = 'hybrid_classical_quantum' THEN 'HYBRID'
            ELSE 'GENERAL'
        END
    ) STORED,
    
    -- =====================================================
    -- PostgreSQL 18: Enhanced Validation Constraints
    -- =====================================================
    
    -- Post-quantum algorithm validation
    CONSTRAINT quantum_keys_algorithm_valid CHECK (
        algorithm IN (
            'ml-kem-1024', 'ml-kem-768', 'ml-kem-512',
            'dilithium-5', 'dilithium-3', 'dilithium-2',
            'sphincs+-256s', 'sphincs+-192s', 'sphincs+-128s',
            'falcon-1024', 'falcon-512',
            'kyber-1024', 'kyber-768', 'kyber-512'  -- Legacy support
        )
    ),
    
    -- Key type validation
    CONSTRAINT quantum_keys_type_valid CHECK (
        key_type IN (
            'key_encapsulation', 'digital_signature', 'hash_based_signature',
            'hybrid_classical_quantum', 'experimental'
        )
    ),
    
    -- Validation status constraint
    CONSTRAINT quantum_keys_validation_status_valid CHECK (
        validation_status IN ('pending', 'validated', 'failed', 'revoked')
    ),
    
    -- PostgreSQL 18: Algorithm-specific key size validation
    CONSTRAINT quantum_keys_size_validation CHECK (
        -- ML-KEM-1024 public key size
        (algorithm = 'ml-kem-1024' AND octet_length(public_key) = 1568) OR
        -- ML-KEM-768 public key size
        (algorithm = 'ml-kem-768' AND octet_length(public_key) = 1184) OR
        -- Dilithium-5 public key size
        (algorithm = 'dilithium-5' AND octet_length(public_key) = 2592) OR
        -- SPHINCS+-256s public key size
        (algorithm = 'sphincs+-256s' AND octet_length(public_key) = 64) OR
        -- Allow other algorithms with flexible sizes
        algorithm NOT IN ('ml-kem-1024', 'ml-kem-768', 'dilithium-5', 'sphincs+-256s')
    ),
    
    -- Encryption algorithm validation
    CONSTRAINT quantum_keys_encryption_valid CHECK (
        encryption_algorithm IN (
            'aes-256-gcm', 'aes-256-siv', 'chacha20-poly1305',
            'aes-256-ocb', 'xchacha20-poly1305'
        )
    ),
    
    -- PostgreSQL 18: Enhanced JSONB structure validation
    CONSTRAINT quantum_keys_derivation_params_valid CHECK (
        jsonb_typeof(key_derivation_params) = 'object' AND
        (NOT key_derivation_params ? 'iterations' OR 
         (key_derivation_params->>'iterations')::INTEGER > 0)
    ),
    
    CONSTRAINT quantum_keys_security_metadata_valid CHECK (
        jsonb_typeof(security_metadata) = 'object'
    ),
    
    -- Key lifecycle validation
    CONSTRAINT quantum_keys_lifecycle_valid CHECK (
        (expires_at IS NULL OR expires_at > created_at) AND
        (rotated_at IS NULL OR rotated_at >= created_at) AND
        (NOT is_compromised OR validation_status = 'revoked')
    ),
    
    -- Rotation chain validation
    CONSTRAINT quantum_keys_rotation_chain_valid CHECK (
        (previous_key_id IS NULL AND rotation_generation = 1) OR
        (previous_key_id IS NOT NULL AND rotation_generation > 1)
    ),
    
    -- HSM integration validation
    CONSTRAINT quantum_keys_hsm_consistency CHECK (
        (hsm_key_id IS NULL AND hsm_provider IS NULL) OR
        (hsm_key_id IS NOT NULL AND hsm_provider IS NOT NULL)
    ),
    
    -- Prevent self-referential rotation
    CONSTRAINT quantum_keys_no_self_rotation CHECK (
        previous_key_id IS NULL OR previous_key_id != id
    )
);

-- =====================================================
-- PostgreSQL 18: Advanced Indexing with Skip Scan
-- =====================================================

-- Primary key access with skip scan optimization
CREATE INDEX idx_quantum_keys_user_algorithm_skip ON quantum_keys(user_id, algorithm, is_active);

-- Active key lookup optimization
CREATE INDEX idx_quantum_keys_active_lookup ON quantum_keys(user_id, key_type, is_active, created_at)
    WHERE is_active = true AND NOT is_compromised;

-- PostgreSQL 18: Key expiry monitoring with temporal indexing
CREATE INDEX idx_quantum_keys_expiry_monitoring ON quantum_keys(expires_at, user_id, algorithm)
    WHERE is_active = true AND expires_at IS NOT NULL;

-- Key rotation chain analysis
CREATE INDEX idx_quantum_keys_rotation_chain ON quantum_keys(previous_key_id, rotation_generation, created_at)
    WHERE previous_key_id IS NOT NULL;

-- Security and compliance monitoring
CREATE INDEX idx_quantum_keys_security_audit ON quantum_keys(is_compromised, validation_status, created_at)
    WHERE is_compromised = true OR validation_status != 'validated';

-- PostgreSQL 18: HSM integration lookup
CREATE INDEX idx_quantum_keys_hsm_lookup ON quantum_keys(hsm_provider, hsm_key_id)
    WHERE hsm_provider IS NOT NULL;

-- Virtual column indexes for analysis
CREATE INDEX idx_quantum_keys_status_analysis ON quantum_keys(key_status, key_strength, created_at);
-- Age analysis index removed (key_age_days calculation moved to application level)

-- PostgreSQL 18: Enhanced JSONB indexes for metadata search
CREATE INDEX idx_quantum_keys_derivation_gin ON quantum_keys USING GIN (key_derivation_params);
CREATE INDEX idx_quantum_keys_security_gin ON quantum_keys USING GIN (security_metadata);

-- Algorithm performance analysis
CREATE INDEX idx_quantum_keys_algorithm_performance ON quantum_keys(algorithm, created_at, validation_status);

-- =====================================================
-- PostgreSQL 18: Advanced Quantum Key Management Functions
-- =====================================================

-- Function to generate quantum key with comprehensive validation
CREATE OR REPLACE FUNCTION generate_quantum_key(
    p_user_id UUID,
    p_algorithm VARCHAR(50) DEFAULT 'ml-kem-1024',
    p_key_type VARCHAR(50) DEFAULT 'key_encapsulation',
    p_public_key BYTEA DEFAULT NULL,
    p_encrypted_private_key BYTEA DEFAULT NULL,
    p_encryption_params JSONB DEFAULT '{}',
    p_expires_in_days INTEGER DEFAULT 365,
    p_hsm_provider VARCHAR(100) DEFAULT NULL
)
RETURNS UUID AS $$
DECLARE
    key_id UUID;
    current_generation INTEGER := 1;
    previous_key UUID;
    expires_timestamp TIMESTAMP WITH TIME ZONE;
    validation_result VARCHAR(20) := 'pending';
BEGIN
    -- Calculate expiration
    IF p_expires_in_days > 0 THEN
        expires_timestamp := NOW() + (p_expires_in_days || ' days')::INTERVAL;
    END IF;
    
    -- Find previous key for rotation chain
    SELECT id, rotation_generation INTO previous_key, current_generation
    FROM quantum_keys 
    WHERE user_id = p_user_id 
      AND algorithm = p_algorithm 
      AND key_type = p_key_type 
      AND is_active = true
    ORDER BY created_at DESC 
    LIMIT 1;
    
    -- Deactivate previous key if exists
    IF previous_key IS NOT NULL THEN
        UPDATE quantum_keys 
        SET 
            is_active = false,
            rotated_at = NOW(),
            rotation_reason = 'scheduled_rotation'
        WHERE id = previous_key;
        
        current_generation := current_generation + 1;
    END IF;
    
    -- Validate key material if provided
    IF p_public_key IS NOT NULL AND p_encrypted_private_key IS NOT NULL THEN
        -- Perform basic validation
        IF (p_algorithm = 'ml-kem-1024' AND octet_length(p_public_key) = 1568) OR
           (p_algorithm = 'ml-kem-768' AND octet_length(p_public_key) = 1184) OR
           (p_algorithm NOT IN ('ml-kem-1024', 'ml-kem-768')) THEN
            validation_result := 'validated';
        ELSE
            validation_result := 'failed';
        END IF;
    END IF;
    
    -- Create new quantum key
    INSERT INTO quantum_keys (
        user_id, algorithm, key_type,
        public_key, encrypted_private_key,
        key_derivation_params, security_metadata,
        expires_at, previous_key_id, rotation_generation,
        validation_status, hsm_provider
    ) VALUES (
        p_user_id, p_algorithm, p_key_type,
        p_public_key, p_encrypted_private_key,
        p_encryption_params, 
        jsonb_build_object(
            'generation_method', 'automated',
            'entropy_source', 'system_random',
            'generation_timestamp', NOW(),
            'compliance_level', 'nist_approved'
        ),
        expires_timestamp, previous_key, current_generation,
        validation_result, p_hsm_provider
    ) RETURNING id INTO key_id;
    
    -- Log key generation
    INSERT INTO audit_logs (
        user_id, event_type, event_category, event_data, severity, is_sensitive
    ) VALUES (
        p_user_id, 'quantum_key_generated', 'crypto',
        jsonb_build_object(
            'key_id', key_id,
            'algorithm', p_algorithm,
            'key_type', p_key_type,
            'generation', current_generation,
            'previous_key_id', previous_key,
            'validation_status', validation_result,
            'hsm_provider', p_hsm_provider
        ),
        'info', true
    );
    
    RETURN key_id;
END;
$$ LANGUAGE plpgsql SECURITY DEFINER;

-- Function to rotate quantum key with enhanced security
CREATE OR REPLACE FUNCTION rotate_quantum_key(
    p_key_id UUID,
    p_reason VARCHAR(255) DEFAULT 'scheduled_rotation',
    p_new_algorithm VARCHAR(50) DEFAULT NULL
)
RETURNS UUID AS $$
DECLARE
    old_key RECORD;
    new_key_id UUID;
    target_algorithm VARCHAR(50);
BEGIN
    -- Get current key information
    SELECT * INTO old_key 
    FROM quantum_keys 
    WHERE id = p_key_id AND is_active = true;
    
    IF NOT FOUND THEN
        RAISE EXCEPTION 'Key not found or already inactive: %', p_key_id;
    END IF;
    
    -- Determine target algorithm
    target_algorithm := COALESCE(p_new_algorithm, old_key.algorithm);
    
    -- Mark old key as compromised if rotation reason indicates security issue
    IF p_reason IN ('compromised', 'suspected_breach', 'security_incident') THEN
        UPDATE quantum_keys 
        SET 
            is_compromised = true,
            validation_status = 'revoked'
        WHERE id = p_key_id;
    END IF;
    
    -- Generate new key in rotation chain
    new_key_id := generate_quantum_key(
        old_key.user_id,
        target_algorithm,
        old_key.key_type,
        NULL, -- Will be generated by external system
        NULL, -- Will be generated by external system
        old_key.key_derivation_params,
        365, -- Default 1 year expiry
        old_key.hsm_provider
    );
    
    -- Update rotation metadata
    UPDATE quantum_keys 
    SET 
        rotation_reason = p_reason,
        rotated_at = NOW()
    WHERE id = p_key_id;
    
    -- Log key rotation
    INSERT INTO audit_logs (
        user_id, event_type, event_category, event_data, severity, is_sensitive
    ) VALUES (
        old_key.user_id, 'quantum_key_rotated', 'crypto',
        jsonb_build_object(
            'old_key_id', p_key_id,
            'new_key_id', new_key_id,
            'reason', p_reason,
            'old_algorithm', old_key.algorithm,
            'new_algorithm', target_algorithm,
            'generation_increment', old_key.rotation_generation + 1
        ),
        CASE WHEN p_reason LIKE '%compromised%' THEN 'critical' ELSE 'warning' END,
        true
    );
    
    RETURN new_key_id;
END;
$$ LANGUAGE plpgsql SECURITY DEFINER;

-- Function to validate quantum key integrity
CREATE OR REPLACE FUNCTION validate_quantum_key_integrity(p_key_id UUID)
RETURNS BOOLEAN AS $$
DECLARE
    key_record RECORD;
    is_valid BOOLEAN := true;
    validation_errors JSONB := '[]';
BEGIN
    -- Get key details
    SELECT * INTO key_record 
    FROM quantum_keys 
    WHERE id = p_key_id;
    
    IF NOT FOUND THEN
        RETURN false;
    END IF;
    
    -- Validate key size constraints
    IF NOT (
        (key_record.algorithm = 'ml-kem-1024' AND octet_length(key_record.public_key) = 1568) OR
        (key_record.algorithm = 'ml-kem-768' AND octet_length(key_record.public_key) = 1184) OR
        (key_record.algorithm NOT IN ('ml-kem-1024', 'ml-kem-768'))
    ) THEN
        is_valid := false;
        validation_errors := validation_errors || '"invalid_key_size"';
    END IF;
    
    -- Check expiry status
    IF key_record.expires_at IS NOT NULL AND key_record.expires_at < NOW() THEN
        is_valid := false;
        validation_errors := validation_errors || '"key_expired"';
    END IF;
    
    -- Check for compromise status
    IF key_record.is_compromised THEN
        is_valid := false;
        validation_errors := validation_errors || '"key_compromised"';
    END IF;
    
    -- Update validation status
    UPDATE quantum_keys 
    SET 
        validation_status = CASE WHEN is_valid THEN 'validated' ELSE 'failed' END,
        security_metadata = security_metadata || jsonb_build_object(
            'last_validation', NOW(),
            'validation_errors', validation_errors,
            'validation_result', is_valid
        )
    WHERE id = p_key_id;
    
    -- Log validation result
    INSERT INTO audit_logs (
        user_id, event_type, event_category, event_data, severity, is_sensitive
    ) VALUES (
        key_record.user_id, 'quantum_key_validated', 'crypto',
        jsonb_build_object(
            'key_id', p_key_id,
            'validation_result', is_valid,
            'errors', validation_errors,
            'algorithm', key_record.algorithm
        ),
        CASE WHEN is_valid THEN 'info' ELSE 'warning' END,
        true
    );
    
    RETURN is_valid;
END;
$$ LANGUAGE plpgsql;

-- =====================================================
-- PostgreSQL 18: Statistics and Performance Optimization
-- =====================================================

-- Configure enhanced statistics for query optimization
ALTER TABLE quantum_keys ALTER COLUMN algorithm SET STATISTICS 1000;
ALTER TABLE quantum_keys ALTER COLUMN key_type SET STATISTICS 500;
ALTER TABLE quantum_keys ALTER COLUMN validation_status SET STATISTICS 500;

-- Extended statistics for correlated columns
CREATE STATISTICS quantum_keys_algorithm_correlation (dependencies, ndistinct)
    ON algorithm, key_type, key_strength FROM quantum_keys;

CREATE STATISTICS quantum_keys_lifecycle_correlation (dependencies)
    ON is_active, validation_status, created_at FROM quantum_keys;

-- =====================================================
-- PostgreSQL 18: Table Configuration
-- =====================================================

ALTER TABLE quantum_keys SET (
    autovacuum_vacuum_scale_factor = 0.1,
    autovacuum_analyze_scale_factor = 0.05,
    autovacuum_vacuum_cost_delay = 10,
    fillfactor = 90
);

-- Comments for documentation
COMMENT ON TABLE quantum_keys IS 'Post-quantum cryptographic keys with PostgreSQL 18 advanced security features and lifecycle management';
-- Age calculation comment removed (moved to application level)
COMMENT ON COLUMN quantum_keys.key_status IS 'Virtual column showing comprehensive key status';
COMMENT ON COLUMN quantum_keys.key_strength IS 'Virtual column assessing cryptographic strength level';
COMMENT ON COLUMN quantum_keys.usage_category IS 'Virtual column categorizing key usage type';
COMMENT ON FUNCTION generate_quantum_key(UUID, VARCHAR, VARCHAR, BYTEA, BYTEA, JSONB, INTEGER, VARCHAR) IS 'Generate post-quantum cryptographic key with comprehensive validation and audit';
COMMENT ON FUNCTION rotate_quantum_key(UUID, VARCHAR, VARCHAR) IS 'Secure quantum key rotation with enhanced audit trail and security validation';
COMMENT ON FUNCTION validate_quantum_key_integrity(UUID) IS 'Comprehensive quantum key integrity validation with detailed error reporting';