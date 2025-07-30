/**
 * KEMBridge Error Recovery System
 * Production-grade error recovery and resilience patterns
 */
use crate::errors::ServiceResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use uuid::Uuid;

/// Recovery strategy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecoveryStrategy {
    /// Immediate retry without delay
    ImmediateRetry { max_attempts: u32 },
    /// Exponential backoff with jitter
    ExponentialBackoff {
        initial_delay: Duration,
        max_delay: Duration,
        multiplier: f64,
        max_attempts: u32,
        jitter: bool,
    },
    /// Circuit breaker pattern
    CircuitBreaker {
        failure_threshold: u32,
        success_threshold: u32,
        timeout: Duration,
    },
    /// Linear backoff
    LinearBackoff { delay: Duration, max_attempts: u32 },
    /// Custom recovery logic
    Custom { name: String, max_attempts: u32 },
    /// Manual intervention required
    ManualIntervention,
}

/// Error categories for recovery classification
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ErrorCategory {
    /// Network connectivity issues
    Network,
    /// Authentication/authorization failures
    Authentication,
    /// Blockchain-specific errors
    Blockchain,
    /// External service errors
    ExternalService,
    /// Internal service errors
    InternalService,
    /// Validation errors
    Validation,
    /// Rate limiting
    RateLimit,
    /// Resource exhaustion
    ResourceExhaustion,
    /// Configuration errors
    Configuration,
    /// Unknown/unclassified errors
    Unknown,
}

/// Recovery action result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecoveryResult {
    /// Recovery successful, operation can continue
    Success,
    /// Recovery partially successful, retry recommended
    PartialSuccess { message: String },
    /// Recovery failed, but retryable
    Retryable { delay: Duration, message: String },
    /// Recovery failed, not retryable
    Failed { message: String },
    /// Manual intervention required
    ManualInterventionRequired { message: String, contact: String },
}

/// Failure context for recovery decisions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailureContext {
    /// Unique transaction identifier
    pub transaction_id: Uuid,
    /// Type of operation that failed  
    pub operation_type: String,
    /// Error category for classification
    pub error_category: ErrorCategory,
    /// Original error message
    pub error_message: String,
    /// When the failure occurred
    pub failed_at: SystemTime,
    /// Current retry attempt
    pub retry_count: u32,
    /// Maximum retry attempts allowed
    pub max_retries: u32,
    /// Additional context data
    pub context: HashMap<String, String>,
    /// Service/component that failed
    pub service_name: String,
    /// Error severity level
    pub severity: u8, // 1-10 scale
}

/// Recovery action definition
#[derive(Debug, Clone)]
pub struct RecoveryAction {
    pub name: String,
    pub description: String,
    pub strategy: RecoveryStrategy,
    pub applicable_categories: Vec<ErrorCategory>,
    pub timeout: Duration,
    pub priority: u8, // 1-10, higher is more priority
}

/// Recovery system state
#[derive(Debug)]
struct RecoveryState {
    /// Active failure contexts being tracked
    active_failures: HashMap<Uuid, FailureContext>,
    /// Recovery actions by category
    recovery_actions: HashMap<ErrorCategory, Vec<RecoveryAction>>,
    /// Circuit breaker states
    circuit_states: HashMap<String, CircuitBreakerState>,
    /// Recovery statistics
    stats: RecoveryStats,
}

/// Circuit breaker state
#[derive(Debug, Clone)]
struct CircuitBreakerState {
    pub state: CircuitState,
    pub failure_count: u32,
    pub success_count: u32,
    pub last_failure: Option<Instant>,
    pub next_attempt: Option<Instant>,
}

#[derive(Debug, Clone, PartialEq)]
enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

/// Recovery statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RecoveryStats {
    pub total_failures: u64,
    pub successful_recoveries: u64,
    pub failed_recoveries: u64,
    pub manual_interventions: u64,
    pub recovery_by_category: HashMap<ErrorCategory, u64>,
    pub average_recovery_time: Duration,
}

/// Production-grade error recovery system
pub struct RecoverySystem {
    state: Arc<RwLock<RecoveryState>>,
    config: RecoveryConfig,
}

