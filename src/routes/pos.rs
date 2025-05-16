use axum::{
    extract::{Path, State},
    Json,
};
use std::sync::Arc;
use std::str::FromStr;
use namada_sdk::core::address::Address;
use namada_sdk::rpc;

use crate::AppState;
use crate::models::{ApiError, LivenessInfoResponse, ValidatorLiveness, ValidatorResponse, ValidatorDetailsResponse, ValidatorMetadata, ValidatorsResponse};

pub async fn get_liveness_info(
    State(state): State<Arc<AppState>>,
) -> Result<Json<LivenessInfoResponse>, ApiError> {
    match state.namada_client.get_liveness_info().await {
        Ok(liveness_info) => {
            let response = LivenessInfoResponse {
                liveness_window_len: liveness_info.liveness_window_len,
                liveness_threshold: liveness_info.liveness_threshold.to_string(),
                validators: liveness_info.validators.into_iter().map(|v| ValidatorLiveness {
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

pub async fn get_validator_by_tm_addr(
    State(state): State<Arc<AppState>>,
    Path(tm_addr): Path<String>,
) -> Result<Json<ValidatorResponse>, ApiError> {
    // Sanitize input to prevent issues
    if tm_addr.contains('/') || tm_addr.is_empty() {
        return Err(ApiError::BadRequest("Invalid Tendermint address format".to_string()));
    }

    match state.namada_client.get_validator_by_tm_addr(tm_addr).await {
        Ok(Some(validator_addr)) => {
            let response = ValidatorResponse {
                address: validator_addr.to_string(),
            };
            Ok(Json(response))
        },
        Ok(None) => Err(ApiError::NotFound("Validator not found".to_string())),
        Err(e) => Err(ApiError::QueryError(e.to_string())),
    }
}

pub async fn get_validator_details(
    State(state): State<Arc<AppState>>,
    Path(address_str): Path<String>,
) -> Result<Json<ValidatorDetailsResponse>, ApiError> {
    // Parse address
    let address = match Address::from_str(&address_str) {
        Ok(addr) => addr,
        Err(_) => return Err(ApiError::BadRequest("Invalid address format".to_string())),
    };

    // Check if address is a validator
    let is_validator = match state.namada_client.is_validator(&address).await {
        Ok(is_val) => is_val,
        Err(e) => return Err(ApiError::QueryError(e.to_string())),
    };

    if !is_validator {
        return Err(ApiError::NotFound("Address is not a validator".to_string()));
    }

    // Get validator state
    let validator_state = match state.namada_client.get_validator_state(&address).await {
        Ok(state) => state,
        Err(e) => return Err(ApiError::QueryError(e.to_string())),
    };

    // Get stake
    let stake = match state.namada_client.get_validator_stake(validator_state.epoch, &address).await {
        Ok(stake) => stake.unwrap_or_default(),
        Err(e) => return Err(ApiError::QueryError(e.to_string())),
    };

    // Get metadata and commission
    let (metadata, commission) = match state.namada_client.query_metadata(&address).await {
        Ok(result) => result,
        Err(e) => return Err(ApiError::QueryError(e.to_string())),
    };

    // Construct response
    let response = ValidatorDetailsResponse {
        address: address.to_string(),
        state: validator_state.state.to_string(),
        stake: stake.to_string(),
        commission_rate: commission.commission_rate.to_string(),
        max_commission_change_per_epoch: commission.max_commission_change_per_epoch.to_string(),
        metadata: metadata.map(|m| ValidatorMetadata {
            email: m.email,
            description: m.description,
            website: m.website,
            discord_handle: m.discord_handle,
        }),
    };

    Ok(Json(response))
}

pub async fn get_validators(
    State(state): State<Arc<AppState>>,
) -> Result<Json<ValidatorsResponse>, ApiError> {
    match state.namada_client.get_all_validators().await {
        Ok(validators) => {
            let response = ValidatorsResponse {
                validators: validators.into_iter().map(|v| v.to_string()).collect(),
            };
            Ok(Json(response))
        },
        Err(e) => Err(ApiError::QueryError(e.to_string())),
    }
} 