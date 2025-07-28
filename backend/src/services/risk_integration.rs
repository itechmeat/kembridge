// src/services/risk_integration.rs - Risk Analysis Integration with Bridge Service (Phase 5.2.2)
use std::sync::{Arc, Mutex};
use uuid::Uuid;
use tracing::{info, warn, error, debug, instrument};

use crate::config::AppConfig;
use crate::models::risk::{
    RiskAnalysisRequest, RiskAnalysisResponse, RiskAnalysisError, 
    RiskLevel, RiskThresholds, UserRiskProfileResponse,
};
use crate::models::review::{CreateReviewRequest, ReviewPriority};
use crate::services::{RiskClient, ManualReviewService};
use kembridge_bridge::{BridgeError, SwapOperation, SwapStatus};
use kembridge_database::TransactionService;
use bigdecimal::BigDecimal;

/// Service for integrating risk analysis into bridge operations
#[derive(Clone)]
pub struct RiskIntegrationService {
    risk_client: RiskClient,
    manual_review_service: Option<Arc<ManualReviewService>>,
    transaction_service: Arc<TransactionService>,
    risk_thresholds: Arc<Mutex<RiskThresholds>>,
    enable_risk_analysis: bool,
    fallback_to_allow: bool,
    admin_bypass_enabled: bool,
}

impl RiskIntegrationService {
    /// Create new risk integration service
    pub fn new(config: &AppConfig, db_pool: sqlx::PgPool) -> Result<Self, RiskAnalysisError> {
        let risk_client = RiskClient::new(config)?;
        let transaction_service = Arc::new(TransactionService::new(db_pool));
        
        let risk_thresholds = config.risk_thresholds();
        
        info!(
            enable_risk_analysis = config.enable_ai_risk_analysis,
            admin_bypass_enabled = config.risk_admin_bypass_enabled,
            risk_thresholds = ?risk_thresholds,
            "RiskIntegrationService initialized"
        );

        Ok(Self {
            risk_client,
            manual_review_service: None, // Will be set later via set_manual_review_service
            transaction_service,
            risk_thresholds: Arc::new(Mutex::new(risk_thresholds)),
            enable_risk_analysis: config.enable_ai_risk_analysis,
            fallback_to_allow: true, // Fail-open by default for now
            admin_bypass_enabled: config.risk_admin_bypass_enabled,
        })
    }

    /// Create new risk integration service with manual review service (Phase 5.2.4)
    pub fn new_with_manual_review(
        config: &AppConfig, 
        db_pool: sqlx::PgPool,
        manual_review_service: Arc<ManualReviewService>
    ) -> Result<Self, RiskAnalysisError> {
        let mut service = Self::new(config, db_pool)?;
        service.manual_review_service = Some(manual_review_service);
        info!("RiskIntegrationService initialized with ManualReviewService integration");
        Ok(service)
    }

    /// Perform risk analysis for a bridge operation before execution
    #[instrument(skip(self, swap_operation), fields(swap_id = %swap_operation.swap_id))]
    pub async fn analyze_bridge_risk(
        &self,
        swap_operation: &SwapOperation,
    ) -> Result<RiskAnalysisResponse, BridgeError> {
        if !self.enable_risk_analysis {
            debug!("Risk analysis disabled, allowing operation");
            return Ok(self.create_bypass_response(swap_operation));
        }

        let request = self.create_risk_request(swap_operation)?;

        debug!(
            swap_id = %swap_operation.swap_id,
            user_id = %swap_operation.user_id,
            amount = swap_operation.amount,
            "Performing risk analysis for bridge operation"
        );

        match self.risk_client.analyze_risk(request).await {
            Ok(response) => {
                info!(
                    swap_id = %swap_operation.swap_id,
                    risk_score = response.risk_score,
                    risk_level = ?response.risk_level,
                    "Risk analysis completed"
                );
                
                // Update transaction risk score in database (Phase 5.2.5)
                if let Err(e) = self.update_transaction_risk_score(swap_operation, &response).await {
                    warn!(
                        swap_id = %swap_operation.swap_id,
                        error = %e,
                        "Failed to update transaction risk score in database"
                    );
                }
                
                Ok(response)
            }
            Err(e) => {
                error!(
                    swap_id = %swap_operation.swap_id,
                    error = %e,
                    "Risk analysis failed"
                );

                if self.fallback_to_allow {
                    warn!(
                        swap_id = %swap_operation.swap_id,
                        "Risk analysis failed, falling back to allow (fail-open mode)" // TODO (MOCK WARNING): Remove fallback mode
                    );
                    Ok(self.create_fallback_response(swap_operation)) // TODO (MOCK WARNING): Remove fallback response
                } else {
                    Err(BridgeError::ValidationError(format!("Risk analysis failed: {}", e)))
                }
            }
        }
    }

