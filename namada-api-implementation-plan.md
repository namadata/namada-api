
```# Namada API Implementation Plan

## Overview

This document outlines the implementation plan for building a Rust-based REST API to expose functionality from the Namada SDK, with a focus on the Proof-of-Stake (PoS) module. The API will enable Python (and other language) clients to query the Namada blockchain without needing to implement direct Rust bindings.

A key feature of this API is the ability to connect to any Namada RPC endpoint (local or remote) by configuring the RPC URL at runtime rather than being limited to only local nodes.

## Project Structure

```
namada-api/
├── Cargo.toml           # Project dependencies
├── src/
│   ├── main.rs          # Entry point & server startup
│   ├── config.rs        # Configuration handling (including RPC URL)
│   ├── client.rs        # Namada RPC client setup
│   ├── routes/          # API route handlers
│   │   ├── mod.rs       # Routes module
│   │   ├── pos.rs       # PoS-related routes
│   │   ├── token.rs     # Token-related routes
│   │   └── common.rs    # Common route utilities
│   ├── models/          # API data models
│   │   ├── mod.rs       # Models module
│   │   ├── pos.rs       # PoS-related models
│   │   └── error.rs     # Error models
│   └── services/        # Business logic
│       ├── mod.rs       # Services module
│       └── namada.rs    # Namada SDK interaction
└── README.md            # Project documentation
```

## Dependencies

```toml
[dependencies]
# Web framework
axum = "0.6"
tower-http = { version = "0.4", features = ["cors", "trace"] }
tokio = { version = "1", features = ["full"] }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Configuration
config = "0.13"
dotenvy = "0.15"
clap = { version = "4.3", features = ["derive", "env"] }

# Namada SDK
namada-sdk = "0.14.0"
tendermint-rpc = { version = "0.29", features = ["http-client"] }

# Logging & Tracing
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# Utilities
thiserror = "1.0"
reqwest = { version = "0.11", features = ["json"] }
```

## Implementation Phases

### Phase 1: Project Setup and Namada RPC Client Configuration

1. **Set up project structure**
   - Initialize Cargo project
   - Configure dependencies
   - Set up logging and error handling

2. **Implement configuration handling**
   - Create configuration struct with RPC URL and other settings
   - Support loading from environment variables (.env file)
   - Support command-line arguments for configuration overrides
   - Add validation for RPC URL format

3. **Implement Namada RPC client**
   - Create a client module that wraps the Namada SDK client
   - Support configurable RPC endpoints (local and remote)
   - Implement connection pooling and reconnection logic
   - Add health check for RPC endpoint connectivity
   - Create an application state that holds the client for use in routes

4. **Implement server initialization**
   - Configure Axum router with CORS
   - Set up health check endpoint that verifies RPC connectivity
   - Implement basic error handling middleware
   - Create server startup with graceful shutdown

5. **Create basic endpoints**
   - Implement `/api/health` endpoint (including RPC health)
   - Implement `/api/epoch` endpoint for current epoch
   - Implement `/api/native_token` endpoint

### Phase 2: PoS Module Implementation (Priority)

Focus on the PoS module with priority on the endpoints needed for liveness information and validator lookup by Tendermint address:

1. **Validators Endpoints**

   ```
   GET /api/pos/validators                # Get all validators
   GET /api/pos/validators/{address}      # Get validator details
   GET /api/pos/liveness_info             # Get validators liveness info
   GET /api/pos/validators/tm/{tm_addr}   # Find validator by Tendermint address
   ```

2. **Implementation Details for Key Endpoints**

   a. **Liveness Info Endpoint** (`GET /api/pos/liveness_info`)
   - Maps to the SDK's `liveness_info` function
   - Returns validators consensus participation metrics
   - Include window length and threshold in response

   b. **Validator by Tendermint Address** (`GET /api/pos/validators/tm/{tm_addr}`)
   - Maps to SDK's `validator_by_tm_addr` function
   - Find Namada validator address from Tendermint address
   - Properly handle input validation (important from SDK code review)

   c. **Validator Details** (`GET /api/pos/validators/{address}`)
   - Combine multiple SDK calls:
     - `is_validator`
     - `validator_stake`
     - `validator_commission`
     - `validator_metadata`
     - `validator_state`

3. **Delegations and Bonds Endpoints**

   ```
   GET /api/pos/delegations/{address}             # Get delegations for address
   GET /api/pos/bonds/{source}/to/{validator}     # Get bonds between addresses
   GET /api/pos/rewards/{validator}/{delegator}   # Get rewards earned
   ```

### Phase 3: Extended PoS Functionality

1. **Staking and Validator Set Endpoints**

   ```
   GET /api/pos/validator_sets/consensus          # Get consensus validators
   GET /api/pos/validator_sets/below_capacity     # Get below-capacity validators
   GET /api/pos/total_stake                       # Get total stake in the system
   ```

2. **Enhanced Bond and Unbond Endpoints**

   ```
   GET /api/pos/unbonds/{source}/to/{validator}   # Get unbonding details
   GET /api/pos/withdrawable/{source}/{validator} # Get withdrawable tokens
   ```

### Phase 4: Additional Endpoints and Refinement

1. **Add token-related endpoints**
   - Token balances
   - Token transfers
   - Total supply

2. **Add governance endpoints**
   - Proposals
   - Votes
   - Parameters

3. **API refinements**
   - Add pagination for list endpoints
   - Implement caching for frequently accessed data
   - Add sorting and filtering options

## Implementation Details: Namada RPC Client and PoS Routes

### 1. Namada Client Implementation

```rust
// src/client.rs
use std::sync::Arc;
use namada_sdk::rpc;
use namada_core::address::Address;
use thiserror::Error;
use tracing::{debug, error, info};