/// Recovery system configuration
#[derive(Debug, Clone)]
pub struct RecoveryConfig {
    /// Maximum concurrent recovery operations
    pub max_concurrent_recoveries: usize,
    /// Default timeout for recovery operations
    pub default_timeout: Duration,
    /// Enable automatic recovery
    pub auto_recovery_enabled: bool,
    /// Enable circuit breaker functionality
    pub circuit_breaker_enabled: bool,
    /// Failure cleanup interval
    pub cleanup_interval: Duration,
}

impl Default for RecoveryConfig {
    fn default() -> Self {
        Self {
            max_concurrent_recoveries: 10,
            default_timeout: Duration::from_secs(30),
            auto_recovery_enabled: true,
            circuit_breaker_enabled: true,
            cleanup_interval: Duration::from_secs(300), // 5 minutes
        }
    }
}

impl RecoverySystem {
    /// Create new recovery system with default configuration
    pub fn new() -> Self {
        Self::with_config(RecoveryConfig::default())
    }

    /// Create new recovery system with custom configuration
    pub fn with_config(config: RecoveryConfig) -> Self {
        let state = RecoveryState {
            active_failures: HashMap::new(),
            recovery_actions: Self::default_recovery_actions(),
            circuit_states: HashMap::new(),
            stats: RecoveryStats::default(),
        };

        Self {
            state: Arc::new(RwLock::new(state)),
            config,
        }
    }

    /// Register a failure and attempt recovery
    pub async fn handle_failure(&self, context: FailureContext) -> ServiceResult<RecoveryResult> {
        let transaction_id = context.transaction_id;
        info!(
            "🔄 Processing failure for transaction {}: {}",
            transaction_id, context.error_message
        );

        // Update failure tracking
        {
            let mut state = self.state.write().await;
            state
                .active_failures
                .insert(transaction_id, context.clone());
            state.stats.total_failures += 1;
            *state
                .stats
                .recovery_by_category
                .entry(context.error_category.clone())
                .or_insert(0) += 1;
        }

        // Check circuit breaker state
        if self.config.circuit_breaker_enabled {
            if let Some(circuit_result) = self.check_circuit_breaker(&context).await? {
                return Ok(circuit_result);
            }
        }

        // Attempt recovery
        let recovery_result = self.attempt_recovery(&context).await?;

        // Update statistics
        {
            let mut state = self.state.write().await;
            match &recovery_result {
                RecoveryResult::Success => {
                    state.stats.successful_recoveries += 1;
                    state.active_failures.remove(&transaction_id);
                }
                RecoveryResult::Failed { .. } => {
                    state.stats.failed_recoveries += 1;
                    state.active_failures.remove(&transaction_id);
                }
                RecoveryResult::ManualInterventionRequired { .. } => {
                    state.stats.manual_interventions += 1;
                }
                _ => {} // Keep tracking for retryable results
            }
        }

        info!(
            "✅ Recovery completed for transaction {}: {:?}",
            transaction_id, recovery_result
        );
        Ok(recovery_result)
    }

    /// Attempt recovery for a specific failure context
    async fn attempt_recovery(&self, context: &FailureContext) -> ServiceResult<RecoveryResult> {
        let actions = {
            let state = self.state.read().await;
            state
                .recovery_actions
                .get(&context.error_category)
                .cloned()
                .unwrap_or_default()
        };

        if actions.is_empty() {
            warn!(
                "No recovery actions available for category: {:?}",
                context.error_category
            );
            return Ok(RecoveryResult::Failed {
                message: format!(
                    "No recovery strategy available for error category: {:?}",
                    context.error_category
                ),
            });
        }

        // Sort actions by priority (highest first)
        let mut sorted_actions = actions;
        sorted_actions.sort_by(|a, b| b.priority.cmp(&a.priority));

        for action in sorted_actions {
            debug!(
                "Attempting recovery action '{}' for transaction {}",
                action.name, context.transaction_id
            );

            match self.execute_recovery_action(&action, context).await {
                Ok(RecoveryResult::Success) => {
                    info!(
                        "🎉 Recovery action '{}' succeeded for transaction {}",
                        action.name, context.transaction_id
                    );
                    return Ok(RecoveryResult::Success);
                }
                Ok(result) => {
                    debug!("Recovery action '{}' returned: {:?}", action.name, result);
                    // Continue to next action for non-success results
                }
                Err(e) => {
                    warn!("Recovery action '{}' failed with error: {}", action.name, e);
                    // Continue to next action
                }
            }
        }

        // All recovery actions failed
        Ok(RecoveryResult::Failed {
            message: "All recovery actions exhausted".to_string(),
        })
    }

