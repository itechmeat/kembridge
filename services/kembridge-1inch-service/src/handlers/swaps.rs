use axum::{
    extract::{State, Path},
    Json,
};
use crate::errors::{OneinchServiceError, Result};
use crate::services::AppState;
use crate::types::{SwapExecutionRequest, SwapResponse, SignedSwapExecutionRequest};
use kembridge_common::ServiceResponse;
use tracing::{info, error};
use validator::Validate;

pub async fn execute_swap(
    State(state): State<AppState>,
    Json(request): Json<SwapExecutionRequest>,
) -> Result<Json<ServiceResponse<SwapResponse>>> {
    info!("Swap execution request for quote: {} by user: {}", 
          request.quote_id, request.user_address);

    // Validate request
    request.validate()
        .map_err(|e| OneinchServiceError::validation_error("request", &e.to_string()))?;

    // Execute swap
    match state.swap_executor.execute_swap(&request).await {
        Ok(swap_response) => {
            info!("Swap executed successfully: {} (tx: {:?})", 
                  swap_response.order_hash, swap_response.transaction_hash);
            Ok(Json(ServiceResponse::success(swap_response)))
        }
        Err(e) => {
            error!("Failed to execute swap: {}", e);
            Err(e)
        }
    }
}

pub async fn execute_signed_swap(
    State(state): State<AppState>,
    Json(request): Json<SignedSwapExecutionRequest>,
) -> Result<Json<ServiceResponse<SwapResponse>>> {
    info!("Signed swap execution request by user: {}", request.user_address);

    // Validate request
    validate_signed_swap_request(&request)?;

    // Execute signed swap
    match state.swap_executor.execute_signed_swap(&request).await {
        Ok(swap_response) => {
            info!("Signed swap executed successfully: {} (tx: {:?})", 
                  swap_response.order_hash, swap_response.transaction_hash);
            Ok(Json(ServiceResponse::success(swap_response)))
        }
        Err(e) => {
            error!("Failed to execute signed swap: {}", e);
            Err(e)
        }
    }
}

pub async fn get_order_status(
    State(state): State<AppState>,
    Path(order_hash): Path<String>,
) -> Result<Json<ServiceResponse<SwapResponse>>> {
    info!("Order status request for: {}", order_hash);

    // Validate order hash format
    if !is_valid_order_hash(&order_hash) {
        return Err(OneinchServiceError::validation_error(
            "order_hash", 
            "Invalid order hash format"
        ));
    }

    // Get order status
    match state.swap_executor.get_order_status(&order_hash).await {
        Ok(status) => {
            info!("Order status retrieved: {} - {:?}", order_hash, status.status);
            Ok(Json(ServiceResponse::success(status)))
        }
        Err(e) => {
            error!("Failed to get order status for {}: {}", order_hash, e);
            Err(e)
        }
    }
}

fn validate_signed_swap_request(request: &SignedSwapExecutionRequest) -> Result<()> {
    // Validate user address
    if !is_valid_ethereum_address(&request.user_address) {
        return Err(OneinchServiceError::validation_error(
            "user_address", 
            "Invalid Ethereum address format"
        ));
    }

    // Validate signature format
    if !is_valid_signature(&request.signature) {
        return Err(OneinchServiceError::validation_error(
            "signature", 
            "Invalid signature format"
        ));
    }

    // Validate transaction data
    if !is_valid_transaction_data(&request.transaction_data) {
        return Err(OneinchServiceError::validation_error(
            "transaction_data", 
            "Invalid transaction data format"
        ));
    }

    // Validate chain ID
    if !is_supported_chain_id(request.chain_id) {
        return Err(OneinchServiceError::ChainNotSupported {
            chain_id: request.chain_id,
        });
    }

    Ok(())
}

fn is_valid_order_hash(order_hash: &str) -> bool {
    order_hash.starts_with("0x") && order_hash.len() == 66 && 
    order_hash.chars().skip(2).all(|c| c.is_ascii_hexdigit())
}

fn is_valid_ethereum_address(address: &str) -> bool {
    address.starts_with("0x") && address.len() == 42 && 
    address.chars().skip(2).all(|c| c.is_ascii_hexdigit())
}

fn is_valid_signature(signature: &str) -> bool {
    // Ethereum signature is 132 characters (0x + 130 hex chars)
    signature.starts_with("0x") && signature.len() == 132 && 
    signature.chars().skip(2).all(|c| c.is_ascii_hexdigit())
}

fn is_valid_transaction_data(data: &str) -> bool {
    // Transaction data should be hex-encoded
    data.starts_with("0x") && data.len() >= 2 && 
    data.chars().skip(2).all(|c| c.is_ascii_hexdigit())
}

fn is_supported_chain_id(chain_id: u64) -> bool {
    // Major chains supported by 1inch
    matches!(chain_id, 1 | 56 | 137 | 10 | 42161 | 43114 | 250)
}

