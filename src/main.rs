use std::sync::Arc;
use warp::{Filter, Rejection, Reply};
use tracing::{info, error};
use namada_core::address::Address;
use std::str::FromStr;
use std::convert::Infallible;
use clap::Parser;
use serde::Deserialize;

mod models;
mod client;
mod config;
#[cfg(test)]
mod tests;

use models::pos::*;
use models::error::{ApiError, handle_rejection};
use config::{CliArgs, Config};

/// Application state shared across all handlers
#[derive(Clone)]
pub struct AppState {
    namada_client: Arc<client::NamadaClient>,
}

#[derive(Debug, Deserialize)]
pub struct ValidatorsQuery {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
}

impl ValidatorsQuery {
    pub fn validate(&self) -> Result<(), ApiError> {
        if let Some(page) = self.page {
            if page == 0 {
                return Err(ApiError::InvalidPagination("Page number must be greater than 0".to_string()));
            }
        }
        
        if let Some(per_page) = self.per_page {
            if per_page == 0 {
                return Err(ApiError::InvalidPagination("Items per page must be greater than 0".to_string()));
            }
            if per_page > 50 {
                return Err(ApiError::InvalidPagination("Items per page cannot exceed 50".to_string()));
            }
        }
        
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    // Parse command line arguments
    let args = CliArgs::parse();
    let config = Config::load(args)?;
    
    info!("Starting Namada API with RPC URL: {}", config.rpc_url);
    
    // Create Namada client with configured URL
    let namada_client = Arc::new(client::NamadaClient::new(config.rpc_url).await?);
    
    // Create application state
    let state = Arc::new(AppState { namada_client });
    
    // Documentation route
    let docs = warp::path("api")
        .and(warp::path("docs"))
        .and(warp::get())
        .and_then(serve_docs);
    
    // Health routes
    let health = warp::path("api")
        .and(warp::path("health"))
        .and(warp::path("api_status"))
        .and(warp::get())
        .and_then(health_check);
        
    let rpc_health = warp::path("api")
        .and(warp::path("health"))
        .and(warp::path("rpc_status"))
        .and(warp::get())
        .and(with_state(state.clone()))
        .and_then(rpc_health_check);
    
    // PoS routes
    let liveness_info = warp::path("api")
        .and(warp::path("pos"))
        .and(warp::path("liveness_info"))
        .and(warp::get())
        .and(with_state(state.clone()))
        .and_then(get_liveness_info);
        
    let validator_by_tm = warp::path("api")
        .and(warp::path("pos"))
        .and(warp::path("validator_by_tm_addr"))
        .and(warp::path::param::<String>())
        .and(warp::get())
        .and(with_state(state.clone()))
        .and_then(|tm_addr: String, state: Arc<AppState>| async move {
            get_validator_by_tm_addr(state, tm_addr).await
        });
        
    let validator_details = warp::path("api")
        .and(warp::path("pos"))
        .and(warp::path("validators"))
        .and(warp::path::param::<String>())
        .and(warp::get())
        .and(with_state(state.clone()))
        .and_then(|address: String, state: Arc<AppState>| async move {
            get_validator_details(state, address).await
        });
        
    let all_validators = warp::path("api")
        .and(warp::path("pos"))
        .and(warp::path("validators"))
        .and(warp::get())
        .and(with_state(state.clone()))
        .and_then(get_all_validators);
        
    let validators_details = warp::path("api")
        .and(warp::path("pos"))
        .and(warp::path("validators_details"))
        .and(warp::get())
        .and(warp::query::<ValidatorsQuery>())
        .and(with_state(state.clone()))
        .and_then(|query: ValidatorsQuery, state: Arc<AppState>| async move {
            get_validators_details(state, query).await
        });

    let consensus_validator_set = warp::path("api")
        .and(warp::path("pos"))
        .and(warp::path("validator_set"))
        .and(warp::path("consensus"))
        .and(warp::get())
        .and(with_state(state.clone()))
        .and_then(get_consensus_validator_set);

    let below_capacity_validator_set = warp::path("api")
        .and(warp::path("pos"))
        .and(warp::path("validator_set"))
        .and(warp::path("below_capacity"))
        .and(warp::get())
        .and(with_state(state.clone()))
        .and_then(get_below_capacity_validator_set);
    
    // Combine all routes
    let routes = docs
        .or(health)
        .or(rpc_health)
        .or(liveness_info)
        .or(validator_by_tm)
        .or(validator_details)
        .or(all_validators)
        .or(validators_details)
        .or(consensus_validator_set)
        .or(below_capacity_validator_set)
        .with(warp::cors().allow_any_origin())
        .recover(handle_rejection);
    
    // Start the server
    let addr = ([127, 0, 0, 1], config.port);
    info!("Starting server on {:?}", addr);
    
    warp::serve(routes).run(addr).await;
    
    Ok(())
}

/// Helper function to inject application state into handlers
pub fn with_state(state: Arc<AppState>) -> impl Filter<Extract = (Arc<AppState>,), Error = Infallible> + Clone {
    warp::any().map(move || state.clone())
}

/// Basic health check endpoint
/// 
/// # Endpoint
/// `GET /api/health/api_status`
/// 
/// # Response
/// ```json
/// {
///     "status": "ok",
///     "version": "0.1.0"
/// }
/// ```
pub async fn health_check() -> Result<impl Reply, Rejection> {
    Ok(warp::reply::json(&serde_json::json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION")
    })))
}