    /// Perform complete risk analysis and automatically handle manual review queue (Phase 5.2.4)
    #[instrument(skip(self, swap_operation), fields(swap_id = %swap_operation.swap_id))]
    pub async fn analyze_bridge_risk_with_review_queue(
        &self,
        swap_operation: &SwapOperation,
    ) -> Result<OperationDecision, BridgeError> {
        // First, perform standard risk analysis
        let risk_response = self.analyze_bridge_risk(swap_operation).await?;
        
        // Make the decision based on risk analysis
        let decision = self.should_allow_operation(&risk_response);
        
        // Update transaction risk score in database (Phase 5.2.5)
        if let Err(e) = self.update_transaction_risk_score(swap_operation, &risk_response).await {
            warn!(
                swap_id = %swap_operation.swap_id,
                error = %e,
                "Failed to update transaction risk score in database during review queue processing"
            );
        }
        
        // If manual review is required, automatically add to queue
        if decision.requires_review() {
            if let Some(ref manual_review_service) = self.manual_review_service {
                let review_request = self.create_manual_review_request(swap_operation, &risk_response)?;
                
                match manual_review_service.add_to_review_queue(review_request).await {
                    Ok(review_entry) => {
                        info!(
                            swap_id = %swap_operation.swap_id,
                            review_id = %review_entry.id,
                            risk_score = risk_response.risk_score,
                            priority = ?review_entry.priority,
                            "Transaction automatically added to manual review queue"
                        );
                    }
                    Err(e) => {
                        error!(
                            swap_id = %swap_operation.swap_id,
                            error = %e,
                            "Failed to add transaction to manual review queue"
                        );
                        // Don't fail the entire process if queue addition fails
                    }
                }
            } else {
                warn!(
                    swap_id = %swap_operation.swap_id,
                    "Manual review required but ManualReviewService not available"
                );
            }
        }
        
        // Log the decision for audit trail
        self.log_risk_decision(swap_operation, &risk_response, &decision, None);
        
        Ok(decision)
    }

    /// Check if operation should be allowed based on risk analysis
    #[instrument(skip(self, risk_response))]
    pub fn should_allow_operation(&self, risk_response: &RiskAnalysisResponse) -> OperationDecision {
        self.should_allow_operation_with_user(risk_response, None)
    }

