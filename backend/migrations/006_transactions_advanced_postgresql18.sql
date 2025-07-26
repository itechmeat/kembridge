-- Transactions table with PostgreSQL 18 advanced features
-- Cross-chain bridge transactions with quantum cryptography and AI risk analysis

CREATE TABLE transactions (
    -- PostgreSQL 18: UUIDv7 for transaction ID with timestamp ordering
    id UUID PRIMARY KEY DEFAULT generate_uuidv7(),
    user_id UUID NOT NULL REFERENCES users(id),
    
    -- Cross-chain transaction details
    source_chain VARCHAR(50) NOT NULL,
    destination_chain VARCHAR(50) NOT NULL,
    source_token VARCHAR(100) NOT NULL,
    destination_token VARCHAR(100) NOT NULL,
    
    -- PostgreSQL 18: High-precision amounts with risk_score domain
    amount_in DECIMAL(36, 18) NOT NULL,
    amount_out DECIMAL(36, 18),
    exchange_rate DECIMAL(36, 18),
    
    -- Fee structure with detailed breakdown
    bridge_fee_amount DECIMAL(36, 18) DEFAULT 0,
    network_fee_amount DECIMAL(36, 18) DEFAULT 0,
    quantum_protection_fee DECIMAL(36, 18) DEFAULT 0,
    
    -- Blockchain transaction identifiers
    source_tx_hash VARCHAR(255),
    destination_tx_hash VARCHAR(255),
    bridge_tx_hash VARCHAR(255),
    
    -- PostgreSQL 18: Enhanced transaction state management
    status VARCHAR(50) NOT NULL DEFAULT 'pending',
    status_history JSONB DEFAULT '[]',
    
    -- Quantum cryptography integration
    quantum_key_id quantum_key_id REFERENCES quantum_keys(id),
    encrypted_payload BYTEA,
    encryption_metadata JSONB DEFAULT '{}',
    
    -- PostgreSQL 18: AI risk analysis with risk_score domain
    risk_score risk_score DEFAULT 0.0000,
    risk_factors JSONB DEFAULT '{}',
    ai_analysis_version VARCHAR(20) DEFAULT 'v1.0',
    
    -- PostgreSQL 18: Enhanced temporal management
    created_at transaction_timestamp,
    updated_at transaction_timestamp,
    confirmed_at TIMESTAMP WITH TIME ZONE,
    completed_at TIMESTAMP WITH TIME ZONE,
    expires_at TIMESTAMP WITH TIME ZONE,
    
    -- 1inch Fusion+ integration
    oneinch_order_id VARCHAR(255),
    oneinch_quote_id VARCHAR(255),
    fusion_metadata JSONB DEFAULT '{}',
    
    -- NEAR Chain Signatures integration
    near_chain_signature JSONB DEFAULT '{}',
    near_account_id VARCHAR(255),
    
    -- PostgreSQL 18: Virtual generated columns for transaction analysis
    transaction_value_usd DECIMAL(20, 2) GENERATED ALWAYS AS (
        CASE 
            WHEN source_token = 'ETH' THEN amount_in * 2000 -- Simplified USD conversion
            WHEN source_token = 'NEAR' THEN amount_in * 4
            ELSE amount_in
        END
    ) STORED,
    
    processing_time_minutes INTEGER GENERATED ALWAYS AS (
        CASE 
            WHEN completed_at IS NOT NULL THEN
                EXTRACT(EPOCH FROM (completed_at - created_at)) / 60
            ELSE NULL
        END
    ) STORED,
    
    transaction_category VARCHAR(20) GENERATED ALWAYS AS (
        CASE 
            WHEN (CASE 
                WHEN source_token = 'ETH' THEN amount_in * 2000 
                WHEN source_token = 'NEAR' THEN amount_in * 4
                ELSE amount_in
            END) > 10000 THEN 'LARGE'
            WHEN (CASE 
                WHEN source_token = 'ETH' THEN amount_in * 2000 
                WHEN source_token = 'NEAR' THEN amount_in * 4
                ELSE amount_in
            END) > 1000 THEN 'MEDIUM'
            ELSE 'SMALL'
        END
    ) STORED,
    
    bridge_direction VARCHAR(20) GENERATED ALWAYS AS (
        CASE 
            WHEN source_chain = 'ethereum' AND destination_chain = 'near' THEN 'ETH_TO_NEAR'
            WHEN source_chain = 'near' AND destination_chain = 'ethereum' THEN 'NEAR_TO_ETH'
            ELSE 'OTHER'
        END
    ) STORED,
    
    risk_category VARCHAR(10) GENERATED ALWAYS AS (
        CASE 
            WHEN risk_score > 0.7 THEN 'HIGH'
            WHEN risk_score > 0.3 THEN 'MEDIUM'
            ELSE 'LOW'
        END
    ) STORED,
    
    -- =====================================================
    -- PostgreSQL 18: Enhanced Validation Constraints
    -- =====================================================
    
    -- Chain validation with expanded support
    CONSTRAINT transactions_valid_chains CHECK (
        source_chain IN ('ethereum', 'near', 'polygon', 'bsc', 'arbitrum') AND
        destination_chain IN ('ethereum', 'near', 'polygon', 'bsc', 'arbitrum')
    ),
    
    -- Must be cross-chain transaction
    CONSTRAINT transactions_different_chains CHECK (
        source_chain != destination_chain
    ),
    
    -- Amount validation
    CONSTRAINT transactions_positive_amounts CHECK (
        amount_in > 0 AND 
        (amount_out IS NULL OR amount_out > 0) AND
        bridge_fee_amount >= 0 AND 
        network_fee_amount >= 0 AND
        quantum_protection_fee >= 0
    ),
    
    -- Status validation with comprehensive states
    CONSTRAINT transactions_valid_status CHECK (
        status IN (
            'pending', 'validating', 'locked', 'processing', 
            'confirming', 'confirmed', 'completed', 
            'failed', 'cancelled', 'expired', 'refunded'
        )
    ),
    
    -- PostgreSQL 18: Enhanced timestamp validation
    CONSTRAINT transactions_temporal_consistency CHECK (
        updated_at >= created_at AND
        (confirmed_at IS NULL OR confirmed_at >= created_at) AND
        (completed_at IS NULL OR completed_at >= created_at) AND
        (expires_at IS NULL OR expires_at > created_at) AND
        (completed_at IS NULL OR confirmed_at IS NULL OR completed_at >= confirmed_at)
    ),
    
    -- Status completion logic
    CONSTRAINT transactions_completion_logic CHECK (
        (status IN ('completed', 'failed', 'cancelled', 'refunded') AND completed_at IS NOT NULL) OR
        (status NOT IN ('completed', 'failed', 'cancelled', 'refunded') AND completed_at IS NULL)
    ),
    
    -- Exchange rate logic validation
    CONSTRAINT transactions_exchange_rate_logic CHECK (
        (amount_out IS NULL AND exchange_rate IS NULL) OR
        (amount_out IS NOT NULL AND exchange_rate IS NOT NULL AND 
         ABS(amount_out - (amount_in * exchange_rate)) < 0.0001)
    ),
    
    -- PostgreSQL 18: Enhanced JSONB validation
    CONSTRAINT transactions_status_history_array CHECK (
        jsonb_typeof(status_history) = 'array'
    ),
    
    CONSTRAINT transactions_risk_factors_object CHECK (
        jsonb_typeof(risk_factors) = 'object'
    ),
    
    CONSTRAINT transactions_encryption_metadata_valid CHECK (
        jsonb_typeof(encryption_metadata) = 'object' AND
        (quantum_key_id IS NULL OR encrypted_payload IS NOT NULL)
    ),
    
    CONSTRAINT transactions_fusion_metadata_valid CHECK (
        jsonb_typeof(fusion_metadata) = 'object'
    ),
    
    CONSTRAINT transactions_near_signature_valid CHECK (
        jsonb_typeof(near_chain_signature) = 'object'
    )
);

