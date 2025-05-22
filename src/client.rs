use thiserror::Error;
use tracing::{error, info};
use tendermint_rpc::{HttpClient, Url};
use namada_core::address::Address;
use namada_core::chain::Epoch;
use namada_proof_of_stake::types::{LivenessInfo, ValidatorMetaData, CommissionPair, ValidatorStateInfo};
use namada_sdk::rpc;
use namada_sdk::queries::RPC;
use std::str::FromStr;
use tokio::task::spawn_blocking;

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
    rpc_client: HttpClient,
    rpc_url: String,
}

impl NamadaClient {
    pub async fn new(rpc_url: String) -> Result<Self, ClientError> {
        info!("Initializing Namada client with RPC URL: {}", rpc_url);
        let url = Url::from_str(&rpc_url)
            .map_err(|e| ClientError::InvalidUrl(e.to_string()))?;
        let rpc_client = HttpClient::new(url)
            .map_err(|e| ClientError::ConnectionError(e.to_string()))?;
        Ok(Self { rpc_client, rpc_url })
    }

    pub async fn check_connection(&self) -> Result<(), ClientError> {
        // Try a simple health check (e.g., get epoch)
        self.query_epoch().await.map(|_| ())
    }

    pub fn rpc_url(&self) -> &str {
        &self.rpc_url
    }

    pub async fn query_epoch(&self) -> Result<Epoch, ClientError> {
        let client = self.rpc_client.clone();
        spawn_blocking(move || {
            tokio::runtime::Handle::current().block_on(async {
                rpc::query_epoch(&client).await
            })
        })
        .await
        .map_err(|e| ClientError::QueryError(e.to_string()))?
        .map_err(|e| ClientError::QueryError(e.to_string()))
    }

    pub async fn get_liveness_info(&self) -> Result<LivenessInfo, ClientError> {
        let client = self.rpc_client.clone();
        spawn_blocking(move || {
            tokio::runtime::Handle::current().block_on(async {
                rpc::get_validators_liveness_info(&client).await
            })
        })
        .await
        .map_err(|e| ClientError::QueryError(e.to_string()))?
        .map_err(|e| ClientError::QueryError(e.to_string()))
    }

    pub async fn get_all_validators(&self, epoch: Option<Epoch>) -> Result<Vec<Address>, ClientError> {
        let epoch = match epoch {
            Some(e) => e,
            None => self.query_epoch().await?,
        };
        let client = self.rpc_client.clone();
        spawn_blocking(move || {
            tokio::runtime::Handle::current().block_on(async {
                rpc::get_all_validators(&client, epoch).await
            })
        })
        .await
        .map_err(|e| ClientError::QueryError(e.to_string()))?
        .map_err(|e| ClientError::QueryError(e.to_string()))
        .map(|set| set.into_iter().collect())
    }

    pub async fn is_validator(&self, address: &Address) -> Result<bool, ClientError> {
        let client = self.rpc_client.clone();
        let address = address.clone();
        spawn_blocking(move || {
            tokio::runtime::Handle::current().block_on(async {
                rpc::is_validator(&client, &address).await
            })
        })
        .await
        .map_err(|e| ClientError::QueryError(e.to_string()))?
        .map_err(|e| ClientError::QueryError(e.to_string()))
    }

    pub async fn get_validator_state(&self, address: &Address, epoch: Option<Epoch>) -> Result<ValidatorStateInfo, ClientError> {
        let client = self.rpc_client.clone();
        let address = address.clone();
        spawn_blocking(move || {
            tokio::runtime::Handle::current().block_on(async {
                rpc::get_validator_state(&client, &address, epoch).await
            })
        })
        .await
        .map_err(|e| ClientError::QueryError(e.to_string()))?
        .map_err(|e| ClientError::QueryError(e.to_string()))
    }

    pub async fn get_validator_stake(&self, epoch: Epoch, address: &Address) -> Result<namada_core::token::Amount, ClientError> {
        let client = self.rpc_client.clone();
        let address = address.clone();
        spawn_blocking(move || {
            tokio::runtime::Handle::current().block_on(async {
                rpc::get_validator_stake(&client, epoch, &address).await
            })
        })
        .await
        .map_err(|e| ClientError::QueryError(e.to_string()))?
        .map_err(|e| ClientError::QueryError(e.to_string()))
    }

    pub async fn query_metadata(&self, address: &Address, epoch: Option<Epoch>) -> Result<(Option<ValidatorMetaData>, CommissionPair), ClientError> {
        let client = self.rpc_client.clone();
        let address = address.clone();
        spawn_blocking(move || {
            tokio::runtime::Handle::current().block_on(async {
                rpc::query_metadata(&client, &address, epoch).await
            })
        })
        .await
        .map_err(|e| ClientError::QueryError(e.to_string()))?
        .map_err(|e| ClientError::QueryError(e.to_string()))
    }

    pub async fn get_delegation_validators(&self, address: &Address, epoch: Option<Epoch>) -> Result<Vec<Address>, ClientError> {
        let epoch = match epoch {
            Some(e) => e,
            None => self.query_epoch().await?,
        };
        let client = self.rpc_client.clone();
        let address = address.clone();
        spawn_blocking(move || {
            tokio::runtime::Handle::current().block_on(async {
                rpc::get_delegation_validators(&client, &address, epoch).await
            })
        })
        .await
        .map_err(|e| ClientError::QueryError(e.to_string()))?
        .map_err(|e| ClientError::QueryError(e.to_string()))
        .map(|set| set.into_iter().collect())
    }

    pub async fn get_consensus_validator_set(&self, epoch: Option<Epoch>) -> Result<Vec<namada_proof_of_stake::types::WeightedValidator>, ClientError> {
        let epoch = match epoch {
            Some(e) => e,
            None => self.query_epoch().await?,
        };
        let client = self.rpc_client.clone();
        spawn_blocking(move || {
            tokio::runtime::Handle::current().block_on(async {
                RPC.vp().pos().consensus_validator_set(&client, &Some(epoch)).await
            })
        })
        .await
        .map_err(|e| ClientError::QueryError(e.to_string()))?
        .map_err(|e| ClientError::QueryError(e.to_string()))
        .map(|set| set.into_iter().collect())
    }

    pub async fn get_below_capacity_validator_set(&self, epoch: Option<Epoch>) -> Result<Vec<namada_proof_of_stake::types::WeightedValidator>, ClientError> {
        let epoch = match epoch {
            Some(e) => e,
            None => self.query_epoch().await?,
        };
        let client = self.rpc_client.clone();
        spawn_blocking(move || {
            tokio::runtime::Handle::current().block_on(async {
                RPC.vp().pos().below_capacity_validator_set(&client, &Some(epoch)).await
            })
        })
        .await
        .map_err(|e| ClientError::QueryError(e.to_string()))?
        .map_err(|e| ClientError::QueryError(e.to_string()))
        .map(|set| set.into_iter().collect())
    }

    pub async fn validator_by_tm_addr(&self, tm_addr: String) -> Result<Option<Address>, ClientError> {
        let client = self.rpc_client.clone();
        spawn_blocking(move || {
            tokio::runtime::Handle::current().block_on(async {
                RPC.vp().pos().validator_by_tm_addr(&client, &tm_addr).await
            })
        })
        .await
        .map_err(|e| ClientError::QueryError(e.to_string()))?
        .map_err(|e| ClientError::QueryError(e.to_string()))
    }
} 