/// RPC connection health check endpoint
/// 
/// # Endpoint
/// `GET /api/health/rpc_status`
/// 
/// # Response
/// Success:
/// ```json
/// {
///     "status": "ok",
///     "rpc_url": "https://rpc-1.namada.nodes.guru"
/// }
/// ```
/// 
/// Error:
/// ```json
/// {
///     "status": "error",
///     "message": "RPC connection error: ...",
///     "rpc_url": "https://rpc-1.namada.nodes.guru"
/// }
/// ```
pub async fn rpc_health_check(state: Arc<AppState>) -> Result<impl Reply, Rejection> {
    // Try to connect to Namada RPC
    let response = match state.namada_client.check_connection().await {
        Ok(_) => {
            serde_json::json!({
                "status": "ok",
                "rpc_url": state.namada_client.rpc_url()
            })
        },
        Err(err) => {
            error!("RPC health check failed: {}", err);
            
            // Return error with status code
            return Err(warp::reject::custom(
                ApiError::RpcConnectionError(err.to_string())
            ));
        }
    };
    
    Ok(warp::reply::json(&response))
}

/// Get liveness information for validators
/// 
/// # Endpoint
/// `GET /api/pos/liveness_info`
/// 
/// # Response
/// ```json
/// {
///     "liveness_window_len": 100,
///     "liveness_threshold": "0.9",
///     "validators": [
///         {
///             "native_address": "tnam1q...",
///             "comet_address": "tnam1q...",
///             "missed_votes": 0
///         }
///     ]
/// }
/// ```
pub async fn get_liveness_info(state: Arc<AppState>) -> Result<impl Reply, Rejection> {
    // Query liveness info
    let liveness_info = state.namada_client.get_liveness_info().await
        .map_err(|err| {
            error!("Failed to get liveness info: {}", err);
            warp::reject::custom(
                ApiError::QueryError(format!("Failed to fetch liveness info: {}", err))
            )
        })?;
    
    // Build validators list from the response
    let validators = liveness_info.validators.iter()
        .map(|v| ValidatorLiveness {
            native_address: v.native_address.to_string(),
            comet_address: v.comet_address.clone(),
            missed_votes: v.missed_votes,
        })
        .collect();
    
    // Create the response
    let response = LivenessInfoResponse {
        liveness_window_len: liveness_info.liveness_window_len,
        liveness_threshold: liveness_info.liveness_threshold.to_string(),
        validators,
    };
    
    Ok(warp::reply::json(&response))
}

