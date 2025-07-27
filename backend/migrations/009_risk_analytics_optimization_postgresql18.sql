-- Migration: 009_risk_analytics_optimization_postgresql18.sql
-- Description: Optimize analytics queries for risk scoring (Phase 5.2.5)
-- PostgreSQL 18 Beta 1 features: Advanced indexing, query optimization

-- Create specialized indexes for risk analytics queries
CREATE INDEX idx_transactions_risk_analytics 
    ON transactions(risk_score DESC, created_at DESC) 
    WHERE risk_score IS NOT NULL;

CREATE INDEX idx_transactions_risk_level_time 
    ON transactions(
        (CASE 
            WHEN risk_score <= 0.2 THEN 'very_low'
            WHEN risk_score <= 0.4 THEN 'low'
            WHEN risk_score <= 0.6 THEN 'medium'
            WHEN risk_score <= 0.8 THEN 'high'
            ELSE 'very_high'
        END),
        created_at DESC
    );

CREATE INDEX idx_transactions_status_risk_time 
    ON transactions(status, risk_score DESC, created_at DESC);

CREATE INDEX idx_transactions_user_risk_analytics 
    ON transactions(user_id, risk_score DESC, created_at DESC);

CREATE INDEX idx_transactions_amount_risk_analytics 
    ON transactions(amount_in, risk_score DESC) 
    WHERE risk_score > 0.6;

-- Create partial index for high-risk transactions monitoring
CREATE INDEX idx_transactions_high_risk_monitoring 
    ON transactions(risk_score DESC, created_at DESC, status, user_id)
    WHERE risk_score > 0.6;

-- Create index for risk score history analytics
CREATE INDEX idx_risk_score_history_analytics 
    ON risk_score_history(created_at DESC, new_risk_score DESC);

-- Create functional index for risk categorization
CREATE INDEX idx_transactions_risk_category_func 
    ON transactions(
        (CASE 
            WHEN risk_score <= 0.3 THEN 'low'
            WHEN risk_score <= 0.7 THEN 'medium'
            ELSE 'high'
        END),
        created_at DESC
    );

-- Create materialized view for risk analytics dashboard
CREATE MATERIALIZED VIEW risk_analytics_dashboard AS
SELECT 
    DATE_TRUNC('hour', created_at) as hour_bucket,
    COUNT(*) as total_transactions,
    AVG(risk_score) as avg_risk_score,
    COUNT(*) FILTER (WHERE risk_score > 0.7) as high_risk_count,
    COUNT(*) FILTER (WHERE risk_score > 0.3 AND risk_score <= 0.7) as medium_risk_count,
    COUNT(*) FILTER (WHERE risk_score <= 0.3) as low_risk_count,
    MAX(risk_score) as max_risk_score,
    MIN(risk_score) as min_risk_score,
    COUNT(*) FILTER (WHERE status = 'completed') as completed_count,
    COUNT(*) FILTER (WHERE status = 'failed') as failed_count,
    COUNT(*) FILTER (WHERE status = 'pending') as pending_count,
    SUM(amount_in) as total_volume,
    COUNT(DISTINCT user_id) as unique_users,
    COUNT(*) FILTER (WHERE risk_score > 0.7 AND status = 'completed') as high_risk_completed,
    COUNT(*) FILTER (WHERE risk_score > 0.7 AND status = 'failed') as high_risk_failed
FROM transactions
WHERE created_at >= NOW() - INTERVAL '7 days'
GROUP BY DATE_TRUNC('hour', created_at)
ORDER BY hour_bucket DESC;

-- Create unique index on materialized view
CREATE UNIQUE INDEX idx_risk_analytics_dashboard_hour 
    ON risk_analytics_dashboard(hour_bucket);

-- Create function to refresh materialized view
CREATE OR REPLACE FUNCTION refresh_risk_analytics_dashboard()
RETURNS void AS $$
BEGIN
    REFRESH MATERIALIZED VIEW CONCURRENTLY risk_analytics_dashboard;
END;
$$ LANGUAGE plpgsql;

-- Create optimized function for risk score distribution
CREATE OR REPLACE FUNCTION get_risk_score_distribution_optimized(hours_back INTEGER DEFAULT 24)
RETURNS TABLE (
    risk_level TEXT,
    count BIGINT,
    avg_score DECIMAL,
    total_volume DECIMAL
) AS $$
BEGIN
    RETURN QUERY
    SELECT 
        CASE 
            WHEN t.risk_score <= 0.2 THEN 'very_low'
            WHEN t.risk_score <= 0.4 THEN 'low'
            WHEN t.risk_score <= 0.6 THEN 'medium'
            WHEN t.risk_score <= 0.8 THEN 'high'
            ELSE 'very_high'
        END as risk_level,
        COUNT(*) as count,
        AVG(t.risk_score) as avg_score,
        SUM(t.amount_in) as total_volume
    FROM transactions t
    WHERE t.created_at >= NOW() - INTERVAL '1 hour' * hours_back
    GROUP BY 
        CASE 
            WHEN t.risk_score <= 0.2 THEN 'very_low'
            WHEN t.risk_score <= 0.4 THEN 'low'
            WHEN t.risk_score <= 0.6 THEN 'medium'
            WHEN t.risk_score <= 0.8 THEN 'high'
            ELSE 'very_high'
        END
    ORDER BY avg_score ASC;
