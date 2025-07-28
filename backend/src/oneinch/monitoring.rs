// src/oneinch/monitoring.rs - Performance monitoring and metrics

use async_trait::async_trait;
use std::time::{Duration, Instant};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use serde::{Serialize, Deserialize};

/// Performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub operation: String,
    pub duration: Duration,
    pub success: bool,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub metadata: HashMap<String, String>,
}

/// Aggregated statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedStats {
    pub operation: String,
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub average_duration: Duration,
    pub min_duration: Duration,
    pub max_duration: Duration,
    pub success_rate: f64,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

/// Trait for performance monitoring
#[async_trait]
pub trait PerformanceMonitor: Send + Sync {
    /// Record a metric
    async fn record_metric(&self, metric: PerformanceMetrics);
    
    /// Get statistics for an operation
    async fn get_stats(&self, operation: &str) -> Option<AggregatedStats>;
    
    /// Get all statistics
    async fn get_all_stats(&self) -> Vec<AggregatedStats>;
    
    /// Clean up old metrics
    async fn cleanup_old_metrics(&self, older_than: Duration);
}

/// Simple in-memory performance monitor
pub struct InMemoryPerformanceMonitor {
    metrics: Arc<Mutex<Vec<PerformanceMetrics>>>,
    stats_cache: Arc<Mutex<HashMap<String, AggregatedStats>>>,
    max_metrics: usize,
}

impl InMemoryPerformanceMonitor {
    pub fn new(max_metrics: usize) -> Self {
        Self {
            metrics: Arc::new(Mutex::new(Vec::new())),
            stats_cache: Arc::new(Mutex::new(HashMap::new())),
            max_metrics,
        }
    }

    /// Recalculate statistics for an operation
    fn recalculate_stats(&self, operation: &str) {
        let metrics = self.metrics.lock().unwrap();
        let operation_metrics: Vec<_> = metrics
            .iter()
            .filter(|m| m.operation == operation)
            .collect();

        if operation_metrics.is_empty() {
            return;
        }

        let total_requests = operation_metrics.len() as u64;
        let successful_requests = operation_metrics.iter().filter(|m| m.success).count() as u64;
        let failed_requests = total_requests - successful_requests;

        let durations: Vec<Duration> = operation_metrics.iter().map(|m| m.duration).collect();
        let total_duration: Duration = durations.iter().sum();
        let average_duration = total_duration / durations.len() as u32;
        let min_duration = *durations.iter().min().unwrap();
        let max_duration = *durations.iter().max().unwrap();

        let success_rate = if total_requests > 0 {
            successful_requests as f64 / total_requests as f64
        } else {
            0.0
        };

        let stats = AggregatedStats {
            operation: operation.to_string(),
            total_requests,
            successful_requests,
            failed_requests,
            average_duration,
            min_duration,
            max_duration,
            success_rate,
            last_updated: chrono::Utc::now(),
        };

        let mut cache = self.stats_cache.lock().unwrap();
        cache.insert(operation.to_string(), stats);
    }
}

#[async_trait]
impl PerformanceMonitor for InMemoryPerformanceMonitor {
    async fn record_metric(&self, metric: PerformanceMetrics) {
        {
            let mut metrics = self.metrics.lock().unwrap();
            metrics.push(metric.clone());

            // Limit the number of metrics
            if metrics.len() > self.max_metrics {
                metrics.remove(0);
            }
        }

        // Recalculate statistics
        self.recalculate_stats(&metric.operation);
    }

    async fn get_stats(&self, operation: &str) -> Option<AggregatedStats> {
        let cache = self.stats_cache.lock().unwrap();
        cache.get(operation).cloned()
    }

    async fn get_all_stats(&self) -> Vec<AggregatedStats> {
        let cache = self.stats_cache.lock().unwrap();
        cache.values().cloned().collect()
    }

