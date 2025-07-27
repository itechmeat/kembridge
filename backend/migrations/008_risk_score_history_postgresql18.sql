-- Migration: 008_risk_score_history_postgresql18.sql
-- Description: Risk Score History table for transaction analytics (Phase 5.2.5)
-- PostgreSQL 18 Beta 1 features: Advanced temporal queries, JSONB optimizations, UUIDv7 support

-- Create risk_score_history table for tracking risk score changes over time
CREATE TABLE risk_score_history (
    id UUID PRIMARY KEY DEFAULT generate_uuidv7(),
    transaction_id UUID NOT NULL REFERENCES transactions(id) ON DELETE CASCADE,
    old_risk_score risk_score NULL,
    new_risk_score risk_score NOT NULL,
    risk_factors JSONB NOT NULL DEFAULT '{}',
    ai_analysis_version VARCHAR(20) DEFAULT 'v1.0',
    change_reason VARCHAR(255),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create indexes for efficient querying
CREATE INDEX idx_risk_score_history_transaction_id ON risk_score_history(transaction_id);
CREATE INDEX idx_risk_score_history_created_at ON risk_score_history(created_at);
CREATE INDEX idx_risk_score_history_new_risk_score ON risk_score_history(new_risk_score);

-- PostgreSQL 18 skip scan optimization index for multi-column queries
CREATE INDEX idx_risk_score_history_transaction_time ON risk_score_history(transaction_id, created_at);

-- Create trigger to automatically log risk score changes
CREATE OR REPLACE FUNCTION log_risk_score_change()
RETURNS TRIGGER AS $$
BEGIN
    -- Only log if risk_score actually changed
    IF OLD.risk_score IS DISTINCT FROM NEW.risk_score THEN
        INSERT INTO risk_score_history (
            transaction_id,
            old_risk_score,
            new_risk_score,
            risk_factors,
            ai_analysis_version,
            change_reason
        )
        VALUES (
            NEW.id,
            OLD.risk_score,
            NEW.risk_score,
            NEW.risk_factors,
            NEW.ai_analysis_version,
            'risk_score_updated'
        );
    END IF;
    
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Create trigger on transactions table to automatically log risk score changes
CREATE TRIGGER trigger_log_risk_score_change
    AFTER UPDATE ON transactions
    FOR EACH ROW
    EXECUTE FUNCTION log_risk_score_change();

-- Create function for risk score trend analysis
CREATE OR REPLACE FUNCTION get_risk_score_trends(days_back INTEGER DEFAULT 7)
RETURNS TABLE (
    trend_date DATE,
    avg_risk_score DECIMAL,
    transaction_count BIGINT,
    high_risk_count BIGINT,
    max_risk_score DECIMAL,
    min_risk_score DECIMAL
) AS $$
BEGIN
    RETURN QUERY
    SELECT 
        DATE_TRUNC('day', t.created_at)::DATE as trend_date,
        AVG(t.risk_score) as avg_risk_score,
        COUNT(*) as transaction_count,
        COUNT(*) FILTER (WHERE t.risk_score > 0.7) as high_risk_count,
        MAX(t.risk_score) as max_risk_score,
        MIN(t.risk_score) as min_risk_score
    FROM transactions t
    WHERE t.created_at >= NOW() - INTERVAL '1 day' * days_back
    GROUP BY DATE_TRUNC('day', t.created_at)
    ORDER BY trend_date ASC;
END;
$$ LANGUAGE plpgsql;

-- Create function for risk score history analysis
CREATE OR REPLACE FUNCTION get_risk_score_history_analysis(transaction_uuid UUID)
RETURNS TABLE (
    change_timestamp TIMESTAMPTZ,
    old_risk_score DECIMAL,
    new_risk_score DECIMAL,
    risk_factors JSONB,
    ai_analysis_version VARCHAR(20),
    change_reason VARCHAR(255)
) AS $$
BEGIN
    RETURN QUERY
    SELECT 
        rsh.created_at as change_timestamp,
        rsh.old_risk_score,
        rsh.new_risk_score,
        rsh.risk_factors,
        rsh.ai_analysis_version,
        rsh.change_reason
    FROM risk_score_history rsh
    WHERE rsh.transaction_id = transaction_uuid
    ORDER BY rsh.created_at ASC;
END;
$$ LANGUAGE plpgsql;

-- Create aggregated view for risk analytics
CREATE VIEW risk_analytics_summary AS
SELECT
    t.id as transaction_id,
    t.user_id,
    t.source_chain,
    t.destination_chain,
    t.amount_in,
    t.risk_score as current_risk_score,
    t.risk_factors as current_risk_factors,
    t.ai_analysis_version as current_ai_version,
    t.status,
    t.created_at as transaction_created_at,
    (
        SELECT COUNT(*) 
        FROM risk_score_history rsh 
        WHERE rsh.transaction_id = t.id
    ) as risk_score_changes,
    (
        SELECT rsh.old_risk_score
        FROM risk_score_history rsh
        WHERE rsh.transaction_id = t.id
        ORDER BY rsh.created_at ASC
        LIMIT 1
    ) as initial_risk_score,
    (
        SELECT rsh.created_at
        FROM risk_score_history rsh
        WHERE rsh.transaction_id = t.id
        ORDER BY rsh.created_at DESC
        LIMIT 1
    ) as last_risk_update
FROM transactions t;

-- Add extended statistics for query optimization (PostgreSQL 18 feature)
CREATE STATISTICS risk_score_stats ON new_risk_score, created_at FROM risk_score_history;
CREATE STATISTICS transaction_risk_stats ON risk_score, source_chain, destination_chain FROM transactions;

-- Note: RLS policies can be added later when auth functions are implemented

-- Add comments for documentation
COMMENT ON TABLE risk_score_history IS 'Historical log of risk score changes for transaction analytics (Phase 5.2.5)';
COMMENT ON COLUMN risk_score_history.transaction_id IS 'Reference to the transaction that had its risk score changed';
COMMENT ON COLUMN risk_score_history.old_risk_score IS 'Previous risk score before the change (NULL for initial score)';
COMMENT ON COLUMN risk_score_history.new_risk_score IS 'New risk score after the change';
COMMENT ON COLUMN risk_score_history.risk_factors IS 'AI analysis factors that contributed to the risk score';
COMMENT ON COLUMN risk_score_history.ai_analysis_version IS 'Version of AI model that generated this risk score';
COMMENT ON COLUMN risk_score_history.change_reason IS 'Reason for the risk score change (e.g., initial_assessment, risk_score_updated)';

COMMENT ON FUNCTION log_risk_score_change() IS 'Trigger function to automatically log risk score changes';
COMMENT ON FUNCTION get_risk_score_trends(INTEGER) IS 'Function to get risk score trends over time for analytics';
COMMENT ON FUNCTION get_risk_score_history_analysis(UUID) IS 'Function to get detailed risk score history for a specific transaction';
COMMENT ON VIEW risk_analytics_summary IS 'Aggregated view for risk analytics with transaction and risk score history data';