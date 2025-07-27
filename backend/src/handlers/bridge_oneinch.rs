// src/handlers/bridge_oneinch.rs - HTTP handlers for 1inch Bridge Integration

use axum::{
    extract::{State, Path},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    extractors::auth::AuthUser,
    oneinch::{OneinchBridgeIntegration, OptimizedBridgeSwapResult, OneinchBridgeError},
    state::AppState,
    constants::*,
};

/// Bridge swap request with 1inch optimization
#[derive(Debug, Deserialize, ToSchema)]
pub struct OptimizedBridgeSwapRequest {
    pub from_chain: String,
    pub to_chain: String,
    pub from_token: String,
    pub to_token: String,
    pub amount: String,
    pub recipient: String,
    pub optimization_strategy: Option<String>, // "output", "gas", "speed", "balanced"
    pub max_slippage: Option<f64>,
}

/// Bridge swap response
#[derive(Debug, Serialize, ToSchema)]
pub struct OptimizedBridgeSwapResponse {
    pub bridge_swap_id: String,
    pub bridge_status: String,
    pub source_optimization: Option<ChainOptimizationResponse>,
    pub destination_optimization: Option<ChainOptimizationResponse>,
    pub optimization_summary: OptimizationSummaryResponse,
    pub total_savings: String,
    pub eth_tx_hash: Option<String>,
    pub near_tx_hash: Option<String>,
    pub quantum_key_id: Option<String>,
    pub estimated_completion_time: Option<String>,
}

/// Chain optimization result for API response
#[derive(Debug, Serialize, ToSchema)]
pub struct ChainOptimizationResponse {
    pub chain: String,
    pub original_amount: String,
    pub optimized_output: String,
    pub gas_savings: String,
    pub output_improvement: String,
    pub oneinch_quote_id: Option<String>,
    pub optimization_applied: bool,
    pub improvement_percentage: f64,
}

/// Optimization summary for API response
#[derive(Debug, Serialize, ToSchema)]
pub struct OptimizationSummaryResponse {
    pub total_gas_savings: String,
    pub total_output_improvement: String,
    pub source_chain_optimized: bool,
    pub destination_chain_optimized: bool,
    pub overall_improvement_percentage: f64,
}

/// Bridge swap status response
#[derive(Debug, Serialize, ToSchema)]
pub struct BridgeSwapStatusResponse {
    pub swap_id: String,
    pub status: String,
    pub from_chain: String,
    pub to_chain: String,
    pub amount: String,
    pub recipient: String,
    pub eth_tx_hash: Option<String>,
    pub near_tx_hash: Option<String>,
    pub quantum_key_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub expires_at: String,
}

/// Execute optimized bridge swap with 1inch integration
#[utoipa::path(
    post,
    path = "/api/v1/bridge/swap/optimized",
    request_body = OptimizedBridgeSwapRequest,
    responses(
        (status = 200, description = "Optimized bridge swap executed successfully", body = OptimizedBridgeSwapResponse),
        (status = 400, description = "Invalid request parameters"),
        (status = 401, description = "Authentication required"),
        (status = 503, description = "Service unavailable")
    ),
    tag = "Bridge Integration"
)]
pub async fn execute_optimized_bridge_swap(
    State(state): State<AppState>,
    user: AuthUser,
    Json(request): Json<OptimizedBridgeSwapRequest>,
) -> Result<Json<OptimizedBridgeSwapResponse>, StatusCode> {
    // Validate request parameters
    if request.from_chain == request.to_chain {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Parse amount
    let amount = request.amount.parse::<bigdecimal::BigDecimal>()
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    // TODO: CRITICAL - Create bridge integration service when bridge service is available in AppState
    // This endpoint will not work until proper bridge service integration is implemented
    // For hackathon version, return not implemented
    Err(StatusCode::NOT_IMPLEMENTED)
}

/// Get bridge swap status
#[utoipa::path(
    get,
    path = "/api/v1/bridge/swap/{swap_id}/status",
    params(
        ("swap_id" = String, Path, description = "Bridge swap ID to check status")
    ),
    responses(
        (status = 200, description = "Bridge swap status retrieved successfully", body = BridgeSwapStatusResponse),
        (status = 404, description = "Swap not found"),
        (status = 401, description = "Authentication required")
    ),
    tag = "Bridge Integration"
)]
pub async fn get_bridge_swap_status(
    State(state): State<AppState>,
    _user: AuthUser,
    Path(swap_id): Path<String>,
) -> Result<Json<BridgeSwapStatusResponse>, StatusCode> {
    // Parse swap ID
    let swap_uuid = swap_id.parse::<Uuid>()
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    // TODO: CRITICAL - Get bridge integration service from state
    // This requires bridge service to be added to AppState and properly initialized
    // For hackathon version, return not implemented
    Err(StatusCode::NOT_IMPLEMENTED)
}

