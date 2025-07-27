-- Manual Review Tables Migration (PostgreSQL 18 Beta 1)
-- Creates tables for manual review queue and related functionality

-- Review queue entry table
CREATE TABLE IF NOT EXISTS review_queue (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    transaction_id UUID NOT NULL,
    user_id UUID NOT NULL,
    risk_score DECIMAL(5,4) NOT NULL CHECK (risk_score >= 0 AND risk_score <= 1),
    status VARCHAR(50) NOT NULL CHECK (status IN ('Pending', 'InReview', 'Approved', 'Rejected', 'Escalated', 'Expired')),
    priority VARCHAR(50) NOT NULL CHECK (priority IN ('Critical', 'High', 'Medium', 'Low')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL,
    assigned_to UUID NULL,
    assigned_at TIMESTAMPTZ NULL,
    reviewed_by UUID NULL,
    reviewed_at TIMESTAMPTZ NULL,
    review_reason TEXT NULL,
    escalation_count INTEGER NOT NULL DEFAULT 0,
    last_escalated_at TIMESTAMPTZ NULL,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    
    -- Constraints
    CONSTRAINT review_queue_assignment_check CHECK (
        (status = 'InReview' AND assigned_to IS NOT NULL) OR 
        (status != 'InReview')
    ),
    CONSTRAINT review_queue_reviewed_check CHECK (
        (status IN ('Approved', 'Rejected') AND reviewed_by IS NOT NULL AND reviewed_at IS NOT NULL) OR 
        (status NOT IN ('Approved', 'Rejected'))
    ),
    CONSTRAINT review_queue_escalation_check CHECK (
        (escalation_count > 0 AND last_escalated_at IS NOT NULL) OR 
        (escalation_count = 0)
    )
);

-- Review decisions table (for audit trail)
CREATE TABLE IF NOT EXISTS review_decisions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    review_id UUID NOT NULL REFERENCES review_queue(id) ON DELETE CASCADE,
    transaction_id UUID NOT NULL,
    decision VARCHAR(50) NOT NULL CHECK (decision IN ('Approved', 'Rejected')),
    reason TEXT NOT NULL,
    reviewed_by UUID NOT NULL,
    reviewed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    metadata JSONB NULL DEFAULT '{}'::jsonb
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_review_queue_status ON review_queue(status);
CREATE INDEX IF NOT EXISTS idx_review_queue_priority ON review_queue(priority);
CREATE INDEX IF NOT EXISTS idx_review_queue_created_at ON review_queue(created_at);
CREATE INDEX IF NOT EXISTS idx_review_queue_expires_at ON review_queue(expires_at);
CREATE INDEX IF NOT EXISTS idx_review_queue_assigned_to ON review_queue(assigned_to);
CREATE INDEX IF NOT EXISTS idx_review_queue_transaction_id ON review_queue(transaction_id);
CREATE INDEX IF NOT EXISTS idx_review_queue_user_id ON review_queue(user_id);

-- Composite indexes for common queries
CREATE INDEX IF NOT EXISTS idx_review_queue_status_priority ON review_queue(status, priority);
CREATE INDEX IF NOT EXISTS idx_review_queue_status_created_at ON review_queue(status, created_at);

-- Index for review decisions
CREATE INDEX IF NOT EXISTS idx_review_decisions_review_id ON review_decisions(review_id);
CREATE INDEX IF NOT EXISTS idx_review_decisions_transaction_id ON review_decisions(transaction_id);
CREATE INDEX IF NOT EXISTS idx_review_decisions_reviewed_at ON review_decisions(reviewed_at);

-- Function to automatically update updated_at timestamp
CREATE OR REPLACE FUNCTION update_review_queue_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Trigger to automatically update updated_at
DROP TRIGGER IF EXISTS trigger_review_queue_updated_at ON review_queue;
CREATE TRIGGER trigger_review_queue_updated_at
    BEFORE UPDATE ON review_queue
    FOR EACH ROW
    EXECUTE FUNCTION update_review_queue_updated_at();

-- Function to check for expired reviews (for escalation)
CREATE OR REPLACE FUNCTION get_expired_reviews()
RETURNS TABLE(id UUID, transaction_id UUID, user_id UUID, created_at TIMESTAMPTZ, priority VARCHAR(50)) AS $$
BEGIN
    RETURN QUERY
    SELECT rq.id, rq.transaction_id, rq.user_id, rq.created_at, rq.priority
    FROM review_queue rq
    WHERE rq.expires_at < NOW()
      AND rq.status IN ('Pending', 'InReview')
    ORDER BY rq.priority DESC, rq.created_at ASC;
END;
$$ LANGUAGE plpgsql;

-- Function to get queue statistics
CREATE OR REPLACE FUNCTION get_review_queue_stats()
RETURNS TABLE(
    total_pending BIGINT,
    total_in_review BIGINT,
    total_escalated BIGINT,
    total_expired BIGINT,
    avg_resolution_time_hours NUMERIC,
    critical_count BIGINT,
    high_priority_count BIGINT
) AS $$
BEGIN
    RETURN QUERY
    SELECT 
        COUNT(*) FILTER (WHERE status = 'Pending') as total_pending,
        COUNT(*) FILTER (WHERE status = 'InReview') as total_in_review,
        COUNT(*) FILTER (WHERE status = 'Escalated') as total_escalated,
        COUNT(*) FILTER (WHERE status = 'Expired') as total_expired,
        AVG(EXTRACT(EPOCH FROM (reviewed_at - created_at)) / 3600) FILTER (WHERE reviewed_at IS NOT NULL) as avg_resolution_time_hours,
        COUNT(*) FILTER (WHERE priority = 'Critical') as critical_count,
        COUNT(*) FILTER (WHERE priority = 'High') as high_priority_count
    FROM review_queue;
END;
$$ LANGUAGE plpgsql;

-- Insert sample data for testing (optional, can be removed in production)
INSERT INTO review_queue (
    transaction_id, user_id, risk_score, status, priority, 
    expires_at, review_reason, metadata
) VALUES 
(
    gen_random_uuid(),
    gen_random_uuid(),
    0.85,
    'Pending',
    'High',
    NOW() + INTERVAL '6 hours',
    'High risk score requires manual review',
    '{"source_chain": "ethereum", "destination_chain": "near", "amount": 1000.0}'::jsonb
),
(
    gen_random_uuid(),
    gen_random_uuid(),
    0.92,
    'Pending',
    'Critical',
    NOW() + INTERVAL '2 hours',
    'Critical risk score requires immediate review',
    '{"source_chain": "ethereum", "destination_chain": "near", "amount": 5000.0}'::jsonb
);