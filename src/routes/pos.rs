use axum::{
    extract::{Path, State},
    Json,
    routing::get,
    Router,
    http::StatusCode,
    response::IntoResponse
};
use std::sync::Arc;
use serde_json::json;
use crate::state::AppState;
use crate::models::pos::*;
use namada_core::address::Address;
use std::str::FromStr;

pub fn router() -> Router<Arc<AppState>> {
    // Use a different pattern for handlers where we define them inline
    Router::new()
        .route("/api/pos/liveness_info", get(
            |State(state): State<Arc<AppState>>| async move {
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
                        (StatusCode::OK, Json(json!(response)))
                    },
                    Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Query error: {}", e)})))
                }
            }
        ))
        .route("/api/pos/validators/:address", get(
            |State(state): State<Arc<AppState>>, Path(address_str): Path<String>| async move {
                // Parse address
                let address = match Address::from_str(&address_str) {
                    Ok(addr) => addr,
                    Err(_) => return (StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid address format"})))
                };
                
                // Check if address is a validator
                match state.namada_client.is_validator(&address).await {
                    Ok(is_validator) => {
                        if !is_validator {
                            return (StatusCode::NOT_FOUND, Json(json!({"error": "Address is not a validator"})));
                        }
                        
                        // Get validator state
                        let validator_state_result = state.namada_client.get_validator_state(&address, None).await;
                        let (validator_state_opt, epoch) = match validator_state_result {
                            Ok(info) => info,
                            Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Query error: {}", e)})))
                        };
                        
                        // Get validator stake
                        let stake = match state.namada_client.get_validator_stake(epoch, &address).await {
                            Ok(stake) => stake,
                            Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Query error: {}", e)})))
                        };
                        
                        // Get validator metadata
                        let metadata_result = state.namada_client.query_metadata(&address, None).await;
                        let (metadata, commission) = match metadata_result {
                            Ok(data) => data,
                            Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Query error: {}", e)})))
                        };
                        
                        // Prepare response
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
                        (StatusCode::OK, Json(json!(response)))
                    },
                    Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Query error: {}", e)})))
                }
            }
        ))
        .route("/api/pos/validators", get(
            |State(state): State<Arc<AppState>>| async move {
                let epoch = state.namada_client.query_epoch().await.ok();
                
                match state.namada_client.get_all_validators(epoch).await {
                    Ok(validators) => {
                        let result: Vec<ValidatorResponse> = validators.into_iter().map(|addr| ValidatorResponse { 
                            address: addr.to_string() 
                        }).collect();
                        (StatusCode::OK, Json(json!(result)))
                    },
                    Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Query error: {}", e)})))
                }
            }
        ))
        .route("/api/pos/delegations/:address", get(
            |State(state): State<Arc<AppState>>, Path(address_str): Path<String>| async move {
                // Parse address
                let address = match Address::from_str(&address_str) {
                    Ok(addr) => addr,
                    Err(_) => return (StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid address format"})))
                };
                
                let epoch = state.namada_client.query_epoch().await.ok();
                
                match state.namada_client.get_delegation_validators(&address, epoch).await {
                    Ok(delegations) => {
                        let delegations_vec = delegations.into_iter().map(|validator| Delegation {
                            validator: validator.to_string(),
                            amount: "0".to_string(), // Since get_delegation_validators only returns validator addresses
                        }).collect();
                        (StatusCode::OK, Json(json!(DelegationsResponse { delegations: delegations_vec })))
                    },
                    Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Query error: {}", e)})))
                }
            }
        ))
} 