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

use models::pos::*;
use models::error::{ApiError, handle_rejection};
use config::{CliArgs, Config};

/// Application state shared across all handlers
#[derive(Clone)]
struct AppState {
    namada_client: Arc<client::NamadaClient>,
}

#[derive(Debug, Deserialize)]
struct ValidatorsQuery {
    page: Option<u32>,
    per_page: Option<u32>,
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
        .and(warp::get())
        .and_then(health_check);
        
    let rpc_health = warp::path("api")
        .and(warp::path("health"))
        .and(warp::path("rpc"))
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
        .and(warp::path("validators"))
        .and(warp::path("tm"))
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
        
    let validators = warp::path("api")
        .and(warp::path("pos"))
        .and(warp::path("validators"))
        .and(warp::get())
        .and(warp::query::<ValidatorsQuery>())
        .and(with_state(state.clone()))
        .and_then(|query: ValidatorsQuery, state: Arc<AppState>| async move {
            get_validators(state, query.page, query.per_page).await
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
        .or(validators)
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
fn with_state(state: Arc<AppState>) -> impl Filter<Extract = (Arc<AppState>,), Error = Infallible> + Clone {
    warp::any().map(move || state.clone())
}

/// Basic health check endpoint
/// 
/// # Endpoint
/// `GET /api/health`
/// 
/// # Response
/// ```json
/// {
///     "status": "ok",
///     "version": "0.1.0"
/// }
/// ```
async fn health_check() -> Result<impl Reply, Rejection> {
    Ok(warp::reply::json(&serde_json::json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION")
    })))
}

/// RPC connection health check endpoint
/// 
/// # Endpoint
/// `GET /api/health/rpc`
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
async fn rpc_health_check(state: Arc<AppState>) -> Result<impl Reply, Rejection> {
    match state.namada_client.check_connection().await {
        Ok(_) => Ok(warp::reply::json(&serde_json::json!({
            "status": "ok",
            "rpc_url": state.namada_client.rpc_url()
        }))),
        Err(e) => {
            error!("RPC health check failed: {}", e);
            Ok(warp::reply::json(&serde_json::json!({
                "status": "error",
                "message": format!("RPC connection error: {}", e),
                "rpc_url": state.namada_client.rpc_url()
            })))
        }
    }
}

/// Get validator liveness information
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
async fn get_liveness_info(state: Arc<AppState>) -> Result<impl Reply, Rejection> {
    let liveness_info = state.namada_client.get_liveness_info().await
        .map_err(|e| warp::reject::custom(ApiError::QueryError(e.to_string())))?;
    
    Ok(warp::reply::json(&LivenessInfoResponse {
        liveness_window_len: liveness_info.liveness_window_len,
        liveness_threshold: liveness_info.liveness_threshold.to_string(),
        validators: liveness_info.validators.into_iter().map(|v| ValidatorLiveness {
            native_address: v.native_address.to_string(),
            comet_address: v.comet_address,
            missed_votes: v.missed_votes,
        }).collect(),
    }))
}

/// Get validator address by Tendermint address
/// 
/// # Endpoint
/// `GET /api/pos/validators/tm/{tm_addr}`
/// 
/// # Parameters
/// - `tm_addr`: Tendermint address of the validator
/// 
/// # Response
/// Success:
/// ```json
/// {
///     "address": "tnam1q..."
/// }
/// ```
/// 
/// Error:
/// ```json
/// {
///     "error": "Validator not found"
/// }
/// ```
async fn get_validator_by_tm_addr(
    state: Arc<AppState>,
    tm_addr: String,
) -> Result<impl Reply, Rejection> {
    let epoch = state.namada_client.query_epoch().await
        .map_err(|e| warp::reject::custom(ApiError::QueryError(e.to_string())))?;
    
    // Get all validators
    let validators = state.namada_client.get_all_validators(Some(epoch)).await
        .map_err(|e| warp::reject::custom(ApiError::QueryError(e.to_string())))?;
    
    // Get liveness info to match Tendermint addresses
    let liveness_info = state.namada_client.get_liveness_info().await
        .map_err(|e| warp::reject::custom(ApiError::QueryError(e.to_string())))?;
    
    // Find validator with matching Tendermint address
    let validator = liveness_info.validators.iter()
        .find(|v| v.comet_address == tm_addr)
        .ok_or_else(|| warp::reject::custom(ApiError::NotFound("Validator not found".to_string())))?;
    
    // Verify the validator is in the active set
    if !validators.contains(&validator.native_address) {
        return Err(warp::reject::custom(ApiError::NotFound("Validator not in active set".to_string())));
    }
    
    Ok(warp::reply::json(&ValidatorResponse {
        address: validator.native_address.to_string(),
    }))
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
    let address = Address::from_str(&address)
        .map_err(|e| warp::reject::custom(ApiError::BadRequest(format!("Invalid address format: {}", e))))?;
    
    let epoch = state.namada_client.query_epoch().await
        .map_err(|e| warp::reject::custom(ApiError::QueryError(e.to_string())))?;
    
    let is_validator = state.namada_client.is_validator(&address).await
        .map_err(|e| warp::reject::custom(ApiError::QueryError(e.to_string())))?;
    
    if !is_validator {
        return Err(warp::reject::custom(ApiError::NotFound("Address is not a validator".to_string())));
    }
    
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

/// Get list of all validators
/// 
/// # Endpoint
/// `GET /api/pos/validators?page={page}&per_page={per_page}`
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
async fn get_validators(
    state: Arc<AppState>,
    page: Option<u32>,
    per_page: Option<u32>,
) -> Result<impl Reply, Rejection> {
    // Set default values and validate parameters
    let page = page.unwrap_or(1);
    let per_page = per_page.unwrap_or(10).min(50); // Cap at 50 validators per page
    
    let epoch = state.namada_client.query_epoch().await
        .map_err(|e| warp::reject::custom(ApiError::QueryError(e.to_string())))?;
    
    let validators = state.namada_client.get_all_validators(Some(epoch)).await
        .map_err(|e| warp::reject::custom(ApiError::QueryError(e.to_string())))?;
    
    let total = validators.len();
    let total_pages = (total as f64 / per_page as f64).ceil() as u32;
    
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
            pre {
                background: #f8f9fa;
                padding: 15px;
                border-radius: 4px;
                overflow-x: auto;
            }
            code {
                font-family: "SFMono-Regular", Consolas, "Liberation Mono", Menlo, monospace;
            }
        </style>
    </head>
    <body>
        <h1>Namada API Documentation</h1>
        
        <h2>Health Endpoints</h2>
        
        <div class="endpoint">
            <h3>Basic Health Check</h3>
            <p><span class="method">GET</span> <span class="path">/api/health</span></p>
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
            <p><span class="method">GET</span> <span class="path">/api/health/rpc</span></p>
            <p>Check if the RPC connection is working.</p>
            <div class="response">
                <h4>Response:</h4>
                <pre><code>{
    "status": "ok",
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
            <p><span class="method">GET</span> <span class="path">/api/pos/validators/tm/{tm_addr}</span></p>
            <p>Get validator information by their Tendermint address.</p>
            <div class="response">
                <h4>Response:</h4>
                <pre><code>{
    "address": "tnam1q..."
}</code></pre>
            </div>
        </div>

        <div class="endpoint">
            <h3>Get Validator Details</h3>
            <p><span class="method">GET</span> <span class="path">/api/pos/validators/{address}</span></p>
            <p>Get detailed information about a specific validator.</p>
            <div class="response">
                <h4>Response:</h4>
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
            </div>
        </div>

        <div class="endpoint">
            <h3>Get All Validators</h3>
            <p><span class="method">GET</span> <span class="path">/api/pos/validators?page={page}&per_page={per_page}</span></p>
            <p>Get information about all validators.</p>
            <div class="response">
                <h4>Response:</h4>
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