    /// Execute a specific recovery action
    async fn execute_recovery_action(
        &self,
        action: &RecoveryAction,
        context: &FailureContext,
    ) -> ServiceResult<RecoveryResult> {
        match &action.strategy {
            RecoveryStrategy::ImmediateRetry { max_attempts } => {
                if context.retry_count >= *max_attempts {
                    return Ok(RecoveryResult::Failed {
                        message: format!("Maximum retry attempts ({}) exceeded", max_attempts),
                    });
                }

                Ok(RecoveryResult::Retryable {
                    delay: Duration::from_millis(0),
                    message: "Immediate retry recommended".to_string(),
                })
            }

            RecoveryStrategy::ExponentialBackoff {
                initial_delay,
                max_delay,
                multiplier,
                max_attempts,
                jitter,
            } => {
                if context.retry_count >= *max_attempts {
                    return Ok(RecoveryResult::Failed {
                        message: format!("Maximum retry attempts ({}) exceeded", max_attempts),
                    });
                }

                let base_delay =
                    initial_delay.as_millis() as f64 * multiplier.powi(context.retry_count as i32);
                let mut delay = Duration::from_millis(base_delay as u64).min(*max_delay);

                if *jitter {
                    let jitter_ms = (delay.as_millis() as f64 * 0.1 * rand::random::<f64>()) as u64;
                    delay += Duration::from_millis(jitter_ms);
                }

                Ok(RecoveryResult::Retryable {
                    delay,
                    message: format!("Exponential backoff retry in {:?}", delay),
                })
            }

            RecoveryStrategy::LinearBackoff {
                delay,
                max_attempts,
            } => {
                if context.retry_count >= *max_attempts {
                    return Ok(RecoveryResult::Failed {
                        message: format!("Maximum retry attempts ({}) exceeded", max_attempts),
                    });
                }

                Ok(RecoveryResult::Retryable {
                    delay: *delay,
                    message: format!("Linear backoff retry in {:?}", delay),
                })
            }

            RecoveryStrategy::CircuitBreaker { .. } => {
                // Circuit breaker logic is handled separately
                Ok(RecoveryResult::Success)
            }

            RecoveryStrategy::Custom { name, max_attempts } => {
                if context.retry_count >= *max_attempts {
                    return Ok(RecoveryResult::Failed {
                        message: format!(
                            "Maximum retry attempts ({}) exceeded for custom strategy '{}'",
                            max_attempts, name
                        ),
                    });
                }

                // Custom recovery logic would be implemented here
                Ok(RecoveryResult::PartialSuccess {
                    message: format!("Custom recovery strategy '{}' applied", name),
                })
            }

            RecoveryStrategy::ManualIntervention => {
                Ok(RecoveryResult::ManualInterventionRequired {
                    message: "Manual intervention required to resolve this error".to_string(),
                    contact: "system-admin@kembridge.io".to_string(),
                })
            }
        }
    }