/// Find validator by Tendermint address
/// 
/// # Endpoint
/// `GET /api/pos/validator_by_tm_addr/{tm_addr}`
/// 
/// # Parameters
/// - `tm_addr`: Tendermint consensus address (40 hex characters, e.g. "CAFAD8DA813BAE48779A4219A74632D5DCA49737")
/// 
/// # Response
/// ```json
/// {
///     "address": "tnam1q..."
/// }
/// ```
pub async fn get_validator_by_tm_addr(
    state: Arc<AppState>,
    tm_addr: String,
) -> Result<impl Reply, Rejection> {
    // Sanitize Tendermint address - should be 40 hex characters
    if !tm_addr.chars().all(|c| c.is_ascii_hexdigit()) || tm_addr.len() != 40 {
        return Err(warp::reject::custom(
            ApiError::InvalidTendermintAddress(format!("Invalid Tendermint address format: {}. Expected 40 hex characters.", tm_addr))
        ));
    }

    // Use the client's validator_by_tm_addr method to find the validator
    let validator_address = state.namada_client.validator_by_tm_addr(tm_addr.clone()).await
        .map_err(|err| {
            error!("Failed to query validator by Tendermint address: {}", err);
            warp::reject::custom(
                ApiError::QueryError(format!("Failed to query validator by Tendermint address: {}", err))
            )
        })?;
    
    match validator_address {
        Some(address) => {
            // Found the validator
            Ok(warp::reply::json(&ValidatorResponse {
                address: address.to_string(),
            }))
        },
        None => {
            // No validator found with the given tendermint address
            Err(warp::reject::custom(
                ApiError::NotFound(format!("No validator found with Tendermint address: {}", tm_addr))
            ))
        }
    }
}

/// Get detailed validator information
/// 
/// # Endpoint
/// `GET /api/pos/validators/{address}`
/// 
/// # Parameters
/// - `address`: Namada address of the validator
/// 
/// # Response
/// ```json
/// {
///     "address": "tnam1q...",
///     "state": "active",
///     "stake": "1000000",
///     "commission_rate": "0.05",
///     "max_commission_change_per_epoch": "0.01",
///     "metadata": {
///         "email": "validator@example.com",
///         "description": "Professional validator",
///         "website": "https://example.com",
///         "discord_handle": "validator#1234"
///     }
/// }
/// ```
async fn get_validator_details(
    state: Arc<AppState>,
    address: String,
) -> Result<impl Reply, Rejection> {
    // Validate address format
    let address = Address::from_str(&address)
        .map_err(|e| warp::reject::custom(ApiError::InvalidAddress(format!("Invalid address format: {}", e))))?;
    
    // Check if address is a validator
    let is_validator = state.namada_client.is_validator(&address).await
        .map_err(|e| warp::reject::custom(ApiError::QueryError(e.to_string())))?;
    
    if !is_validator {
        return Err(warp::reject::custom(ApiError::NotFound(format!("Address {} is not a validator", address))));
    }
    
    let epoch = state.namada_client.query_epoch().await
        .map_err(|e| warp::reject::custom(ApiError::QueryError(e.to_string())))?;
    
    let (state_info, epoch) = state.namada_client.get_validator_state(&address, Some(epoch)).await
        .map_err(|e| warp::reject::custom(ApiError::QueryError(e.to_string())))?;
    
    let stake = state.namada_client.get_validator_stake(epoch, &address).await
        .map_err(|e| warp::reject::custom(ApiError::QueryError(e.to_string())))?;
    
    let (metadata, commission) = state.namada_client.query_metadata(&address, Some(epoch)).await
        .map_err(|e| warp::reject::custom(ApiError::QueryError(e.to_string())))?;
    
    Ok(warp::reply::json(&ValidatorDetailsResponse {
        address: address.to_string(),
        state: state_info.map_or("unknown".to_string(), |s| format!("{:?}", s)),
        stake: stake.to_string(),
        commission_rate: commission.commission_rate.map_or("0".to_string(), |r| r.to_string()),
        max_commission_change_per_epoch: commission.max_commission_change_per_epoch.map_or("0".to_string(), |r| r.to_string()),
        metadata: metadata.map(|m| ValidatorMetadata {
            email: m.email,
            description: m.description,
            website: m.website,
            discord_handle: m.discord_handle,
        }),
    }))
}

