use axum::{extract::{State, Path}, Json};
use crate::{errors::Result, services::AppState, types::{TokenPrice, PriceComparison}};
use kembridge_common::ServiceResponse;

pub async fn get_token_price(
    State(state): State<AppState>,
    Path(token): Path<String>,
) -> Result<Json<ServiceResponse<TokenPrice>>> {
    let price = state.price_oracle.get_token_price(1, &token).await?;
    Ok(Json(ServiceResponse::success(price)))
}

pub async fn compare_prices(
    State(state): State<AppState>,
    Path((from_token, to_token)): Path<(String, String)>,
) -> Result<Json<ServiceResponse<PriceComparison>>> {
    let comparison = state.price_oracle.compare_prices(1, &from_token, &to_token).await?;
    Ok(Json(ServiceResponse::success(comparison)))
}