    /// Check if operation should be allowed with optional admin user context
    #[instrument(skip(self, risk_response, is_admin_user))]
    pub fn should_allow_operation_with_user(
        &self, 
        risk_response: &RiskAnalysisResponse, 
        is_admin_user: Option<bool>
    ) -> OperationDecision {
        let risk_score = risk_response.risk_score;
        let _risk_level = &risk_response.risk_level;

        let thresholds = self.risk_thresholds.lock().unwrap();
        
        // Admin bypass logic
        if let Some(true) = is_admin_user {
            if self.admin_bypass_enabled && risk_score >= thresholds.auto_block_threshold {
                info!(
                    risk_score = risk_score,
                    auto_block_threshold = thresholds.auto_block_threshold,
                    "Admin bypass applied for high-risk transaction"
                );
                return OperationDecision::Allow {
                    risk_score,
                };
            }
        }
        
        // Standard threshold-based decisions
        if risk_score >= thresholds.auto_block_threshold {
            warn!(
                risk_score = risk_score,
                auto_block_threshold = thresholds.auto_block_threshold,
                "Transaction automatically blocked due to high risk score"
            );
            OperationDecision::Block {
                reason: format!("Risk score {} exceeds auto-block threshold {}", 
                    risk_score, thresholds.auto_block_threshold),
                risk_score,
            }
        } else if risk_score >= thresholds.manual_review_threshold {
            warn!(
                risk_score = risk_score,
                manual_review_threshold = thresholds.manual_review_threshold,
                "Transaction requires manual review due to elevated risk score"
            );
            OperationDecision::RequireManualReview {
                reason: format!("Risk score {} requires manual review", risk_score),
                risk_score,
            }
        } else {
            debug!(
                risk_score = risk_score,
                "Transaction approved with acceptable risk score"
            );
            OperationDecision::Allow {
                risk_score,
            }
        }
    }

    /// Get user risk profile for enhanced analysis
    #[instrument(skip(self))]
    pub async fn get_user_risk_profile(
        &self,
        user_id: Uuid,
        include_history_days: Option<i32>,
    ) -> Result<UserRiskProfileResponse, BridgeError> {
        if !self.enable_risk_analysis {
            return Err(BridgeError::ValidationError("Risk analysis disabled".to_string()));
        }

        self.risk_client
            .get_user_risk_profile(user_id, include_history_days)
            .await
            .map_err(|e| BridgeError::ValidationError(format!("Failed to get user risk profile: {}", e)))
    }

    /// Update user profile after transaction completion
    #[instrument(skip(self, transaction_data))]
    pub async fn update_user_profile_after_transaction(
        &self,
        user_id: Uuid,
        transaction_data: serde_json::Value,
    ) -> Result<(), BridgeError> {
        if !self.enable_risk_analysis {
            return Ok(());
        }

        debug!(user_id = %user_id, "Updating user risk profile after transaction");

        self.risk_client
            .update_user_profile(user_id, transaction_data)
            .await
            .map_err(|e| {
                warn!(
                    user_id = %user_id,
                    error = %e,
                    "Failed to update user risk profile"
                );
                // Don't fail the transaction for profile update errors
                BridgeError::ValidationError(format!("Profile update failed: {}", e))
            })?;

        Ok(())
    }

    /// Check if AI Engine is healthy
    pub async fn health_check(&self) -> bool {
        if !self.enable_risk_analysis {
            return true; // Consider healthy if disabled
        }

        self.risk_client.health_check().await.unwrap_or(false)
    }

    /// Create manual review request from swap operation and risk analysis (Phase 5.2.4)
    fn create_manual_review_request(
        &self, 
        swap_operation: &SwapOperation, 
        risk_response: &RiskAnalysisResponse
    ) -> Result<CreateReviewRequest, BridgeError> {
        let priority = ReviewPriority::from_risk_score(risk_response.risk_score);
        
        let reason = format!(
            "Risk analysis indicates elevated risk (score: {:.2}, level: {:?}). Requires manual review before proceeding.",
            risk_response.risk_score,
            risk_response.risk_level
        );

        let metadata = serde_json::json!({
            "risk_analysis": {
                "risk_score": risk_response.risk_score,
                "risk_level": risk_response.risk_level,
                "risk_factors": risk_response.risk_factors,
                "recommendation": risk_response.recommendation,
                "analysis_timestamp": risk_response.analysis_timestamp,
                "model_version": risk_response.model_version,
            },
            "transaction": {
                "swap_id": swap_operation.swap_id,
                "from_chain": swap_operation.from_chain,
                "to_chain": swap_operation.to_chain,
                "amount": swap_operation.amount,
                "recipient": swap_operation.recipient,
                "status": swap_operation.status.to_string(),
                "created_at": swap_operation.created_at,
                "expires_at": swap_operation.expires_at,
                "quantum_key_id": swap_operation.quantum_key_id,
                "eth_tx_hash": swap_operation.eth_tx_hash,
                "near_tx_hash": swap_operation.near_tx_hash,
            }
        });

        Ok(CreateReviewRequest {
            transaction_id: swap_operation.swap_id,
            user_id: swap_operation.user_id,
            risk_score: risk_response.risk_score,
            priority: Some(priority),
            reason,
            metadata: Some(metadata),
        })
    }