/// Get list of all validators (simple list)
/// 
/// # Endpoint
/// `GET /api/pos/validators`
/// 
/// # Response
/// ```json
/// {
///     "validators": [
///         "tnam1q...",
///         "tnam1q..."
///     ]
/// }
/// ```
async fn get_all_validators(state: Arc<AppState>) -> Result<impl Reply, Rejection> {
    let epoch = state.namada_client.query_epoch().await
        .map_err(|e| warp::reject::custom(ApiError::QueryError(e.to_string())))?;
    
    let validators = state.namada_client.get_all_validators(Some(epoch)).await
        .map_err(|e| warp::reject::custom(ApiError::QueryError(e.to_string())))?;
    
    Ok(warp::reply::json(&serde_json::json!({
        "validators": validators.into_iter().map(|addr| addr.to_string()).collect::<Vec<String>>()
    })))
}

/// Get detailed information about all validators with pagination
/// 
/// # Endpoint
/// `GET /api/pos/validators_details?page={page}&per_page={per_page}`
/// 
/// # Parameters
/// - `page`: Page number (default: 1)
/// - `per_page`: Number of validators per page (default: 10, max: 50)
/// 
/// # Response
/// ```json
/// {
///     "validators": [
///         {
///             "address": "tnam1q...",
///             "state": "active",
///             "stake": "1000000",
///             "commission_rate": "0.05",
///             "max_commission_change_per_epoch": "0.01",
///             "metadata": {
///                 "email": "validator@example.com",
///                 "description": "Professional validator",
///                 "website": "https://example.com",
///                 "discord_handle": "validator#1234"
///             }
///         }
///     ],
///     "pagination": {
///         "total": 100,
///         "page": 1,
///         "per_page": 10,
///         "total_pages": 10
///     }
/// }
/// ```
async fn get_validators_details(
    state: Arc<AppState>,
    query: ValidatorsQuery,
) -> Result<impl Reply, Rejection> {
    // Validate query parameters
    query.validate()?;
    
    // Set default values
    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(10);
    
    let epoch = state.namada_client.query_epoch().await
        .map_err(|e| warp::reject::custom(ApiError::QueryError(e.to_string())))?;
    
    let validators = state.namada_client.get_all_validators(Some(epoch)).await
        .map_err(|e| warp::reject::custom(ApiError::QueryError(e.to_string())))?;
    
    let total = validators.len();
    let total_pages = (total as f64 / per_page as f64).ceil() as u32;
    
    // Validate page number against total pages
    if page > total_pages && total_pages > 0 {
        return Err(warp::reject::custom(ApiError::InvalidPagination(
            format!("Page number {} exceeds total pages {}", page, total_pages)
        )));
    }
    
    // Calculate start and end indices for the current page
    let start = ((page - 1) * per_page) as usize;
    let end = (start + per_page as usize).min(total);
    
    // Get validators for the current page
    let mut responses = Vec::new();
    for address in validators.into_iter().skip(start).take(end - start) {
        let (state_info, epoch) = state.namada_client.get_validator_state(&address, Some(epoch)).await
            .map_err(|e| warp::reject::custom(ApiError::QueryError(e.to_string())))?;
        
        let stake = state.namada_client.get_validator_stake(epoch, &address).await
            .map_err(|e| warp::reject::custom(ApiError::QueryError(e.to_string())))?;
        
        let (metadata, commission) = state.namada_client.query_metadata(&address, Some(epoch)).await
            .map_err(|e| warp::reject::custom(ApiError::QueryError(e.to_string())))?;
        
        responses.push(ValidatorDetailsResponse {
            address: address.to_string(),
            state: state_info.map_or("unknown".to_string(), |s| format!("{:?}", s)),
            stake: stake.to_string(),
            commission_rate: commission.commission_rate.map_or("0".to_string(), |r| r.to_string()),
            max_commission_change_per_epoch: commission.max_commission_change_per_epoch.map_or("0".to_string(), |r| r.to_string()),
            metadata: metadata.map(|m| ValidatorMetadata {
                email: m.email,
                description: m.description,
                website: m.website,
                discord_handle: m.discord_handle,
            }),
        });
    }
    
    Ok(warp::reply::json(&serde_json::json!({
        "validators": responses,
        "pagination": {
            "total": total,
            "page": page,
            "per_page": per_page,
            "total_pages": total_pages
        }
    })))
}

