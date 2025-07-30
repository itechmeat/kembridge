use crate::{ServiceError, ServiceResult};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{error, info};

// Utilities for all services

pub fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

pub fn current_timestamp_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

// Address validation
pub fn validate_ethereum_address(address: &str) -> ServiceResult<()> {
    if !address.starts_with("0x") {
        return Err(ServiceError::Validation {
            field: "ethereum_address".to_string(),
            message: "Address must start with 0x".to_string(),
        });
    }

    if address.len() != 42 {
        return Err(ServiceError::Validation {
            field: "ethereum_address".to_string(),
            message: "Address must be 42 characters long".to_string(),
        });
    }

    // Simple hex character validation
    for c in address.chars().skip(2) {
        if !c.is_ascii_hexdigit() {
            return Err(ServiceError::Validation {
                field: "ethereum_address".to_string(),
                message: "Address contains invalid hex characters".to_string(),
            });
        }
    }

    Ok(())
}

pub fn validate_near_address(address: &str) -> ServiceResult<()> {
    if address.is_empty() {
        return Err(ServiceError::Validation {
            field: "near_address".to_string(),
            message: "Address cannot be empty".to_string(),
        });
    }

    // NEAR addresses can be account names or implicit accounts (hex)
    if address.len() == 64 && address.chars().all(|c| c.is_ascii_hexdigit()) {
        // Implicit account (64-char hex)
        return Ok(());
    }

    // Named account validation
    if address.len() < 2 || address.len() > 64 {
        return Err(ServiceError::Validation {
            field: "near_address".to_string(),
            message: "Named account must be 2-64 characters".to_string(),
        });
    }

    // Basic character validation for named accounts
    for c in address.chars() {
        if !c.is_ascii_alphanumeric() && c != '-' && c != '_' && c != '.' {
            return Err(ServiceError::Validation {
                field: "near_address".to_string(),
                message: "Named account contains invalid characters".to_string(),
            });
        }
    }

    Ok(())
}

// Error formatting for logs
pub fn log_error<'a, T>(result: &'a ServiceResult<T>, context: &str) -> &'a ServiceResult<T> {
    if let Err(e) = result {
        error!("Error in {}: {}", context, e);
    }
    result
}

pub fn log_success(context: &str) {
    info!("Success in {}", context);
}

// HTTP response helpers
#[derive(Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub message: Option<String>,
    pub timestamp: u64,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            message: None,
            timestamp: current_timestamp(),
        }
    }

    pub fn error(message: String) -> Self {
        Self {
            success: false,
            data: None,
            message: Some(message),
            timestamp: current_timestamp(),
        }
    }
}

// Environment helpers
pub fn is_development() -> bool {
    std::env::var("ENVIRONMENT")
        .unwrap_or_else(|_| "development".to_string())
        .to_lowercase()
        == "development"
}

pub fn is_production() -> bool {
    std::env::var("ENVIRONMENT")
        .unwrap_or_else(|_| "development".to_string())
        .to_lowercase()
        == "production"
}

// Service URL helpers
pub fn get_service_url(service_name: &str, default_port: u16) -> String {
    let env_var = format!("{}_SERVICE_URL", service_name.to_uppercase());
    std::env::var(&env_var).unwrap_or_else(|_| format!("http://localhost:{}", default_port))
}

// Async retry utility
pub async fn retry_async<F, Fut, T, E>(
    mut operation: F,
    max_retries: u32,
    initial_delay_ms: u64,
) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
    E: std::fmt::Display,
{
    let mut delay = initial_delay_ms;
    
    for attempt in 0..=max_retries {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                if attempt == max_retries {
                    error!("Operation failed after {} retries: {}", max_retries + 1, e);
                    return Err(e);
                }
                
                info!("Attempt {} failed, retrying in {}ms: {}", attempt + 1, delay, e);
                tokio::time::sleep(std::time::Duration::from_millis(delay)).await;
                delay *= 2; // Exponential backoff
            }
        }
    }

    unreachable!()
}