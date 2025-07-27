// src/services/risk_client.rs - HTTP Client for AI Engine Integration (Phase 5.2.1)
use std::time::Duration;
use reqwest::{Client, Url};
use tracing::{info, warn, error, debug, instrument};
use uuid::Uuid;

use crate::config::AppConfig;
use crate::constants::*;
use crate::models::risk::{
    RiskAnalysisRequest, RiskAnalysisResponse, RiskAnalysisError,
    UserRiskProfileRequest, UserRiskProfileResponse,
};

/// HTTP client for AI Engine communication
#[derive(Clone)]
pub struct RiskClient {
    client: Client,
    base_url: Url,
    api_key: Option<String>,
    timeout: Duration,
    max_retries: u32,
}

impl RiskClient {
    /// Create new RiskClient with configuration
    pub fn new(config: &AppConfig) -> Result<Self, RiskAnalysisError> {
        let base_url = config.ai_engine_url
            .parse::<Url>()
            .map_err(|e| RiskAnalysisError::InvalidResponse(format!("Invalid AI Engine URL: {}", e)))?;

        let client = Client::builder()
            .timeout(Duration::from_millis(config.ai_engine_timeout_ms))
            .connection_verbose(false)
            .pool_idle_timeout(Duration::from_secs(RISK_CLIENT_POOL_IDLE_TIMEOUT_SEC))
            .pool_max_idle_per_host(RISK_CLIENT_POOL_MAX_IDLE_PER_HOST)
            .build()
            .map_err(|e| RiskAnalysisError::NetworkError(format!("Failed to create HTTP client: {}", e)))?;

        info!(
            base_url = %base_url,
            timeout_ms = config.ai_engine_timeout_ms,
            max_retries = config.ai_engine_max_retries,
            "RiskClient initialized"
        );

        Ok(Self {
            client,
            base_url,
            api_key: config.ai_engine_api_key.clone(),
            timeout: Duration::from_millis(config.ai_engine_timeout_ms),
            max_retries: config.ai_engine_max_retries,
        })
    }

    /// Analyze risk for a transaction
    #[instrument(skip(self, request), fields(transaction_id = %request.transaction_id))]
    pub async fn analyze_risk(&self, request: RiskAnalysisRequest) -> Result<RiskAnalysisResponse, RiskAnalysisError> {
        let url = self.base_url.join("/api/risk/analyze")
            .map_err(|e| RiskAnalysisError::InvalidResponse(format!("Invalid URL: {}", e)))?;

        debug!(
            transaction_id = %request.transaction_id,
            user_id = %request.user_id,
            amount = request.amount,
            "Sending risk analysis request to AI Engine"
        );

        let mut last_error = None;

        for attempt in 1..=self.max_retries {
            match self.send_risk_request(&url, &request, attempt).await {
                Ok(response) => {
                    info!(
                        transaction_id = %request.transaction_id,
                        risk_score = response.risk_score,
                        risk_level = ?response.risk_level,
                        attempt = attempt,
                        "Risk analysis completed successfully"
                    );
                    return Ok(response);
                }
                Err(e) => {
                    warn!(
                        transaction_id = %request.transaction_id,
                        attempt = attempt,
                        max_retries = self.max_retries,
                        error = %e,
                        "Risk analysis attempt failed"
                    );
                    last_error = Some(e);

                    if attempt < self.max_retries {
                        // Exponential backoff
                        let delay = Duration::from_millis(RISK_CLIENT_RETRY_BASE_DELAY_MS * (2_u64.pow(attempt - 1)));
                        tokio::time::sleep(delay).await;
                    }
                }
            }
        }

        let error = last_error.unwrap_or_else(|| RiskAnalysisError::AnalysisFailed("Unknown error".to_string()));
        error!(
            transaction_id = %request.transaction_id,
            error = %error,
            "Risk analysis failed after all retries"
        );
        Err(error)
    }

    /// Send individual risk request with retry logic
    async fn send_risk_request(
        &self,
        url: &Url,
        request: &RiskAnalysisRequest,
        attempt: u32,
    ) -> Result<RiskAnalysisResponse, RiskAnalysisError> {
        let mut req_builder = self.client
            .post(url.clone())
            .json(request)
            .timeout(self.timeout);

        // Add API key if configured
        if let Some(ref api_key) = self.api_key {
            req_builder = req_builder.header("X-API-Key", api_key);
        }

        req_builder = req_builder.header("X-Request-ID", Uuid::new_v4().to_string());
        req_builder = req_builder.header("X-Retry-Attempt", attempt.to_string());

        let response = req_builder.send().await?;

        let status = response.status();
        
        if status.is_success() {
            let risk_response: RiskAnalysisResponse = response.json().await
                .map_err(|e| RiskAnalysisError::InvalidResponse(format!("Failed to parse response: {}", e)))?;
            
            // Validate response data
            if risk_response.risk_score < 0.0 || risk_response.risk_score > 1.0 {
                return Err(RiskAnalysisError::InvalidResponse(
                    format!("Invalid risk score: {}", risk_response.risk_score)
                ));
            }

            Ok(risk_response)
        } else if status == 429 {
            Err(RiskAnalysisError::RateLimitExceeded)
        } else if status == 401 || status == 403 {
            Err(RiskAnalysisError::AuthenticationFailed)
        } else {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            Err(RiskAnalysisError::AnalysisFailed(format!("HTTP {}: {}", status, error_text)))
        }
    }

