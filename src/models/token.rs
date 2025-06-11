use serde::{Deserialize, Serialize};

/// Response for token balance query
#[derive(Debug, Serialize, Deserialize)]
pub struct TokenBalanceResponse {
    /// Token address
    pub token: String,
    /// Owner address
    pub owner: String, 
    /// Token balance amount
    pub balance: String,
    /// Block height at which the balance was queried (optional)
    pub height: Option<u64>,
}

/// Request parameters for token balance query
#[derive(Debug, Deserialize)]
pub struct TokenBalanceQuery {
    /// Token address
    pub token: String,
    /// Owner address
    pub owner: String,
    /// Optional block height
    pub height: Option<u64>,
}

impl TokenBalanceQuery {
    /// Validate the query parameters
    pub fn validate(&self) -> Result<(), crate::models::error::ApiError> {
        use std::str::FromStr;
        use namada_core::address::Address;

        // Validate token address format
        Address::from_str(&self.token)
            .map_err(|e| crate::models::error::ApiError::InvalidAddress(
                format!("Invalid token address format: {}", e)
            ))?;

        // Validate owner address format
        Address::from_str(&self.owner)
            .map_err(|e| crate::models::error::ApiError::InvalidAddress(
                format!("Invalid owner address format: {}", e)
            ))?;

        Ok(())
    }
}

/// Response for token total supply query
#[derive(Debug, Serialize, Deserialize)]
pub struct TokenTotalSupplyResponse {
    /// Token address
    pub token: String,
    /// Total supply amount
    pub total_supply: String,
}

/// Response for native token query
#[derive(Debug, Serialize, Deserialize)]
pub struct NativeTokenResponse {
    /// Native token address
    pub address: String,
} 