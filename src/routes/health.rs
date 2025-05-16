use axum::{
    extract::State,
    Json,
};
use std::sync::Arc;
use serde_json::json;
use tracing::warn;

use crate::AppState;

pub async fn check_health() -> Json<serde_json::Value> {
    Json(json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION"),
    }))
}

pub async fn check_rpc_health(
    State(state): State<Arc<AppState>>,
) -> Json<serde_json::Value> {
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