    /// Get user risk profile
    #[instrument(skip(self), fields(user_id = %user_id))]
    pub async fn get_user_risk_profile(&self, user_id: Uuid, include_history_days: Option<i32>) -> Result<UserRiskProfileResponse, RiskAnalysisError> {
        let url = self.base_url.join(&format!("/api/risk/profile/{}", user_id))
            .map_err(|e| RiskAnalysisError::InvalidResponse(format!("Invalid URL: {}", e)))?;

        let request = UserRiskProfileRequest {
            user_id,
            include_history_days,
        };

        debug!(
            user_id = %user_id,
            include_history_days = ?include_history_days,
            "Fetching user risk profile from AI Engine"
        );

        let mut req_builder = self.client
            .get(url)
            .query(&[("include_history_days", include_history_days)])
            .timeout(self.timeout);

        if let Some(ref api_key) = self.api_key {
            req_builder = req_builder.header("X-API-Key", api_key);
        }

        let response = req_builder.send().await?;

        let status = response.status();
        if status.is_success() {
            let profile: UserRiskProfileResponse = response.json().await
                .map_err(|e| RiskAnalysisError::InvalidResponse(format!("Failed to parse profile: {}", e)))?;
            
            info!(
                user_id = %user_id,
                risk_score = profile.overall_risk_score,
                risk_level = ?profile.risk_level,
                transaction_count = profile.transaction_count,
                "User risk profile retrieved successfully"
            );

            Ok(profile)
        } else {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            Err(RiskAnalysisError::AnalysisFailed(format!("HTTP {}: {}", status, error_text)))
        }
    }

    /// Check if AI Engine is healthy
    #[instrument(skip(self))]
    pub async fn health_check(&self) -> Result<bool, RiskAnalysisError> {
        let url = self.base_url.join("/health")
            .map_err(|e| RiskAnalysisError::InvalidResponse(format!("Invalid URL: {}", e)))?;

        debug!("Performing AI Engine health check");

        let response = self.client
            .get(url)
            .timeout(Duration::from_secs(RISK_CLIENT_HEALTH_CHECK_TIMEOUT_SEC))
            .send()
            .await?;

        let is_healthy = response.status().is_success();
        
        if is_healthy {
            debug!("AI Engine health check passed");
        } else {
            warn!(status = %response.status(), "AI Engine health check failed");
        }

        Ok(is_healthy)
    }

    /// Update user risk profile after transaction
    #[instrument(skip(self))]
    pub async fn update_user_profile(&self, user_id: Uuid, transaction_data: serde_json::Value) -> Result<(), RiskAnalysisError> {
        let url = self.base_url.join(&format!("/api/risk/profile/{}/update", user_id))
            .map_err(|e| RiskAnalysisError::InvalidResponse(format!("Invalid URL: {}", e)))?;

        debug!(
            user_id = %user_id,
            "Updating user risk profile after transaction"
        );

        let mut req_builder = self.client
            .post(url)
            .json(&transaction_data)
            .timeout(self.timeout);

        if let Some(ref api_key) = self.api_key {
            req_builder = req_builder.header("X-API-Key", api_key);
        }

        let response = req_builder.send().await?;

        let status = response.status();
        if status.is_success() {
            info!(user_id = %user_id, "User risk profile updated successfully");
            Ok(())
        } else {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            Err(RiskAnalysisError::AnalysisFailed(format!("Profile update failed: HTTP {}: {}", status, error_text)))
        }
    }

    /// Get base URL for debugging
    pub fn base_url(&self) -> &Url {
        &self.base_url
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AppConfig;

    #[test]
    fn test_risk_client_creation() {
        let mut config = AppConfig::default();
        config.ai_engine_url = DEFAULT_AI_ENGINE_URL.to_string();
        config.ai_engine_timeout_ms = DEFAULT_AI_ENGINE_TIMEOUT_MS;
        config.ai_engine_max_retries = DEFAULT_AI_ENGINE_MAX_RETRIES;

        let client = RiskClient::new(&config);
        assert!(client.is_ok());
        
        let client = client.unwrap();
        assert_eq!(client.base_url.as_str(), &format!("{}/", DEFAULT_AI_ENGINE_URL));
        assert_eq!(client.timeout, Duration::from_millis(DEFAULT_AI_ENGINE_TIMEOUT_MS));
        assert_eq!(client.max_retries, DEFAULT_AI_ENGINE_MAX_RETRIES);
    }

    #[test]
    fn test_invalid_url() {
        let mut config = AppConfig::default();
        config.ai_engine_url = "invalid-url".to_string();

        let client = RiskClient::new(&config);
        assert!(client.is_err());
    }
}