    async fn cleanup_old_metrics(&self, older_than: Duration) {
        let cutoff = chrono::Utc::now() - chrono::Duration::from_std(older_than).unwrap();
        
        let mut metrics = self.metrics.lock().unwrap();
        metrics.retain(|m| m.timestamp > cutoff);

        // Recalculate statistics for all operations
        let operations: Vec<String> = metrics.iter().map(|m| m.operation.clone()).collect();
        drop(metrics);

        for operation in operations.into_iter().collect::<std::collections::HashSet<_>>() {
            self.recalculate_stats(&operation);
        }
    }
}

/// Decorator for automatic function monitoring
pub struct MonitoredOperation<T> {
    monitor: Arc<dyn PerformanceMonitor>,
    operation_name: String,
    _phantom: std::marker::PhantomData<T>,
}

impl<T> MonitoredOperation<T> {
    pub fn new(monitor: Arc<dyn PerformanceMonitor>, operation_name: String) -> Self {
        Self {
            monitor,
            operation_name,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Execute operation with monitoring
    pub async fn execute<F, Fut>(&self, operation: F) -> Result<T, Box<dyn std::error::Error + Send + Sync>>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T, Box<dyn std::error::Error + Send + Sync>>>,
    {
        let start = Instant::now();
        let result = operation().await;
        let duration = start.elapsed();

        let mut metadata = HashMap::new();
        metadata.insert("duration_ms".to_string(), duration.as_millis().to_string());

        let metric = PerformanceMetrics {
            operation: self.operation_name.clone(),
            duration,
            success: result.is_ok(),
            timestamp: chrono::Utc::now(),
            metadata,
        };

        self.monitor.record_metric(metric).await;
        result
    }

    /// Execute operation with additional metadata
    pub async fn execute_with_metadata<F, Fut>(
        &self,
        operation: F,
        metadata: HashMap<String, String>,
    ) -> Result<T, Box<dyn std::error::Error + Send + Sync>>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T, Box<dyn std::error::Error + Send + Sync>>>,
    {
        let start = Instant::now();
        let result = operation().await;
        let duration = start.elapsed();

        let mut final_metadata = metadata;
        final_metadata.insert("duration_ms".to_string(), duration.as_millis().to_string());

        let metric = PerformanceMetrics {
            operation: self.operation_name.clone(),
            duration,
            success: result.is_ok(),
            timestamp: chrono::Utc::now(),
            metadata: final_metadata,
        };

        self.monitor.record_metric(metric).await;
        result
    }
}

/// Trait for components that support monitoring
pub trait Monitorable {
    fn get_monitor(&self) -> Option<Arc<dyn PerformanceMonitor>>;
    
    /// Create a monitored operation
    fn create_monitored_operation<T>(&self, operation_name: &str) -> Option<MonitoredOperation<T>> {
        self.get_monitor().map(|monitor| {
            MonitoredOperation::new(monitor, operation_name.to_string())
        })
    }
}

/// Alerts based on metrics
#[derive(Debug, Clone)]
pub struct Alert {
    pub level: AlertLevel,
    pub operation: String,
    pub message: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AlertLevel {
    Info,
    Warning,
    Error,
    Critical,
}

/// Rules for alerts
pub struct AlertRule {
    pub operation_pattern: String,
    pub condition: AlertCondition,
    pub level: AlertLevel,
    pub message_template: String,
}

#[derive(Debug, Clone)]
pub enum AlertCondition {
    SuccessRateBelow(f64),
    AverageDurationAbove(Duration),
    ErrorRateAbove(f64),
    TotalRequestsAbove(u64),
}

/// Alert system
pub struct AlertSystem {
    rules: Vec<AlertRule>,
    monitor: Arc<dyn PerformanceMonitor>,
}

impl AlertSystem {
    pub fn new(monitor: Arc<dyn PerformanceMonitor>) -> Self {
        Self {
            rules: Vec::new(),
            monitor,
        }
    }

    pub fn add_rule(&mut self, rule: AlertRule) {
        self.rules.push(rule);
    }

    /// Check all rules and generate alerts
    pub async fn check_alerts(&self) -> Vec<Alert> {
        let mut alerts = Vec::new();
        let all_stats = self.monitor.get_all_stats().await;

        for stats in all_stats {
            for rule in &self.rules {
                if self.matches_pattern(&stats.operation, &rule.operation_pattern) {
                    if let Some(alert) = self.check_rule(&stats, rule) {
                        alerts.push(alert);
                    }
                }
            }
        }

        alerts
    }

    fn matches_pattern(&self, operation: &str, pattern: &str) -> bool {
        // Simple pattern matching (can be extended to regex)
        pattern == "*" || operation.contains(pattern)
    }

    fn check_rule(&self, stats: &AggregatedStats, rule: &AlertRule) -> Option<Alert> {
        let triggered = match &rule.condition {
            AlertCondition::SuccessRateBelow(threshold) => stats.success_rate < *threshold,
            AlertCondition::AverageDurationAbove(threshold) => stats.average_duration > *threshold,
            AlertCondition::ErrorRateAbove(threshold) => {
                let error_rate = stats.failed_requests as f64 / stats.total_requests as f64;
                error_rate > *threshold
            },
            AlertCondition::TotalRequestsAbove(threshold) => stats.total_requests > *threshold,
        };

        if triggered {
            let mut metadata = HashMap::new();
            metadata.insert("success_rate".to_string(), stats.success_rate.to_string());
            metadata.insert("average_duration_ms".to_string(), stats.average_duration.as_millis().to_string());
            metadata.insert("total_requests".to_string(), stats.total_requests.to_string());

            Some(Alert {
                level: rule.level.clone(),
                operation: stats.operation.clone(),
                message: rule.message_template.replace("{operation}", &stats.operation),
                timestamp: chrono::Utc::now(),
                metadata,
            })
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_performance_monitor() {
        let monitor = InMemoryPerformanceMonitor::new(100);

        // Record several metrics
        let metric1 = PerformanceMetrics {
            operation: "test_op".to_string(),
            duration: Duration::from_millis(100),
            success: true,
            timestamp: chrono::Utc::now(),
            metadata: HashMap::new(),
        };

        let metric2 = PerformanceMetrics {
            operation: "test_op".to_string(),
            duration: Duration::from_millis(200),
            success: false,
            timestamp: chrono::Utc::now(),
            metadata: HashMap::new(),
        };

        monitor.record_metric(metric1).await;
        monitor.record_metric(metric2).await;

        // Check statistics
        let stats = monitor.get_stats("test_op").await.unwrap();
        assert_eq!(stats.total_requests, 2);
        assert_eq!(stats.successful_requests, 1);
        assert_eq!(stats.failed_requests, 1);
        assert_eq!(stats.success_rate, 0.5);
    }

    #[tokio::test]
    async fn test_monitored_operation() {
        let monitor = Arc::new(InMemoryPerformanceMonitor::new(100));
        let monitored_op = MonitoredOperation::<String>::new(monitor.clone(), "test_operation".to_string());

        let result = monitored_op.execute(|| async {
            sleep(Duration::from_millis(10)).await;
            Ok("success".to_string())
        }).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "success");

        // Check that the metric was recorded
        let stats = monitor.get_stats("test_operation").await.unwrap();
        assert_eq!(stats.total_requests, 1);
        assert_eq!(stats.successful_requests, 1);
    }

    #[tokio::test]
    async fn test_alert_system() {
        let monitor = Arc::new(InMemoryPerformanceMonitor::new(100));
        let mut alert_system = AlertSystem::new(monitor.clone());

        // Add alert rule
        alert_system.add_rule(AlertRule {
            operation_pattern: "slow_op".to_string(),
            condition: AlertCondition::AverageDurationAbove(Duration::from_millis(150)),
            level: AlertLevel::Warning,
            message_template: "Operation {operation} is too slow".to_string(),
        });

        // Record slow operation
        let slow_metric = PerformanceMetrics {
            operation: "slow_op".to_string(),
            duration: Duration::from_millis(200),
            success: true,
            timestamp: chrono::Utc::now(),
            metadata: HashMap::new(),
        };

        monitor.record_metric(slow_metric).await;

        // Check alerts
        let alerts = alert_system.check_alerts().await;
        assert_eq!(alerts.len(), 1);
        assert_eq!(alerts[0].level, AlertLevel::Warning);
        assert!(alerts[0].message.contains("slow_op"));
    }
}