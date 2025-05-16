use warp::{Rejection, Reply, http::StatusCode};
use serde::Serialize;
use std::convert::Infallible;

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("Bad request: {0}")]
    BadRequest(String),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Query error: {0}")]
    QueryError(String),
    #[error("Invalid address format: {0}")]
    InvalidAddress(String),
    #[error("Invalid pagination parameters: {0}")]
    InvalidPagination(String),
    #[error("Invalid Tendermint address: {0}")]
    InvalidTendermintAddress(String),
    #[error("RPC connection error: {0}")]
    RpcConnectionError(String),
    #[error("Internal server error: {0}")]
    InternalError(String),
}

impl warp::reject::Reject for ApiError {}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    details: Option<String>,
}

pub async fn handle_rejection(err: Rejection) -> Result<impl Reply, Infallible> {
    let (code, message, details) = if err.is_not_found() {
        (StatusCode::NOT_FOUND, "Not Found".to_string(), None)
    } else if let Some(e) = err.find::<ApiError>() {
        match e {
            ApiError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.clone(), None),
            ApiError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone(), None),
            ApiError::QueryError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, "Query error occurred".to_string(), Some(msg.clone())),
            ApiError::InvalidAddress(msg) => (StatusCode::BAD_REQUEST, "Invalid address format".to_string(), Some(msg.clone())),
            ApiError::InvalidPagination(msg) => (StatusCode::BAD_REQUEST, "Invalid pagination parameters".to_string(), Some(msg.clone())),
            ApiError::InvalidTendermintAddress(msg) => (StatusCode::BAD_REQUEST, "Invalid Tendermint address".to_string(), Some(msg.clone())),
            ApiError::RpcConnectionError(msg) => (StatusCode::SERVICE_UNAVAILABLE, "RPC connection error".to_string(), Some(msg.clone())),
            ApiError::InternalError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string(), Some(msg.clone())),
        }
    } else {
        (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error".to_string(), None)
    };

    let json = warp::reply::json(&ErrorResponse {
        error: message,
        details,
    });

    Ok(warp::reply::with_status(json, code))
} 