-- =====================================================
-- PostgreSQL 18: Advanced Indexing with Skip Scan
-- =====================================================

-- Primary transaction lookups with skip scan optimization
CREATE INDEX idx_transactions_user_status_skip ON transactions(user_id, status, created_at);

-- Cross-chain flow analysis
CREATE INDEX idx_transactions_bridge_flow ON transactions(source_chain, destination_chain, status, created_at);

-- PostgreSQL 18: Risk analysis and monitoring
CREATE INDEX idx_transactions_risk_monitoring ON transactions(risk_score, risk_category, created_at)
    WHERE risk_score > 0.3;

-- Transaction value analysis
CREATE INDEX idx_transactions_value_analysis ON transactions(transaction_value_usd, transaction_category, created_at);

-- Blockchain hash lookups (most frequent)
CREATE INDEX idx_transactions_source_hash ON transactions(source_tx_hash) 
    WHERE source_tx_hash IS NOT NULL;

CREATE INDEX idx_transactions_destination_hash ON transactions(destination_tx_hash) 
    WHERE destination_tx_hash IS NOT NULL;

-- PostgreSQL 18: Temporal range queries for analytics
CREATE INDEX idx_transactions_temporal_range ON transactions(created_at, bridge_direction, status);

-- Quantum cryptography integration
CREATE INDEX idx_transactions_quantum_encrypted ON transactions(quantum_key_id, encryption_metadata)
    WHERE quantum_key_id IS NOT NULL;

