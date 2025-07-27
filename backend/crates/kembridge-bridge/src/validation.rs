use crate::BridgeError;
use std::collections::HashSet;

pub struct ValidationService {
    supported_chains: HashSet<String>,
    min_amount: u128,
    max_amount: u128,
}

impl ValidationService {
    pub fn new() -> Self {
        let mut supported_chains = HashSet::new();
        supported_chains.insert("ethereum".to_string());
        supported_chains.insert("near".to_string());
        
        Self {
            supported_chains,
            min_amount: 1_000_000_000_000_000, // 0.001 ETH minimum
            max_amount: 1_000_000_000_000_000_000_000, // 1000 ETH maximum
        }
    }

    pub async fn validate_swap_params(
        &self,
        from_chain: &str,
        to_chain: &str,
        amount: u128,
        recipient: &str,
    ) -> Result<(), BridgeError> {
        // Validate chains
        self.validate_chain(from_chain)?;
        self.validate_chain(to_chain)?;
        
        // Validate chains are different
        if from_chain == to_chain {
            return Err(BridgeError::ValidationError(
                "Source and destination chains must be different".to_string()
            ));
        }

        // Validate amount
        self.validate_amount(amount)?;

        // Validate recipient address
        self.validate_recipient_address(to_chain, recipient)?;

        tracing::info!("Validation passed for swap: {} {} -> {}, amount: {}", 
                      from_chain, to_chain, recipient, amount);
        
        Ok(())
    }

    fn validate_chain(&self, chain: &str) -> Result<(), BridgeError> {
        if !self.supported_chains.contains(chain) {
            return Err(BridgeError::UnsupportedChain {
                chain: chain.to_string(),
            });
        }
        Ok(())
    }

    fn validate_amount(&self, amount: u128) -> Result<(), BridgeError> {
        if amount < self.min_amount {
            return Err(BridgeError::ValidationError(
                format!("Amount {} is below minimum {}", amount, self.min_amount)
            ));
        }

        if amount > self.max_amount {
            return Err(BridgeError::ValidationError(
                format!("Amount {} exceeds maximum {}", amount, self.max_amount)
            ));
        }

        Ok(())
    }

    fn validate_recipient_address(&self, chain: &str, address: &str) -> Result<(), BridgeError> {
        match chain {
            "ethereum" => self.validate_ethereum_address(address),
            "near" => self.validate_near_address(address),
            _ => Err(BridgeError::ValidationError(
                format!("Unknown chain: {}", chain)
            )),
        }
    }

    fn validate_ethereum_address(&self, address: &str) -> Result<(), BridgeError> {
        // Basic Ethereum address validation
        if !address.starts_with("0x") {
            return Err(BridgeError::ValidationError(
                "Ethereum address must start with 0x".to_string()
            ));
        }

        if address.len() != 42 {
            return Err(BridgeError::ValidationError(
                "Ethereum address must be 42 characters long".to_string()
            ));
        }

        // Check if it's valid hex (excluding 0x prefix)
        let hex_part = &address[2..];
        if !hex_part.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(BridgeError::ValidationError(
                "Ethereum address contains invalid characters".to_string()
            ));
        }

