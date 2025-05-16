use std::sync::Arc;
use thiserror::Error;
use tracing::{debug, error, info};
use reqwest::Client;
use tendermint_rpc::{HttpClient, Url};
use namada_core::address::Address;
use namada_core::chain::Epoch;
use namada_proof_of_stake::types::{LivenessInfo, ValidatorMetaData, CommissionPair, ValidatorStateInfo};
use namada_sdk::rpc;
use std::str::FromStr;

#[derive(Error, Debug)]
pub enum ClientError {
    #[error("RPC connection error: {0}")]
    ConnectionError(String),
    #[error("RPC query error: {0}")]
    QueryError(String),
    #[error("Invalid RPC URL: {0}")]
    InvalidUrl(String),
}

pub struct NamadaClient {
    rpc_client: Client,
    rpc_url: String,
}

impl NamadaClient {
    pub fn new(rpc_url: String) -> Self {
        info!("Initializing Namada client with RPC URL: {}", rpc_url);
        Self {
            rpc_client: Client::new(),
            rpc_url,
        }
    }

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

    pub fn rpc_url(&self) -> &str {
        &self.rpc_url
    }

    pub async fn query_epoch(&self) -> Result<Epoch, ClientError> {
        rpc::query_epoch(&self.rpc_client).await
            .map_err(|e| ClientError::QueryError(e.to_string()))
    }

    pub async fn get_liveness_info(&self) -> Result<LivenessInfo, ClientError> {
        rpc::get_validators_liveness_info(&self.rpc_client).await
            .map_err(|e| ClientError::QueryError(e.to_string()))
    }

    pub async fn get_all_validators(&self, epoch: Option<Epoch>) -> Result<Vec<Address>, ClientError> {
        let epoch = match epoch {
            Some(e) => e,
            None => self.query_epoch().await?,
        };
        let set = rpc::get_all_validators(&self.rpc_client, epoch).await
            .map_err(|e| ClientError::QueryError(e.to_string()))?;
        Ok(set.into_iter().collect())
    }

    pub async fn is_validator(&self, address: &Address) -> Result<bool, ClientError> {
        rpc::is_validator(&self.rpc_client, address).await
            .map_err(|e| ClientError::QueryError(e.to_string()))
    }

    pub async fn get_validator_state(&self, address: &Address, epoch: Option<Epoch>) -> Result<ValidatorStateInfo, ClientError> {
        rpc::get_validator_state(&self.rpc_client, address, epoch).await
            .map_err(|e| ClientError::QueryError(e.to_string()))
    }

    pub async fn get_validator_stake(&self, epoch: Epoch, address: &Address) -> Result<namada_core::token::Amount, ClientError> {
        rpc::get_validator_stake(&self.rpc_client, epoch, address).await
            .map_err(|e| ClientError::QueryError(e.to_string()))
    }

    pub async fn query_metadata(&self, address: &Address, epoch: Option<Epoch>) -> Result<(Option<ValidatorMetaData>, CommissionPair), ClientError> {
        rpc::query_metadata(&self.rpc_client, address, epoch).await
            .map_err(|e| ClientError::QueryError(e.to_string()))
    }

    pub async fn get_delegation_validators(&self, address: &Address, epoch: Option<Epoch>) -> Result<Vec<Address>, ClientError> {
        let epoch = match epoch {
            Some(e) => e,
            None => self.query_epoch().await?,
        };
        let set = rpc::get_delegation_validators(&self.rpc_client, address, epoch).await
            .map_err(|e| ClientError::QueryError(e.to_string()))?;
        Ok(set.into_iter().collect())
    }
} 