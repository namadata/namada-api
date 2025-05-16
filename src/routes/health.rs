use axum::{extract::State, Json, routing::get, Router, response::IntoResponse};
use std::sync::Arc;
use serde_json::json;
use tracing::warn;

use crate::state::AppState;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/health", get(check_health))
        .route("/api/health/rpc", get(check_rpc_health))
}

pub async fn check_health() -> impl IntoResponse {
    Json(json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION"),
    }))
}

pub async fn check_rpc_health(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    match state.namada_client.check_connection().await {
        Ok(_) => Json(json!({
            "status": "ok",
            "rpc_url": state.namada_client.rpc_url(),
        })),
        Err(e) => {
            warn!("RPC health check failed: {}", e);
            Json(json!({
                "status": "error",
                "message": format!("RPC connection error: {}", e),
                "rpc_url": state.namada_client.rpc_url(),
            }))
        }
    }
} 