// src/models/risk.rs - Risk Analysis Models for AI Engine Integration (Phase 5.2)
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use utoipa::ToSchema;

/// Request to AI Engine for risk analysis
#[derive(Debug, Clone, Serialize)]
pub struct RiskAnalysisRequest {
    pub transaction_id: Uuid,
    pub user_id: Uuid,
    pub source_chain: String,
    pub destination_chain: String,
    pub amount: f64,
    pub source_address: Option<String>,
    pub destination_address: Option<String>,
    pub transaction_metadata: serde_json::Value,
}

/// Response from AI Engine with risk analysis
#[derive(Debug, Clone, Deserialize)]
pub struct RiskAnalysisResponse {
    pub transaction_id: Uuid,
    pub risk_score: f64,
    pub risk_level: RiskLevel,
    pub risk_factors: Vec<RiskFactor>,
    pub recommendation: RiskRecommendation,
    pub analysis_timestamp: DateTime<Utc>,
    pub model_version: String,
}

/// Risk level enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub enum RiskLevel {
    #[serde(rename = "low")]
    Low,
    #[serde(rename = "medium")] 
    Medium,
    #[serde(rename = "high")]
    High,
    #[serde(rename = "critical")]
    Critical,
}

impl RiskLevel {
    /// Convert risk score to risk level based on thresholds
    pub fn from_score(score: f64) -> Self {
        match score {
            s if s < 0.3 => RiskLevel::Low,
            s if s < 0.6 => RiskLevel::Medium,
            s if s < 0.8 => RiskLevel::High,
            _ => RiskLevel::Critical,
        }
    }

    /// Check if risk level requires manual review
    pub fn requires_manual_review(&self) -> bool {
        matches!(self, RiskLevel::High | RiskLevel::Critical)
    }

    /// Check if risk level should be auto-blocked
    pub fn should_auto_block(&self) -> bool {
        matches!(self, RiskLevel::Critical)
    }
}

/// Individual risk factor identified by AI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskFactor {
    pub factor_type: String,
    pub severity: f64,
    pub description: String,
    pub confidence: f64,
}

/// AI recommendation for transaction handling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskRecommendation {
    #[serde(rename = "allow")]
    Allow,
    #[serde(rename = "review")]
    RequireManualReview,
    #[serde(rename = "block")]
    Block,
    #[serde(rename = "escalate")]
    Escalate,
}

/// User risk profile request
#[derive(Debug, Clone, Serialize)]
pub struct UserRiskProfileRequest {
    pub user_id: Uuid,
    pub include_history_days: Option<i32>,
}

/// User risk profile response from AI Engine
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UserRiskProfileResponse {
    pub user_id: Uuid,
    pub overall_risk_score: f64,
    pub risk_level: RiskLevel,
    pub transaction_count: u32,
    pub total_volume: f64,
    pub first_transaction: Option<DateTime<Utc>>,
    pub last_transaction: Option<DateTime<Utc>>,
    pub risk_trend: RiskTrend,
    pub behavioral_flags: Vec<BehavioralFlag>,
    pub profile_updated_at: DateTime<Utc>,
}

/// Risk trend analysis
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub enum RiskTrend {
    #[serde(rename = "improving")]
    Improving,
    #[serde(rename = "stable")]
    Stable,
    #[serde(rename = "deteriorating")]
    Deteriorating,
    #[serde(rename = "unknown")]
    Unknown,
}

/// Behavioral flags from AI analysis
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct BehavioralFlag {
    pub flag_type: String,
    pub severity: f64,
    pub first_detected: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
    pub description: String,
}

/// Risk thresholds configuration
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct RiskThresholds {
    pub low_threshold: f64,
    pub medium_threshold: f64,
    pub high_threshold: f64,
    pub auto_block_threshold: f64,
    pub manual_review_threshold: f64,
}

impl Default for RiskThresholds {
    fn default() -> Self {
        Self {
            low_threshold: 0.3,
            medium_threshold: 0.6,
            high_threshold: 0.8,
            auto_block_threshold: 0.9,
            manual_review_threshold: 0.7,
        }
    }
}

/// Error types for risk analysis
#[derive(Debug, thiserror::Error)]
pub enum RiskAnalysisError {
    #[error("AI Engine unavailable: {0}")]
    EngineUnavailable(String),

    #[error("Risk analysis timeout")]
    Timeout,

    #[error("Invalid risk response: {0}")]
    InvalidResponse(String),

    #[error("Risk analysis failed: {0}")]
    AnalysisFailed(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Authentication failed")]
    AuthenticationFailed,

    #[error("Rate limit exceeded")]
    RateLimitExceeded,
}

impl From<reqwest::Error> for RiskAnalysisError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            RiskAnalysisError::Timeout
        } else if err.is_connect() {
            RiskAnalysisError::EngineUnavailable(err.to_string())
        } else {
            RiskAnalysisError::NetworkError(err.to_string())
        }
    }
}