/// Get consensus validator set
/// 
/// # Endpoint
/// `GET /api/pos/validator_set/consensus`
/// 
/// # Response
/// ```json
/// {
///     "validators": [
///         {
///             "address": "tnam1q...",
///             "stake": "1000000"
///         }
///     ]
/// }
/// ```
async fn get_consensus_validator_set(state: Arc<AppState>) -> Result<impl Reply, Rejection> {
    let validators = state.namada_client.get_consensus_validator_set(None).await
        .map_err(|e| warp::reject::custom(ApiError::QueryError(e.to_string())))?;
    
    let response = ValidatorSetResponse {
        validators: validators.into_iter().map(|v| WeightedValidatorResponse {
            address: v.address.to_string(),
            stake: v.bonded_stake.to_string(),
        }).collect(),
    };
    
    Ok(warp::reply::json(&response))
}

/// Get below-capacity validator set
/// 
/// # Endpoint
/// `GET /api/pos/validator_set/below_capacity`
/// 
/// # Response
/// ```json
/// {
///     "validators": [
///         {
///             "address": "tnam1q...",
///             "stake": "1000000"
///         }
///     ]
/// }
/// ```
async fn get_below_capacity_validator_set(state: Arc<AppState>) -> Result<impl Reply, Rejection> {
    let validators = state.namada_client.get_below_capacity_validator_set(None).await
        .map_err(|e| warp::reject::custom(ApiError::QueryError(e.to_string())))?;
    
    let response = ValidatorSetResponse {
        validators: validators.into_iter().map(|v| WeightedValidatorResponse {
            address: v.address.to_string(),
            stake: v.bonded_stake.to_string(),
        }).collect(),
    };
    
    Ok(warp::reply::json(&response))
}

