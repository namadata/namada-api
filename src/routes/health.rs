use axum::{
    extract::State,
    Json,
    routing::get,
    Router,
    http::StatusCode,
    response::IntoResponse
};
use std::sync::Arc;
use serde_json::json;
use tracing::warn;

use crate::state::AppState;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/health", get(
            || async {
                (
                    StatusCode::OK, 
                    Json(json!({
                        "status": "ok",
                        "version": env!("CARGO_PKG_VERSION")
                    }))
                )
            }
        ))
        .route("/api/health/rpc", get(
            |State(state): State<Arc<AppState>>| async move {
                match state.namada_client.check_connection().await {
                    Ok(_) => (
                        StatusCode::OK,
                        Json(json!({
                            "status": "ok",
                            "rpc_url": state.namada_client.rpc_url()
                        }))
                    ),
                    Err(e) => {
                        warn!("RPC health check failed: {}", e);
                        (
                            StatusCode::OK,
                            Json(json!({
                                "status": "error",
                                "message": format!("RPC connection error: {}", e),
                                "rpc_url": state.namada_client.rpc_url()
                            }))
                        )
                    }
                }
            }
        ))
} 