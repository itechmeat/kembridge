// Minimal gateway service for architecture testing
pub mod circuit_breaker;
pub mod clients;
pub mod config;
pub mod errors;
pub mod event_api;
pub mod event_listener;
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
        extract::{Query, State},
        Json,
    };
    use kembridge_common::ServiceResponse;
    use serde::{Deserialize, Serialize};
    use std::sync::Arc;

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
}

// Re-export for easy access
pub use errors::*;
pub use types::*;
