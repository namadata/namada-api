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
}

impl warp::reject::Reject for ApiError {}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

pub async fn handle_rejection(err: Rejection) -> Result<impl Reply, Infallible> {
    let (code, message) = if err.is_not_found() {
        (StatusCode::NOT_FOUND, "Not Found".to_string())
    } else if let Some(e) = err.find::<ApiError>() {
        match e {
            ApiError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            ApiError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
            ApiError::QueryError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()),
        }
    } else {
        (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error".to_string())
    };

    let json = warp::reply::json(&ErrorResponse {
        error: message,
    });

    Ok(warp::reply::with_status(json, code))
} 