/// Serve API documentation
/// 
/// # Endpoint
/// `GET /api/docs`
async fn serve_docs() -> Result<impl Reply, Rejection> {
    let html = r#"
    <!DOCTYPE html>
    <html>
    <head>
        <title>Namada API Documentation</title>
        <style>
            body {
                font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif;
                line-height: 1.6;
                max-width: 1200px;
                margin: 0 auto;
                padding: 20px;
                color: #333;
            }
            h1 {
                color: #2c3e50;
                border-bottom: 2px solid #eee;
                padding-bottom: 10px;
            }
            h2 {
                color: #34495e;
                margin-top: 30px;
            }
            .endpoint {
                background: #f8f9fa;
                border: 1px solid #e9ecef;
                border-radius: 4px;
                padding: 20px;
                margin: 20px 0;
            }
            .method {
                font-weight: bold;
                color: #2ecc71;
            }
            .path {
                font-family: monospace;
                background: #e9ecef;
                padding: 2px 6px;
                border-radius: 3px;
            }
            .response {
                background: #f1f8e9;
                border: 1px solid #dcedc8;
                border-radius: 4px;
                padding: 15px;
                margin: 10px 0;
            }
            .error {
                background: #ffebee;
                border: 1px solid #ffcdd2;
                border-radius: 4px;
                padding: 15px;
                margin: 10px 0;
            }
            pre {
                background: #f8f9fa;
                padding: 15px;
                border-radius: 4px;
                overflow-x: auto;
            }
            code {
                font-family: "SFMono-Regular", Consolas, "Liberation Mono", Menlo, monospace;
            }
            .params {
                margin: 10px 0;
            }
            .param {
                margin: 5px 0;
            }
            .param-name {
                font-weight: bold;
                color: #2c3e50;
            }
            .param-desc {
                color: #666;
            }
        </style>
    </head>
    <body>
        <h1>Namada API Documentation</h1>
        
        <h2>Health Endpoints</h2>
        
        <div class="endpoint">
            <h3>Basic Health Check</h3>
            <p><span class="method">GET</span> <span class="path">/api/health/api_status</span></p>
            <p>Check if the API is running.</p>
            <div class="response">
                <h4>Response:</h4>
                <pre><code>{
    "status": "ok",
    "version": "0.1.0"
}</code></pre>
            </div>
        </div>

        <div class="endpoint">
            <h3>RPC Health Check</h3>
            <p><span class="method">GET</span> <span class="path">/api/health/rpc_status</span></p>
            <p>Check if the RPC connection is working.</p>
            <div class="response">
                <h4>Success Response:</h4>
                <pre><code>{
    "status": "ok",
    "rpc_url": "https://rpc-1.namada.nodes.guru"
}</code></pre>
                <h4>Error Response:</h4>
                <pre><code>{
    "status": "error",
    "message": "RPC connection error: ...",
    "rpc_url": "https://rpc-1.namada.nodes.guru"
}</code></pre>
            </div>
        </div>

        <h2>Proof of Stake Endpoints</h2>

        <div class="endpoint">
            <h3>Get Validator Liveness Information</h3>
            <p><span class="method">GET</span> <span class="path">/api/pos/liveness_info</span></p>
            <p>Get liveness information for all validators.</p>
            <div class="response">
                <h4>Response:</h4>
                <pre><code>{
    "liveness_window_len": 100,
    "liveness_threshold": "0.9",
    "validators": [
        {
            "native_address": "tnam1q...",
            "comet_address": "tnam1q...",
            "missed_votes": 0
        }
    ]
}</code></pre>
            </div>
        </div>

        <div class="endpoint">
            <h3>Get Validator by Tendermint Address</h3>
            <p><span class="method">GET</span> <span class="path">/api/pos/validator_by_tm_addr/{tm_addr}</span></p>
            <p>Get validator information by their Tendermint address.</p>
            <div class="params">
                <div class="param">
                    <span class="param-name">tm_addr</span>: <span class="param-desc">Tendermint address of the validator (40 hex characters)</span>
                </div>
            </div>
            <div class="response">
                <h4>Success Response:</h4>
                <pre><code>{
    "address": "tnam1q..."
}</code></pre>
                <h4>Error Responses:</h4>
                <pre><code>{
    "error": "Invalid Tendermint address",
    "details": "Tendermint address must be 40 hex characters"
}</code></pre>
                <pre><code>{
    "error": "Not found",
    "details": "No validator found with Tendermint address CAFA..."
}</code></pre>
            </div>
        </div>

        <div class="endpoint">
            <h3>Get Validator Details</h3>
            <p><span class="method">GET</span> <span class="path">/api/pos/validators/{address}</span></p>
            <p>Get detailed information about a specific validator.</p>
            <div class="params">
                <div class="param">
                    <span class="param-name">address</span>: <span class="param-desc">Namada address of the validator</span>
                </div>
            </div>
            <div class="response">
                <h4>Success Response:</h4>
                <pre><code>{
    "address": "tnam1q...",
    "state": "active",
    "stake": "1000000",
    "commission_rate": "0.05",
    "max_commission_change_per_epoch": "0.01",
    "metadata": {
        "email": "validator@example.com",
        "description": "Professional validator",
        "website": "https://example.com",
        "discord_handle": "validator#1234"
    }
}</code></pre>
                <h4>Error Responses:</h4>
                <pre><code>{
    "error": "Invalid address format",
    "details": "Invalid address format: expected bech32m encoding"
}</code></pre>
                <pre><code>{
    "error": "Not found",
    "details": "Address tnam1q... is not a validator"
}</code></pre>
            </div>
        </div>

        <div class="endpoint">
            <h3>Get All Validators</h3>
            <p><span class="method">GET</span> <span class="path">/api/pos/validators</span></p>
            <p>Get a simple list of all validators. This endpoint returns just the addresses without additional details.</p>
            <div class="response">
                <h4>Response:</h4>
                <pre><code>{
    "validators": [
        "tnam1q...",
        "tnam1q..."
    ]
}</code></pre>
            </div>
        </div>

        <div class="endpoint">
            <h3>Get All Validators with Details</h3>
            <p><span class="method">GET</span> <span class="path">/api/pos/validators_details?page={page}&per_page={per_page}</span></p>
            <p>Get detailed information about all validators with pagination.</p>
            <div class="params">
                <div class="param">
                    <span class="param-name">page</span>: <span class="param-desc">Page number (default: 1, must be greater than 0)</span>
                </div>
                <div class="param">
                    <span class="param-name">per_page</span>: <span class="param-desc">Number of validators per page (default: 10, max: 50)</span>
                </div>
            </div>
            <div class="response">
                <h4>Success Response:</h4>
                <pre><code>{
    "validators": [
        {
            "address": "tnam1q...",
            "state": "active",
            "stake": "1000000",
            "commission_rate": "0.05",
            "max_commission_change_per_epoch": "0.01",
            "metadata": {
                "email": "validator@example.com",
                "description": "Professional validator",
                "website": "https://example.com",
                "discord_handle": "validator#1234"
            }
        }
    ],
    "pagination": {
        "total": 100,
        "page": 1,
        "per_page": 10,
        "total_pages": 10
    }
}</code></pre>
                <h4>Error Responses:</h4>
                <pre><code>{
    "error": "Invalid pagination parameters",
    "details": "Page number must be greater than 0"
}</code></pre>
                <pre><code>{
    "error": "Invalid pagination parameters",
    "details": "Items per page cannot exceed 50"
}</code></pre>
                <pre><code>{
    "error": "Invalid pagination parameters",
    "details": "Page number 11 exceeds total pages 10"
}</code></pre>
            </div>
        </div>

        <div class="endpoint">
            <h3>Get Consensus Validator Set</h3>
            <p><span class="method">GET</span> <span class="path">/api/pos/validator_set/consensus</span></p>
            <p>Get all validators in the consensus set with their bonded stake.</p>
            <div class="response">
                <h4>Response:</h4>
                <pre><code>{
    "validators": [
        {
            "address": "tnam1q...",
            "stake": "1000000"
        }
    ]
}</code></pre>
            </div>
        </div>

        <div class="endpoint">
            <h3>Get Below-Capacity Validator Set</h3>
            <p><span class="method">GET</span> <span class="path">/api/pos/validator_set/below_capacity</span></p>
            <p>Get all validators in the below-capacity set with their bonded stake.</p>
            <div class="response">
                <h4>Response:</h4>
                <pre><code>{
    "validators": [
        {
            "address": "tnam1q...",
            "stake": "1000000"
        }
    ]
}</code></pre>
            </div>
        </div>
    </body>
    </html>
    "#;
    
    Ok(warp::reply::html(html))
} 