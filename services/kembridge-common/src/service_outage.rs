/**
 * KEMBridge Service Outage Handling System
 * Graceful degradation and fallback mechanisms for service unavailability
 */

use crate::errors::{ServiceError, ServiceResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};
use tokio::sync::{Mutex, RwLock};
use tracing::{info, warn, debug};
use uuid::Uuid;

// Service health status
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ServiceHealth {
    Healthy,
    Degraded { issues: Vec<String> },
    Unavailable { reason: String, since: SystemTime },
    Unknown,
}

// Degradation levels
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DegradationLevel {
    None,
    Minimal,     // Minor performance impact
    Moderate,    // Some features disabled
    Severe,      // Major functionality limited
    Critical,    // Core service unavailable
}

// Fallback strategies for different services
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FallbackStrategy {
    /// Use cached data with staleness warning
    CachedData { max_age: Duration },
    /// Use alternative service
    AlternativeService { service_name: String },
    /// Use static/default data
    StaticData { data: String },
    /// Graceful feature disable
    DisableFeature { feature_name: String },
    /// Queue requests for later processing
    QueueRequests { max_queue_size: usize },
    /// Return degraded response
    DegradedResponse { message: String },
    /// Fail fast with clear error message
    FailFast { error_message: String },
}

// Service configuration for outage handling
#[derive(Debug, Clone)]
pub struct ServiceOutageConfig {
    pub service_name: String,
    pub health_check_url: String,
    pub health_check_interval: Duration,
    pub failure_threshold: u32,
    pub recovery_threshold: u32,
    pub fallback_strategy: FallbackStrategy,
    pub enable_auto_recovery: bool,
    pub max_outage_duration: Duration,
}

// Service status tracking
#[derive(Debug, Clone)]
pub struct ServiceStatus {
    pub service_name: String,
    pub health: ServiceHealth,
    pub degradation_level: DegradationLevel,
    pub last_health_check: Instant,
    pub failure_count: u32,
    pub success_count: u32,
    pub outage_started: Option<Instant>,
    pub last_error: Option<String>,
    pub fallback_active: bool,
}

// Outage event for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutageEvent {
    pub service_name: String,
    pub event_type: OutageEventType,
    pub timestamp: SystemTime,
    pub details: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum OutageEventType {
    ServiceDown,
    ServiceRecovered,
    FallbackActivated,
    FallbackDeactivated,
    DegradationLevelChanged,
}

// Main service outage handler
#[derive(Debug)]
pub struct ServiceOutageHandler {
    // Service configurations
    configs: Arc<RwLock<HashMap<String, ServiceOutageConfig>>>,
    // Current service status
    statuses: Arc<RwLock<HashMap<String, ServiceStatus>>>,
    // Outage event history
    events: Arc<Mutex<Vec<OutageEvent>>>,
    // Health check tasks
    health_check_handles: Arc<Mutex<HashMap<String, tokio::task::JoinHandle<()>>>>,
    // Fallback data cache
    fallback_cache: Arc<RwLock<HashMap<String, (String, Instant)>>>,
    // Request queues for unavailable services
    request_queues: Arc<RwLock<HashMap<String, Vec<QueuedRequest>>>>,
}

#[derive(Debug, Clone)]
struct QueuedRequest {
    id: Uuid,
    service_name: String,
    request_data: String,
    queued_at: Instant,
    retry_count: u32,
}

