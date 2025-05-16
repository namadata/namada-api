use axum::{
    Router,
    routing::get,
    extract::{State, Path},
    Json,
    response::IntoResponse,
    http::StatusCode,
};
use std::sync::Arc;
use std::str::FromStr;
use namada_core::address::Address;
use namada_core::chain::Epoch;
use crate::state::AppState;
use crate::models::pos::{
    LivenessInfoResponse, 
    ValidatorLiveness, 
    ValidatorDetailsResponse, 
    ValidatorMetadata,
    ValidatorResponse
};
use crate::models::error::ApiError;
use tracing::{info, error};

/// Get validators liveness information
async fn get_validators_liveness(
    State(state): State<Arc<AppState>>
) -> Result<Json<LivenessInfoResponse>, ApiError> {
    info!("Fetching validators liveness information");
    
    let liveness_info = state.namada_client.get_liveness_info().await
        .map_err(|e| {
            error!("Failed to get liveness info: {}", e);
            ApiError::InternalServerError(format!("Failed to get liveness info: {}", e))
        })?;
    
    let liveness_window_len = liveness_info.window_len;
    let liveness_threshold = liveness_info.threshold.to_string();
    
    let validators = liveness_info.validator_liveness
        .into_iter()
        .map(|(validator_addr, comet_addr, missed_votes)| ValidatorLiveness {
            native_address: validator_addr.to_string(),
            comet_address: comet_addr.to_string(),
            missed_votes,
        })
        .collect();
    
    Ok(Json(LivenessInfoResponse {
        liveness_window_len,
        liveness_threshold,
        validators,
    }))
}

/// Get validator details by address
async fn get_validator_details(
    State(state): State<Arc<AppState>>,
    Path(address): Path<String>,
) -> Result<Json<ValidatorDetailsResponse>, ApiError> {
    info!("Fetching validator details for address: {}", address);
    
    // Parse address
    let validator_address = Address::from_str(&address)
        .map_err(|_| ApiError::BadRequest(format!("Invalid validator address: {}", address)))?;
    
    // Check if the address is a validator
    let is_validator = state.namada_client.is_validator(&validator_address).await
        .map_err(|e| {
            error!("Failed to check if address is validator: {}", e);
            ApiError::InternalServerError(format!("Failed to check validator status: {}", e))
        })?;
    
    if !is_validator {
        return Err(ApiError::NotFound(format!("Address {} is not a validator", address)));
    }
    
    // Get current epoch
    let epoch = state.namada_client.query_epoch().await
        .map_err(|e| {
            error!("Failed to query epoch: {}", e);
            ApiError::InternalServerError(format!("Failed to query epoch: {}", e))
        })?;
    
    // Get validator state
    let validator_state = state.namada_client.get_validator_state(&validator_address, Some(epoch)).await
        .map_err(|e| {
            error!("Failed to get validator state: {}", e);
            ApiError::InternalServerError(format!("Failed to get validator state: {}", e))
        })?;
    
    // Get validator stake
    let stake = state.namada_client.get_validator_stake(epoch, &validator_address).await
        .map_err(|e| {
            error!("Failed to get validator stake: {}", e);
            ApiError::InternalServerError(format!("Failed to get validator stake: {}", e))
        })?;
    
    // Get validator metadata and commission
    let (metadata, commission) = state.namada_client.query_metadata(&validator_address, Some(epoch)).await
        .map_err(|e| {
            error!("Failed to query validator metadata: {}", e);
            ApiError::InternalServerError(format!("Failed to query validator metadata: {}", e))
        })?;
    
    // Format the response
    let metadata_response = metadata.map(|m| ValidatorMetadata {
        email: m.email,
        description: m.description,
        website: m.website,
        discord_handle: m.discord_handle,
    });
    
    Ok(Json(ValidatorDetailsResponse {
        address: validator_address.to_string(),
        state: format!("{:?}", validator_state),
        stake: stake.to_string(),
        commission_rate: commission.commission.to_string(),
        max_commission_change_per_epoch: commission.max_commission_change_per_epoch.to_string(),
        metadata: metadata_response,
    }))
}

/// Find validator by Tendermint address
async fn find_validator_by_tm_address(
    State(state): State<Arc<AppState>>,
    Path(tm_address): Path<String>,
) -> Result<Json<ValidatorResponse>, ApiError> {
    info!("Finding validator by Tendermint address: {}", tm_address);
    
    // Get liveness info which contains the mapping of validators to Tendermint addresses
    let liveness_info = state.namada_client.get_liveness_info().await
        .map_err(|e| {
            error!("Failed to get liveness info: {}", e);
            ApiError::InternalServerError(format!("Failed to get liveness info: {}", e))
        })?;
    
    // Find the validator with the matching Tendermint address
    let validator_addr = liveness_info.validator_liveness
        .into_iter()
        .find_map(|(validator_addr, comet_addr, _)| {
            if comet_addr.to_string() == tm_address {
                Some(validator_addr)
            } else {
                None
            }
        });
    
    match validator_addr {
        Some(addr) => Ok(Json(ValidatorResponse {
            address: addr.to_string(),
        })),
        None => Err(ApiError::NotFound(format!("No validator found with Tendermint address: {}", tm_address))),
    }
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/pos/liveness_info", get(get_validators_liveness))
        .route("/api/pos/validators/:address", get(get_validator_details))
        .route("/api/pos/validators/tm/:tm_address", get(find_validator_by_tm_address))
} 