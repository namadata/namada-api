use axum::{extract::{Path, State}, Json, routing::get, Router, response::{IntoResponse, Response}, http::StatusCode};
use std::sync::Arc;
use crate::state::AppState;
use crate::models::pos::*;
use crate::models::error::ApiError;
use namada_core::address::Address;
use std::str::FromStr;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/pos/liveness_info", get(get_liveness_info))
        .route("/api/pos/validators/:address", get(get_validator_details))
        .route("/api/pos/validators", get(get_validators))
        .route("/api/pos/delegations/:address", get(get_delegations))
}

pub async fn get_liveness_info(
    State(state): State<Arc<AppState>>,
) -> Response {
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
            Json(response).into_response()
        },
        Err(e) => ApiError::QueryError(e.to_string()).into_response()
    }
}

pub async fn get_validator_details(
    State(state): State<Arc<AppState>>,
    Path(address_str): Path<String>,
) -> Response {
    let address = match Address::from_str(&address_str) {
        Ok(addr) => addr,
        Err(_) => return ApiError::BadRequest("Invalid address format".to_string()).into_response()
    };
    
    match state.namada_client.is_validator(&address).await {
        Ok(is_validator) => {
            if !is_validator {
                return ApiError::NotFound("Address is not a validator".to_string()).into_response();
            }
            
            let validator_state_result = state.namada_client.get_validator_state(&address, None).await;
            let (validator_state_opt, epoch) = match validator_state_result {
                Ok(info) => info,
                Err(e) => return ApiError::QueryError(e.to_string()).into_response()
            };
            
            let stake = match state.namada_client.get_validator_stake(epoch, &address).await {
                Ok(stake) => stake,
                Err(e) => return ApiError::QueryError(e.to_string()).into_response()
            };
            
            let metadata_result = state.namada_client.query_metadata(&address, None).await;
            let (metadata, commission) = match metadata_result {
                Ok(data) => data,
                Err(e) => return ApiError::QueryError(e.to_string()).into_response()
            };
            
            let response = ValidatorDetailsResponse {
                address: address.to_string(),
                state: validator_state_opt
                    .map(|s| format!("{:?}", s))
                    .unwrap_or_else(|| "Unknown".to_string()),
                stake: stake.to_string(),
                commission_rate: commission.commission_rate.as_ref().map(|d| d.to_string()).unwrap_or_default(),
                max_commission_change_per_epoch: commission.max_commission_change_per_epoch.as_ref().map(|d| d.to_string()).unwrap_or_default(),
                metadata: metadata.map(|m| ValidatorMetadata {
                    email: m.email,
                    description: m.description,
                    website: m.website,
                    discord_handle: m.discord_handle,
                }),
            };
            Json(response).into_response()
        },
        Err(e) => ApiError::QueryError(e.to_string()).into_response()
    }
}

pub async fn get_validators(
    State(state): State<Arc<AppState>>,
) -> Response {
    let epoch = state.namada_client.query_epoch().await.ok();
    
    match state.namada_client.get_all_validators(epoch).await {
        Ok(validators) => {
            let result: Vec<ValidatorResponse> = validators.into_iter().map(|addr| ValidatorResponse { 
                address: addr.to_string() 
            }).collect();
            Json(result).into_response()
        },
        Err(e) => ApiError::QueryError(e.to_string()).into_response()
    }
}

pub async fn get_delegations(
    State(state): State<Arc<AppState>>,
    Path(address_str): Path<String>,
) -> Response {
    let address = match Address::from_str(&address_str) {
        Ok(addr) => addr,
        Err(_) => return ApiError::BadRequest("Invalid address format".to_string()).into_response()
    };
    
    let epoch = state.namada_client.query_epoch().await.ok();
    
    match state.namada_client.get_delegation_validators(&address, epoch).await {
        Ok(delegations) => {
            let delegations_vec = delegations.into_iter().map(|validator| Delegation {
                validator: validator.to_string(),
                amount: "0".to_string(), // Since get_delegation_validators only returns validator addresses
            }).collect();
            Json(DelegationsResponse { delegations: delegations_vec }).into_response()
        },
        Err(e) => ApiError::QueryError(e.to_string()).into_response()
    }
} 