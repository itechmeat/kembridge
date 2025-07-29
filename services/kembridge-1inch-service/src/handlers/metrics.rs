use axum::{extract::State, response::Response, http::StatusCode};
use crate::{services::AppState};

pub async fn get_metrics(
    State(_state): State<AppState>,
) -> Result<Response, StatusCode> {
    // Mock Prometheus metrics
    let metrics = r#"
# HELP oneinch_requests_total Total number of requests to 1inch service
# TYPE oneinch_requests_total counter
oneinch_requests_total{method="quote"} 150
oneinch_requests_total{method="swap"} 75

# HELP oneinch_request_duration_seconds Request duration in seconds
# TYPE oneinch_request_duration_seconds histogram
oneinch_request_duration_seconds_bucket{le="0.1"} 50
oneinch_request_duration_seconds_bucket{le="0.5"} 120
oneinch_request_duration_seconds_bucket{le="1.0"} 180
oneinch_request_duration_seconds_bucket{le="+Inf"} 200

# HELP oneinch_cache_hits_total Total cache hits
# TYPE oneinch_cache_hits_total counter
oneinch_cache_hits_total 320

# HELP oneinch_cache_misses_total Total cache misses
# TYPE oneinch_cache_misses_total counter
oneinch_cache_misses_total 80
"#;

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "text/plain; version=0.0.4")
        .body(metrics.into())
        .unwrap())
}