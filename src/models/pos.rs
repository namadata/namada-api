use serde::Serialize;

#[derive(Serialize)]
pub struct LivenessInfoResponse {
    pub liveness_window_len: u64,
    pub liveness_threshold: String,
    pub validators: Vec<ValidatorLiveness>,
}

#[derive(Serialize)]
pub struct ValidatorLiveness {
    pub native_address: String,
    pub comet_address: String,
    pub missed_votes: u64,
}

#[derive(Serialize)]
pub struct ValidatorResponse {
    pub address: String,
}

#[derive(Serialize)]
pub struct ValidatorDetailsResponse {
    pub address: String,
    pub state: String,
    pub stake: String,
    pub commission_rate: String,
    pub max_commission_change_per_epoch: String,
    pub metadata: Option<ValidatorMetadata>,
}

#[derive(Serialize)]
pub struct ValidatorMetadata {
    pub email: String,
    pub description: Option<String>,
    pub website: Option<String>,
    pub discord_handle: Option<String>,
}

#[derive(Serialize)]
pub struct DelegationsResponse {
    pub delegations: Vec<Delegation>,
}

#[derive(Serialize)]
pub struct Delegation {
    pub validator: String,
    pub amount: String,
}

#[derive(Debug, Serialize)]
pub struct ValidatorSetResponse {
    pub validators: Vec<WeightedValidatorResponse>,
}

#[derive(Debug, Serialize)]
pub struct WeightedValidatorResponse {
    pub address: String,
    pub stake: String,
} 