-- 1inch Fusion+ integration lookups
CREATE INDEX idx_transactions_oneinch_orders ON transactions(oneinch_order_id, oneinch_quote_id)
    WHERE oneinch_order_id IS NOT NULL;

-- NEAR Chain Signatures lookups
CREATE INDEX idx_transactions_near_signatures ON transactions(near_account_id, created_at)
    WHERE near_account_id IS NOT NULL;

-- PostgreSQL 18: Enhanced JSONB indexes
CREATE INDEX idx_transactions_risk_factors_gin ON transactions USING GIN (risk_factors);
CREATE INDEX idx_transactions_fusion_metadata_gin ON transactions USING GIN (fusion_metadata);
CREATE INDEX idx_transactions_near_signature_gin ON transactions USING GIN (near_chain_signature);
CREATE INDEX idx_transactions_status_history_gin ON transactions USING GIN (status_history);

-- Virtual column indexes for performance analysis
CREATE INDEX idx_transactions_processing_time ON transactions(processing_time_minutes, transaction_category)
    WHERE processing_time_minutes IS NOT NULL;

CREATE INDEX idx_transactions_large_amounts ON transactions(transaction_value_usd, risk_category)
    WHERE transaction_category = 'LARGE';

-- Expiry and cleanup monitoring
CREATE INDEX idx_transactions_expiring ON transactions(expires_at, status)
    WHERE expires_at IS NOT NULL AND status IN ('pending', 'locked', 'processing');

-- =====================================================
-- PostgreSQL 18: Advanced Transaction Management Functions
-- =====================================================

-- Function to create transaction with comprehensive validation
CREATE OR REPLACE FUNCTION create_bridge_transaction(
    p_user_id UUID,
    p_source_chain VARCHAR(50),
    p_destination_chain VARCHAR(50),
    p_source_token VARCHAR(100),
    p_destination_token VARCHAR(100),
    p_amount_in DECIMAL(36, 18),
    p_expected_amount_out DECIMAL(36, 18) DEFAULT NULL,
    p_quantum_key_id UUID DEFAULT NULL,
    p_expires_in_hours INTEGER DEFAULT 24,
    p_oneinch_quote_id VARCHAR(255) DEFAULT NULL
)
RETURNS UUID AS $$
DECLARE
    transaction_id UUID;
    expires_timestamp TIMESTAMP WITH TIME ZONE;
    initial_risk_score DECIMAL(5,4);
    status_entry JSONB;
BEGIN
    -- Calculate expiration
    expires_timestamp := NOW() + (p_expires_in_hours || ' hours')::INTERVAL;
    
    -- Initial risk assessment (simplified)
    initial_risk_score := CASE 
        WHEN p_amount_in > 10 THEN 0.5000  -- Large amounts higher risk
        WHEN p_amount_in > 1 THEN 0.2000   -- Medium amounts medium risk
        ELSE 0.1000                        -- Small amounts low risk
    END;
    
    -- Create initial status history entry
    status_entry := jsonb_build_array(
        jsonb_build_object(
            'status', 'pending',
            'timestamp', NOW(),
            'reason', 'transaction_created'
        )
    );
    
    -- Create transaction
    INSERT INTO transactions (
        user_id, source_chain, destination_chain,
        source_token, destination_token, amount_in, amount_out,
        quantum_key_id, expires_at, risk_score,
        status_history, oneinch_quote_id,
        risk_factors, ai_analysis_version
    ) VALUES (
        p_user_id, p_source_chain, p_destination_chain,
        p_source_token, p_destination_token, p_amount_in, p_expected_amount_out,
        p_quantum_key_id, expires_timestamp, initial_risk_score,
        status_entry, p_oneinch_quote_id,
        jsonb_build_object(
            'amount_factor', CASE WHEN p_amount_in > 10 THEN 0.3 ELSE 0.1 END,
            'cross_chain_factor', 0.1,
            'user_history_factor', 0.0,
            'initial_assessment', 'automated'
        ),
        'v1.0'
    ) RETURNING id INTO transaction_id;
    
    -- Log transaction creation
    INSERT INTO audit_logs (
        user_id, transaction_id, event_type, event_category,
        event_data, severity, is_sensitive
    ) VALUES (
        p_user_id, transaction_id, 'transaction_created', 'finance',
        jsonb_build_object(
            'transaction_id', transaction_id,
            'bridge_direction', p_source_chain || '_to_' || p_destination_chain,
            'amount', p_amount_in,
            'token_pair', p_source_token || '_' || p_destination_token,
            'initial_risk_score', initial_risk_score,
            'quantum_protected', p_quantum_key_id IS NOT NULL,
            'oneinch_integration', p_oneinch_quote_id IS NOT NULL
        ),
        'info', false
    );
    
    RETURN transaction_id;
