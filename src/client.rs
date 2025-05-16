use std::sync::Arc;
use namada_sdk::rpc;
use namada_sdk::core::address::Address;
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
    pub async fn query_epoch(&self) -> Result<namada_sdk::core::chain::Epoch, ClientError> {
        rpc::query_epoch(self.client()).await
            .map_err(|e| ClientError::QueryError(e.to_string()))
    }
    
    /// Get validators liveness information
    pub async fn get_liveness_info(&self) -> Result<namada_sdk::proof_of_stake::types::LivenessInfo, ClientError> {
        rpc::get_validators_liveness_info(self.client()).await
            .map_err(|e| ClientError::QueryError(e.to_string()))
    }
    
    /// Find validator by Tendermint address
    pub async fn get_validator_by_tm_addr(&self, tm_addr: String) -> Result<Option<Address>, ClientError> {
        rpc::query_validator_by_tm_addr(self.client(), tm_addr).await
            .map_err(|e| ClientError::QueryError(e.to_string()))
    }
    
    /// Get validator state
    pub async fn get_validator_state(&self, address: &Address) -> Result<namada_sdk::proof_of_stake::types::ValidatorState, ClientError> {
        rpc::get_validator_state(self.client(), address, None).await
            .map_err(|e| ClientError::QueryError(e.to_string()))
    }
    
    /// Check if address is a validator
    pub async fn is_validator(&self, address: &Address) -> Result<bool, ClientError> {
        rpc::is_validator(self.client(), address).await
            .map_err(|e| ClientError::QueryError(e.to_string()))
    }
    
    /// Get validator stake
    pub async fn get_validator_stake(&self, epoch: namada_sdk::core::chain::Epoch, address: &Address) -> Result<Option<namada_sdk::core::token::Amount>, ClientError> {
        rpc::get_validator_stake(self.client(), epoch, address).await
            .map_err(|e| ClientError::QueryError(e.to_string()))
    }
    
    /// Get validator metadata and commission
    pub async fn query_metadata(&self, address: &Address) -> Result<(Option<namada_sdk::proof_of_stake::types::ValidatorMetaData>, namada_sdk::proof_of_stake::types::CommissionPair), ClientError> {
        rpc::query_metadata(self.client(), address, None).await
            .map_err(|e| ClientError::QueryError(e.to_string()))
    }
    
    /// Get all validators
    pub async fn get_all_validators(&self) -> Result<Vec<Address>, ClientError> {
        rpc::get_all_validators(self.client()).await
            .map_err(|e| ClientError::QueryError(e.to_string()))
    }
    
    /// Get native token
    pub async fn get_native_token(&self) -> Result<Address, ClientError> {
        rpc::get_native_token(self.client()).await
            .map_err(|e| ClientError::QueryError(e.to_string()))
    }
} 