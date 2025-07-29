// Simple Circuit Breaker implementation for Gateway service
use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CircuitState {
    Closed,   // Normal operation
    Open,     // Failures detected, blocking requests
    HalfOpen, // Testing recovery
}

#[derive(Debug)]
pub struct CircuitBreakerConfig {
    pub failure_threshold: u32,     // Number of failures to open circuit
    pub success_threshold: u32,     // Number of successes to close circuit
    pub timeout: Duration,          // How long to stay open
    pub window_duration: Duration,  // Time window for failure counting
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            success_threshold: 2,
            timeout: Duration::from_secs(30),
            window_duration: Duration::from_secs(60),
        }
    }
}

#[derive(Debug)]
struct CircuitBreakerState {
    state: CircuitState,
    failure_count: u32,
    success_count: u32,
    last_failure_time: Option<Instant>,
    window_start: Instant,
}

impl CircuitBreakerState {
    fn new() -> Self {
        Self {
            state: CircuitState::Closed,
            failure_count: 0,
            success_count: 0,
            last_failure_time: None,
            window_start: Instant::now(),
        }
    }
}

#[derive(Debug)]
pub struct CircuitBreaker {
    config: CircuitBreakerConfig,
    states: Arc<Mutex<HashMap<String, CircuitBreakerState>>>,
}

impl CircuitBreaker {
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            config,
            states: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn is_request_allowed(&self, service: &str) -> bool {
        let mut states = self.states.lock();
        let state = states.entry(service.to_string()).or_insert_with(CircuitBreakerState::new);
        
        let now = Instant::now();
        
        // Reset window if needed
        if now.duration_since(state.window_start) > self.config.window_duration {
            state.failure_count = 0;
            state.success_count = 0;
            state.window_start = now;
        }

        match state.state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                // Check if timeout has passed
                if let Some(last_failure) = state.last_failure_time {
                    if now.duration_since(last_failure) > self.config.timeout {
                        tracing::info!("Circuit breaker for {} transitioning to HALF_OPEN", service);
                        state.state = CircuitState::HalfOpen;
                        state.success_count = 0;
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            CircuitState::HalfOpen => true,
        }
    }

    pub fn record_success(&self, service: &str) {
        let mut states = self.states.lock();
        let state = states.entry(service.to_string()).or_insert_with(CircuitBreakerState::new);
        
        state.success_count += 1;

        match state.state {
            CircuitState::HalfOpen => {
                if state.success_count >= self.config.success_threshold {
                    tracing::info!("Circuit breaker for {} transitioning to CLOSED", service);
                    state.state = CircuitState::Closed;
                    state.failure_count = 0;
                    state.success_count = 0;
                }
            }
            CircuitState::Open => {
                // Shouldn't happen, but reset if it does
                tracing::warn!("Received success for {} while circuit is OPEN", service);
                state.state = CircuitState::Closed;
                state.failure_count = 0;
            }
            CircuitState::Closed => {
                // Normal operation, just track the success
            }
        }
    }

    pub fn record_failure(&self, service: &str) {
        let mut states = self.states.lock();
        let state = states.entry(service.to_string()).or_insert_with(CircuitBreakerState::new);
        
        state.failure_count += 1;
        state.last_failure_time = Some(Instant::now());

        match state.state {
            CircuitState::Closed => {
                if state.failure_count >= self.config.failure_threshold {
                    tracing::warn!("Circuit breaker for {} transitioning to OPEN (failures: {})", 
                                 service, state.failure_count);
                    state.state = CircuitState::Open;
                }
            }
            CircuitState::HalfOpen => {
                tracing::warn!("Circuit breaker for {} transitioning back to OPEN", service);
                state.state = CircuitState::Open;
                state.success_count = 0;
            }
            CircuitState::Open => {
                // Already open, just track the failure
            }
        }
    }

    pub fn get_state(&self, service: &str) -> CircuitState {
        let states = self.states.lock();
        states.get(service)
              .map(|s| s.state)
              .unwrap_or(CircuitState::Closed)
    }

    pub fn get_stats(&self, service: &str) -> (CircuitState, u32, u32) {
        let states = self.states.lock();
        if let Some(state) = states.get(service) {
            (state.state, state.failure_count, state.success_count)
        } else {
            (CircuitState::Closed, 0, 0)
        }
    }

    // Helper method to create a fallback response
    pub fn create_fallback_response<T>(&self, service: &str, fallback_data: T) -> T {
        tracing::warn!("Circuit breaker OPEN for {}, returning fallback response", service);
        fallback_data
    }
}