impl ServiceOutageHandler {
    pub fn new() -> Self {
        Self {
            configs: Arc::new(RwLock::new(HashMap::new())),
            statuses: Arc::new(RwLock::new(HashMap::new())),
            events: Arc::new(Mutex::new(Vec::new())),
            health_check_handles: Arc::new(Mutex::new(HashMap::new())),
            fallback_cache: Arc::new(RwLock::new(HashMap::new())),
            request_queues: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a service for outage monitoring
    pub async fn register_service(&self, config: ServiceOutageConfig) {
        let service_name = config.service_name.clone();
        
        info!("Registering service for outage monitoring: {}", service_name);

        // Store configuration
        {
            let mut configs = self.configs.write().await;
            configs.insert(service_name.clone(), config.clone());
        }

        // Initialize status
        {
            let mut statuses = self.statuses.write().await;
            statuses.insert(service_name.clone(), ServiceStatus {
                service_name: service_name.clone(),
                health: ServiceHealth::Unknown,
                degradation_level: DegradationLevel::None,
                last_health_check: Instant::now(),
                failure_count: 0,
                success_count: 0,
                outage_started: None,
                last_error: None,
                fallback_active: false,
            });
        }

        // Start health check task
        self.start_health_check_task(config).await;
    }

    /// Start health check task for a service
    async fn start_health_check_task(&self, config: ServiceOutageConfig) {
        let service_name = config.service_name.clone();
        let service_name_for_handle = service_name.clone();
        let health_check_url = config.health_check_url.clone();
        let interval = config.health_check_interval;
        
        let statuses = Arc::clone(&self.statuses);
        let events = Arc::clone(&self.events);
        
        let handle = tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(interval);
            
            loop {
                interval_timer.tick().await;
                
                debug!("Performing health check for service: {}", service_name);
                
                // Perform health check
                let health_result = Self::perform_health_check(&health_check_url).await;
                
                // Update service status
                {
                    let mut statuses = statuses.write().await;
                    if let Some(status) = statuses.get_mut(&service_name) {
                        status.last_health_check = Instant::now();
                        
                        match health_result {
                            Ok(_) => {
                                status.success_count += 1;
                                
                                // Service recovered
                                if matches!(status.health, ServiceHealth::Unavailable { .. }) {
                                    info!("Service {} has recovered", service_name);
                                    status.health = ServiceHealth::Healthy;
                                    status.degradation_level = DegradationLevel::None;
                                    status.outage_started = None;
                                    status.fallback_active = false;
                                    
                                    // Log recovery event
                                    let event = OutageEvent {
                                        service_name: service_name.clone(),
                                        event_type: OutageEventType::ServiceRecovered,
                                        timestamp: SystemTime::now(),
                                        details: HashMap::new(),
                                    };
                                    
                                    if let Ok(mut events) = events.try_lock() {
                                        events.push(event);
                                    }
                                }
                            }
                            Err(error) => {
                                status.failure_count += 1;
                                status.last_error = Some(error.clone());
                                
                                // Check if we should mark service as unavailable
                                if status.failure_count >= config.failure_threshold &&
                                   !matches!(status.health, ServiceHealth::Unavailable { .. }) {
                                    
                                    warn!("Service {} marked as unavailable after {} failures", 
                                          service_name, status.failure_count);
                                    
                                    let now = Instant::now();
                                    status.health = ServiceHealth::Unavailable {
                                        reason: error.clone(),
                                        since: SystemTime::now(),
                                    };
                                    status.degradation_level = DegradationLevel::Critical;
                                    status.outage_started = Some(now);
                                    
                                    // Log outage event
                                    let mut details = HashMap::new();
                                    details.insert("reason".to_string(), error);
                                    details.insert("failure_count".to_string(), status.failure_count.to_string());
                                    
                                    let event = OutageEvent {
                                        service_name: service_name.clone(),
                                        event_type: OutageEventType::ServiceDown,
                                        timestamp: SystemTime::now(),
                                        details,
                                    };
                                    
                                    if let Ok(mut events) = events.try_lock() {
                                        events.push(event);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        });

        // Store handle for cleanup
        {
            let mut handles = self.health_check_handles.lock().await;
            handles.insert(service_name_for_handle, handle);
        }
    }

    /// Perform HTTP health check
    async fn perform_health_check(url: &str) -> Result<(), String> {
        debug!("Health check request to: {}", url);
        
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .map_err(|e| format!("HTTP client creation failed: {}", e))?;

        let response = client
            .get(url)
            .send()
            .await
            .map_err(|e| format!("Health check request failed: {}", e))?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(format!("Health check returned status: {}", response.status()))
        }
    }

    /// Handle service request with fallback if service is unavailable
    pub async fn handle_request(&self, service_name: &str, request_data: String) -> ServiceResult<String> {
        // Check service status
        let status = {
            let statuses = self.statuses.read().await;
            statuses.get(service_name).cloned()
        };

        let status = match status {
            Some(s) => s,
            None => {
                return Err(ServiceError::NotFound {
                    resource: format!("Service: {}", service_name),
                });
            }
        };

        // If service is healthy, proceed normally
        if matches!(status.health, ServiceHealth::Healthy) {
            return Ok(format!("Normal response from {}", service_name));
        }

        // Service is unavailable, apply fallback strategy
        let config = {
            let configs = self.configs.read().await;
            configs.get(service_name).cloned()
        };

        let config = config.ok_or_else(|| ServiceError::Internal {
            message: format!("No configuration found for service: {}", service_name),
        })?;

        self.apply_fallback_strategy(service_name, &config.fallback_strategy, &request_data).await
    }

    /// Apply fallback strategy for unavailable service
    async fn apply_fallback_strategy(&self, service_name: &str, strategy: &FallbackStrategy, request_data: &str) -> ServiceResult<String> {
        info!("Applying fallback strategy {:?} for service: {}", strategy, service_name);

        match strategy {
            FallbackStrategy::CachedData { max_age } => {
                self.get_cached_response(service_name, *max_age).await
            }
            FallbackStrategy::AlternativeService { service_name: alt_service } => {
                info!("Redirecting to alternative service: {}", alt_service);
                Box::pin(self.handle_request(alt_service, request_data.to_string())).await
            }
            FallbackStrategy::StaticData { data } => {
                Ok(format!("Static fallback data: {}", data))
            }
            FallbackStrategy::DisableFeature { feature_name } => {
                Ok(format!("Feature '{}' temporarily disabled due to service outage", feature_name))
            }
            FallbackStrategy::QueueRequests { max_queue_size } => {
                self.queue_request(service_name, request_data, *max_queue_size).await
            }
            FallbackStrategy::DegradedResponse { message } => {
                Ok(format!("Degraded service response: {}", message))
            }
            FallbackStrategy::FailFast { error_message } => {
                Err(ServiceError::ServiceUnavailable {
                    service: format!("{}: {}", service_name, error_message),
                })
            }
        }
    }

    /// Get cached response if available
    async fn get_cached_response(&self, service_name: &str, max_age: Duration) -> ServiceResult<String> {
        let cache = self.fallback_cache.read().await;
        
        if let Some((data, cached_at)) = cache.get(service_name) {
            if cached_at.elapsed() <= max_age {
                return Ok(format!("Cached response (age: {:?}): {}", cached_at.elapsed(), data));
            }
        }

        Err(ServiceError::NotFound {
            resource: format!("Valid cached data for service: {}", service_name),
        })
    }

    /// Queue request for later processing
    async fn queue_request(&self, service_name: &str, request_data: &str, max_queue_size: usize) -> ServiceResult<String> {
        let mut queues = self.request_queues.write().await;
        let queue = queues.entry(service_name.to_string()).or_insert_with(Vec::new);

        if queue.len() >= max_queue_size {
            return Err(ServiceError::ServiceUnavailable {
                service: format!("{}: Request queue full", service_name),
            });
        }

        let request = QueuedRequest {
            id: Uuid::new_v4(),
            service_name: service_name.to_string(),
            request_data: request_data.to_string(),
            queued_at: Instant::now(),
            retry_count: 0,
        };

        queue.push(request.clone());
        
        Ok(format!("Request queued (ID: {}, Position: {})", request.id, queue.len()))
    }

    /// Update cached data for a service
    pub async fn update_cache(&self, service_name: &str, data: String) {
        let mut cache = self.fallback_cache.write().await;
        cache.insert(service_name.to_string(), (data, Instant::now()));
        debug!("Updated fallback cache for service: {}", service_name);
    }

    /// Get service status
    pub async fn get_service_status(&self, service_name: &str) -> Option<ServiceStatus> {
        let statuses = self.statuses.read().await;
        statuses.get(service_name).cloned()
    }

    /// Get all service statuses
    pub async fn get_all_statuses(&self) -> HashMap<String, ServiceStatus> {
        let statuses = self.statuses.read().await;
        statuses.clone()
    }

    /// Get outage events
    pub async fn get_outage_events(&self) -> Vec<OutageEvent> {
        let events = self.events.lock().await;
        events.clone()
    }

    /// Force service status update (for testing)
    pub async fn set_service_status(&self, service_name: &str, health: ServiceHealth) {
        let mut statuses = self.statuses.write().await;
        if let Some(status) = statuses.get_mut(service_name) {
            status.health = health;
            status.last_health_check = Instant::now();
        }
    }

    /// Process queued requests when service recovers
    pub async fn process_queued_requests(&self, service_name: &str) -> usize {
        let mut queues = self.request_queues.write().await;
        
        if let Some(queue) = queues.get_mut(service_name) {
            let queue_size = queue.len();
            
            if queue_size > 0 {
                info!("Processing {} queued requests for recovered service: {}", queue_size, service_name);
                // TODO: Implement actual request processing logic
                queue.clear();
            }
            
            queue_size
        } else {
            0
        }
    }

    /// Shutdown handler and cleanup resources
    pub async fn shutdown(&self) {
        info!("Shutting down service outage handler");
        
        // Cancel all health check tasks
        let mut handles = self.health_check_handles.lock().await;
        for (service_name, handle) in handles.drain() {
            debug!("Cancelling health check task for service: {}", service_name);
            handle.abort();
        }
    }
}

impl Default for ServiceOutageHandler {
    fn default() -> Self {
        Self::new()
    }
}

// Utility functions for creating configurations
impl ServiceOutageConfig {
    pub fn new(service_name: String, health_check_url: String) -> Self {
        Self {
            service_name,
            health_check_url,
            health_check_interval: Duration::from_secs(30),
            failure_threshold: 3,
            recovery_threshold: 2,
            fallback_strategy: FallbackStrategy::FailFast {
                error_message: "Service temporarily unavailable".to_string(),
            },
            enable_auto_recovery: true,
            max_outage_duration: Duration::from_secs(300),
        }
    }

    pub fn with_fallback_strategy(mut self, strategy: FallbackStrategy) -> Self {
        self.fallback_strategy = strategy;
        self
    }

    pub fn with_health_check_interval(mut self, interval: Duration) -> Self {
        self.health_check_interval = interval;
        self
    }

    pub fn with_failure_threshold(mut self, threshold: u32) -> Self {
        self.failure_threshold = threshold;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_service_outage_handler_creation() {
        let handler = ServiceOutageHandler::new();
        let statuses = handler.get_all_statuses().await;
        assert_eq!(statuses.len(), 0);
    }

    #[tokio::test]
    async fn test_service_registration() {
        let handler = ServiceOutageHandler::new();
        
        let config = ServiceOutageConfig::new(
            "test_service".to_string(),
            "http://localhost:8080/health".to_string(),
        );

        handler.register_service(config).await;
        
        let status = handler.get_service_status("test_service").await;
        assert!(status.is_some());
        assert_eq!(status.unwrap().service_name, "test_service");
    }

    #[tokio::test]
    async fn test_fallback_static_data() {
        let handler = ServiceOutageHandler::new();
        
        let config = ServiceOutageConfig::new(
            "test_service".to_string(),
            "http://localhost:8080/health".to_string(),
        ).with_fallback_strategy(FallbackStrategy::StaticData {
            data: "fallback_data".to_string(),
        });

        handler.register_service(config).await;
        
        // Force service to unavailable
        handler.set_service_status("test_service", ServiceHealth::Unavailable {
            reason: "Test outage".to_string(),
            since: SystemTime::now(),
        }).await;

        // Test fallback
        let result = handler.handle_request("test_service", "test_request".to_string()).await;
        assert!(result.is_ok());
        assert!(result.unwrap().contains("fallback_data"));
    }

    #[tokio::test]
    async fn test_request_queuing() {
        let handler = ServiceOutageHandler::new();
        
        let config = ServiceOutageConfig::new(
            "queue_service".to_string(),
            "http://localhost:8080/health".to_string(),
        ).with_fallback_strategy(FallbackStrategy::QueueRequests {
            max_queue_size: 10,
        });

        handler.register_service(config).await;
        
        // Force service to unavailable
        handler.set_service_status("queue_service", ServiceHealth::Unavailable {
            reason: "Test outage".to_string(),
            since: SystemTime::now(),
        }).await;

        // Test request queuing
        let result = handler.handle_request("queue_service", "queued_request".to_string()).await;
        assert!(result.is_ok());
        assert!(result.unwrap().contains("Request queued"));
    }
}