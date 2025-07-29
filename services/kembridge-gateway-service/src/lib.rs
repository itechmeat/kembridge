// Minimal gateway service for architecture testing
pub mod circuit_breaker;
pub mod config;
pub mod errors;
pub mod clients;

// Gateway types for proxying requests
pub mod types {
    use serde::{Deserialize, Serialize};
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ProxyRequest {
        pub service: String,
        pub path: String,
        pub method: String,
        pub body: Option<String>,
    }
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ProxyResponse {
        pub status: u16,
        pub data: serde_json::Value,
    }
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ServiceStatus {
        pub service: String,
        pub healthy: bool,
        pub response_time_ms: u64,
    }
}

// Minimal handlers for gateway functionality  
pub mod handlers {
    use axum::{extract::{Query, State}, Json};
    use crate::types::{ProxyRequest, ProxyResponse, ServiceStatus};
    use crate::circuit_breaker::{CircuitBreaker, CircuitBreakerConfig};
    use kembridge_common::ServiceResponse;
    use std::sync::Arc;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Deserialize)]
    pub struct NonceRequest {
        pub wallet_address: String,
        pub chain_type: String,
    }

    #[derive(Debug, Serialize)]
    pub struct NonceResponse {
        pub nonce: String,
        pub message: String,
    }

    #[derive(Debug, Deserialize)]
    pub struct VerifyWalletRequest {
        pub wallet_address: String,
        pub signature: String,
        pub nonce: String,
        pub chain_type: String,
        pub message: String,
    }

    #[derive(Debug, Serialize)]
    pub struct VerifyWalletResponse {
        pub verified: bool,
        pub wallet_address: String,
        pub chain_type: String,
        pub session_token: Option<String>,
    }

    #[derive(Debug, Serialize)]
    pub struct BridgeToken {
        pub id: String,
        pub symbol: String,
        pub name: String,
        pub decimals: u8,
        pub chain: String,
        pub contract_address: Option<String>,
        pub is_native: bool,
        pub icon_url: Option<String>,
        pub balance: Option<String>,
        pub usd_price: Option<f64>,
    }
    
    pub async fn simple_proxy(
        State(circuit_breaker): State<Arc<CircuitBreaker>>,
        Query(request): Query<ProxyRequest>,
    ) -> Result<Json<ServiceResponse<ProxyResponse>>, crate::errors::GatewayServiceError> {
        // Check if circuit breaker allows the request
        if !circuit_breaker.is_request_allowed(&request.service) {
            tracing::warn!("Circuit breaker OPEN for service: {}", request.service);
            return Ok(Json(ServiceResponse::error(format!("Circuit breaker OPEN for service: {}", request.service))));
        }
        
        // Simulate request processing (in real implementation, make HTTP call here)
        let response = ProxyResponse {
            status: 200,
            data: serde_json::json!({
                "proxied_to": request.service,
                "path": request.path,
                "result": "mock response with circuit breaker protection"
            }),
        };
        
        // Record success for circuit breaker
        circuit_breaker.record_success(&request.service);
        
        Ok(Json(ServiceResponse::success(response)))
    }
    
    pub async fn services_status() -> Result<Json<ServiceResponse<Vec<ServiceStatus>>>, crate::errors::GatewayServiceError> {
        let services = vec![
            ServiceStatus {
                service: "1inch-service".to_string(),
                healthy: true,
                response_time_ms: 25,
            },
            ServiceStatus {
                service: "blockchain-service".to_string(),
                healthy: true,
                response_time_ms: 30,
            },
            ServiceStatus {
                service: "crypto-service".to_string(),
                healthy: true,
                response_time_ms: 15,
            },
            ServiceStatus {
                service: "auth-service".to_string(),
                healthy: true,
                response_time_ms: 20,
            },
        ];
        
        Ok(Json(ServiceResponse::success(services)))
    }
    
    pub async fn health() -> Json<ServiceResponse<String>> {
        Json(ServiceResponse::success("🔥 Gateway Service with HOT RELOAD - OK".to_string()))
    }
    
    pub async fn circuit_breaker_status(
        State(circuit_breaker): State<Arc<CircuitBreaker>>,
    ) -> Json<ServiceResponse<Vec<serde_json::Value>>> {
        let services = ["1inch-service", "blockchain-service", "crypto-service", "auth-service"];
        let mut status_list = Vec::new();
        
        for service in services {
            let (state, failures, successes) = circuit_breaker.get_stats(service);
            status_list.push(serde_json::json!({
                "service": service,
                "state": format!("{:?}", state),
                "failure_count": failures,
                "success_count": successes
            }));
        }
        
        Json(ServiceResponse::success(status_list))
    }

    // Auth API handlers
    pub async fn get_nonce(
        Query(params): Query<NonceRequest>,
    ) -> Result<Json<ServiceResponse<NonceResponse>>, crate::errors::GatewayServiceError> {
        tracing::info!("🔑 Getting nonce for wallet: {} chain: {}", params.wallet_address, params.chain_type);
        
        // Generate a 32-byte nonce for NEAR compatibility
        let nonce_bytes: [u8; 32] = rand::random();
        let nonce = hex::encode(nonce_bytes);
        
        let message = format!(
            "KEMBridge Authentication\n\nWallet: {}\nChain: {}\nNonce: {}\nTimestamp: {}",
            params.wallet_address,
            params.chain_type,
            nonce,
            chrono::Utc::now().to_rfc3339()
        );
        
        let response = NonceResponse {
            nonce,
            message,
        };
        
        tracing::info!("✅ Nonce generated successfully (32 bytes)");
        Ok(Json(ServiceResponse::success(response)))
    }

    pub async fn verify_wallet(
        Json(payload): Json<VerifyWalletRequest>,
    ) -> Result<Json<ServiceResponse<VerifyWalletResponse>>, crate::errors::GatewayServiceError> {
        tracing::info!("🔐 Verifying wallet signature for: {} chain: {}", payload.wallet_address, payload.chain_type);
        
        // Basic validation
        if payload.wallet_address.is_empty() || payload.signature.is_empty() || payload.nonce.is_empty() {
            tracing::warn!("❌ Invalid verification request: missing required fields");
            return Ok(Json(ServiceResponse::error("Missing required fields for verification".to_string())));
        }

        // For demo purposes, we'll accept all signatures as valid
        // In production, this would verify the signature cryptographically
        let is_valid_signature = !payload.signature.is_empty() && payload.signature.starts_with("0x");
        
        if is_valid_signature {
            tracing::info!("✅ Wallet signature verified successfully");
            
            // Generate a mock session token
            let session_token = format!("session_{}", uuid::Uuid::new_v4());
            
            let response = VerifyWalletResponse {
                verified: true,
                wallet_address: payload.wallet_address,
                chain_type: payload.chain_type,
                session_token: Some(session_token),
            };
            
            Ok(Json(ServiceResponse::success(response)))
        } else {
            tracing::warn!("❌ Invalid wallet signature");
            
            let response = VerifyWalletResponse {
                verified: false,
                wallet_address: payload.wallet_address,
                chain_type: payload.chain_type,
                session_token: None,
            };
            
            Ok(Json(ServiceResponse::success(response)))
        }
    }

    /// Mock endpoint for bridge supported tokens
    pub async fn get_bridge_tokens() -> Result<Json<ServiceResponse<Vec<BridgeToken>>>, crate::errors::GatewayServiceError> {
        tracing::info!("🪙 Getting supported bridge tokens");

        // Mock data for testing
        let tokens = vec![
            BridgeToken {
                id: "eth-ethereum".to_string(),
                symbol: "ETH".to_string(),
                name: "Ethereum".to_string(),
                decimals: 18,
                chain: "ethereum".to_string(),
                contract_address: None,
                is_native: true,
                icon_url: Some("https://cryptologos.cc/logos/ethereum-eth-logo.svg".to_string()),
                balance: Some("1.5".to_string()),
                usd_price: Some(2500.0),
            },
            BridgeToken {
                id: "usdc-ethereum".to_string(),
                symbol: "USDC".to_string(),
                name: "USD Coin".to_string(),
                decimals: 6,
                chain: "ethereum".to_string(),
                contract_address: Some("0xA0b86a33E6417c9D6b4a1eC4a4B4B1a5B1A1B1A1".to_string()),
                is_native: false,
                icon_url: Some("https://cryptologos.cc/logos/usd-coin-usdc-logo.svg".to_string()),
                balance: Some("100.0".to_string()),
                usd_price: Some(1.0),
            },
            BridgeToken {
                id: "near-near".to_string(),
                symbol: "NEAR".to_string(),
                name: "NEAR Protocol".to_string(),
                decimals: 24,
                chain: "near".to_string(),
                contract_address: None,
                is_native: true,
                icon_url: Some("https://cryptologos.cc/logos/near-protocol-near-logo.svg".to_string()),
                balance: Some("50.0".to_string()),
                usd_price: Some(3.5),
            },
            BridgeToken {
                id: "usdc-near".to_string(),
                symbol: "USDC".to_string(),
                name: "USD Coin (NEAR)".to_string(),
                decimals: 6,
                chain: "near".to_string(),
                contract_address: Some("usdc.near".to_string()),
                is_native: false,
                icon_url: Some("https://cryptologos.cc/logos/usd-coin-usdc-logo.svg".to_string()),
                balance: Some("75.0".to_string()),
                usd_price: Some(1.0),
            },
        ];

        Ok(Json(ServiceResponse::success(tokens)))
    }

    /// Mock endpoint for bridge history
    pub async fn get_bridge_history() -> Result<Json<ServiceResponse<serde_json::Value>>, crate::errors::GatewayServiceError> {
        tracing::info!("📜 Getting bridge transaction history");

        let history = serde_json::json!({
            "transactions": [],
            "total_count": 0,
            "page": 1,
            "page_size": 10
        });

        Ok(Json(ServiceResponse::success(history)))
    }
}

// Re-export for easy access
pub use types::*;
pub use errors::*;