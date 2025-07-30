-- Quantum Keys Table for ML-KEM-1024 storage
-- Based on old backend database schema

CREATE TABLE IF NOT EXISTS quantum_keys (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NULL,                    -- NULL for system keys
    algorithm VARCHAR(50) NOT NULL,       -- "ML-KEM-1024"
    key_type VARCHAR(50) NOT NULL,        -- "ml_kem_1024"
    public_key BYTEA NOT NULL,            -- Raw public key bytes
    encrypted_private_key BYTEA NOT NULL, -- Encrypted private key
    encryption_algorithm VARCHAR(50) NOT NULL DEFAULT 'base64', -- For now
    security_metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NULL,
    rotated_at TIMESTAMPTZ NULL, 
    is_active BOOLEAN NOT NULL DEFAULT true,
    is_compromised BOOLEAN NOT NULL DEFAULT false,
    rotation_generation INTEGER NOT NULL DEFAULT 1,
    usage_category VARCHAR(50) NOT NULL DEFAULT 'general'
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_quantum_keys_user_id ON quantum_keys(user_id);
CREATE INDEX IF NOT EXISTS idx_quantum_keys_active ON quantum_keys(is_active) WHERE is_active = true;
CREATE INDEX IF NOT EXISTS idx_quantum_keys_expires ON quantum_keys(expires_at) WHERE expires_at IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_quantum_keys_rotation ON quantum_keys(created_at, is_active);

-- Ensure only one active key per user per category (optional constraint)
CREATE UNIQUE INDEX IF NOT EXISTS idx_quantum_keys_user_category_active 
ON quantum_keys(user_id, usage_category) 
WHERE is_active = true AND user_id IS NOT NULL;