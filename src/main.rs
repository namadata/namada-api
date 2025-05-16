mod client;
mod config;
mod state;
mod models;
mod routes;

use axum::Router;
use clap::Parser;
use std::sync::Arc;
use std::net::SocketAddr;
use tower_http::cors::{CorsLayer, Any};
use tower_http::trace::TraceLayer;
use tracing::{info, error};

use crate::config::{CliArgs, Config};
use crate::state::AppState;
use crate::routes::{health, pos};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    let args = CliArgs::parse();
    let config = Config::load(args)?;
    info!("Configuration loaded with RPC URL: {}", config.rpc_url);
    
    let state = Arc::new(AppState::new(config.rpc_url).await?);
    
    match state.namada_client.check_connection().await {
        Ok(_) => info!("Successfully connected to Namada RPC endpoint"),
        Err(e) => {
            error!("Failed to connect to Namada RPC endpoint: {}", e);
        }
    }
    
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);
    let app = Router::new()
        .merge(health::router())
        .merge(pos::router())
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state);
    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    info!("Starting server on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;
    Ok(())
} 