    /// Create risk analysis request from swap operation
    fn create_risk_request(&self, swap_operation: &SwapOperation) -> Result<RiskAnalysisRequest, BridgeError> {
        // Convert amount from wei to float for AI analysis
        let amount_f64 = (swap_operation.amount as f64) / 1_000_000_000_000_000_000.0;

        let transaction_metadata = serde_json::json!({
            "swap_id": swap_operation.swap_id,
            "from_chain": swap_operation.from_chain,
            "to_chain": swap_operation.to_chain,
            "recipient": swap_operation.recipient,
            "status": swap_operation.status.to_string(),
            "created_at": swap_operation.created_at,
            "expires_at": swap_operation.expires_at,
            "quantum_key_id": swap_operation.quantum_key_id,
            "eth_tx_hash": swap_operation.eth_tx_hash,
            "near_tx_hash": swap_operation.near_tx_hash,
        });

        Ok(RiskAnalysisRequest {
            transaction_id: swap_operation.swap_id,
            user_id: swap_operation.user_id,
            source_chain: swap_operation.from_chain.clone(),
            destination_chain: swap_operation.to_chain.clone(),
            amount: amount_f64,
            source_address: None, // TODO (check): Extract from transaction data
            destination_address: Some(swap_operation.recipient.clone()),
            transaction_metadata,
        })
    }

    /// Create bypass response when risk analysis is disabled
    fn create_bypass_response(&self, swap_operation: &SwapOperation) -> RiskAnalysisResponse {
        RiskAnalysisResponse {
            transaction_id: swap_operation.swap_id,
            risk_score: 0.1, // Low risk when bypassed
            risk_level: RiskLevel::Low,
            risk_factors: vec![],
            recommendation: crate::models::risk::RiskRecommendation::Allow,
            analysis_timestamp: chrono::Utc::now(),
            model_version: "bypass".to_string(),
        }
    }

    /// TODO (MOCK WARNING): Create fallback response when risk analysis fails - remove this fallback mechanism
    fn create_fallback_response(&self, swap_operation: &SwapOperation) -> RiskAnalysisResponse {
        RiskAnalysisResponse {
            transaction_id: swap_operation.swap_id,
            risk_score: 0.5, // Medium risk when fallback
            risk_level: RiskLevel::Medium,
            risk_factors: vec![],
            recommendation: crate::models::risk::RiskRecommendation::Allow,
            analysis_timestamp: chrono::Utc::now(),
            model_version: "fallback".to_string(),
        }
    }

    /// Get risk thresholds for configuration
    pub fn risk_thresholds(&self) -> RiskThresholds {
        self.risk_thresholds.lock().unwrap().clone()
    }

    /// Update risk thresholds (for admin configuration)
    pub fn update_risk_thresholds(&self, new_thresholds: RiskThresholds) {
        let mut thresholds = self.risk_thresholds.lock().unwrap();
        info!(old_thresholds = ?*thresholds, new_thresholds = ?new_thresholds, "Updating risk thresholds");
        *thresholds = new_thresholds;
    }