END;
$$ LANGUAGE plpgsql SECURITY DEFINER;

-- Function to update transaction status with comprehensive audit
CREATE OR REPLACE FUNCTION update_transaction_status(
    p_transaction_id UUID,
    p_new_status VARCHAR(50),
    p_reason VARCHAR(255) DEFAULT NULL,
    p_tx_hash VARCHAR(255) DEFAULT NULL,
    p_risk_score DECIMAL(5,4) DEFAULT NULL,
    p_metadata JSONB DEFAULT '{}'
)
RETURNS BOOLEAN AS $$
DECLARE
    tx_record RECORD;
    new_status_entry JSONB;
    updated_status_history JSONB;
BEGIN
    -- Get current transaction state
    SELECT * INTO tx_record
    FROM transactions 
    WHERE id = p_transaction_id;
    
    IF NOT FOUND THEN
        RETURN false;
    END IF;
    
    -- Validate status transition
    IF NOT is_valid_status_transition(tx_record.status, p_new_status) THEN
        RAISE EXCEPTION 'Invalid status transition from % to %', tx_record.status, p_new_status;
    END IF;
    
    -- Create new status history entry
    new_status_entry := jsonb_build_object(
        'status', p_new_status,
        'timestamp', NOW(),
        'reason', COALESCE(p_reason, 'status_update'),
        'tx_hash', p_tx_hash,
        'metadata', p_metadata
    );
    
    -- Update status history
    updated_status_history := tx_record.status_history || new_status_entry;
    
    -- Update transaction
    UPDATE transactions 
    SET 
        status = p_new_status,
        updated_at = NOW(),
        status_history = updated_status_history,
        confirmed_at = CASE 
            WHEN p_new_status = 'confirmed' THEN NOW()
            ELSE confirmed_at
        END,
        completed_at = CASE 
            WHEN p_new_status IN ('completed', 'failed', 'cancelled', 'refunded') THEN NOW()
            ELSE completed_at
        END,
        risk_score = COALESCE(p_risk_score, risk_score),
        source_tx_hash = CASE 
            WHEN p_tx_hash IS NOT NULL AND source_tx_hash IS NULL THEN p_tx_hash
            ELSE source_tx_hash
        END,
        destination_tx_hash = CASE 
            WHEN p_tx_hash IS NOT NULL AND p_new_status = 'completed' THEN p_tx_hash
            ELSE destination_tx_hash
        END
    WHERE id = p_transaction_id;
    
    -- Log status change
    INSERT INTO audit_logs (
        user_id, transaction_id, event_type, event_category,
        event_data, severity, is_sensitive
    ) VALUES (
        tx_record.user_id, p_transaction_id, 'transaction_status_changed', 'finance',
        jsonb_build_object(
            'old_status', tx_record.status,
            'new_status', p_new_status,
            'reason', p_reason,
            'tx_hash', p_tx_hash,
            'risk_score', p_risk_score,
            'processing_time_minutes', 
                CASE 
                    WHEN p_new_status IN ('completed', 'failed') THEN
                        EXTRACT(EPOCH FROM (NOW() - tx_record.created_at)) / 60
                    ELSE NULL
                END
        ),
        CASE 
            WHEN p_new_status = 'failed' THEN 'error'
            WHEN p_new_status = 'completed' THEN 'info'
            ELSE 'info'
        END,
        false
    );
    
    RETURN true;
END;
$$ LANGUAGE plpgsql SECURITY DEFINER;

-- Helper function to validate status transitions
CREATE OR REPLACE FUNCTION is_valid_status_transition(
    current_status VARCHAR(50),
    new_status VARCHAR(50)
)
RETURNS BOOLEAN AS $$
BEGIN
    RETURN CASE 
        WHEN current_status = 'pending' THEN new_status IN ('validating', 'cancelled', 'expired')
        WHEN current_status = 'validating' THEN new_status IN ('locked', 'failed', 'cancelled')
        WHEN current_status = 'locked' THEN new_status IN ('processing', 'failed', 'expired')
        WHEN current_status = 'processing' THEN new_status IN ('confirming', 'failed')
        WHEN current_status = 'confirming' THEN new_status IN ('confirmed', 'failed')
        WHEN current_status = 'confirmed' THEN new_status IN ('completed', 'failed')
        WHEN current_status IN ('completed', 'failed', 'cancelled', 'expired', 'refunded') THEN false
        ELSE true
    END;
