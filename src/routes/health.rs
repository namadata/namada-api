use axum::{
    Router,
    routing::get,
    extract::State,
    Json,
};
use std::sync::Arc;
use crate::state::AppState;
use serde::Serialize;

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    rpc_url: String,
}

async fn health_check(State(state): State<Arc<AppState>>) -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        rpc_url: state.namada_client.rpc_url().to_string(),
    })
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/health", get(health_check))
} 