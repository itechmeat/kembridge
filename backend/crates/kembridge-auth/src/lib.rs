pub mod web3;
pub mod jwt;
pub mod models;

use axum::{routing::post, Router};

pub fn routes<T: Clone + Send + Sync + 'static>() -> Router<T> {
    Router::new()
        .route("/nonce", post(|| async { "nonce endpoint - todo" }))
        .route("/verify", post(|| async { "verify endpoint - todo" }))
}