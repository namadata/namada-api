use axum::{
    extract::State,
    Json,
};
use std::sync::Arc;
use serde_json::json;

use crate::AppState;
use crate::models::ApiError;

pub async fn get_epoch(
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, ApiError> {
    match state.namada_client.query_epoch().await {
        Ok(epoch) => Ok(Json(json!({
            "epoch": epoch.0,
        }))),
        Err(e) => Err(ApiError::QueryError(e.to_string())),
    }
}

pub async fn get_native_token(
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, ApiError> {
    match state.namada_client.get_native_token().await {
        Ok(token) => Ok(Json(json!({
            "native_token": token.to_string(),
        }))),
        Err(e) => Err(ApiError::QueryError(e.to_string())),
    }
} 