END;
$$ LANGUAGE plpgsql;

-- Create optimized function for top risky transactions
CREATE OR REPLACE FUNCTION get_top_risky_transactions_optimized(limit_count INTEGER DEFAULT 50)
RETURNS TABLE (
    id UUID,
    user_id UUID,
    source_chain VARCHAR(50),
    destination_chain VARCHAR(50),
    amount_in DECIMAL,
    risk_score risk_score,
    status VARCHAR(50),
    created_at TIMESTAMPTZ,
    ethereum_address VARCHAR(42),
    near_account_id VARCHAR(64)
) AS $$
BEGIN
    RETURN QUERY
    SELECT 
        t.id,
        t.user_id,
        t.source_chain,
        t.destination_chain,
        t.amount_in,
        t.risk_score,
        t.status,
        t.created_at,
        eth_auth.wallet_address as ethereum_address,
        near_auth.wallet_address as near_account_id
    FROM transactions t
    JOIN users u ON t.user_id = u.id
    LEFT JOIN user_auth_methods eth_auth ON t.user_id = eth_auth.user_id AND eth_auth.chain_type = 'ethereum'
    LEFT JOIN user_auth_methods near_auth ON t.user_id = near_auth.user_id AND near_auth.chain_type = 'near'
    WHERE t.risk_score > 0.6
    ORDER BY t.risk_score DESC, t.created_at DESC
    LIMIT limit_count;
END;
$$ LANGUAGE plpgsql;

-- Create view for real-time risk monitoring
CREATE VIEW real_time_risk_monitoring AS
SELECT 
    t.id,
    t.user_id,
    t.source_chain,
    t.destination_chain,
    t.amount_in,
    t.risk_score,
    t.status,
    t.created_at,
    eth_auth.wallet_address as ethereum_address,
    near_auth.wallet_address as near_account_id,
    CASE 
        WHEN t.risk_score <= 0.3 THEN 'low'
        WHEN t.risk_score <= 0.7 THEN 'medium'
        ELSE 'high'
    END as risk_category,
    CASE 
        WHEN t.created_at >= NOW() - INTERVAL '1 hour' THEN 'recent'
        WHEN t.created_at >= NOW() - INTERVAL '24 hours' THEN 'today'
        ELSE 'older'
    END as time_category
FROM transactions t
JOIN users u ON t.user_id = u.id
LEFT JOIN user_auth_methods eth_auth ON t.user_id = eth_auth.user_id AND eth_auth.chain_type = 'ethereum'
LEFT JOIN user_auth_methods near_auth ON t.user_id = near_auth.user_id AND near_auth.chain_type = 'near'
WHERE t.risk_score > 0.5
ORDER BY t.risk_score DESC, t.created_at DESC;

-- Add extended statistics for better query planning
CREATE STATISTICS risk_analytics_stats (dependencies) 
ON risk_score, status, created_at FROM transactions;

CREATE STATISTICS risk_user_stats (dependencies) 
ON user_id, risk_score, amount_in FROM transactions;

CREATE STATISTICS risk_chain_stats (dependencies) 
ON source_chain, destination_chain, risk_score FROM transactions;

-- Create function for automated index maintenance
CREATE OR REPLACE FUNCTION maintain_risk_analytics_indexes()
RETURNS void AS $$
BEGIN
    -- Refresh materialized view
    PERFORM refresh_risk_analytics_dashboard();
    
    -- Update statistics
    ANALYZE transactions;
    ANALYZE risk_score_history;
    ANALYZE users;
    
    -- Log maintenance
    INSERT INTO audit_logs (event_type, event_data, created_at)
    VALUES ('risk_analytics_maintenance', 
            json_build_object('timestamp', NOW(), 'action', 'indexes_maintained'),
            NOW());
END;
$$ LANGUAGE plpgsql;

-- Add comments for documentation
COMMENT ON MATERIALIZED VIEW risk_analytics_dashboard IS 'Pre-computed risk analytics for dashboard performance (Phase 5.2.5)';
COMMENT ON FUNCTION get_risk_score_distribution_optimized(INTEGER) IS 'Optimized function for risk score distribution charts';
COMMENT ON FUNCTION get_top_risky_transactions_optimized(INTEGER) IS 'Optimized function for top risky transactions monitoring';
COMMENT ON VIEW real_time_risk_monitoring IS 'Real-time view for risk monitoring dashboard';
COMMENT ON FUNCTION maintain_risk_analytics_indexes() IS 'Automated maintenance function for risk analytics indexes';

-- Schedule maintenance (would be done via cron job or pg_cron in production)
-- This is just documentation for setup
-- SELECT cron.schedule('refresh-risk-analytics', '*/15 * * * *', 'SELECT maintain_risk_analytics_indexes();');