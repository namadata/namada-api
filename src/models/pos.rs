use serde::Serialize;

#[derive(Serialize)]
pub struct LivenessInfoResponse {
    pub liveness_window_len: u64,
    pub liveness_threshold: String, // Using String for Dec
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
    pub email: Option<String>,
    pub description: Option<String>,
    pub website: Option<String>,
    pub discord_handle: Option<String>,
}

#[derive(Serialize)]
pub struct ValidatorsResponse {
    pub validators: Vec<String>,
} 