use axum::{
    routing::{get, post},
    Router,
    extract::{State, Path},
    response::IntoResponse,
    Json,
};
use std::sync::Arc;
use tower_http::cors::{CorsLayer, Any};
use tower_http::trace::TraceLayer;
use tracing::{info, error};

mod models;
mod client;
mod config;

use models::pos::*;
use models::error::ApiError;

#[derive(Clone)]
struct AppState {
    namada_client: Arc<client::NamadaClient>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    // Create Namada client (for now with a default URL)
    let namada_client = Arc::new(client::NamadaClient::new("http://localhost:26657".to_string()));
    
    // Create application state
    let state = Arc::new(AppState { namada_client });
    
    // Configure CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);
    
    // Build the router
    let app = Router::new()
        // Health routes
        .route("/api/health", get(health_check))
        .route("/api/health/rpc", get(rpc_health_check))
        
        // PoS routes
        .route("/api/pos/liveness_info", get(get_liveness_info))
        .route("/api/pos/validators/tm/:address", get(get_validator_by_tm_addr))
        .route("/api/pos/validators/:address", get(get_validator_details))
        .route("/api/pos/validators", get(get_validators))
        
        // Add middleware
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state);
    
    // Start the server
    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], 3000));
    info!("Starting server on {}", addr);
    
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;
    
    Ok(())
}

// Health check handlers
async fn health_check() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

async fn rpc_health_check(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    match state.namada_client.check_connection().await {
        Ok(_) => Json(serde_json::json!({
            "status": "ok",
            "rpc_url": state.namada_client.rpc_url()
        })).into_response(),
        Err(e) => {
            error!("RPC health check failed: {}", e);
            Json(serde_json::json!({
                "status": "error",
                "message": format!("RPC connection error: {}", e),
                "rpc_url": state.namada_client.rpc_url()
            })).into_response()
        }
    }
}

// PoS handlers
async fn get_liveness_info(
    State(state): State<Arc<AppState>>,
) -> Result<Json<LivenessInfoResponse>, ApiError> {
    // For now, return mock data
    Ok(Json(LivenessInfoResponse {
        liveness_window_len: 100,
        liveness_threshold: "0.67".to_string(),
        validators: vec![
            ValidatorLiveness {
                native_address: "tnam1q...".to_string(),
                comet_address: "tnam1q...".to_string(),
                missed_votes: 0,
            }
        ],
    }))
}

async fn get_validator_by_tm_addr(
    State(state): State<Arc<AppState>>,
    Path(tm_addr): Path<String>,
) -> Result<Json<ValidatorResponse>, ApiError> {
    // For now, return mock data
    Ok(Json(ValidatorResponse {
        address: "tnam1q...".to_string(),
    }))
}

async fn get_validator_details(
    State(state): State<Arc<AppState>>,
    Path(address): Path<String>,
) -> Result<Json<ValidatorDetailsResponse>, ApiError> {
    // For now, return mock data
    Ok(Json(ValidatorDetailsResponse {
        address: address,
        state: "active".to_string(),
        stake: "1000000".to_string(),
        commission_rate: "0.05".to_string(),
        max_commission_change_per_epoch: "0.01".to_string(),
        metadata: Some(ValidatorMetadata {
            email: "validator@example.com".to_string(),
            description: Some("A test validator".to_string()),
            website: Some("https://example.com".to_string()),
            discord_handle: Some("validator#1234".to_string()),
        }),
    }))
}

async fn get_validators(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<ValidatorDetailsResponse>>, ApiError> {
    // For now, return mock data
    Ok(Json(vec![
        ValidatorDetailsResponse {
            address: "tnam1q...".to_string(),
            state: "active".to_string(),
            stake: "1000000".to_string(),
            commission_rate: "0.05".to_string(),
            max_commission_change_per_epoch: "0.01".to_string(),
            metadata: Some(ValidatorMetadata {
                email: "validator@example.com".to_string(),
                description: Some("A test validator".to_string()),
                website: Some("https://example.com".to_string()),
                discord_handle: Some("validator#1234".to_string()),
            }),
        }
    ]))
} 