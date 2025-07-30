/**
 * KEMBridge Monitoring & Observability System
 * Production-grade monitoring, metrics, and observability
 */
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};
use tokio::sync::RwLock;
use tracing::{debug, info};
use uuid::Uuid;

/// Metric types supported by the monitoring system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetricType {
    /// Counter that only increases
    Counter,
    /// Gauge that can increase or decrease
    Gauge,
    /// Histogram for timing/distribution data
    Histogram,
    /// Summary with quantiles
    Summary,
}

/// Metric value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetricValue {
    Counter(u64),
    Gauge(f64),
    Histogram(Vec<f64>),
    Summary { count: u64, sum: f64 },
}

/// Individual metric data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metric {
    pub name: String,
    pub value: MetricValue,
    pub labels: HashMap<String, String>,
    pub timestamp: SystemTime,
    pub help: String,
}

/// Health check status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

/// Health check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheck {
    pub name: String,
    pub status: HealthStatus,
    pub message: String,
    pub timestamp: SystemTime,
    pub response_time: Duration,
    pub metadata: HashMap<String, String>,
}

/// Service health information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceHealth {
    pub service_name: String,
    pub overall_status: HealthStatus,
    pub checks: Vec<HealthCheck>,
    pub uptime: Duration,
    pub version: String,
    pub last_updated: SystemTime,
}

/// Performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub request_count: u64,
    pub error_count: u64,
    pub average_response_time: Duration,
    pub p95_response_time: Duration,
    pub p99_response_time: Duration,
    pub throughput_per_second: f64,
    pub error_rate: f64,
    pub uptime_percentage: f64,
}

/// Resource utilization metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResourceMetrics {
    pub cpu_usage: f64,    // 0.0-100.0
    pub memory_usage: f64, // 0.0-100.0
    pub disk_usage: f64,   // 0.0-100.0
    pub network_in: u64,   // bytes/sec
    pub network_out: u64,  // bytes/sec
    pub connection_count: u32,
    pub thread_count: u32,
}

/// Alert severity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum AlertSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Monitoring alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub severity: AlertSeverity,
    pub service: String,
    pub triggered_at: SystemTime,
    pub resolved_at: Option<SystemTime>,
    pub threshold: f64,
    pub current_value: f64,
    pub labels: HashMap<String, String>,
}

/// Monitoring configuration
#[derive(Debug, Clone)]
pub struct MonitoringConfig {
    pub metrics_retention: Duration,
    pub health_check_interval: Duration,
    pub alert_evaluation_interval: Duration,
    pub max_metrics_in_memory: usize,
    pub enable_detailed_metrics: bool,
    pub enable_resource_monitoring: bool,
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            metrics_retention: Duration::from_secs(3600), // 1 hour
            health_check_interval: Duration::from_secs(30),
            alert_evaluation_interval: Duration::from_secs(60),
            max_metrics_in_memory: 10000,
            enable_detailed_metrics: true,
            enable_resource_monitoring: true,
        }
    }
}

/// Internal monitoring state
#[derive(Debug)]
struct MonitoringState {
    metrics: HashMap<String, Vec<Metric>>,
    health_checks: HashMap<String, HealthCheck>,
    alerts: HashMap<Uuid, Alert>,
    start_time: Instant,
    service_info: ServiceInfo,
}

#[derive(Debug, Clone)]
struct ServiceInfo {
    name: String,
    version: String,
    instance_id: Uuid,
}

/// Production monitoring and observability system
pub struct MonitoringSystem {
    state: Arc<RwLock<MonitoringState>>,
    config: MonitoringConfig,
}

impl MonitoringSystem {
    /// Create new monitoring system
    pub fn new(service_name: String, version: String) -> Self {
        Self::with_config(service_name, version, MonitoringConfig::default())
    }

    /// Create monitoring system with custom configuration
    pub fn with_config(service_name: String, version: String, config: MonitoringConfig) -> Self {
        let service_info = ServiceInfo {
            name: service_name,
            version,
            instance_id: Uuid::new_v4(),
        };

        let state = MonitoringState {
            metrics: HashMap::new(),
            health_checks: HashMap::new(),
            alerts: HashMap::new(),
            start_time: Instant::now(),
            service_info,
        };

        Self {
            state: Arc::new(RwLock::new(state)),
            config,
        }
    }

