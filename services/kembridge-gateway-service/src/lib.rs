// Minimal gateway service for architecture testing
pub mod circuit_breaker;
pub mod clients;
pub mod config;
pub mod errors;
pub mod event_api;
pub mod event_listener;
pub mod jwt_validation;
pub mod middleware;
pub mod websocket;

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
    use crate::circuit_breaker::CircuitBreaker;
    use crate::types::{ProxyRequest, ProxyResponse, ServiceStatus};
    use axum::{
        extract::{Query, State, Path},
        Json,
    };
    use kembridge_common::ServiceResponse;
    use serde::{Deserialize, Serialize};
    use std::sync::Arc;
    use chrono;

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
        State((circuit_breaker, _)): State<(
            Arc<CircuitBreaker>,
            crate::websocket::ConnectionManager,
        )>,
        Query(request): Query<ProxyRequest>,
    ) -> Result<Json<ServiceResponse<ProxyResponse>>, crate::errors::GatewayServiceError> {
        // Check if circuit breaker allows the request
        if !circuit_breaker.is_request_allowed(&request.service) {
            tracing::warn!("Circuit breaker OPEN for service: {}", request.service);
            return Ok(Json(ServiceResponse::error(format!(
                "Circuit breaker OPEN for service: {}",
                request.service
            ))));
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

    pub async fn services_status(
    ) -> Result<Json<ServiceResponse<Vec<ServiceStatus>>>, crate::errors::GatewayServiceError> {
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
        Json(ServiceResponse::success(
            "🔥 Gateway Service with HOT RELOAD - OK".to_string(),
        ))
    }

    pub async fn circuit_breaker_status(
        State((circuit_breaker, _)): State<(
            Arc<CircuitBreaker>,
            crate::websocket::ConnectionManager,
        )>,
    ) -> Json<ServiceResponse<Vec<serde_json::Value>>> {
        let services = [
            "1inch-service",
            "blockchain-service",
            "crypto-service",
            "auth-service",
        ];
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
        // Validate required parameters
        if params.wallet_address.is_empty() {
            tracing::warn!("❌ Missing wallet_address parameter");
            return Err(crate::errors::GatewayServiceError::ValidationError(
                "Missing required parameter: wallet_address".to_string()
            ));
        }
        
        if params.chain_type.is_empty() {
            tracing::warn!("❌ Missing chain_type parameter");
            return Err(crate::errors::GatewayServiceError::ValidationError(
                "Missing required parameter: chain_type".to_string()
            ));
        }
        
        // Validate chain type
        let supported_chains = ["ethereum", "near"];
        if !supported_chains.contains(&params.chain_type.as_str()) {
            tracing::warn!("❌ Invalid chain_type: {}", params.chain_type);
            return Err(crate::errors::GatewayServiceError::ValidationError(
                format!("Invalid chain_type '{}'. Supported: {:?}", params.chain_type, supported_chains)
            ));
        }
        
        // Validate wallet address format
        if params.chain_type == "ethereum" {
            if !params.wallet_address.starts_with("0x") || params.wallet_address.len() != 42 {
                tracing::warn!("❌ Invalid Ethereum wallet address: {}", params.wallet_address);
                return Err(crate::errors::GatewayServiceError::ValidationError(
                    "Invalid Ethereum wallet address format".to_string()
                ));
            }
        } else if params.chain_type == "near" {
            if params.wallet_address.len() < 2 || params.wallet_address.len() > 64 {
                tracing::warn!("❌ Invalid NEAR wallet address: {}", params.wallet_address);
                return Err(crate::errors::GatewayServiceError::ValidationError(
                    "Invalid NEAR wallet address format".to_string()
                ));
            }
        }
        
        tracing::info!(
            "🔑 Getting nonce for wallet: {} chain: {}",
            params.wallet_address,
            params.chain_type
        );

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

        let response = NonceResponse { nonce, message };

        tracing::info!("✅ Nonce generated successfully (32 bytes)");
        Ok(Json(ServiceResponse::success(response)))
    }

    pub async fn verify_wallet(
        Json(payload): Json<VerifyWalletRequest>,
    ) -> Result<Json<ServiceResponse<VerifyWalletResponse>>, crate::errors::GatewayServiceError>
    {
        tracing::info!(
            "🔐 Verifying wallet signature for: {} chain: {}",
            payload.wallet_address,
            payload.chain_type
        );

        // Basic validation
        if payload.wallet_address.is_empty()
            || payload.signature.is_empty()
            || payload.nonce.is_empty()
        {
            tracing::warn!("❌ Invalid verification request: missing required fields");
            return Ok(Json(ServiceResponse::error(
                "Missing required fields for verification".to_string(),
            )));
        }

        // For demo purposes, we'll accept all signatures as valid
        // In production, this would verify the signature cryptographically
        let is_valid_signature =
            !payload.signature.is_empty() && payload.signature.starts_with("0x");

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
    pub async fn get_bridge_tokens(
    ) -> Result<Json<ServiceResponse<Vec<BridgeToken>>>, crate::errors::GatewayServiceError> {
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
                icon_url: Some(
                    "https://cryptologos.cc/logos/near-protocol-near-logo.svg".to_string(),
                ),
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

    #[derive(Debug, Deserialize)]
    pub struct BridgeQuoteRequest {
        pub from_chain: Option<String>,
        pub to_chain: Option<String>,
        pub from_token: Option<String>,
        pub to_token: Option<String>,
        pub from_amount: Option<String>,
    }

    /// Bridge quote endpoint with validation
    pub async fn get_bridge_quote(
        Query(params): Query<BridgeQuoteRequest>,
    ) -> Result<Json<ServiceResponse<serde_json::Value>>, crate::errors::GatewayServiceError> {
        tracing::info!("💱 Getting bridge quote with params: {:?}", params);

        // Validate required parameters
        let from_chain = params.from_chain.as_ref()
            .ok_or_else(|| crate::errors::GatewayServiceError::ValidationError(
                "Missing required parameter: from_chain".to_string()
            ))?;
        
        let to_chain = params.to_chain.as_ref()
            .ok_or_else(|| crate::errors::GatewayServiceError::ValidationError(
                "Missing required parameter: to_chain".to_string()
            ))?;
            
        let from_token = params.from_token.as_ref()
            .ok_or_else(|| crate::errors::GatewayServiceError::ValidationError(
                "Missing required parameter: from_token".to_string()
            ))?;
            
        let to_token = params.to_token.as_ref()
            .ok_or_else(|| crate::errors::GatewayServiceError::ValidationError(
                "Missing required parameter: to_token".to_string()
            ))?;
            
        let from_amount = params.from_amount.as_ref()
            .ok_or_else(|| crate::errors::GatewayServiceError::ValidationError(
                "Missing required parameter: from_amount".to_string()
            ))?;

        // Validate chains
        let supported_chains = ["ethereum", "near"];
        if !supported_chains.contains(&from_chain.as_str()) {
            return Err(crate::errors::GatewayServiceError::ValidationError(
                format!("Invalid from_chain '{}'. Supported: {:?}", from_chain, supported_chains)
            ));
        }
        if !supported_chains.contains(&to_chain.as_str()) {
            return Err(crate::errors::GatewayServiceError::ValidationError(
                format!("Invalid to_chain '{}'. Supported: {:?}", to_chain, supported_chains)
            ));
        }

        // Validate and parse amount
        let amount: f64 = from_amount.parse()
            .map_err(|_| crate::errors::GatewayServiceError::ValidationError(
                format!("Invalid from_amount '{}'. Must be a valid number", from_amount)
            ))?;

        if amount <= 0.0 {
            return Err(crate::errors::GatewayServiceError::ValidationError(
                "from_amount must be greater than 0".to_string()
            ));
        }
        
        // Validate extreme amounts
        if amount > 1_000_000.0 {
            return Err(crate::errors::GatewayServiceError::ValidationError(
                "from_amount exceeds maximum allowed value of 1,000,000".to_string()
            ));
        }
        
        if amount < 0.000001 {
            return Err(crate::errors::GatewayServiceError::ValidationError(
                "from_amount is below minimum allowed value of 0.000001".to_string()
            ));
        }

        // Validate token pairs
        let supported_tokens = ["ETH", "NEAR", "USDC", "USDT"];
        if !supported_tokens.contains(&from_token.as_str()) {
            return Err(crate::errors::GatewayServiceError::ValidationError(
                format!("Unsupported from_token '{}'. Supported: {:?}", from_token, supported_tokens)
            ));
        }
        if !supported_tokens.contains(&to_token.as_str()) {
            return Err(crate::errors::GatewayServiceError::ValidationError(
                format!("Unsupported to_token '{}'. Supported: {:?}", to_token, supported_tokens)
            ));
        }

        // Mock quote response
        let quote = serde_json::json!({
            "quote_id": format!("quote_{}", chrono::Utc::now().timestamp()),
            "from_chain": from_chain,
            "to_chain": to_chain,
            "from_token": from_token,
            "to_token": to_token,
            "from_amount": from_amount,
            "to_amount": (amount * 1164.39).to_string(),
            "exchange_rate": 1164.39,
            "estimated_fees": {
                "gas_fee": "0.005",
                "bridge_fee": "0.001",
                "protocol_fee": "0.004",
                "total_fee": "0.01"
            },
            "price_impact": 0.01,
            "estimated_time_minutes": 12,
            "expires_at": (chrono::Utc::now() + chrono::Duration::minutes(10)).to_rfc3339(),
            "quantum_protection_enabled": true,
            "route_info": {
                "risk_score": 0.15
            }
        });

        tracing::info!("✅ Bridge quote generated successfully");
        Ok(Json(ServiceResponse::success(quote)))
    }

    /// Mock endpoint for bridge history
    pub async fn get_bridge_history(
    ) -> Result<Json<ServiceResponse<serde_json::Value>>, crate::errors::GatewayServiceError> {
        tracing::info!("📜 Getting bridge transaction history");

        let history = serde_json::json!({
            "transactions": [],
            "total_count": 0,
            "page": 1,
            "page_size": 10
        });

        Ok(Json(ServiceResponse::success(history)))
    }

    /// Get quantum cryptography system status - PROXY TO REAL CRYPTO-SERVICE
    pub async fn get_crypto_status(
        State((circuit_breaker, _)): State<(
            Arc<CircuitBreaker>,
            crate::websocket::ConnectionManager,
        )>,
    ) -> Result<Json<ServiceResponse<serde_json::Value>>, crate::errors::GatewayServiceError> {
        tracing::info!("🔐 Proxying to real crypto-service for quantum status");

        // Check circuit breaker
        if !circuit_breaker.is_request_allowed("crypto-service") {
            tracing::warn!("Circuit breaker OPEN for crypto-service");
            return Ok(Json(ServiceResponse::error(
                "Crypto service temporarily unavailable".to_string(),
            )));
        }

        // Make request to real crypto-service
        let client = reqwest::Client::new();
        let crypto_service_url = std::env::var("CRYPTO_SERVICE_URL")
            .unwrap_or_else(|_| "http://crypto-service-dev:4003".to_string());

        match client
            .get(&format!("{}/status", crypto_service_url))
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await
        {
            Ok(response) => {
                if response.status().is_success() {
                    match response.json::<ServiceResponse<serde_json::Value>>().await {
                        Ok(crypto_response) => {
                            circuit_breaker.record_success("crypto-service");

                            // Transform response to match old backend format
                            let transformed_data = if let Some(data) = crypto_response.data {
                                if let Some(quantum_protection) = data.get("quantum_protection") {
                                    serde_json::json!({
                                        "quantumProtection": {
                                            "isActive": quantum_protection.get("is_active").unwrap_or(&serde_json::Value::Bool(false)),
                                            "algorithm": quantum_protection.get("algorithm").unwrap_or(&serde_json::Value::String("ML-KEM-1024".to_string())),
                                            "keyRotationDate": quantum_protection.get("key_rotation_date").unwrap_or(&serde_json::Value::String(chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string())),
                                            "nextRotationDue": quantum_protection.get("next_rotation_due").unwrap_or(&serde_json::Value::String((chrono::Utc::now() + chrono::Duration::days(30)).format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string())),
                                            "encryptionStrength": quantum_protection.get("encryption_strength").unwrap_or(&serde_json::Value::Number(serde_json::Number::from(1024)))
                                        },
                                        "overall": data.get("overall").unwrap_or(&serde_json::Value::String("UNKNOWN".to_string())),
                                        "isOnline": data.get("is_online").unwrap_or(&serde_json::Value::Bool(true)),
                                        "lastUpdate": data.get("last_update").unwrap_or(&serde_json::Value::String(chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string())),
                                        "systemHealth": data.get("system_health").unwrap_or(&serde_json::json!({
                                            "backend": true,
                                            "aiEngine": true,
                                            "blockchain": true
                                        }))
                                    })
                                } else {
                                    data
                                }
                            } else {
                                serde_json::json!({
                                    "error": "No data received from crypto service"
                                })
                            };

                            Ok(Json(ServiceResponse::success(transformed_data)))
                        }
                        Err(e) => {
                            circuit_breaker.record_failure("crypto-service");
                            tracing::error!("Failed to parse crypto service response: {}", e);
                            Ok(Json(ServiceResponse::error(format!(
                                "Invalid response from crypto service: {}",
                                e
                            ))))
                        }
                    }
                } else {
                    circuit_breaker.record_failure("crypto-service");
                    tracing::error!("Crypto service returned error: {}", response.status());
                    Ok(Json(ServiceResponse::error(format!(
                        "Crypto service error: {}",
                        response.status()
                    ))))
                }
            }
            Err(e) => {
                circuit_breaker.record_failure("crypto-service");
                tracing::error!("Failed to connect to crypto service: {}", e);
                Ok(Json(ServiceResponse::error(format!(
                    "Failed to connect to crypto service: {}",
                    e
                ))))
            }
        }
    }

    /// Check quantum key rotation status - PROXY TO REAL CRYPTO-SERVICE
    pub async fn check_key_rotation(
        State((circuit_breaker, _)): State<(
            Arc<CircuitBreaker>,
            crate::websocket::ConnectionManager,
        )>,
        Json(payload): Json<serde_json::Value>,
    ) -> Result<Json<ServiceResponse<serde_json::Value>>, crate::errors::GatewayServiceError> {
        tracing::info!("🔑 Proxying to real crypto-service for key rotation check");

        // Check circuit breaker
        if !circuit_breaker.is_request_allowed("crypto-service") {
            tracing::warn!("Circuit breaker OPEN for crypto-service");
            return Ok(Json(ServiceResponse::error(
                "Crypto service temporarily unavailable".to_string(),
            )));
        }

        // Make request to real crypto-service
        let client = reqwest::Client::new();
        let crypto_service_url = std::env::var("CRYPTO_SERVICE_URL")
            .unwrap_or_else(|_| "http://crypto-service-dev:4003".to_string());

        // Use payload from request or default values
        let rotation_request = if payload.is_object() && !payload.as_object().unwrap().is_empty() {
            payload
        } else {
            serde_json::json!({"threshold_days": 30})
        };

        match client
            .post(&format!("{}/keys/check-rotation", crypto_service_url))
            .json(&rotation_request)
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await
        {
            Ok(response) => {
                if response.status().is_success() {
                    match response.json::<ServiceResponse<serde_json::Value>>().await {
                        Ok(crypto_response) => {
                            circuit_breaker.record_success("crypto-service");

                            // Transform response to match old backend format if needed
                            let transformed_data = if let Some(data) = crypto_response.data {
                                serde_json::json!({
                                    "rotationDue": data.get("rotation_due").unwrap_or(&serde_json::Value::Bool(false)),
                                    "lastRotation": chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string(),
                                    "nextRotation": data.get("next_rotation_date").unwrap_or(&serde_json::Value::String((chrono::Utc::now() + chrono::Duration::days(30)).format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string())),
                                    "activeKeys": data.get("total_keys").unwrap_or(&serde_json::Value::Number(serde_json::Number::from(0)))
                                })
                            } else {
                                serde_json::json!({
                                    "error": "No data received from crypto service"
                                })
                            };

                            Ok(Json(ServiceResponse::success(transformed_data)))
                        }
                        Err(e) => {
                            circuit_breaker.record_failure("crypto-service");
                            tracing::error!("Failed to parse crypto service response: {}", e);
                            Ok(Json(ServiceResponse::error(format!(
                                "Invalid response from crypto service: {}",
                                e
                            ))))
                        }
                    }
                } else {
                    circuit_breaker.record_failure("crypto-service");
                    tracing::error!("Crypto service returned error: {}", response.status());
                    Ok(Json(ServiceResponse::error(format!(
                        "Crypto service error: {}",
                        response.status()
                    ))))
                }
            }
            Err(e) => {
                circuit_breaker.record_failure("crypto-service");
                tracing::error!("Failed to connect to crypto service: {}", e);
                Ok(Json(ServiceResponse::error(format!(
                    "Failed to connect to crypto service: {}",
                    e
                ))))
            }
        }
    }

    /// Get crypto keys list - PROXY TO REAL CRYPTO-SERVICE
    pub async fn get_crypto_keys(
        State((circuit_breaker, _)): State<(
            Arc<CircuitBreaker>,
            crate::websocket::ConnectionManager,
        )>,
    ) -> Result<Json<ServiceResponse<serde_json::Value>>, crate::errors::GatewayServiceError> {
        tracing::info!("🔑 Proxying to real crypto-service for keys list");

        // Check circuit breaker
        if !circuit_breaker.is_request_allowed("crypto-service") {
            tracing::warn!("Circuit breaker OPEN for crypto-service");
            return Ok(Json(ServiceResponse::error(
                "Crypto service temporarily unavailable".to_string(),
            )));
        }

        // Make request to real crypto-service
        let client = reqwest::Client::new();
        let crypto_service_url = std::env::var("CRYPTO_SERVICE_URL")
            .unwrap_or_else(|_| "http://crypto-service-dev:4003".to_string());

        match client
            .get(&format!("{}/keys", crypto_service_url))
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await
        {
            Ok(response) => {
                if response.status().is_success() {
                    match response.json::<ServiceResponse<serde_json::Value>>().await {
                        Ok(crypto_response) => {
                            circuit_breaker.record_success("crypto-service");

                            // Transform response to match frontend expectations
                            let transformed_data = if let Some(data) = crypto_response.data {
                                if let Some(keys) = data.get("keys") {
                                    serde_json::json!({
                                        "keys": keys,
                                        "totalCount": data.get("total_count").unwrap_or(&serde_json::Value::Number(serde_json::Number::from(0))),
                                        "activeKeys": data.get("active_keys").unwrap_or(&serde_json::Value::Number(serde_json::Number::from(0))),
                                        "lastUpdate": chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string()
                                    })
                                } else {
                                    data
                                }
                            } else {
                                serde_json::json!({
                                    "keys": [],
                                    "totalCount": 0,
                                    "activeKeys": 0,
                                    "error": "No data received from crypto service"
                                })
                            };

                            Ok(Json(ServiceResponse::success(transformed_data)))
                        }
                        Err(e) => {
                            circuit_breaker.record_failure("crypto-service");
                            tracing::error!("Failed to parse crypto service response: {}", e);
                            Ok(Json(ServiceResponse::error(format!(
                                "Invalid response from crypto service: {}",
                                e
                            ))))
                        }
                    }
                } else {
                    circuit_breaker.record_failure("crypto-service");
                    tracing::error!("Crypto service returned error: {}", response.status());
                    Ok(Json(ServiceResponse::error(format!(
                        "Crypto service error: {}",
                        response.status()
                    ))))
                }
            }
            Err(e) => {
                circuit_breaker.record_failure("crypto-service");
                tracing::error!("Failed to connect to crypto service: {}", e);
                Ok(Json(ServiceResponse::error(format!(
                    "Failed to connect to crypto service: {}",
                    e
                ))))
            }
        }
    }

    /// Generate new crypto key - PROXY TO REAL CRYPTO-SERVICE
    pub async fn generate_crypto_key(
        State((circuit_breaker, _)): State<(
            Arc<CircuitBreaker>,
            crate::websocket::ConnectionManager,
        )>,
        Json(payload): Json<serde_json::Value>,
    ) -> Result<Json<ServiceResponse<serde_json::Value>>, crate::errors::GatewayServiceError> {
        tracing::info!("🔐 Proxying to real crypto-service for key generation");

        // Check circuit breaker
        if !circuit_breaker.is_request_allowed("crypto-service") {
            tracing::warn!("Circuit breaker OPEN for crypto-service");
            return Ok(Json(ServiceResponse::error(
                "Crypto service temporarily unavailable".to_string(),
            )));
        }

        // Make request to real crypto-service
        let client = reqwest::Client::new();
        let crypto_service_url = std::env::var("CRYPTO_SERVICE_URL")
            .unwrap_or_else(|_| "http://crypto-service-dev:4003".to_string());

        // Ensure we have the required fields for key generation
        let key_request = if payload.get("key_type").is_none() {
            serde_json::json!({
                "key_type": "ml_kem_1024",
                "usage_category": payload.get("purpose").unwrap_or(&serde_json::Value::String("gateway_generated".to_string())),
                "expires_in_days": 90
            })
        } else {
            payload
        };

        match client
            .post(&format!("{}/keys/generate", crypto_service_url))
            .json(&key_request)
            .timeout(std::time::Duration::from_secs(15))
            .send()
            .await
        {
            Ok(response) => {
                if response.status().is_success() {
                    match response.json::<ServiceResponse<serde_json::Value>>().await {
                        Ok(crypto_response) => {
                            circuit_breaker.record_success("crypto-service");

                            // Transform response to match frontend expectations
                            let transformed_data = if let Some(data) = crypto_response.data {
                                serde_json::json!({
                                    "keyId": data.get("key_id").unwrap_or(&serde_json::Value::String(uuid::Uuid::new_v4().to_string())),
                                    "algorithm": data.get("algorithm").unwrap_or(&serde_json::Value::String("ML-KEM-1024".to_string())),
                                    "publicKey": data.get("public_key").unwrap_or(&serde_json::Value::String("".to_string())),
                                    "keyType": data.get("key_type").unwrap_or(&serde_json::Value::String("ml_kem_1024".to_string())),
                                    "createdAt": data.get("created_at").unwrap_or(&serde_json::Value::String(chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string())),
                                    "encryptionStrength": data.get("encryption_strength").unwrap_or(&serde_json::Value::Number(serde_json::Number::from(1024))),
                                    "isActive": true
                                })
                            } else {
                                serde_json::json!({
                                    "error": "No data received from crypto service"
                                })
                            };

                            Ok(Json(ServiceResponse::success(transformed_data)))
                        }
                        Err(e) => {
                            circuit_breaker.record_failure("crypto-service");
                            tracing::error!("Failed to parse crypto service response: {}", e);
                            Ok(Json(ServiceResponse::error(format!(
                                "Invalid response from crypto service: {}",
                                e
                            ))))
                        }
                    }
                } else {
                    circuit_breaker.record_failure("crypto-service");
                    tracing::error!("Crypto service returned error: {}", response.status());
                    Ok(Json(ServiceResponse::error(format!(
                        "Crypto service error: {}",
                        response.status()
                    ))))
                }
            }
            Err(e) => {
                circuit_breaker.record_failure("crypto-service");
                tracing::error!("Failed to connect to crypto service: {}", e);
                Ok(Json(ServiceResponse::error(format!(
                    "Failed to connect to crypto service: {}",
                    e
                ))))
            }
        }
    }

    /// Trigger manual key rotation - PROXY TO REAL CRYPTO-SERVICE
    pub async fn trigger_key_rotation(
        State((circuit_breaker, _)): State<(
            Arc<CircuitBreaker>,
            crate::websocket::ConnectionManager,
        )>,
    ) -> Result<Json<ServiceResponse<serde_json::Value>>, crate::errors::GatewayServiceError> {
        tracing::info!("🔄 Proxying to real crypto-service for key rotation");

        // Check circuit breaker
        if !circuit_breaker.is_request_allowed("crypto-service") {
            tracing::warn!("Circuit breaker OPEN for crypto-service");
            return Ok(Json(ServiceResponse::error(
                "Crypto service temporarily unavailable".to_string(),
            )));
        }

        // Make request to real crypto-service
        let client = reqwest::Client::new();
        let crypto_service_url = std::env::var("CRYPTO_SERVICE_URL")
            .unwrap_or_else(|_| "http://crypto-service-dev:4003".to_string());

        // First generate a new key to get a key_id for rotation
        let new_key = match client
            .post(&format!("{}/keys/generate", crypto_service_url))
            .json(&serde_json::json!({
                "key_type": "ml_kem_1024",
                "usage_category": "system_rotation",
                "expires_in_days": 90
            }))
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await
        {
            Ok(response) if response.status().is_success() => {
                match response.json::<ServiceResponse<serde_json::Value>>().await {
                    Ok(key_response) => {
                        if let Some(data) = key_response.data {
                            data.get("key_id")
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string())
                        } else {
                            None
                        }
                    }
                    Err(_) => None,
                }
            }
            _ => None,
        };

        if let Some(key_id) = new_key {
            // Now rotate the key
            match client
                .post(&format!("{}/keys/rotate", crypto_service_url))
                .json(&serde_json::json!({
                    "key_id": key_id,
                    "reason": "Manual rotation triggered via gateway"
                }))
                .timeout(std::time::Duration::from_secs(10))
                .send()
                .await
            {
                Ok(response) => {
                    if response.status().is_success() {
                        match response.json::<ServiceResponse<serde_json::Value>>().await {
                            Ok(crypto_response) => {
                                circuit_breaker.record_success("crypto-service");

                                // Transform response to match old backend format if needed
                                let transformed_data = if let Some(data) = crypto_response.data {
                                    serde_json::json!({
                                        "success": data.get("success").unwrap_or(&serde_json::Value::Bool(true)),
                                        "newKeyId": data.get("new_key_id").unwrap_or(&serde_json::Value::String(uuid::Uuid::new_v4().to_string())),
                                        "rotatedAt": data.get("rotation_completed_at").unwrap_or(&serde_json::Value::String(chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string())),
                                        "algorithm": data.get("algorithm").unwrap_or(&serde_json::Value::String("ML-KEM-1024".to_string()))
                                    })
                                } else {
                                    serde_json::json!({
                                        "error": "No data received from crypto service"
                                    })
                                };

                                Ok(Json(ServiceResponse::success(transformed_data)))
                            }
                            Err(e) => {
                                circuit_breaker.record_failure("crypto-service");
                                tracing::error!("Failed to parse crypto service response: {}", e);
                                Ok(Json(ServiceResponse::error(format!(
                                    "Invalid response from crypto service: {}",
                                    e
                                ))))
                            }
                        }
                    } else {
                        circuit_breaker.record_failure("crypto-service");
                        tracing::error!("Crypto service returned error: {}", response.status());
                        Ok(Json(ServiceResponse::error(format!(
                            "Crypto service error: {}",
                            response.status()
                        ))))
                    }
                }
                Err(e) => {
                    circuit_breaker.record_failure("crypto-service");
                    tracing::error!("Failed to connect to crypto service: {}", e);
                    Ok(Json(ServiceResponse::error(format!(
                        "Failed to connect to crypto service: {}",
                        e
                    ))))
                }
            }
        } else {
            circuit_breaker.record_failure("crypto-service");
            tracing::error!("Failed to generate new key for rotation");
            Ok(Json(ServiceResponse::error(
                "Failed to generate new key for rotation".to_string(),
            )))
        }
    }

    /// Test endpoint for error handling system
    pub async fn test_error_handling(
        Query(params): Query<std::collections::HashMap<String, String>>,
    ) -> Result<Json<ServiceResponse<serde_json::Value>>, crate::errors::GatewayServiceError> {
        let error_type = params.get("type").unwrap_or(&"validation".to_string()).clone();
        
        tracing::info!("🧪 Testing error handling system with type: {}", error_type);
        
        match error_type.as_str() {
            "validation" => Err(crate::errors::GatewayServiceError::ValidationError(
                "Test validation error: Invalid parameters provided".to_string()
            )),
            "auth" => Err(crate::errors::GatewayServiceError::AuthenticationRequired),
            "network" => Err(crate::errors::GatewayServiceError::ServiceTimeout {
                service: "test_service".to_string()
            }),
            "service" => Err(crate::errors::GatewayServiceError::ServiceUnavailable {
                service: "test_service".to_string()
            }),
            _ => Ok(Json(ServiceResponse::success(serde_json::json!({
                "message": "Error handling test completed successfully",
                "tested_type": error_type,
                "recovery_available": true
            }))))
        }
    }

    /// Analyze transaction risk via AI Engine
    pub async fn analyze_transaction_risk(
        State((circuit_breaker, _)): State<(
            Arc<CircuitBreaker>,
            crate::websocket::ConnectionManager,
        )>,
        Json(request): Json<TransactionRiskRequest>,
    ) -> Result<Json<ServiceResponse<RiskAnalysisResponse>>, crate::errors::GatewayServiceError> {
        tracing::info!("🔍 Proxying transaction risk analysis to AI Engine");

        // Check circuit breaker
        if !circuit_breaker.is_request_allowed("ai-engine") {
            tracing::warn!("Circuit breaker OPEN for AI Engine");
            return Ok(Json(ServiceResponse::error(
                "AI Engine temporarily unavailable".to_string(),
            )));
        }

        // Make request to AI Engine
        let client = reqwest::Client::new();
        let ai_engine_url = std::env::var("AI_ENGINE_URL")
            .unwrap_or_else(|_| "http://ai-engine:8001".to_string());

        match client
            .post(&format!("{}/api/risk/analyze", ai_engine_url))
            .json(&serde_json::json!({
                "from_address": request.from_address,
                "to_address": request.to_address,
                "amount": request.amount,
                "token_symbol": request.token,
                "chain": request.chain,
                "user_id": request.user_id.unwrap_or_else(|| "current_user".to_string())
            }))
            .timeout(std::time::Duration::from_secs(30))
            .send()
            .await
        {
            Ok(response) => {
                if response.status().is_success() {
                    match response.json::<RiskAnalysisResponse>().await {
                        Ok(risk_response) => {
                            circuit_breaker.record_success("ai-engine");
                            Ok(Json(ServiceResponse::success(risk_response)))
                        }
                        Err(e) => {
                            circuit_breaker.record_failure("ai-engine");
                            tracing::error!("Failed to parse AI Engine response: {}", e);
                            Ok(Json(ServiceResponse::error(format!(
                                "Invalid response from AI Engine: {}",
                                e
                            ))))
                        }
                    }
                } else {
                    circuit_breaker.record_failure("ai-engine");
                    tracing::error!("AI Engine returned error: {}", response.status());
                    Ok(Json(ServiceResponse::error(format!(
                        "AI Engine error: {}",
                        response.status()
                    ))))
                }
            }
            Err(e) => {
                circuit_breaker.record_failure("ai-engine");
                tracing::error!("AI Engine request failed: {}", e);
                Ok(Json(ServiceResponse::error(format!(
                    "Failed to connect to AI Engine: {}",
                    e
                ))))
            }
        }
    }

    /// Get current user risk profile
    pub async fn get_current_user_risk_profile(
        State((circuit_breaker, _)): State<(
            Arc<CircuitBreaker>,
            crate::websocket::ConnectionManager,
        )>,
        headers: HeaderMap,
    ) -> Result<Json<ServiceResponse<UserRiskProfileResponse>>, crate::errors::GatewayServiceError> {
        tracing::info!("📊 Getting current user risk profile");

        // Extract user_id from JWT token
        let auth_header = headers.get("authorization")
            .and_then(|h| h.to_str().ok())
            .and_then(|h| h.strip_prefix("Bearer "))
            .ok_or_else(|| crate::errors::GatewayServiceError::AuthenticationRequired)?;
        
        let jwt_secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "default_secret".to_string());
        let validation_result = crate::jwt_validation::validate_jwt_token(auth_header, &jwt_secret);
        
        if !validation_result.valid {
            tracing::warn!("❌ Invalid JWT token: {:?}", validation_result.errors);
            return Err(crate::errors::GatewayServiceError::AuthenticationRequired);
        }
        
        let user_id = validation_result.user_id.unwrap_or_else(|| "unknown".to_string());
        
        // Proxy to AI Engine
        get_user_risk_profile_impl(circuit_breaker, user_id).await
    }

    /// Get user risk profile by ID
    pub async fn get_user_risk_profile_by_id(
        State((circuit_breaker, _)): State<(
            Arc<CircuitBreaker>,
            crate::websocket::ConnectionManager,
        )>,
        Path(user_id): Path<String>,
        headers: HeaderMap,
    ) -> Result<Json<ServiceResponse<UserRiskProfileResponse>>, crate::errors::GatewayServiceError> {
        tracing::info!("📊 Getting risk profile for user: {}", user_id);

        // Validate JWT token (optional: check if user can access this profile)
        let auth_header = headers.get("authorization")
            .and_then(|h| h.to_str().ok())
            .and_then(|h| h.strip_prefix("Bearer "));
        
        if let Some(token) = auth_header {
            let jwt_secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "default_secret".to_string());
            let validation_result = crate::jwt_validation::validate_jwt_token(token, &jwt_secret);
            
            if !validation_result.valid {
                tracing::warn!("❌ Invalid JWT token: {:?}", validation_result.errors);
                return Err(crate::errors::GatewayServiceError::AuthenticationRequired);
            }
        }
        
        // Proxy to AI Engine
        get_user_risk_profile_impl(circuit_breaker, user_id).await
    }

    /// Common implementation for getting user risk profile
    async fn get_user_risk_profile_impl(
        circuit_breaker: Arc<CircuitBreaker>,
        user_id: String,
    ) -> Result<Json<ServiceResponse<UserRiskProfileResponse>>, crate::errors::GatewayServiceError> {
        // Check circuit breaker
        if !circuit_breaker.is_request_allowed("ai-engine") {
            tracing::warn!("Circuit breaker OPEN for AI Engine");
            return Ok(Json(ServiceResponse::error(
                "AI Engine temporarily unavailable".to_string(),
            )));
        }

        // Make request to AI Engine
        let client = reqwest::Client::new();
        let ai_engine_url = std::env::var("AI_ENGINE_URL")
            .unwrap_or_else(|_| "http://ai-engine:8001".to_string());

        match client
            .get(&format!("{}/api/risk/profile/{}", ai_engine_url, user_id))
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await
        {
            Ok(response) => {
                if response.status().is_success() {
                    match response.json::<UserRiskProfileResponse>().await {
                        Ok(profile_response) => {
                            circuit_breaker.record_success("ai-engine");
                            Ok(Json(ServiceResponse::success(profile_response)))
                        }
                        Err(e) => {
                            circuit_breaker.record_failure("ai-engine");
                            tracing::error!("Failed to parse AI Engine response: {}", e);
                            Ok(Json(ServiceResponse::error(format!(
                                "Invalid response from AI Engine: {}",
                                e
                            ))))
                        }
                    }
                } else {
                    circuit_breaker.record_failure("ai-engine");
                    tracing::error!("AI Engine returned error: {}", response.status());
                    Ok(Json(ServiceResponse::error(format!(
                        "AI Engine error: {}",
                        response.status()
                    ))))
                }
            }
            Err(e) => {
                circuit_breaker.record_failure("ai-engine");
                tracing::error!("AI Engine request failed: {}", e);
                Ok(Json(ServiceResponse::error(format!(
                    "Failed to connect to AI Engine: {}",
                    e
                ))))
            }
        }
    }

    // User API endpoints for security tests

    use axum::http::HeaderMap;
    
    #[derive(Debug, Serialize)]
    pub struct UserProfile {
        pub user_id: String,
        pub wallet_address: String,
        pub chain_type: String,
        pub created_at: String,
        pub last_login: String,
        pub tier: String,
    }
    
    #[derive(Debug, Serialize)]
    pub struct UserTransaction {
        pub id: String,
        pub from_chain: String,
        pub to_chain: String,
        pub from_token: String,
        pub to_token: String,
        pub amount: String,
        pub status: String,
        pub created_at: String,
        pub completed_at: Option<String>,
    }

    #[derive(Debug, Serialize)]
    pub struct BridgeTransaction {
        pub id: String,
        pub quote_id: String,
        pub status: String,
        pub from_chain: String,
        pub to_chain: String,
        pub from_token: String,
        pub to_token: String,
        pub from_amount: String,
        pub to_amount: String,
        pub from_wallet_address: String,
        pub to_wallet_address: String,
        pub from_transaction_hash: Option<String>,
        pub to_transaction_hash: Option<String>,
        pub created_at: String,
        pub updated_at: String,
        pub estimated_completion_at: Option<String>,
        pub actual_completion_at: Option<String>,
        pub risk_analysis: Option<serde_json::Value>,
        pub quantum_protection_used: bool,
    }
    
    #[derive(Debug, Serialize)]
    pub struct UserBalance {
        pub chain: String,
        pub token: String,
        pub balance: String,
        pub usd_value: f64,
    }

    // Risk Analysis Types
    #[derive(Debug, Serialize, Deserialize)]
    pub struct TransactionRiskRequest {
        pub from_address: String,
        pub to_address: String,
        pub amount: f64,
        pub token: String,
        pub chain: String,
        pub user_id: Option<String>,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct RiskScore {
        pub value: f64,
        pub level: String,
        pub confidence: f64,
        pub timestamp: String,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct RiskAnalysisResponse {
        pub risk_score: f64,
        pub risk_level: String,
        pub reasons: Vec<String>,
        pub approved: bool,
        pub ml_confidence: Option<f64>,
        pub is_anomaly: Option<bool>,
        pub recommended_action: String,
        pub analysis_timestamp: String,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct UserRiskProfileResponse {
        pub user_id: String,
        pub overall_risk_level: String,
        pub transaction_count: i32,
        pub avg_risk_score: f64,
        pub high_risk_transactions: i32,
        pub last_analysis_date: String,
    }
    
    #[derive(Debug, Deserialize)]
    pub struct BridgeInitiateRequest {
        pub from_chain: String,
        pub to_chain: String,
        pub from_token: String,
        pub to_token: String,
        pub amount: String,
        pub recipient_address: String,
    }
    
    /// Get user profile - requires JWT authentication
    pub async fn get_user_profile(
        headers: HeaderMap,
    ) -> Result<Json<ServiceResponse<UserProfile>>, crate::errors::GatewayServiceError> {
        tracing::info!("👤 Getting user profile");
        
        // Extract JWT token from Authorization header
        let auth_header = headers.get("authorization")
            .and_then(|h| h.to_str().ok())
            .and_then(|h| h.strip_prefix("Bearer "))
            .ok_or_else(|| crate::errors::GatewayServiceError::AuthenticationRequired)?;
        
        // Validate JWT token
        let jwt_secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "default_secret".to_string());
        let validation_result = crate::jwt_validation::validate_jwt_token(auth_header, &jwt_secret);
        
        if !validation_result.valid {
            tracing::warn!("❌ Invalid JWT token: {:?}", validation_result.errors);
            return Err(crate::errors::GatewayServiceError::AuthenticationRequired);
        }
        
        let user_id = validation_result.user_id.unwrap_or_else(|| "unknown".to_string());
        let wallet_address = validation_result.wallet_address.unwrap_or_else(|| "unknown".to_string());
        
        let profile = UserProfile {
            user_id,
            wallet_address,
            chain_type: "ethereum".to_string(),
            created_at: chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string(),
            last_login: chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string(),
            tier: "standard".to_string(),
        };
        
        Ok(Json(ServiceResponse::success(profile)))
    }
    
    /// Get user transactions - requires JWT authentication
    pub async fn get_user_transactions(
        headers: HeaderMap,
    ) -> Result<Json<ServiceResponse<Vec<UserTransaction>>>, crate::errors::GatewayServiceError> {
        tracing::info!("📜 Getting user transactions");
        
        // Extract JWT token from Authorization header
        let auth_header = headers.get("authorization")
            .and_then(|h| h.to_str().ok())
            .and_then(|h| h.strip_prefix("Bearer "))
            .ok_or_else(|| crate::errors::GatewayServiceError::AuthenticationRequired)?;
        
        // Validate JWT token
        let jwt_secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "default_secret".to_string());
        let validation_result = crate::jwt_validation::validate_jwt_token(auth_header, &jwt_secret);
        
        if !validation_result.valid {
            tracing::warn!("❌ Invalid JWT token: {:?}", validation_result.errors);
            return Err(crate::errors::GatewayServiceError::AuthenticationRequired);
        }
        
        // Mock transactions data
        let transactions = vec![
            UserTransaction {
                id: "tx_1".to_string(),
                from_chain: "ethereum".to_string(),
                to_chain: "near".to_string(),
                from_token: "ETH".to_string(),
                to_token: "NEAR".to_string(),
                amount: "0.1".to_string(),
                status: "completed".to_string(),
                created_at: chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string(),
                completed_at: Some(chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string()),
            }
        ];
        
        Ok(Json(ServiceResponse::success(transactions)))
    }
    
    /// Get user balance - requires JWT authentication
    pub async fn get_user_balance(
        headers: HeaderMap,
    ) -> Result<Json<ServiceResponse<Vec<UserBalance>>>, crate::errors::GatewayServiceError> {
        tracing::info!("💰 Getting user balance");
        
        // Extract JWT token from Authorization header
        let auth_header = headers.get("authorization")
            .and_then(|h| h.to_str().ok())
            .and_then(|h| h.strip_prefix("Bearer "))
            .ok_or_else(|| crate::errors::GatewayServiceError::AuthenticationRequired)?;
        
        // Validate JWT token
        let jwt_secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "default_secret".to_string());
        let validation_result = crate::jwt_validation::validate_jwt_token(auth_header, &jwt_secret);
        
        if !validation_result.valid {
            tracing::warn!("❌ Invalid JWT token: {:?}", validation_result.errors);
            return Err(crate::errors::GatewayServiceError::AuthenticationRequired);
        }
        
        // Mock balance data
        let balances = vec![
            UserBalance {
                chain: "ethereum".to_string(),
                token: "ETH".to_string(),
                balance: "1.5".to_string(),
                usd_value: 3750.0,
            },
            UserBalance {
                chain: "near".to_string(),
                token: "NEAR".to_string(),
                balance: "50.0".to_string(),
                usd_value: 175.0,
            },
        ];
        
        Ok(Json(ServiceResponse::success(balances)))
    }
    
    /// Initiate bridge transaction - requires JWT authentication
    pub async fn initiate_bridge(
        headers: HeaderMap,
        Json(payload): Json<BridgeInitiateRequest>,
    ) -> Result<Json<ServiceResponse<serde_json::Value>>, crate::errors::GatewayServiceError> {
        tracing::info!("🌉 Initiating bridge transaction");
        
        // Extract JWT token from Authorization header
        let auth_header = headers.get("authorization")
            .and_then(|h| h.to_str().ok())
            .and_then(|h| h.strip_prefix("Bearer "))
            .ok_or_else(|| crate::errors::GatewayServiceError::AuthenticationRequired)?;
        
        // Validate JWT token
        let jwt_secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "default_secret".to_string());
        let validation_result = crate::jwt_validation::validate_jwt_token(auth_header, &jwt_secret);
        
        if !validation_result.valid {
            tracing::warn!("❌ Invalid JWT token: {:?}", validation_result.errors);
            return Err(crate::errors::GatewayServiceError::AuthenticationRequired);
        }
        
        // Validate bridge request
        if payload.amount.parse::<f64>().unwrap_or(0.0) <= 0.0 {
            return Err(crate::errors::GatewayServiceError::ValidationError(
                "Amount must be greater than 0".to_string()
            ));
        }
        
        // Generate quantum signature
        let quantum_signature = {
            use base64::{Engine as _, engine::general_purpose};
            general_purpose::STANDARD.encode({
                let mut bytes = [0u8; 128];
                for i in 0..128 {
                    bytes[i] = rand::random::<u8>();
                }
                bytes
            })
        };
        
        let response = serde_json::json!({
            "transaction_id": format!("bridge_{}", uuid::Uuid::new_v4()),
            "status": "initiated",
            "from_chain": payload.from_chain,
            "to_chain": payload.to_chain,
            "from_token": payload.from_token,
            "to_token": payload.to_token,
            "amount": payload.amount,
            "recipient_address": payload.recipient_address,
            "estimated_completion": (chrono::Utc::now() + chrono::Duration::minutes(15)).format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string(),
            "quantum_signature": quantum_signature
        });
        
        Ok(Json(ServiceResponse::success(response)))
    }
    
    /// Get bridge status endpoint
    pub async fn get_bridge_status(
    ) -> Result<Json<ServiceResponse<serde_json::Value>>, crate::errors::GatewayServiceError> {
        tracing::info!("🌉 Getting bridge status");
        
        let status = serde_json::json!({
            "status": "operational",
            "supported_chains": ["ethereum", "near"],
            "supported_tokens": ["ETH", "NEAR", "USDC", "USDT"],
            "last_update": chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string()
        });
        
        Ok(Json(ServiceResponse::success(status)))
    }

    /// Get bridge transaction status by ID
    pub async fn get_bridge_transaction_status(
        Path(transaction_id): Path<String>,
    ) -> Result<Json<ServiceResponse<BridgeTransaction>>, crate::errors::GatewayServiceError> {
        tracing::info!("🔍 Getting bridge transaction status for ID: {}", transaction_id);
        
        // Mock transaction data with realistic progression
        let (status, from_hash, to_hash) = match transaction_id.as_str() {
            id if id.starts_with("tx_") => {
                // Simple ID format - use character count logic
                let hash_base = &id[3..]; // Remove "tx_" prefix
                
                if hash_base.len() % 3 == 0 {
                    // Completed transaction
                    ("completed", 
                     Some(format!("0x{}64f92a8b3c1e{}", hash_base, "a".repeat(32))),
                     Some(format!("{}::completed_tx{}", hash_base, "b".repeat(32))))
                } else if hash_base.len() % 3 == 1 {
                    // Confirmed on source chain only
                    ("confirmed",
                     Some(format!("0x{}64f92a8b3c1e{}", hash_base, "c".repeat(32))),
                     None)
                } else {
                    // Still pending
                    ("pending", None, None)
                }
            },
            id if id.starts_with("swap_") => {
                // UUID format - use hash of UUID for deterministic status
                use std::collections::hash_map::DefaultHasher;
                use std::hash::{Hash, Hasher};
                
                let mut hasher = DefaultHasher::new();
                id.hash(&mut hasher);
                let hash_value = hasher.finish();
                
                match hash_value % 3 {
                    0 => {
                        // Completed transaction  
                        let short_id = &id[5..13]; // Take 8 chars after "swap_"
                        ("completed",
                         Some(format!("0x{}64f92a8b3c1e{}", short_id, "a".repeat(24))),
                         Some(format!("{}::completed_tx{}", short_id, "b".repeat(24))))
                    },
                    1 => {
                        // Confirmed on source chain only
                        let short_id = &id[5..13]; // Take 8 chars after "swap_"
                        ("confirmed",
                         Some(format!("0x{}64f92a8b3c1e{}", short_id, "c".repeat(24))),
                         None)
                    },
                    _ => {
                        // Still pending
                        ("pending", None, None)
                    }
                }
            },
            _ => ("pending", None, None)
        };

        let transaction = BridgeTransaction {
            id: transaction_id.clone(),
            quote_id: format!("quote_{}", transaction_id),
            status: status.to_string(),
            from_chain: "ethereum".to_string(),
            to_chain: "near".to_string(),
            from_token: "ETH".to_string(),
            to_token: "NEAR".to_string(),
            from_amount: "1000000000000000000".to_string(), // 1 ETH in wei
            to_amount: "5000000000000000000000000".to_string(), // 5 NEAR
            from_wallet_address: "0x742d35Cc6634C0532925a3b8D847B3aA27f4d50f".to_string(),
            to_wallet_address: "test.near".to_string(),
            from_transaction_hash: from_hash,
            to_transaction_hash: to_hash,
            created_at: chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string(),
            updated_at: chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string(),
            estimated_completion_at: Some((chrono::Utc::now() + chrono::Duration::minutes(10)).format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string()),
            actual_completion_at: if status == "completed" {
                Some(chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string())
            } else {
                None
            },
            risk_analysis: Some(serde_json::json!({
                "score": 0.15,
                "level": "low",
                "flags": []
            })),
            quantum_protection_used: true,
        };
        
        Ok(Json(ServiceResponse::success(transaction)))
    }
    
    /// Get CSRF token endpoint
    pub async fn get_csrf_token(
    ) -> Result<Json<ServiceResponse<serde_json::Value>>, crate::errors::GatewayServiceError> {
        tracing::info!("🛡️ Generating CSRF token");
        
        let token = crate::middleware::generate_csrf_token();
        crate::middleware::store_csrf_token(token.clone());
        
        let response = serde_json::json!({
            "csrf_token": token,
            "expires_in": 3600, // 1 hour in seconds
            "usage": "single_use",
            "header_name": "x-csrf-token"
        });
        
        Ok(Json(ServiceResponse::success(response)))
    }

    // ===== Bridge Swap =====

    #[derive(Debug, serde::Deserialize)]
    pub struct BridgeSwapRequest {
        pub quote_id: String,
        pub from_wallet_address: String,
        pub to_wallet_address: Option<String>,
        pub max_slippage: f64,
    }

    #[derive(Debug, serde::Serialize)]
    pub struct BridgeSwapResponse {
        pub swap_id: String,
        pub status: String,
        pub tx_preview: serde_json::Value,
        pub expires_at: String,
    }

    /// Real swap handler connecting to 1inch service.
    /// CSRF проверяется middleware, CORS уже настроен.
    pub async fn handle_bridge_swap(
        State((circuit_breaker, _)): State<(
            Arc<CircuitBreaker>,
            crate::websocket::ConnectionManager,
        )>,
        _headers: HeaderMap,
        Json(payload): Json<BridgeSwapRequest>,
    ) -> Result<Json<ServiceResponse<BridgeSwapResponse>>, crate::errors::GatewayServiceError> {
        tracing::info!("🌉 Real bridge swap request via 1inch service");

        // Validate payload
        if payload.quote_id.trim().is_empty() {
            return Err(crate::errors::GatewayServiceError::ValidationError(
                "quote_id is required".to_string(),
            ));
        }
        if payload.from_wallet_address.trim().is_empty() {
            return Err(crate::errors::GatewayServiceError::ValidationError(
                "from_wallet_address is required".to_string(),
            ));
        }
        if !(0.0..=5.0).contains(&payload.max_slippage) {
            return Err(crate::errors::GatewayServiceError::ValidationError(
                "max_slippage must be between 0.0 and 5.0".to_string(),
            ));
        }

        // Check circuit breaker for 1inch service
        if !circuit_breaker.is_request_allowed("1inch-service") {
            tracing::warn!("Circuit breaker OPEN for 1inch-service");
            return Ok(Json(ServiceResponse::error(
                "1inch service temporarily unavailable".to_string(),
            )));
        }

        // Proxy to real 1inch service
        let client = reqwest::Client::new();
        let oneinch_service_url = std::env::var("ONEINCH_SERVICE_URL")
            .unwrap_or_else(|_| "http://1inch-service:4001".to_string());

        // Convert gateway request to 1inch service format
        let recipient_address = payload.to_wallet_address.clone().unwrap_or_else(|| payload.from_wallet_address.clone());
        let oneinch_request = serde_json::json!({
            "quote_id": payload.quote_id,
            "user_address": payload.from_wallet_address,
            "recipient_address": recipient_address,
            "slippage": payload.max_slippage,
        });

        match client
            .post(&format!("{}/api/swaps/execute", oneinch_service_url))
            .json(&oneinch_request)
            .timeout(std::time::Duration::from_secs(30))
            .send()
            .await
        {
            Ok(response) => {
                if response.status().is_success() {
                    match response.json::<ServiceResponse<serde_json::Value>>().await {
                        Ok(oneinch_response) => {
                            circuit_breaker.record_success("1inch-service");

                            if let Some(data) = oneinch_response.data {
                                // Transform 1inch response to gateway format
                                let swap_id = format!("swap_{}", uuid::Uuid::new_v4());
                                
                                let tx_preview = serde_json::json!({
                                    "quote_id": payload.quote_id,
                                    "from": payload.from_wallet_address,
                                    "to": recipient_address,
                                    "max_slippage": payload.max_slippage,
                                    "network_fee_estimate": data.get("actual_gas_fee").unwrap_or(&serde_json::Value::String("0.0042".to_string())),
                                    "bridge_fee_estimate": "0.0010",
                                    "protocol": "1inch-fusion",
                                    "transaction_hash": data.get("transaction_hash"),
                                    "order_hash": data.get("order_hash"),
                                });

                                let resp = BridgeSwapResponse {
                                    swap_id,
                                    status: data.get("status").and_then(|s| s.as_str()).unwrap_or("initiated").to_string(),
                                    tx_preview,
                                    expires_at: (chrono::Utc::now() + chrono::Duration::minutes(10)).to_rfc3339(),
                                };

                                tracing::info!("✅ Real 1inch swap executed: {}", resp.swap_id);
                                Ok(Json(ServiceResponse::success(resp)))
                            } else {
                                circuit_breaker.record_failure("1inch-service");
                                Ok(Json(ServiceResponse::error(
                                    "No data received from 1inch service".to_string(),
                                )))
                            }
                        }
                        Err(e) => {
                            circuit_breaker.record_failure("1inch-service");
                            tracing::error!("Failed to parse 1inch service response: {}", e);
                            Ok(Json(ServiceResponse::error(format!(
                                "Invalid response from 1inch service: {}",
                                e
                            ))))
                        }
                    }
                } else {
                    circuit_breaker.record_failure("1inch-service");
                    tracing::error!("1inch service returned error: {}", response.status());
                    Ok(Json(ServiceResponse::error(format!(
                        "1inch service error: {}",
                        response.status()
                    ))))
                }
            }
            Err(e) => {
                circuit_breaker.record_failure("1inch-service");
                tracing::error!("Failed to connect to 1inch service: {}", e);
                Ok(Json(ServiceResponse::error(format!(
                    "Failed to connect to 1inch service: {}",
                    e
                ))))
            }
        }
    }
}

// Re-export for easy access
pub use errors::*;
pub use types::*;