// Additional types for signed swaps
// Using SignedSwapExecutionRequest from types.rs to avoid duplication

// Batch swap operations
#[derive(Debug, serde::Deserialize, Validate)]
pub struct BatchSwapRequest {
    #[validate(length(min = 1, max = 10))] // Limit batch size
    pub swaps: Vec<SwapExecutionRequest>,
    
    pub execution_strategy: BatchExecutionStrategy,
    pub fail_fast: Option<bool>, // Stop on first failure
}

#[derive(Debug, serde::Deserialize)]
pub enum BatchExecutionStrategy {
    Sequential,  // Execute one by one
    Parallel,    // Execute all at once
    Optimized,   // Optimal order based on gas/dependencies
}

#[derive(Debug, serde::Serialize)]
pub struct BatchSwapResponse {
    pub batch_id: String,
    pub total_swaps: usize,
    pub successful_swaps: usize,
    pub failed_swaps: usize,
    pub results: Vec<BatchSwapResult>,
    pub total_execution_time_ms: u64,
}

#[derive(Debug, serde::Serialize)]
pub struct BatchSwapResult {
    pub swap_index: usize,
    pub result: std::result::Result<SwapResponse, String>,
    pub execution_time_ms: u64,
}

pub async fn execute_batch_swap(
    State(state): State<AppState>,
    Json(request): Json<BatchSwapRequest>,
) -> Result<Json<ServiceResponse<BatchSwapResponse>>> {
    info!("Batch swap execution request: {} swaps with strategy {:?}", 
          request.swaps.len(), request.execution_strategy);

    // Validate request
    request.validate()
        .map_err(|e| OneinchServiceError::validation_error("request", &e.to_string()))?;

    let start_time = std::time::Instant::now();
    let batch_id = uuid::Uuid::new_v4().to_string();
    
    let mut results = Vec::new();
    let mut successful_swaps = 0;
    let mut failed_swaps = 0;

    match request.execution_strategy {
        BatchExecutionStrategy::Sequential => {
            for (index, swap_request) in request.swaps.iter().enumerate() {
                let swap_start = std::time::Instant::now();
                
                let result = state.swap_executor.execute_swap(swap_request).await;
                let execution_time = swap_start.elapsed().as_millis() as u64;
                
                match &result {
                    Ok(_) => {
                        successful_swaps += 1;
                        info!("Batch swap {}/{} completed successfully", index + 1, request.swaps.len());
                    }
                    Err(e) => {
                        failed_swaps += 1;
                        error!("Batch swap {}/{} failed: {}", index + 1, request.swaps.len(), e);
                        
                        // Stop on first failure if fail_fast is enabled
                        if request.fail_fast.unwrap_or(false) {
                            results.push(BatchSwapResult {
                                swap_index: index,
                                result: Err(e.to_string()),
                                execution_time_ms: execution_time,
                            });
                            break;
                        }
                    }
                }
                
                results.push(BatchSwapResult {
                    swap_index: index,
                    result: result.map_err(|e| e.to_string()),
                    execution_time_ms: execution_time,
                });
            }
        }
        
        BatchExecutionStrategy::Parallel => {
            // Execute all swaps in parallel
            let futures: Vec<_> = request.swaps.iter().enumerate().map(|(index, swap_request)| {
                let executor = state.swap_executor.clone();
                let swap_request = swap_request.clone();
                async move {
                    let swap_start = std::time::Instant::now();
                    let result = executor.execute_swap(&swap_request).await;
                    let execution_time = swap_start.elapsed().as_millis() as u64;
                    
                    BatchSwapResult {
                        swap_index: index,
                        result: result.map_err(|e| e.to_string()),
                        execution_time_ms: execution_time,
                    }
                }
            }).collect();
            
            results = futures_util::future::join_all(futures).await;
            
            // Count successes and failures
            for result in &results {
                match &result.result {
                    Ok(_) => successful_swaps += 1,
                    Err(_) => failed_swaps += 1,
                }
            }
        }
        
        BatchExecutionStrategy::Optimized => {
            // For now, use sequential execution
            // In production, this would analyze dependencies and optimize order
            return execute_batch_swap(
                State(state), 
                Json(BatchSwapRequest {
                    swaps: request.swaps,
                    execution_strategy: BatchExecutionStrategy::Sequential,
                    fail_fast: request.fail_fast,
                })
            ).await;
        }
    }

    let total_execution_time = start_time.elapsed().as_millis() as u64;

    let response = BatchSwapResponse {
        batch_id,
        total_swaps: request.swaps.len(),
        successful_swaps,
        failed_swaps,
        results,
        total_execution_time_ms: total_execution_time,
    };

    info!("Batch swap completed: {}/{} successful in {}ms", 
          successful_swaps, request.swaps.len(), total_execution_time);

    Ok(Json(ServiceResponse::success(response)))
}