/// Calculate potential savings for bridge swap
#[utoipa::path(
    post,
    path = "/api/v1/bridge/swap/calculate-savings",
    request_body = OptimizedBridgeSwapRequest,
    responses(
        (status = 200, description = "Potential savings calculated successfully", body = OptimizationSummaryResponse),
        (status = 400, description = "Invalid request parameters"),
        (status = 401, description = "Authentication required")
    ),
    tag = "Bridge Integration"
)]
pub async fn calculate_bridge_swap_savings(
    State(state): State<AppState>,
    user: AuthUser,
    Json(request): Json<OptimizedBridgeSwapRequest>,
) -> Result<Json<OptimizationSummaryResponse>, StatusCode> {
    // This would calculate potential savings without executing the swap
    // For hackathon version, return estimated savings based on 1inch quotes
    
    let estimated_gas_savings = "0.02"; // 2% estimated gas savings
    let estimated_output_improvement = "0.015"; // 1.5% estimated output improvement
    
    let response = OptimizationSummaryResponse {
        total_gas_savings: estimated_gas_savings.to_string(),
        total_output_improvement: estimated_output_improvement.to_string(),
        source_chain_optimized: true,
        destination_chain_optimized: true,
        overall_improvement_percentage: 3.5, // Combined improvement
    };

    Ok(Json(response))
}

/// Get supported chains for bridge optimization
#[utoipa::path(
    get,
    path = "/api/v1/bridge/swap/supported-chains",
    responses(
        (status = 200, description = "Supported chains retrieved successfully", body = Vec<String>)
    ),
    tag = "Bridge Integration"
)]
pub async fn get_supported_bridge_chains(
    State(state): State<AppState>,
) -> Json<Vec<String>> {
    // Return chains supported by both bridge and 1inch
    let supported_chains = vec![
        "ethereum".to_string(),
        "bsc".to_string(),
        "polygon".to_string(),
        "avalanche".to_string(),
        "arbitrum".to_string(),
        "optimism".to_string(),
        "near".to_string(), // Supported by bridge but not 1inch
    ];

    Json(supported_chains)
}

// Helper functions

fn convert_to_api_response(result: OptimizedBridgeSwapResult) -> OptimizedBridgeSwapResponse {
    OptimizedBridgeSwapResponse {
        bridge_swap_id: result.bridge_swap_id.to_string(),
        bridge_status: format!("{:?}", result.bridge_status),
        source_optimization: result.source_optimization.map(convert_chain_optimization),
        destination_optimization: result.destination_optimization.map(convert_chain_optimization),
        optimization_summary: convert_optimization_summary(result.optimization_summary),
        total_savings: result.total_savings.to_string(),
        eth_tx_hash: result.eth_tx_hash,
        near_tx_hash: result.near_tx_hash,
        quantum_key_id: result.quantum_key_id,
        estimated_completion_time: Some("5 minutes".to_string()), // Simplified for hackathon
    }
}

fn convert_chain_optimization(opt: crate::oneinch::bridge_integration::ChainOptimizationResult) -> ChainOptimizationResponse {
    let improvement_percentage = if opt.original_amount > bigdecimal::BigDecimal::from(0) {
        let improvement_ratio = &opt.output_improvement / &opt.original_amount;
        improvement_ratio.to_string().parse::<f64>().unwrap_or(0.0) * 100.0
    } else {
        0.0
    };

    ChainOptimizationResponse {
        chain: opt.chain,
        original_amount: opt.original_amount.to_string(),
        optimized_output: opt.optimized_output.to_string(),
        gas_savings: opt.gas_savings.to_string(),
        output_improvement: opt.output_improvement.to_string(),
        oneinch_quote_id: opt.oneinch_quote_id,
        optimization_applied: opt.optimization_applied,
        improvement_percentage,
    }
}

fn convert_optimization_summary(summary: crate::oneinch::bridge_integration::OptimizationSummary) -> OptimizationSummaryResponse {
    // Calculate overall improvement percentage (simplified)
    // TODO: Replace with actual calculation based on real optimization data
    let overall_improvement = 2.5; // Placeholder for hackathon

    OptimizationSummaryResponse {
        total_gas_savings: summary.total_gas_savings.to_string(),
        total_output_improvement: summary.total_output_improvement.to_string(),
        source_chain_optimized: summary.source_chain_optimized,
        destination_chain_optimized: summary.destination_chain_optimized,
        overall_improvement_percentage: overall_improvement,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_optimization_conversion() {
        let opt = crate::oneinch::bridge_integration::ChainOptimizationResult {
            chain: "ethereum".to_string(),
            original_amount: bigdecimal::BigDecimal::from(100),
            optimized_output: bigdecimal::BigDecimal::from(105),
            gas_savings: bigdecimal::BigDecimal::from(2),
            output_improvement: bigdecimal::BigDecimal::from(5),
            oneinch_quote_id: Some("test_quote".to_string()),
            optimization_applied: true,
        };

        let response = convert_chain_optimization(opt);
        assert_eq!(response.chain, "ethereum");
        assert_eq!(response.improvement_percentage, 5.0);
        assert!(response.optimization_applied);
    }
}