    /// Check circuit breaker state for a service
    async fn check_circuit_breaker(
        &self,
        context: &FailureContext,
    ) -> ServiceResult<Option<RecoveryResult>> {
        let mut state = self.state.write().await;
        let circuit_state = state
            .circuit_states
            .entry(context.service_name.clone())
            .or_insert_with(|| CircuitBreakerState {
                state: CircuitState::Closed,
                failure_count: 0,
                success_count: 0,
                last_failure: None,
                next_attempt: None,
            });

        match circuit_state.state {
            CircuitState::Open => {
                if let Some(next_attempt) = circuit_state.next_attempt {
                    if Instant::now() > next_attempt {
                        circuit_state.state = CircuitState::HalfOpen;
                        info!(
                            "🔄 Circuit breaker transitioning to half-open for service: {}",
                            context.service_name
                        );
                    } else {
                        return Ok(Some(RecoveryResult::Failed {
                            message: format!(
                                "Circuit breaker open for service: {}",
                                context.service_name
                            ),
                        }));
                    }
                } else {
                    return Ok(Some(RecoveryResult::Failed {
                        message: format!(
                            "Circuit breaker open for service: {}",
                            context.service_name
                        ),
                    }));
                }
            }
            CircuitState::HalfOpen => {
                // Allow one attempt through
                info!(
                    "🧪 Circuit breaker half-open, allowing test request for service: {}",
                    context.service_name
                );
            }
            CircuitState::Closed => {
                // Normal operation
            }
        }

        // Record failure
        circuit_state.failure_count += 1;
        circuit_state.last_failure = Some(Instant::now());

        // Check if we should open the circuit
        if circuit_state.failure_count >= 5 {
            // Default threshold
            circuit_state.state = CircuitState::Open;
            circuit_state.next_attempt = Some(Instant::now() + Duration::from_secs(60)); // Default timeout
            warn!(
                "⚡ Circuit breaker opened for service: {}",
                context.service_name
            );

            return Ok(Some(RecoveryResult::Failed {
                message: format!(
                    "Circuit breaker opened due to repeated failures for service: {}",
                    context.service_name
                ),
            }));
        }

        Ok(None)
    }

    /// Get recovery statistics
    pub async fn get_stats(&self) -> RecoveryStats {
        let state = self.state.read().await;
        state.stats.clone()
    }

    /// Get active failure contexts
    pub async fn get_active_failures(&self) -> Vec<FailureContext> {
        let state = self.state.read().await;
        state.active_failures.values().cloned().collect()
    }

    /// Default recovery actions for different error categories
    fn default_recovery_actions() -> HashMap<ErrorCategory, Vec<RecoveryAction>> {
        let mut actions = HashMap::new();

        // Network errors
        actions.insert(
            ErrorCategory::Network,
            vec![RecoveryAction {
                name: "exponential_backoff".to_string(),
                description: "Exponential backoff retry for network failures".to_string(),
                strategy: RecoveryStrategy::ExponentialBackoff {
                    initial_delay: Duration::from_millis(100),
                    max_delay: Duration::from_secs(30),
                    multiplier: 2.0,
                    max_attempts: 5,
                    jitter: true,
                },
                applicable_categories: vec![ErrorCategory::Network],
                timeout: Duration::from_secs(30),
                priority: 8,
            }],
        );

        // Authentication errors
        actions.insert(
            ErrorCategory::Authentication,
            vec![RecoveryAction {
                name: "token_refresh".to_string(),
                description: "Attempt to refresh authentication token".to_string(),
                strategy: RecoveryStrategy::ImmediateRetry { max_attempts: 2 },
                applicable_categories: vec![ErrorCategory::Authentication],
                timeout: Duration::from_secs(10),
                priority: 9,
            }],
        );

        // Rate limiting
        actions.insert(
            ErrorCategory::RateLimit,
            vec![RecoveryAction {
                name: "rate_limit_backoff".to_string(),
                description: "Linear backoff for rate limit recovery".to_string(),
                strategy: RecoveryStrategy::LinearBackoff {
                    delay: Duration::from_secs(1),
                    max_attempts: 10,
                },
                applicable_categories: vec![ErrorCategory::RateLimit],
                timeout: Duration::from_secs(60),
                priority: 7,
            }],
        );

        // External service errors
        actions.insert(
            ErrorCategory::ExternalService,
            vec![RecoveryAction {
                name: "service_circuit_breaker".to_string(),
                description: "Circuit breaker for external service failures".to_string(),
                strategy: RecoveryStrategy::CircuitBreaker {
                    failure_threshold: 5,
                    success_threshold: 2,
                    timeout: Duration::from_secs(60),
                },
                applicable_categories: vec![ErrorCategory::ExternalService],
                timeout: Duration::from_secs(30),
                priority: 6,
            }],
        );

        actions
    }
}

impl Default for RecoverySystem {
    fn default() -> Self {
        Self::new()
    }
}

// Re-export types for backward compatibility with SimpleRecoverySystem usage
pub use ErrorCategory as SimpleErrorCategory;
pub use FailureContext as SimpleFailureContext;
pub use RecoverySystem as SimpleRecoverySystem;