    /// TODO (check): Update transaction risk score in database
    async fn update_transaction_risk_score(
        &self,
        swap_operation: &SwapOperation,
        risk_response: &RiskAnalysisResponse,
    ) -> Result<(), BridgeError> {
        let risk_score = BigDecimal::from(risk_response.risk_score as i64);
        let risk_factors = serde_json::to_value(&risk_response.risk_factors)
            .map_err(|e| BridgeError::ValidationError(format!("Failed to serialize risk factors: {}", e)))?;
        
        self.transaction_service
            .update_risk_score(
                swap_operation.swap_id,
                risk_score,
                Some(risk_factors),
                Some(risk_response.model_version.clone()),
            )
            .await
            .map_err(|e| BridgeError::ValidationError(format!("Failed to update transaction risk score: {}", e)))?;
        
        debug!(
            swap_id = %swap_operation.swap_id,
            risk_score = risk_response.risk_score,
            model_version = %risk_response.model_version,
            "Updated transaction risk score in database"
        );
        
        Ok(())
    }

    /// Log detailed decision information for audit trail
    pub fn log_risk_decision(
        &self,
        swap_operation: &SwapOperation,
        risk_response: &RiskAnalysisResponse,
        decision: &OperationDecision,
        is_admin_user: Option<bool>,
    ) {
        let thresholds = self.risk_thresholds.lock().unwrap();
        
        match decision {
            OperationDecision::Allow { risk_score } => {
                info!(
                    swap_id = %swap_operation.swap_id,
                    user_id = %swap_operation.user_id,
                    risk_score = risk_score,
                    risk_level = ?risk_response.risk_level,
                    decision = "ALLOW",
                    is_admin_user = is_admin_user.unwrap_or(false),
                    admin_bypass_used = is_admin_user.unwrap_or(false) && self.admin_bypass_enabled && *risk_score >= thresholds.auto_block_threshold,
                    thresholds = ?*thresholds,
                    "Risk decision: Transaction approved"
                );
            }
            OperationDecision::RequireManualReview { risk_score, reason } => {
                warn!(
                    swap_id = %swap_operation.swap_id,
                    user_id = %swap_operation.user_id,
                    risk_score = risk_score,
                    risk_level = ?risk_response.risk_level,
                    decision = "MANUAL_REVIEW",
                    reason = reason,
                    is_admin_user = is_admin_user.unwrap_or(false),
                    thresholds = ?*thresholds,
                    "Risk decision: Transaction requires manual review"
                );
            }
            OperationDecision::Block { risk_score, reason } => {
                error!(
                    swap_id = %swap_operation.swap_id,
                    user_id = %swap_operation.user_id,
                    risk_score = risk_score,
                    risk_level = ?risk_response.risk_level,
                    decision = "BLOCK",
                    reason = reason,
                    is_admin_user = is_admin_user.unwrap_or(false),
                    thresholds = ?*thresholds,
                    "Risk decision: Transaction blocked due to high risk"
                );
            }
        }
    }
}

/// Decision result from risk analysis
#[derive(Debug, Clone)]
pub enum OperationDecision {
    /// Allow the operation to proceed
    Allow {
        risk_score: f64,
    },
    /// Require manual review before proceeding
    RequireManualReview {
        reason: String,
        risk_score: f64,
    },
    /// Block the operation
    Block {
        reason: String,
        risk_score: f64,
    },
}

impl OperationDecision {
    /// Check if operation is allowed to proceed
    pub fn is_allowed(&self) -> bool {
        matches!(self, OperationDecision::Allow { .. })
    }

    /// Check if operation requires manual review
    pub fn requires_review(&self) -> bool {
        matches!(self, OperationDecision::RequireManualReview { .. })
    }

    /// Check if operation is blocked
    pub fn is_blocked(&self) -> bool {
        matches!(self, OperationDecision::Block { .. })
    }

