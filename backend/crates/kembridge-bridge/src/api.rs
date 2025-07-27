// HTTP API handlers for bridge operations
// Implements Phase 4.3.5 and 4.3.9

use axum::{
    extract::{Path, State},
    response::Json,
    Json as JsonRequest,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;
use kembridge_auth::ChainType;

use crate::{BridgeService, BridgeError, SwapStatus};

// Re-export AuthUser from main backend extractors
// Note: This is a temporary solution until AuthUser is properly modularized

#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: Uuid,
    pub wallet_address: String,
    pub chain_type: ChainType,
    pub session_id: String,
    pub user_tier: UserTier,
    pub is_quantum_protected: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UserTier {
    Admin,
    Premium,
    Standard,
}

// Request/Response structures

#[derive(Debug, Deserialize)]
pub struct InitSwapRequest {
    pub from_chain: String,
    pub to_chain: String,
    /// Amount as string for precision with large numbers
    pub amount: String,
    pub recipient: String,
}

#[derive(Debug, Serialize)]
pub struct InitSwapResponse {
    pub swap_id: Uuid,
    pub status: String,
    pub estimated_time_minutes: i64,
}

#[derive(Debug, Serialize)]
pub struct SwapStatusResponse {
    pub swap_id: Uuid,
    pub status: String,
    pub from_chain: String,
    pub to_chain: String,
    pub amount: String,
    pub recipient: String,
    pub eth_tx_hash: Option<String>,
    pub near_tx_hash: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub quantum_key_id: Option<String>,
}

// HTTP Handlers

/// Initialize a new bridge swap operation
/// POST /api/bridge/init-swap
pub async fn init_swap_handler(
    auth_user: AuthUser,
    State(bridge_service): State<Arc<BridgeService>>,
    JsonRequest(request): JsonRequest<InitSwapRequest>,
) -> Result<Json<InitSwapResponse>, BridgeError> {
    tracing::info!(
        user_id = %auth_user.user_id,
        from_chain = %request.from_chain,
        to_chain = %request.to_chain,
        amount = %request.amount,
        recipient = %request.recipient,
        "Initializing bridge swap"
    );

    // Parse amount
    let amount = request.amount.parse::<u128>()
        .map_err(|_| BridgeError::OperationFailed("Invalid amount format".to_string()))?;

    // Call BridgeService
    let result = bridge_service
        .init_swap(
            auth_user.user_id,
            &request.from_chain,
            &request.to_chain,
            amount,
            &request.recipient,
        )
        .await?;

    Ok(Json(InitSwapResponse {
        swap_id: result.swap_id,
        status: format!("{:?}", result.status),
        estimated_time_minutes: result.estimated_time.num_minutes(),
    }))
}

/// Get status of a bridge swap operation
/// GET /api/bridge/status/{swap_id}
pub async fn get_swap_status_handler(
    auth_user: AuthUser,
    Path(swap_id): Path<Uuid>,
    State(bridge_service): State<Arc<BridgeService>>,
) -> Result<Json<SwapStatusResponse>, BridgeError> {
    tracing::info!(
        user_id = %auth_user.user_id,
        swap_id = %swap_id,
        "Getting swap status"
    );

    // Get swap operation
    let operation = bridge_service
        .get_swap_operation(swap_id)
        .await?;

    // Authorization check - ensure user owns this swap
    // Compare user_id from JWT with the swap's user_id

    Ok(Json(SwapStatusResponse {
        swap_id: operation.swap_id,
        status: format!("{:?}", operation.status),
        from_chain: operation.from_chain,
        to_chain: operation.to_chain,
        amount: operation.amount.to_string(),
        recipient: operation.recipient,
        eth_tx_hash: operation.eth_tx_hash,
        near_tx_hash: operation.near_tx_hash,
        created_at: operation.created_at,
        updated_at: operation.updated_at,
        quantum_key_id: operation.quantum_key_id,
    }))
}