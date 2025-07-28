// src/routes/quantum.rs - Quantum cryptography routes (Phase 3 placeholder)
use axum::{
    routing::{get, post},
    Router,
};
use crate::state::AppState;

/// Create quantum cryptography routes
/// These will be fully implemented in Phase 3.2 - Quantum Key Management
pub fn create_routes() -> Router<AppState> {
    Router::new()
        // Generate ML-KEM-1024 keypair (Phase 3.2.4)
        .route("/generate-keys", post(crate::handlers::quantum::generate_keypair))
        
        // Encapsulate data (Phase 3.2.5)
        .route("/encapsulate", post(crate::handlers::quantum::encapsulate))
        
        // Decapsulate data (Phase 3.2.6)
        .route("/decapsulate", post(crate::handlers::quantum::decapsulate))
        
        // Get user's quantum keys
        .route("/keys", get(crate::handlers::quantum::get_user_keys))
        
        // Export public key (Phase 3.2.8)
        .route("/keys/{key_id}/public", get(crate::handlers::quantum::export_public_key))
        
        // Key rotation endpoints (Task 3.2.7)
        .route("/keys/rotate", post(crate::handlers::quantum::rotate_key))
        .route("/keys/check-rotation", post(crate::handlers::quantum::check_rotation_needed))
        .route("/admin/check-rotation", post(crate::handlers::quantum::admin_check_rotation))
        
        // Hybrid key rotation endpoint (Task 3.4.4)
        .route("/keys/hybrid-rotate", post(crate::handlers::quantum::hybrid_rotate_key))
        
        // HybridCrypto endpoints (Task 3.4.5)
        .route("/hybrid/encrypt", post(crate::handlers::quantum::hybrid_encrypt))
        .route("/hybrid/decrypt", post(crate::handlers::quantum::hybrid_decrypt))
}