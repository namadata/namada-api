use std::sync::Arc;
use crate::client::NamadaClient;

#[derive(Clone)]
pub struct AppState {
    pub namada_client: Arc<NamadaClient>,
}

impl AppState {
    pub async fn new(rpc_url: String) -> Result<Self, crate::client::ClientError> {
        let client = NamadaClient::new(rpc_url).await?;
        Ok(Self {
            namada_client: Arc::new(client),
        })
    }
} 