#[derive(Error, Debug)]
pub enum ClientError {
    #[error("RPC connection error: {0}")]
    ConnectionError(String),
    
    #[error("RPC query error: {0}")]
    QueryError(String),

    #[error("Invalid RPC URL: {0}")]
    InvalidUrl(String),
}

/// Wrapper for Namada SDK client
pub struct NamadaClient {
    rpc_client: reqwest::Client,
    rpc_url: String,
}

impl NamadaClient {
    /// Create a new Namada client with the specified RPC URL
    pub fn new(rpc_url: String) -> Self {
        info!("Initializing Namada client with RPC URL: {}", rpc_url);
        Self {
            rpc_client: reqwest::Client::new(),
            rpc_url,
        }
    }

    /// Check if the RPC endpoint is reachable
    pub async fn check_connection(&self) -> Result<(), ClientError> {
        debug!("Checking connection to Namada RPC at {}", self.rpc_url);
        match self.rpc_client.get(&self.rpc_url).send().await {
            Ok(response) if response.status().is_success() => {
                info!("Successfully connected to Namada RPC");
                Ok(())
            },
            Ok(response) => {
                let status = response.status();
                let error_msg = format!("RPC endpoint returned non-success status: {}", status);
                error!("{}", error_msg);
                Err(ClientError::ConnectionError(error_msg))
            },
            Err(e) => {
                let error_msg = format!("Failed to connect to RPC endpoint: {}", e);
                error!("{}", error_msg);
                Err(ClientError::ConnectionError(error_msg))
            }
        }
    }

    /// Get the underlying client for use with Namada SDK functions
    pub fn client(&self) -> &reqwest::Client {
        &self.rpc_client
    }

    /// Get the RPC URL
    pub fn rpc_url(&self) -> &str {
        &self.rpc_url
    }
    
    // Examples of convenience methods wrapping SDK functionality
    
    /// Query the current epoch
    pub async fn query_epoch(&self) -> Result<namada_core::chain::Epoch, ClientError> {
        rpc::query_epoch(self.client()).await
            .map_err(|e| ClientError::QueryError(e.to_string()))
    }
    
