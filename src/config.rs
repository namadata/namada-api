use clap::Parser;
use serde::Deserialize;
use std::path::PathBuf;
use tracing::{info, warn};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct CliArgs {
    /// Namada RPC URL
    #[arg(short, long, env = "NAMADA_RPC_URL")]
    pub rpc_url: Option<String>,

    /// API server port
    #[arg(short, long, env = "API_PORT", default_value = "3000")]
    pub port: u16,

    /// Config file path
    #[arg(short, long)]
    pub config: Option<PathBuf>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub rpc_url: String,
    pub port: u16,
    pub cors_allowed_origins: Vec<String>,
}

impl Config {
    pub fn load(args: CliArgs) -> Result<Self, ConfigError> {
        // First load from .env file
        match dotenvy::dotenv() {
            Ok(_) => info!("Loaded .env file"),
            Err(e) => warn!("No .env file found: {}", e),
        }
        
        // Initialize with defaults
        let mut config = Config {
            rpc_url: std::env::var("NAMADA_RPC_URL")
                .unwrap_or_else(|_| {
                    info!("NAMADA_RPC_URL not found in environment, using default");
                    "http://localhost:26657".to_string()
                }),
            port: std::env::var("API_PORT")
                .ok()
                .and_then(|p| p.parse::<u16>().ok())
                .unwrap_or_else(|| {
                    info!("API_PORT not found in environment or invalid, using default");
                    3000
                }),
            cors_allowed_origins: vec!["*".to_string()],
        };
        
        // Override with CLI args
        if let Some(rpc_url) = args.rpc_url {
            info!("Using RPC URL from command line arguments");
            config.rpc_url = rpc_url;
        } else {
            info!("Using RPC URL from environment: {}", config.rpc_url);
        }
        
        if args.port != 0 {
            info!("Using port from command line arguments");
            config.port = args.port;
        } else {
            info!("Using port from environment: {}", config.port);
        }
        
        // Validate configuration
        if !is_valid_url(&config.rpc_url) {
            return Err(ConfigError::InvalidRpcUrl(config.rpc_url));
        }
        
        Ok(config)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ConfigError {
    #[error("Invalid RPC URL: {0}")]
    InvalidRpcUrl(String),
}

fn is_valid_url(url: &str) -> bool {
    url.starts_with("http://") || url.starts_with("https://")
} 