    /// Get risk score from decision
    pub fn risk_score(&self) -> f64 {
        match self {
            OperationDecision::Allow { risk_score } => *risk_score,
            OperationDecision::RequireManualReview { risk_score, .. } => *risk_score,
            OperationDecision::Block { risk_score, .. } => *risk_score,
        }
    }

    /// Get reason if operation is not allowed
    pub fn reason(&self) -> Option<&str> {
        match self {
            OperationDecision::Allow { .. } => None,
            OperationDecision::RequireManualReview { reason, .. } => Some(reason),
            OperationDecision::Block { reason, .. } => Some(reason),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AppConfig;
    use kembridge_bridge::SwapOperation;
    use chrono::Utc;

    #[test]
    fn test_operation_decision() {
        let allow = OperationDecision::Allow { risk_score: 0.2 };
        assert!(allow.is_allowed());
        assert!(!allow.requires_review());
        assert!(!allow.is_blocked());
        assert_eq!(allow.risk_score(), 0.2);
        assert!(allow.reason().is_none());

        let review = OperationDecision::RequireManualReview {
            reason: "High risk".to_string(),
            risk_score: 0.7,
        };
        assert!(!review.is_allowed());
        assert!(review.requires_review());
        assert!(!review.is_blocked());
        assert_eq!(review.risk_score(), 0.7);
        assert_eq!(review.reason().unwrap(), "High risk");

        let block = OperationDecision::Block {
            reason: "Critical risk".to_string(),
            risk_score: 0.9,
        };
        assert!(!block.is_allowed());
        assert!(!block.requires_review());
        assert!(block.is_blocked());
        assert_eq!(block.risk_score(), 0.9);
        assert_eq!(block.reason().unwrap(), "Critical risk");
    }

    #[test]
    fn test_risk_thresholds() {
        let thresholds = RiskThresholds::default();
        
        let low_risk = RiskLevel::from_score(0.2);
        assert_eq!(low_risk, RiskLevel::Low);
        assert!(!low_risk.requires_manual_review());
        assert!(!low_risk.should_auto_block());

        let high_risk = RiskLevel::from_score(0.8);
        assert_eq!(high_risk, RiskLevel::High);
        assert!(high_risk.requires_manual_review());
        assert!(!high_risk.should_auto_block());

        let critical_risk = RiskLevel::from_score(0.95);
        assert_eq!(critical_risk, RiskLevel::Critical);
        assert!(critical_risk.requires_manual_review());
        assert!(critical_risk.should_auto_block());
    }

    fn create_test_swap_operation() -> SwapOperation {
        SwapOperation {
            swap_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            from_chain: "ethereum".to_string(),
            to_chain: "near".to_string(),
            amount: 1_000_000_000_000_000_000, // 1 ETH in wei
            recipient: "user.testnet".to_string(),
            status: SwapStatus::Initialized,
            quantum_key_id: None,
            eth_tx_hash: None,
            near_tx_hash: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            expires_at: Utc::now() + chrono::Duration::minutes(30),
        }
    }

    #[ignore] // Requires database connection
    #[tokio::test]
    async fn test_create_risk_request() {
        let mut config = AppConfig::default();
        config.enable_ai_risk_analysis = false; // Disable for test
        
        // Mock database pool for testing
        let db_pool = sqlx::PgPool::connect("postgres://test").await.unwrap_or_else(|_| {
            // Return a mock pool if connection fails in test environment
            panic!("Mock database not available for testing")
        });
        let service = RiskIntegrationService::new(&config, db_pool).unwrap();
        let swap_op = create_test_swap_operation();
        
        let request = service.create_risk_request(&swap_op).unwrap();
        
        assert_eq!(request.transaction_id, swap_op.swap_id);
        assert_eq!(request.user_id, swap_op.user_id);
        assert_eq!(request.source_chain, "ethereum");
        assert_eq!(request.destination_chain, "near");
        assert_eq!(request.amount, 1.0); // 1 ETH
        assert_eq!(request.destination_address.unwrap(), "user.testnet");
    }
}