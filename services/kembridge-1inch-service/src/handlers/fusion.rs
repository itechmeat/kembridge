use axum::{extract::State, Json};
use crate::{errors::Result, services::AppState, types::CrossChainQuoteRequest};
use kembridge_common::ServiceResponse;

pub async fn get_cross_chain_quote(
    State(_state): State<AppState>,
    Json(_request): Json<CrossChainQuoteRequest>,
) -> Result<Json<ServiceResponse<String>>> {
    Ok(Json(ServiceResponse::success("Cross-chain quote endpoint".to_string())))
}

pub async fn build_cross_chain_order(
    State(_state): State<AppState>,
    Json(_request): Json<serde_json::Value>,
) -> Result<Json<ServiceResponse<String>>> {
    Ok(Json(ServiceResponse::success("Build cross-chain order endpoint".to_string())))
}

pub async fn submit_cross_chain_order(
    State(_state): State<AppState>,
    Json(_request): Json<serde_json::Value>,
) -> Result<Json<ServiceResponse<String>>> {
    Ok(Json(ServiceResponse::success("Submit cross-chain order endpoint".to_string())))
}

pub async fn get_active_orders(
    State(_state): State<AppState>,
) -> Result<Json<ServiceResponse<Vec<String>>>> {
    Ok(Json(ServiceResponse::success(vec![])))
}

pub async fn get_order_by_hash(
    State(_state): State<AppState>,
    axum::extract::Path(_hash): axum::extract::Path<String>,
) -> Result<Json<ServiceResponse<String>>> {
    Ok(Json(ServiceResponse::success("Order details".to_string())))
}