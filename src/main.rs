use axum::{
    routing::get,
    Router,
    extract::State,
};
use clap::Parser;
use std::sync::Arc;
use std::net::SocketAddr;
use tower_http::cors::{CorsLayer, Any};
use tower_http::trace::TraceLayer;
use tracing::{info, error};

mod client;
mod config;
mod routes;
mod models;

use crate::client::NamadaClient;
use crate::config::{CliArgs, Config};

#[derive(Clone)]
pub struct AppState {
    namada_client: Arc<NamadaClient>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    // Parse command line arguments
    let args = CliArgs::parse();
    
    // Load configuration
    let config = Config::load(args)?;
    info!("Configuration loaded with RPC URL: {}", config.rpc_url);
    
    // Create Namada client
    let namada_client = Arc::new(NamadaClient::new(config.rpc_url));
    
    // Check RPC connection before starting the server
    match namada_client.check_connection().await {
        Ok(_) => info!("Successfully connected to Namada RPC endpoint"),
        Err(e) => {
            error!("Failed to connect to Namada RPC endpoint: {}", e);
            // Continue startup even if initial connection fails, as the RPC might become available later
        }
    }
    
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
        .route("/api/health", get(routes::health::check_health))
        .route("/api/health/rpc", get(routes::health::check_rpc_health))
        
        // Basic info routes
        .route("/api/epoch", get(routes::info::get_epoch))
        .route("/api/native_token", get(routes::info::get_native_token))
        
        // PoS routes
        .route("/api/pos/liveness_info", get(routes::pos::get_liveness_info))
        .route("/api/pos/validators/tm/:address", get(routes::pos::get_validator_by_tm_addr))
        .route("/api/pos/validators/:address", get(routes::pos::get_validator_details))
        .route("/api/pos/validators", get(routes::pos::get_validators))
        
        // Add middleware
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state);
    
    // Build address to bind to
    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    info!("Starting server on {}", addr);
    
    // Start the server
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;
    
    Ok(())
}
