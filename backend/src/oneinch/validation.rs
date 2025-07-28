// src/oneinch/validation.rs - API validation traits and implementations

use async_trait::async_trait;
use std::fmt;

/// API key validation result
#[derive(Debug, Clone)]
pub struct ApiKeyValidationResult {
    pub is_valid: bool,
    pub message: String,
    pub details: Option<ValidationDetails>,
}

/// Validation details
#[derive(Debug, Clone)]
pub struct ValidationDetails {
    pub key_format_valid: bool,
    pub key_length_valid: bool,
    pub permissions_valid: bool,
    pub rate_limit_info: Option<RateLimitInfo>,
}

/// Rate limit information
#[derive(Debug, Clone)]
pub struct RateLimitInfo {
    pub requests_remaining: u32,
    pub reset_time: chrono::DateTime<chrono::Utc>,
}

/// Trait for API key validation (following Interface Segregation Principle)
#[async_trait]
pub trait ApiKeyValidator: Send + Sync {
    type Error: fmt::Display + fmt::Debug + Send + Sync;

    /// Validate key format
    fn validate_key_format(&self, api_key: &str) -> Result<bool, Self::Error>;
    
    /// Validate key via API
    async fn validate_key_remote(&self, api_key: &str) -> Result<ApiKeyValidationResult, Self::Error>;
    
    /// Comprehensive validation (format + remote check)
    async fn validate_key_comprehensive(&self, api_key: &str) -> Result<ApiKeyValidationResult, Self::Error> {
        // First check format
        match self.validate_key_format(api_key) {
            Ok(false) => {
                return Ok(ApiKeyValidationResult {
                    is_valid: false,
                    message: "Invalid API key format".to_string(),
                    details: Some(ValidationDetails {
                        key_format_valid: false,
                        key_length_valid: api_key.len() >= 10,
                        permissions_valid: false,
                        rate_limit_info: None,
                    }),
                });
            },
            Err(e) => {
                return Ok(ApiKeyValidationResult {
                    is_valid: false,
                    message: format!("Format validation error: {}", e),
                    details: None,
                });
            },
            Ok(true) => {
                // Format is valid, check remotely
                self.validate_key_remote(api_key).await
            }
        }
    }
}

/// Validator for 1inch API keys
pub struct OneinchApiKeyValidator {
    pub min_key_length: usize,
    pub max_key_length: usize,
}

impl Default for OneinchApiKeyValidator {
    fn default() -> Self {
        Self {
            min_key_length: 10,
            max_key_length: 128,
        }
    }
}

impl OneinchApiKeyValidator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_length_constraints(min_length: usize, max_length: usize) -> Self {
        Self {
            min_key_length: min_length,
            max_key_length: max_length,
        }
    }

    /// Check if key is a test key
    pub fn is_test_key(&self, api_key: &str) -> bool {
        let test_patterns = [
            "test",
            "demo",
            "example",
            "sample",
            "placeholder",
            "your_api_key",
            "api_key_here",
        ];

        let key_lower = api_key.to_lowercase();
        test_patterns.iter().any(|pattern| key_lower.contains(pattern))
    }

    /// Check if key looks realistic
    pub fn looks_realistic(&self, api_key: &str) -> bool {
        // Real API keys usually contain a mix of letters and numbers
        let has_letters = api_key.chars().any(|c| c.is_alphabetic());
        let has_numbers = api_key.chars().any(|c| c.is_numeric());
        let has_reasonable_length = api_key.len() >= self.min_key_length && api_key.len() <= self.max_key_length;
        
        has_letters && has_numbers && has_reasonable_length && !self.is_test_key(api_key)
    }
}

#[async_trait]
impl ApiKeyValidator for OneinchApiKeyValidator {
    type Error = ValidationError;