END;
$$ LANGUAGE plpgsql IMMUTABLE;

-- =====================================================
-- PostgreSQL 18: Enhanced Triggers
-- =====================================================

-- Function to update transaction timestamps
CREATE OR REPLACE FUNCTION update_transaction_timestamp()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    
    -- Validate risk score changes
    IF OLD.risk_score IS DISTINCT FROM NEW.risk_score THEN
        INSERT INTO audit_logs (
            user_id, transaction_id, event_type, event_category,
            event_data, severity, is_sensitive
        ) VALUES (
            NEW.user_id, NEW.id, 'risk_score_updated', 'security',
            jsonb_build_object(
                'old_risk_score', OLD.risk_score,
                'new_risk_score', NEW.risk_score,
                'risk_factors', NEW.risk_factors
            ),
            CASE WHEN NEW.risk_score > 0.7 THEN 'warning' ELSE 'info' END,
            false
        );
    END IF;
    
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Trigger for transaction updates
CREATE TRIGGER transactions_update_timestamp 
    BEFORE UPDATE ON transactions 
    FOR EACH ROW 
    EXECUTE FUNCTION update_transaction_timestamp();

-- =====================================================
-- PostgreSQL 18: Statistics and Performance Optimization
-- =====================================================

-- Configure enhanced statistics
ALTER TABLE transactions ALTER COLUMN status SET STATISTICS 1000;
ALTER TABLE transactions ALTER COLUMN source_chain SET STATISTICS 1000;
ALTER TABLE transactions ALTER COLUMN destination_chain SET STATISTICS 1000;
ALTER TABLE transactions ALTER COLUMN risk_score SET STATISTICS 1000;
ALTER TABLE transactions ALTER COLUMN amount_in SET STATISTICS 1000;

-- Extended statistics for correlated columns
CREATE STATISTICS transactions_bridge_correlation (dependencies, ndistinct)
    ON source_chain, destination_chain, bridge_direction FROM transactions;

CREATE STATISTICS transactions_risk_correlation (dependencies)
    ON risk_score, risk_category, transaction_category FROM transactions;

CREATE STATISTICS transactions_temporal_correlation (dependencies)
    ON created_at, processing_time_minutes, status FROM transactions;

-- =====================================================
-- PostgreSQL 18: Table Configuration and Partitioning
-- =====================================================

-- Configure table for high-performance operations
ALTER TABLE transactions SET (
    autovacuum_vacuum_scale_factor = 0.05,
    autovacuum_analyze_scale_factor = 0.02,
    autovacuum_vacuum_cost_delay = 5,
    fillfactor = 85
);

-- Future partitioning by creation date (for high volume)
-- Uncomment when transaction volume exceeds 1M per month

/*
-- Enable partitioning
CREATE TABLE transactions_partitioned (LIKE transactions INCLUDING ALL)
PARTITION BY RANGE (created_at);

-- Create monthly partitions
CREATE TABLE transactions_y2024m01 PARTITION OF transactions_partitioned
    FOR VALUES FROM ('2024-01-01') TO ('2024-02-01');

CREATE TABLE transactions_y2024m02 PARTITION OF transactions_partitioned
    FOR VALUES FROM ('2024-02-01') TO ('2024-03-01');
*/

-- Comments for documentation
COMMENT ON TABLE transactions IS 'Cross-chain bridge transactions with PostgreSQL 18 advanced features, quantum cryptography, and AI risk analysis';
COMMENT ON COLUMN transactions.transaction_value_usd IS 'Virtual column calculating USD value for analytics';
COMMENT ON COLUMN transactions.processing_time_minutes IS 'Virtual column tracking transaction processing duration';
COMMENT ON COLUMN transactions.transaction_category IS 'Virtual column categorizing transaction size';
COMMENT ON COLUMN transactions.bridge_direction IS 'Virtual column standardizing cross-chain direction';
COMMENT ON COLUMN transactions.risk_category IS 'Virtual column for risk-based classification';
COMMENT ON FUNCTION create_bridge_transaction(UUID, VARCHAR, VARCHAR, VARCHAR, VARCHAR, DECIMAL, DECIMAL, UUID, INTEGER, VARCHAR) IS 'Create cross-chain bridge transaction with comprehensive validation and risk assessment';
COMMENT ON FUNCTION update_transaction_status(UUID, VARCHAR, VARCHAR, VARCHAR, DECIMAL, JSONB) IS 'Update transaction status with validation, audit trail, and automatic timestamp management';