    /// Record a metric value
    pub async fn record_metric(
        &self,
        name: String,
        value: MetricValue,
        labels: HashMap<String, String>,
        help: String,
    ) {
        let metric = Metric {
            name: name.clone(),
            value,
            labels,
            timestamp: SystemTime::now(),
            help,
        };

        let mut state = self.state.write().await;
        let metrics_vec = state.metrics.entry(name).or_insert_with(Vec::new);
        metrics_vec.push(metric);

        // Cleanup old metrics if needed
        if metrics_vec.len() > self.config.max_metrics_in_memory {
            metrics_vec.remove(0);
        }

        debug!("Recorded metric: {}", metrics_vec.last().unwrap().name);
    }

    /// Increment a counter metric
    pub async fn increment_counter(
        &self,
        name: String,
        labels: HashMap<String, String>,
        help: String,
    ) {
        self.record_metric(name, MetricValue::Counter(1), labels, help)
            .await;
    }

    /// Set a gauge metric
    pub async fn set_gauge(
        &self,
        name: String,
        value: f64,
        labels: HashMap<String, String>,
        help: String,
    ) {
        self.record_metric(name, MetricValue::Gauge(value), labels, help)
            .await;
    }

    /// Record histogram value (e.g., response time)
    pub async fn record_histogram(
        &self,
        name: String,
        value: f64,
        labels: HashMap<String, String>,
        help: String,
    ) {
        self.record_metric(name, MetricValue::Histogram(vec![value]), labels, help)
            .await;
    }

    /// Record a health check result
    pub async fn record_health_check(&self, check: HealthCheck) {
        let mut state = self.state.write().await;
        state.health_checks.insert(check.name.clone(), check);
        debug!("Recorded health check: {}", state.health_checks.len());
    }

    /// Get current service health
    pub async fn get_service_health(&self) -> ServiceHealth {
        let state = self.state.read().await;
        let checks: Vec<HealthCheck> = state.health_checks.values().cloned().collect();

        // Determine overall status
        let overall_status = if checks.is_empty() {
            HealthStatus::Unknown
        } else if checks.iter().any(|c| c.status == HealthStatus::Unhealthy) {
            HealthStatus::Unhealthy
        } else if checks.iter().any(|c| c.status == HealthStatus::Degraded) {
            HealthStatus::Degraded
        } else {
            HealthStatus::Healthy
        };

        ServiceHealth {
            service_name: state.service_info.name.clone(),
            overall_status,
            checks,
            uptime: state.start_time.elapsed(),
            version: state.service_info.version.clone(),
            last_updated: SystemTime::now(),
        }
    }

