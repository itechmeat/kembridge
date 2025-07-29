use crate::{ServiceError, ServiceResult, ServiceResponse};
use reqwest::{Client, Response};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{error, info, warn};

#[derive(Clone)]
pub struct ServiceClient {
    client: Client,
    base_url: String,
    timeout: Duration,
    max_retries: u32,
}

impl ServiceClient {
    pub fn new(base_url: String, timeout_ms: u64, max_retries: u32) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_millis(timeout_ms))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            base_url,
            timeout: Duration::from_millis(timeout_ms),
            max_retries,
        }
    }

    pub async fn get<T>(&self, endpoint: &str) -> ServiceResult<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        let url = format!("{}{}", self.base_url, endpoint);
        self.retry_request(|| self.client.get(&url)).await
    }

    pub async fn post<T, R>(&self, endpoint: &str, body: &T) -> ServiceResult<R>
    where
        T: Serialize,
        R: for<'de> Deserialize<'de>,
    {
        let url = format!("{}{}", self.base_url, endpoint);
        self.retry_request(|| self.client.post(&url).json(body)).await
    }

    pub async fn put<T, R>(&self, endpoint: &str, body: &T) -> ServiceResult<R>
    where
        T: Serialize,
        R: for<'de> Deserialize<'de>,
    {
        let url = format!("{}{}", self.base_url, endpoint);
        self.retry_request(|| self.client.put(&url).json(body)).await
    }

    pub async fn delete<T>(&self, endpoint: &str) -> ServiceResult<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        let url = format!("{}{}", self.base_url, endpoint);
        self.retry_request(|| self.client.delete(&url)).await
    }

    async fn retry_request<F, R>(&self, request_fn: F) -> ServiceResult<R>
    where
        F: Fn() -> reqwest::RequestBuilder,
        R: for<'de> Deserialize<'de>,
    {
        let mut last_error = None;

        for attempt in 0..=self.max_retries {
            match self.execute_request(request_fn()).await {
                Ok(response) => return Ok(response),
                Err(e) => {
                    last_error = Some(e);
                    if attempt < self.max_retries {
                        let delay = Duration::from_millis(100 * (2_u64.pow(attempt)));
                        warn!("Request failed, retrying in {:?}. Attempt {}/{}", delay, attempt + 1, self.max_retries + 1);
                        tokio::time::sleep(delay).await;
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| ServiceError::Internal {
            message: "Request failed after all retries".to_string(),
        }))
    }

    async fn execute_request<R>(&self, request_builder: reqwest::RequestBuilder) -> ServiceResult<R>
    where
        R: for<'de> Deserialize<'de>,
    {
        let response = request_builder
            .send()
            .await
            .map_err(|e| {
                error!("Network error: {}", e);
                ServiceError::Network {
                    message: e.to_string(),
                }
            })?;

        self.handle_response(response).await
    }

    async fn handle_response<R>(&self, response: Response) -> ServiceResult<R>
    where
        R: for<'de> Deserialize<'de>,
    {
        let status = response.status();
        let url = response.url().clone();

        if status.is_success() {
            let service_response: ServiceResponse<R> = response
                .json()
                .await
                .map_err(|e| {
                    error!("Failed to deserialize response from {}: {}", url, e);
                    ServiceError::Serialization {
                        message: e.to_string(),
                    }
                })?;

            if service_response.success {
                service_response.data.ok_or_else(|| ServiceError::Internal {
                    message: "Service returned success but no data".to_string(),
                })
            } else {
                Err(ServiceError::ExternalService {
                    service: self.base_url.clone(),
                    message: service_response.error.unwrap_or_else(|| "Unknown error".to_string()),
                })
            }
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Failed to read error response".to_string());

            error!("HTTP error {} from {}: {}", status, url, error_text);

            match status.as_u16() {
                400 => Err(ServiceError::InvalidRequest { message: error_text }),
                401 => Err(ServiceError::AuthenticationFailed { reason: error_text }),
                403 => Err(ServiceError::AuthorizationFailed { reason: error_text }),
                404 => Err(ServiceError::NotFound { resource: url.to_string() }),
                429 => Err(ServiceError::RateLimitExceeded),
                503 => Err(ServiceError::ServiceUnavailable { service: self.base_url.clone() }),
                504 => Err(ServiceError::Timeout { operation: url.to_string() }),
                _ => Err(ServiceError::ExternalService {
                    service: self.base_url.clone(),
                    message: error_text,
                }),
            }
        }
    }

    pub async fn health_check(&self) -> ServiceResult<()> {
        match self.get::<serde_json::Value>("/health").await {
            Ok(_) => {
                info!("Health check passed for {}", self.base_url);
                Ok(())
            }
            Err(e) => {
                warn!("Health check failed for {}: {}", self.base_url, e);
                Err(e)
            }
        }
    }
}