        Ok(())
    }

    fn validate_near_address(&self, address: &str) -> Result<(), BridgeError> {
        // Basic NEAR address validation
        if address.is_empty() {
            return Err(BridgeError::ValidationError(
                "NEAR address cannot be empty".to_string()
            ));
        }

        if address.len() > 64 {
            return Err(BridgeError::ValidationError(
                "NEAR address too long".to_string()
            ));
        }

        // NEAR addresses can be:
        // 1. Implicit accounts (64 char hex)
        // 2. Named accounts (e.g., alice.near)
        // 3. Subaccounts (e.g., app.alice.near)
        
        if address.len() == 64 {
            // Check if it's a valid hex string (implicit account)
            if !address.chars().all(|c| c.is_ascii_hexdigit()) {
                return Err(BridgeError::ValidationError(
                    "NEAR implicit address contains invalid characters".to_string()
                ));
            }
        } else {
            // Check if it's a valid named account
            if !address.chars().all(|c| c.is_alphanumeric() || c == '.' || c == '-' || c == '_') {
                return Err(BridgeError::ValidationError(
                    "NEAR named address contains invalid characters".to_string()
                ));
            }

            // Must end with .near or .testnet for named accounts
            if !address.ends_with(".near") && !address.ends_with(".testnet") {
                return Err(BridgeError::ValidationError(
                    "NEAR named address must end with .near or .testnet".to_string()
                ));
            }
        }

        Ok(())
    }

    pub fn get_supported_chains(&self) -> Vec<String> {
        self.supported_chains.iter().cloned().collect()
    }

    pub fn get_min_amount(&self) -> u128 {
        self.min_amount
    }

    pub fn get_max_amount(&self) -> u128 {
        self.max_amount
    }
}

impl Default for ValidationService {
    fn default() -> Self {
        Self::new()
    }
}

// Clone implementation for Arc usage
impl Clone for ValidationService {
    fn clone(&self) -> Self {
        Self {
            supported_chains: self.supported_chains.clone(),
            min_amount: self.min_amount,
            max_amount: self.max_amount,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

    #[tokio::test]
    async fn test_valid_swap_params() {
        let validator = ValidationService::new();
        
        let result = validator.validate_swap_params(
            "ethereum",
            "near",
            1_000_000_000_000_000_000, // 1 ETH
            "test.near",
        ).await;
        
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_same_chain_validation() {
        let validator = ValidationService::new();
        
        let result = validator.validate_swap_params(
            "ethereum",
            "ethereum",
            1_000_000_000_000_000_000,
            "0x1234567890123456789012345678901234567890",
        ).await;
        
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_unsupported_chain() {
        let validator = ValidationService::new();
        
        let result = validator.validate_swap_params(
            "bitcoin",
            "near",
            1_000_000_000_000_000_000,
            "test.near",
        ).await;
        
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_amount_validation() {
        let validator = ValidationService::new();
        
        // Test minimum amount
        let result = validator.validate_swap_params(
            "ethereum",
            "near",
            100, // Too small
            "test.near",
        ).await;
        assert!(result.is_err());

        // Test maximum amount
        let result = validator.validate_swap_params(
            "ethereum",
            "near",
            u128::MAX, // Too large
            "test.near",
        ).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_ethereum_address_validation() {
        let validator = ValidationService::new();
        
        // Valid address
        assert!(validator.validate_ethereum_address("0x1234567890123456789012345678901234567890").is_ok());
        
        // Invalid addresses
        assert!(validator.validate_ethereum_address("1234567890123456789012345678901234567890").is_err()); // No 0x prefix
        assert!(validator.validate_ethereum_address("0x123456789012345678901234567890123456789").is_err()); // Too short
        assert!(validator.validate_ethereum_address("0x12345678901234567890123456789012345678901").is_err()); // Too long
        assert!(validator.validate_ethereum_address("0x123456789012345678901234567890123456789G").is_err()); // Invalid hex
    }

    #[test]
    fn test_near_address_validation() {
        let validator = ValidationService::new();
        
        // Valid addresses
        assert!(validator.validate_near_address("test.near").is_ok());
        assert!(validator.validate_near_address("alice.testnet").is_ok());
        assert!(validator.validate_near_address("app.alice.near").is_ok());
        assert!(validator.validate_near_address("1234567890123456789012345678901234567890123456789012345678901234").is_ok()); // 64 char hex
        
        // Invalid addresses
        assert!(validator.validate_near_address("").is_err()); // Empty
        assert!(validator.validate_near_address("test").is_err()); // No suffix
        assert!(validator.validate_near_address("test.invalid").is_err()); // Invalid suffix
        assert!(validator.validate_near_address("test@alice.near").is_err()); // Invalid character
        assert!(validator.validate_near_address("123456789012345678901234567890123456789012345678901234567890123G").is_err()); // Invalid hex
    }
}