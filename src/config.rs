use clap::Parser;
use serde::Deserialize;
use std::path::PathBuf;
use thiserror::Error;

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

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Invalid RPC URL: {0}")]
    InvalidRpcUrl(String),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
}

impl Config {
    // Load configuration from file and override with CLI args
    pub fn load(args: CliArgs) -> Result<Self, ConfigError> {
        // First load from .env file
        let _ = dotenvy::dotenv();
        
        // Initialize with defaults
        let mut config = Config {
            rpc_url: std::env::var("NAMADA_RPC_URL")
                .unwrap_or_else(|_| "http://localhost:26657".to_string()),
            port: std::env::var("API_PORT")
                .ok()
                .and_then(|p| p.parse::<u16>().ok())
                .unwrap_or(3000),
            cors_allowed_origins: vec!["*".to_string()],
        };
        
        // Override with config file if provided
        if let Some(config_path) = args.config {
            // Load and merge config from file
            // This is a simplified implementation
            if config_path.exists() {
                return Err(ConfigError::ConfigError(
                    "Config file loading not implemented yet".to_string()
                ));
            }
        }
        
        // Override with CLI args
        if let Some(rpc_url) = args.rpc_url {
            config.rpc_url = rpc_url;
        }
        
        if args.port != 0 {
            config.port = args.port;
        }
        
        // Validate configuration
        if !is_valid_url(&config.rpc_url) {
            return Err(ConfigError::InvalidRpcUrl(config.rpc_url));
        }
        
        Ok(config)
    }
}

fn is_valid_url(url: &str) -> bool {
    url.starts_with("http://") || url.starts_with("https://")
} 