use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{error, info};

/// Blockchain provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub name: String,
    pub chain_id: u64,
    pub rpc_url: String,
    pub ws_url: Option<String>,
    pub priority: u32,
    pub enabled: bool,
}

/// Blockchain provider for RPC interactions
#[derive(Debug, Clone)]
pub struct BlockchainProvider {
    config: ProviderConfig,
    client: reqwest::Client,
}

impl BlockchainProvider {
    /// Create a new blockchain provider
    pub fn new(config: ProviderConfig) -> Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()?;

        Ok(Self { config, client })
    }

    /// Get provider name
    pub fn name(&self) -> &str {
        &self.config.name
    }

    /// Get chain ID
    pub fn chain_id(&self) -> u64 {
        self.config.chain_id
    }

    /// Check if provider is enabled
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    /// Get RPC URL
    pub fn rpc_url(&self) -> &str {
        &self.config.rpc_url
    }

    /// Send RPC request
    pub async fn send_rpc_request(
        &self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value> {
        if !self.config.enabled {
            return Err(anyhow::anyhow!("Provider {} is disabled", self.config.name));
        }

        info!("Sending RPC request to {}: {}", self.config.name, method);

        let request_body = serde_json::json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
            "id": 1
        });

        let response = self
            .client
            .post(&self.config.rpc_url)
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;

        let response_json: serde_json::Value = response.json().await?;

        if let Some(error) = response_json.get("error") {
            error!("RPC error from {}: {}", self.config.name, error);
            return Err(anyhow::anyhow!("RPC error: {}", error));
        }

        Ok(response_json
            .get("result")
            .cloned()
            .unwrap_or(serde_json::Value::Null))
    }

    /// Get latest block number
    pub async fn get_block_number(&self) -> Result<u64> {
        let result = self
            .send_rpc_request("eth_blockNumber", serde_json::Value::Array(vec![]))
            .await?;
        let block_hex = result.as_str().unwrap_or("0x0");
        let block_number = u64::from_str_radix(block_hex.trim_start_matches("0x"), 16)
            .map_err(|e| anyhow::anyhow!("Failed to parse block number: {}", e))?;
        Ok(block_number)
    }

    /// Get gas price
    pub async fn get_gas_price(&self) -> Result<u64> {
        let result = self
            .send_rpc_request("eth_gasPrice", serde_json::Value::Array(vec![]))
            .await?;
        let gas_price_hex = result.as_str().unwrap_or("0x0");
        let gas_price = u64::from_str_radix(gas_price_hex.trim_start_matches("0x"), 16)
            .map_err(|e| anyhow::anyhow!("Failed to parse gas price: {}", e))?;
        Ok(gas_price)
    }
}

/// Provider manager for handling multiple blockchain providers
#[derive(Debug)]
pub struct ProviderManager {
    providers: HashMap<u64, Vec<BlockchainProvider>>,
}

impl ProviderManager {
    /// Create a new provider manager
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
        }
    }

    /// Add a provider
    pub fn add_provider(&mut self, provider: BlockchainProvider) {
        let chain_id = provider.chain_id();
        self.providers.entry(chain_id).or_default().push(provider);
    }

    /// Get providers for a chain
    pub fn get_providers(&self, chain_id: u64) -> Option<&Vec<BlockchainProvider>> {
        self.providers.get(&chain_id)
    }

    /// Get the best provider for a chain based on priority
    pub fn get_best_provider(&self, chain_id: u64) -> Option<&BlockchainProvider> {
        if let Some(providers) = self.providers.get(&chain_id) {
            providers
                .iter()
                .filter(|p| p.is_enabled())
                .min_by_key(|p| p.config.priority)
        } else {
            None
        }
    }

    /// Get all enabled providers for a chain
    pub fn get_enabled_providers(&self, chain_id: u64) -> Vec<&BlockchainProvider> {
        if let Some(providers) = self.providers.get(&chain_id) {
            providers.iter().filter(|p| p.is_enabled()).collect()
        } else {
            Vec::new()
        }
    }
}

impl Default for ProviderManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_config() {
        let config = ProviderConfig {
            name: "ethereum".to_string(),
            chain_id: 1,
            rpc_url: "https://ethereum.rpc".to_string(),
            ws_url: Some("wss://ethereum.ws".to_string()),
            priority: 1,
            enabled: true,
        };

        assert_eq!(config.name, "ethereum");
        assert_eq!(config.chain_id, 1);
        assert_eq!(config.rpc_url, "https://ethereum.rpc");
        assert_eq!(config.ws_url, Some("wss://ethereum.ws".to_string()));
        assert_eq!(config.priority, 1);
        assert!(config.enabled);
    }

    #[test]
    fn test_provider_manager() {
        let mut manager = ProviderManager::new();
        assert!(manager.get_providers(1).is_none());
        assert!(manager.get_best_provider(1).is_none());

        let config = ProviderConfig {
            name: "ethereum".to_string(),
            chain_id: 1,
            rpc_url: "https://ethereum.rpc".to_string(),
            ws_url: None,
            priority: 1,
            enabled: true,
        };

        let provider = BlockchainProvider::new(config).unwrap();
        manager.add_provider(provider);

        assert!(manager.get_providers(1).is_some());
        assert!(manager.get_best_provider(1).is_some());
    }

    #[tokio::test]
    async fn test_blockchain_provider_creation() {
        let config = ProviderConfig {
            name: "test".to_string(),
            chain_id: 1,
            rpc_url: "https://test.rpc".to_string(),
            ws_url: None,
            priority: 1,
            enabled: true,
        };

        let provider = BlockchainProvider::new(config);
        assert!(provider.is_ok());

        let provider = provider.unwrap();
        assert_eq!(provider.name(), "test");
        assert_eq!(provider.chain_id(), 1);
        assert_eq!(provider.rpc_url(), "https://test.rpc");
        assert!(provider.is_enabled());
    }
}