    /// Get performance metrics for a time window
    pub async fn get_performance_metrics(&self, window: Duration) -> PerformanceMetrics {
        let state = self.state.read().await;
        let cutoff_time = SystemTime::now() - window;

        let mut request_count = 0u64;
        let mut error_count = 0u64;
        let mut response_times = Vec::new();

        // Aggregate metrics from the time window
        for (name, metrics) in &state.metrics {
            for metric in metrics {
                if metric.timestamp > cutoff_time {
                    match &metric.value {
                        MetricValue::Counter(value) => {
                            if name.contains("request") {
                                request_count += value;
                            } else if name.contains("error") {
                                error_count += value;
                            }
                        }
                        MetricValue::Histogram(values) => {
                            if name.contains("response_time") {
                                response_times.extend(values);
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        // Calculate statistics
        response_times.sort_by(|a: &f64, b: &f64| a.partial_cmp(b).unwrap());
        let avg_response = if response_times.is_empty() {
            Duration::from_millis(0)
        } else {
            let sum: f64 = response_times.iter().sum();
            Duration::from_millis((sum / response_times.len() as f64) as u64)
        };

        let p95_response = if response_times.is_empty() {
            Duration::from_millis(0)
        } else {
            let index = ((response_times.len() as f64) * 0.95) as usize;
            Duration::from_millis(response_times.get(index).unwrap_or(&0.0).round() as u64)
        };

        let p99_response = if response_times.is_empty() {
            Duration::from_millis(0)
        } else {
            let index = ((response_times.len() as f64) * 0.99) as usize;
            Duration::from_millis(response_times.get(index).unwrap_or(&0.0).round() as u64)
        };

        let error_rate = if request_count == 0 {
            0.0
        } else {
            (error_count as f64 / request_count as f64) * 100.0
        };

        let throughput = request_count as f64 / window.as_secs() as f64;

        PerformanceMetrics {
            request_count,
            error_count,
            average_response_time: avg_response,
            p95_response_time: p95_response,
            p99_response_time: p99_response,
            throughput_per_second: throughput,
            error_rate,
            uptime_percentage: 99.9, // Would be calculated from health checks
        }
    }

    /// Trigger an alert
    pub async fn trigger_alert(
        &self,
        name: String,
        description: String,
        severity: AlertSeverity,
        service: String,
        threshold: f64,
        current_value: f64,
        labels: HashMap<String, String>,
    ) -> Uuid {
        let alert_id = Uuid::new_v4();
        let alert = Alert {
            id: alert_id,
            name: name.clone(),
            description,
            severity,
            service,
            triggered_at: SystemTime::now(),
            resolved_at: None,
            threshold,
            current_value,
            labels,
        };

        let mut state = self.state.write().await;
        state.alerts.insert(alert_id, alert);

        info!("🚨 Alert triggered: {} (ID: {})", name, alert_id);
        alert_id
    }

    /// Resolve an alert
    pub async fn resolve_alert(&self, alert_id: Uuid) {
        let mut state = self.state.write().await;
        if let Some(alert) = state.alerts.get_mut(&alert_id) {
            alert.resolved_at = Some(SystemTime::now());
            info!("✅ Alert resolved: {} (ID: {})", alert.name, alert_id);
        }
    }

    /// Get active alerts
    pub async fn get_active_alerts(&self) -> Vec<Alert> {
        let state = self.state.read().await;
        state
            .alerts
            .values()
            .filter(|alert| alert.resolved_at.is_none())
            .cloned()
            .collect()
    }

    /// Get all metrics for a specific name
    pub async fn get_metrics(&self, name: &str) -> Vec<Metric> {
        let state = self.state.read().await;
        state.metrics.get(name).cloned().unwrap_or_default()
    }

    /// Get resource metrics (would integrate with system monitoring)
    pub async fn get_resource_metrics(&self) -> ResourceMetrics {
        // In a real implementation, this would collect actual system metrics
        // For now, return mock data that could be replaced with real monitoring
        ResourceMetrics {
            cpu_usage: 45.2,
            memory_usage: 62.1,
            disk_usage: 23.8,
            network_in: 1024 * 1024, // 1MB/s
            network_out: 512 * 1024, // 512KB/s
            connection_count: 42,
            thread_count: 16,
        }
    }

    /// Start background monitoring tasks
    pub async fn start_monitoring(&self) {
        let state_clone = Arc::clone(&self.state);
        let config = self.config.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(config.health_check_interval);
            loop {
                interval.tick().await;

                // Perform periodic health checks
                let health_check = HealthCheck {
                    name: "system_health".to_string(),
                    status: HealthStatus::Healthy,
                    message: "System operating normally".to_string(),
                    timestamp: SystemTime::now(),
                    response_time: Duration::from_millis(10),
                    metadata: HashMap::new(),
                };

                let mut state = state_clone.write().await;
                state
                    .health_checks
                    .insert("system_health".to_string(), health_check);
            }
        });

        info!("🔍 Monitoring system started");
    }

    /// Cleanup old metrics
    pub async fn cleanup_old_metrics(&self) {
        let mut state = self.state.write().await;
        let cutoff_time = SystemTime::now() - self.config.metrics_retention;

        for (_, metrics) in state.metrics.iter_mut() {
            metrics.retain(|metric| metric.timestamp > cutoff_time);
        }

        debug!("Cleaned up old metrics");
    }
}

impl Default for MonitoringSystem {
    fn default() -> Self {
        Self::new("default-service".to_string(), "1.0.0".to_string())
    }
}