    /// Get validators liveness information
    pub async fn get_liveness_info(&self) -> Result<namada_proof_of_stake::types::LivenessInfo, ClientError> {
        rpc::get_validators_liveness_info(self.client()).await
            .map_err(|e| ClientError::QueryError(e.to_string()))
    }
    
    /// Find validator by Tendermint address
    pub async fn get_validator_by_tm_addr(&self, tm_addr: String) -> Result<Option<Address>, ClientError> {
        rpc::query_validator_by_tm_addr(self.client(), tm_addr).await
            .map_err(|e| ClientError::QueryError(e.to_string()))
    }
}
```

### 2. Application State

```rust
// src/main.rs or src/state.rs
use std::sync::Arc;
use crate::client::NamadaClient;

pub struct AppState {
    pub namada_client: Arc<NamadaClient>,
}

impl AppState {
    pub fn new(rpc_url: String) -> Self {
        Self {
            namada_client: Arc::new(NamadaClient::new(rpc_url)),
        }
    }
}
```

### 3. Configuration Handler

```rust
// src/config.rs
use clap::Parser;
use serde::Deserialize;
use std::path::PathBuf;

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
    // Add other configuration options as needed
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

#[derive(thiserror::Error, Debug)]
pub enum ConfigError {
    #[error("Invalid RPC URL: {0}")]
    InvalidRpcUrl(String),
    // Add other error types as needed
}

fn is_valid_url(url: &str) -> bool {
    url.starts_with("http://") || url.starts_with("https://")
}
```

### 4. Handler for Liveness Info

```rust
// src/routes/pos.rs
pub async fn get_liveness_info(
    State(state): State<Arc<AppState>>,
) -> Result<Json<models::LivenessInfoResponse>, ApiError> {
    match state.namada_client.get_liveness_info().await {
        Ok(liveness_info) => {
            let response = models::LivenessInfoResponse {
                liveness_window_len: liveness_info.liveness_window_len,
                liveness_threshold: liveness_info.liveness_threshold,
                validators: liveness_info.validators.into_iter().map(|v| models::ValidatorLiveness {
                    native_address: v.native_address.to_string(),
                    comet_address: v.comet_address,
                    missed_votes: v.missed_votes,
                }).collect(),
            };
            Ok(Json(response))
        },
        Err(e) => Err(ApiError::QueryError(e.to_string())),
    }
}
```

### 5. Handler for Validator by Tendermint Address

```rust
// src/routes/pos.rs
pub async fn get_validator_by_tm_addr(
    State(state): State<Arc<AppState>>,
    Path(tm_addr): Path<String>,
) -> Result<Json<models::ValidatorResponse>, ApiError> {
    // Important: Sanitize input to prevent issues seen in the SDK code
    if tm_addr.contains('/') || tm_addr.is_empty() {
        return Err(ApiError::BadRequest("Invalid Tendermint address format".to_string()));
    }

    match state.namada_client.get_validator_by_tm_addr(tm_addr).await {
        Ok(Some(validator_addr)) => {
            let response = models::ValidatorResponse {
                address: validator_addr.to_string(),
            };
            Ok(Json(response))
        },
        Ok(None) => Err(ApiError::NotFound("Validator not found".to_string())),
        Err(e) => Err(ApiError::QueryError(e.to_string())),
    }
}
```

### 6. Handler for Validator Details

```rust
// src/routes/pos.rs
pub async fn get_validator_details(
    State(state): State<Arc<AppState>>,
    Path(address_str): Path<String>,
) -> Result<Json<models::ValidatorDetailsResponse>, ApiError> {
    let client = state.namada_client.client();
    
    // Parse address
    let address = match Address::from_str(&address_str) {
        Ok(addr) => addr,
        Err(_) => return Err(ApiError::BadRequest("Invalid address format".to_string())),
    };

    // Check if address is a validator
    let is_validator = match rpc::is_validator(client, &address).await {
        Ok(is_val) => is_val,
        Err(e) => return Err(ApiError::QueryError(e.to_string())),
    };

    if !is_validator {
        return Err(ApiError::NotFound("Address is not a validator".to_string()));
    }

    // Get validator state
    let validator_state = match rpc::get_validator_state(client, &address, None).await {
        Ok(state) => state,
        Err(e) => return Err(ApiError::QueryError(e.to_string())),
    };

    // Get stake
    let stake = match rpc::get_validator_stake(client, validator_state.epoch, &address).await {
        Ok(stake) => stake.unwrap_or_default(),
        Err(e) => return Err(ApiError::QueryError(e.to_string())),
    };

    // Get metadata and commission
    let (metadata, commission) = match rpc::query_metadata(client, &address, None).await {
        Ok(result) => result,
        Err(e) => return Err(ApiError::QueryError(e.to_string())),
    };

    // Construct response
    let response = models::ValidatorDetailsResponse {
        address: address.to_string(),
        state: validator_state.state.to_string(),
        stake: stake.to_string(),
        commission_rate: commission.commission_rate.to_string(),
        max_commission_change_per_epoch: commission.max_commission_change_per_epoch.to_string(),
        metadata: metadata.map(|m| models::ValidatorMetadata {
            email: m.email,
            description: m.description,
            website: m.website,
            discord_handle: m.discord_handle,
            // Add other fields as needed
        }),
        // Add other fields as needed
    };

    Ok(Json(response))
}
```

## API Response Models

Define clear models for the API responses:

```rust
// src/models/pos.rs
#[derive(Serialize)]
pub struct LivenessInfoResponse {
    pub liveness_window_len: u64,
    pub liveness_threshold: Dec,
    pub validators: Vec<ValidatorLiveness>,
}