    fn validate_key_format(&self, api_key: &str) -> Result<bool, Self::Error> {
        if api_key.is_empty() {
            return Err(ValidationError::EmptyKey);
        }

        if api_key.len() < self.min_key_length {
            return Err(ValidationError::KeyTooShort(api_key.len(), self.min_key_length));
        }

        if api_key.len() > self.max_key_length {
            return Err(ValidationError::KeyTooLong(api_key.len(), self.max_key_length));
        }

        // Check for suspicious patterns
        if self.is_test_key(api_key) {
            return Ok(false); // Formally valid, but it's a test key
        }

        // Check that key contains only allowed characters
        let valid_chars = api_key.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_');
        if !valid_chars {
            return Err(ValidationError::InvalidCharacters);
        }

        Ok(true)
    }

    async fn validate_key_remote(&self, api_key: &str) -> Result<ApiKeyValidationResult, Self::Error> {
        // This function should be implemented in FusionClient
        // Here we return basic validation
        let format_valid = self.validate_key_format(api_key).unwrap_or(false);
        let looks_realistic = self.looks_realistic(api_key);

        Ok(ApiKeyValidationResult {
            is_valid: format_valid && looks_realistic,
            message: if format_valid && looks_realistic {
                "Key format appears valid".to_string()
            } else if !format_valid {
                "Invalid key format".to_string()
            } else {
                "Key appears to be a test/placeholder key".to_string()
            },
            details: Some(ValidationDetails {
                key_format_valid: format_valid,
                key_length_valid: api_key.len() >= self.min_key_length && api_key.len() <= self.max_key_length,
                permissions_valid: looks_realistic, // Approximate assessment
                rate_limit_info: None,
            }),
        })
    }
}

/// Validation errors
#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("API key is empty")]
    EmptyKey,
    
    #[error("API key too short: {0} characters, minimum {1}")]
    KeyTooShort(usize, usize),
    
    #[error("API key too long: {0} characters, maximum {1}")]
    KeyTooLong(usize, usize),
    
    #[error("API key contains invalid characters")]
    InvalidCharacters,
    
    #[error("Network error during validation: {0}")]
    NetworkError(String),
    
    #[error("API validation failed: {0}")]
    ApiError(String),
}

/// Trait for components that can validate their settings
pub trait SelfValidating {
    type ValidationError: fmt::Display + fmt::Debug;
    
    /// Validate component configuration
    fn validate_configuration(&self) -> Result<(), Self::ValidationError>;
    
    /// Check readiness for work
    fn is_ready(&self) -> bool {
        self.validate_configuration().is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_key_format_validation() {
        let validator = OneinchApiKeyValidator::new();

        // Valid keys
        assert!(validator.validate_key_format("valid_api_key_123").unwrap());
        assert!(validator.validate_key_format("sk-1234567890abcdef").unwrap());

        // Invalid keys
        assert!(validator.validate_key_format("").is_err());
        assert!(validator.validate_key_format("short").is_err());
        assert!(!validator.validate_key_format("test_key").unwrap()); // Test key
        assert!(validator.validate_key_format("key with spaces").is_err());
    }

    #[test]
    fn test_test_key_detection() {
        let validator = OneinchApiKeyValidator::new();

        assert!(validator.is_test_key("test_key"));
        assert!(validator.is_test_key("demo_api_key"));
        assert!(validator.is_test_key("your_api_key_here"));
        assert!(!validator.is_test_key("sk-1234567890abcdef"));
    }

    #[test]
    fn test_realistic_key_check() {
        let validator = OneinchApiKeyValidator::new();

        assert!(validator.looks_realistic("sk-1234567890abcdef"));
        assert!(validator.looks_realistic("api_key_abc123def456"));
        assert!(!validator.looks_realistic("test_key"));
        assert!(!validator.looks_realistic("short"));
        assert!(!validator.looks_realistic("onlyletters"));
        assert!(!validator.looks_realistic("123456789"));
    }

    #[tokio::test]
    async fn test_comprehensive_validation() {
        let validator = OneinchApiKeyValidator::new();

        // Test with valid key
        let result = validator.validate_key_comprehensive("sk-1234567890abcdef").await.unwrap();
        assert!(result.is_valid);

        // Test with test key
        let result = validator.validate_key_comprehensive("test_key").await.unwrap();
        assert!(!result.is_valid);

        // Test with short key
        let result = validator.validate_key_comprehensive("short").await.unwrap();
        assert!(!result.is_valid);
    }
}