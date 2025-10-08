use crate::errors::SniperError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub bus: Bus,
    pub execution: Execution,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bus {
    pub kind: String,
    pub nats_url: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Execution {
    pub modes: Vec<String>,
    pub fallback_chain: Vec<String>,
    pub replacement_ttl_secs: u64,
    pub rbf_retry_secs: u64,
}

impl AppConfig {
    pub fn load_default() -> Result<Self, SniperError> {
        let txt = std::fs::read_to_string("configs/app.toml")
            .map_err(|e| SniperError::Config(e.to_string()))?;
        toml::from_str(&txt).map_err(|e| SniperError::Config(e.to_string()))
    }
}