#[derive(Serialize)]
pub struct ValidatorLiveness {
    pub native_address: String,
    pub comet_address: String,
    pub missed_votes: u64,
}

#[derive(Serialize)]
pub struct ValidatorResponse {
    pub address: String,
}

#[derive(Serialize)]
pub struct ValidatorDetailsResponse {
    pub address: String,
    pub state: String,
    pub stake: String,
    pub commission_rate: String,
    pub max_commission_change_per_epoch: String,
    pub metadata: Option<ValidatorMetadata>,
    // Other fields as needed
}

#[derive(Serialize)]
pub struct ValidatorMetadata {
    pub email: Option<String>,
    pub description: Option<String>,
    pub website: Option<String>,
    pub discord_handle: Option<String>,
    // Other fields as needed
}
```

## Error Handling

Implement a comprehensive error handling system:

```rust
// src/models/error.rs
#[derive(Debug, Error)]
pub enum ApiError {
    #[error("Bad request: {0}")]
    BadRequest(String),
    
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Internal server error: {0}")]
    InternalError(String),
    
    #[error("Query error: {0}")]
    QueryError(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            ApiError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            ApiError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            ApiError::InternalError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            ApiError::QueryError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Query error: {}", msg)),
        };

        let body = Json(serde_json::json!({
            "error": error_message
        }));

        (status, body).into_response()
    }
}
```

## Deployment Plan

1. **Development Environment**
   - Local development with direct connection to a Namada node

2. **Testing Environment**
   - Dockerize the API for testing
   - Set up CI/CD pipeline for automated testing

3. **Production Deployment**
   - Deploy using Nginx as a reverse proxy
   - Set up SSL with Let's Encrypt
   - Configure systemd service for automatic restart
   - Implement monitoring and logging

4. **DNS Configuration**
   - Create A record pointing to the server IP
   - Set up HTTPS and automatic certificate renewal

## Python Client Example

Once the API is implemented, Python clients can interact with it easily:

```python
import requests

class NamadaClient:
    def __init__(self, base_url="http://localhost:3000/api"):
        self.base_url = base_url
        
    def get_liveness_info(self):
        """Get validator liveness information"""
        response = requests.get(f"{self.base_url}/pos/liveness_info")
        return response.json()
        
    def get_validator_by_tm_addr(self, tm_addr):
        """Find validator by Tendermint address"""
        response = requests.get(f"{self.base_url}/pos/validators/tm/{tm_addr}")
        return response.json()
        
    def get_validator_details(self, address):
        """Get validator details"""
        response = requests.get(f"{self.base_url}/pos/validators/{address}")
        return response.json()
```

## Next Steps and Extensions

1. **Caching Layer**
   - Implement Redis for frequent queries
   - Add TTL-based invalidation

2. **Authentication**
   - Add API key authentication for production
   - Consider JWT for more complex auth requirements

3. **Rate Limiting**
   - Implement per-client rate limiting
   - Add tiered access levels

4. **Monitoring**
   - Set up Prometheus metrics
   - Create Grafana dashboards

5. **Documentation**
   - Generate OpenAPI/Swagger documentation
   - Create comprehensive usage guides

This implementation plan provides a structured approach to building a Namada API with a focus on the PoS module, particularly liveness information and validator lookup by Tendermint address, as requested.

## Python Client Example

Once the API is implemented, Python clients can interact with it easily:

```python
import requests

class NamadaClient:
    def __init__(self, base_url="http://localhost:3000/api"):
        self.base_url = base_url
        
    def check_health(self):
        """Check if the API is running"""
        response = requests.get(f"{self.base_url}/health")
        return response.json()
        
    def check_rpc_health(self):
        """Check if the Namada RPC is connected"""
        response = requests.get(f"{self.base_url}/health/rpc")
        return response.json()
        
    def get_epoch(self):
        """Get current epoch"""
        response = requests.get(f"{self.base_url}/epoch")
        return response.json()
        
    def get_liveness_info(self):
        """Get validator liveness information"""
        response = requests.get(f"{self.base_url}/pos/liveness_info")
        return response.json()
        
    def get_validator_by_tm_addr(self, tm_addr):
        """Find validator by Tendermint address"""
        response = requests.get(f"{self.base_url}/pos/validators/tm/{tm_addr}")
        return response.json()
        
    def get_validator_details(self, address):
        """Get validator details"""
        response = requests.get(f"{self.base_url}/pos/validators/{address}")
        return response.json()
        
    def get_validators(self):
        """Get all validators"""
        response = requests.get(f"{self.base_url}/pos/validators")
        return response.json()
```

Example usage:

```python
# Connect to a specific Namada API endpoint
client = NamadaClient("https://api.namada-example.com/api")

# Check if API and RPC are healthy
api_health = client.check_health()
rpc_health = client.check_rpc_health()
print(f"API status: {api_health['status']}")
print(f"RPC status: {rpc_health['status']}, URL## Main Server Setup

```rust
// src/main.rs
use axum::{
    routing::{get, post},
    Router,
    Extension,
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
struct AppState {
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
```

## Health Check Implementation

```rust
// src/routes/health.rs
use axum::{
    extract::State,
    Json,
};
use std::sync::Arc;
use serde_json::json;
use tracing::warn;

use crate::AppState;

pub async fn check_health() -> Json<serde_json::Value> {
    Json(json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION"),
    }))
}

pub async fn check_rpc_health(
    State(state): State<Arc<AppState>>,
) -> Json<serde_json::Value> {
    match state.namada_client.check_connection().await {
        Ok(_) => Json(json!({
            "status": "ok",
            "rpc_url": state.namada_client.rpc_url(),
        })),
        Err(e) => {
            warn!("RPC health check failed: {}", e);
            Json(json!({
                "status": "error",
                "message": format!("RPC connection error: {}", e),
                "rpc_url": state.namada_client.rpc_url(),
            }))
        }
    }
}