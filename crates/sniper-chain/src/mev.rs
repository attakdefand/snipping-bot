use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{info, warn};

/// MEV bundle configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MevBundleConfig {
    pub relay_url: String,
    pub auth_key: Option<String>,
    pub enabled: bool,
    pub min_profit_wei: u128,
    pub max_gas_price: u64,
}

/// MEV bundle submission request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MevBundleRequest {
    pub txs: Vec<String>, // RLP encoded transactions
    pub block_number: u64,
    pub min_timestamp: Option<u64>,
    pub max_timestamp: Option<u64>,
    pub reverting_tx_hashes: Vec<String>,
}

/// MEV bundle response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MevBundleResponse {
    pub bundle_hash: String,
    pub success: bool,
    pub error: Option<String>,
}

/// MEV searcher for submitting bundles to relays
#[derive(Debug, Clone)]
pub struct MevSearcher {
    config: MevBundleConfig,
    client: reqwest::Client,
}

impl MevSearcher {
    /// Create a new MEV searcher
    pub fn new(config: MevBundleConfig) -> Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()?;

        Ok(Self { config, client })
    }

    /// Check if MEV functionality is enabled
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    /// Get relay URL
    pub fn relay_url(&self) -> &str {
        &self.config.relay_url
    }

    /// Submit a bundle to the MEV relay
    pub async fn submit_bundle(&self, bundle: MevBundleRequest) -> Result<MevBundleResponse> {
        if !self.config.enabled {
            return Err(anyhow::anyhow!("MEV functionality is disabled"));
        }

        info!(
            "Submitting MEV bundle with {} transactions to block {}",
            bundle.txs.len(),
            bundle.block_number
        );

        let url = format!("{}/bundle", self.config.relay_url);

        let mut request_builder = self
            .client
            .post(&url)
            .header("Content-Type", "application/json");

        // Add authentication if provided
        if let Some(auth_key) = &self.config.auth_key {
            request_builder =
                request_builder.header("Authorization", format!("Bearer {}", auth_key));
        }

        let response = request_builder.json(&bundle).send().await?;

        let status = response.status();
        let response_text = response.text().await?;

        if !status.is_success() {
            warn!(
                "MEV bundle submission failed with status {}: {}",
                status, response_text
            );
            return Ok(MevBundleResponse {
                bundle_hash: String::new(),
                success: false,
                error: Some(format!("HTTP {}: {}", status, response_text)),
            });
        }

        let response_json: serde_json::Value = serde_json::from_str(&response_text)
            .map_err(|e| anyhow::anyhow!("Failed to parse response: {}", e))?;

        let bundle_hash = response_json
            .get("bundleHash")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        Ok(MevBundleResponse {
            bundle_hash,
            success: true,
            error: None,
        })
    }

    /// Simulate a bundle before submission
    pub async fn simulate_bundle(&self, bundle: MevBundleRequest) -> Result<serde_json::Value> {
        if !self.config.enabled {
            return Err(anyhow::anyhow!("MEV functionality is disabled"));
        }

        info!(
            "Simulating MEV bundle with {} transactions",
            bundle.txs.len()
        );

        let url = format!("{}/simulate", self.config.relay_url);

        let mut request_builder = self
            .client
            .post(&url)
            .header("Content-Type", "application/json");

        // Add authentication if provided
        if let Some(auth_key) = &self.config.auth_key {
            request_builder =
                request_builder.header("Authorization", format!("Bearer {}", auth_key));
        }

        let response = request_builder.json(&bundle).send().await?;

        let response_text = response.text().await?;
        let response_json: serde_json::Value = serde_json::from_str(&response_text)
            .map_err(|e| anyhow::anyhow!("Failed to parse simulation response: {}", e))?;

        Ok(response_json)
    }

    /// Check if a bundle is likely to be profitable
    pub fn is_profitable(&self, expected_profit: u128) -> bool {
        expected_profit >= self.config.min_profit_wei
    }

    /// Check if gas price is within acceptable limits
    pub fn is_gas_price_acceptable(&self, gas_price: u64) -> bool {
        gas_price <= self.config.max_gas_price
    }
}

/// MEV relay information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MevRelayInfo {
    pub name: String,
    pub url: String,
    pub is_active: bool,
    pub last_seen: u64, // timestamp
}

/// MEV relay manager for handling multiple relays
#[derive(Debug)]
pub struct MevRelayManager {
    relays: HashMap<String, MevRelayInfo>,
}

impl MevRelayManager {
    /// Create a new relay manager
    pub fn new() -> Self {
        Self {
            relays: HashMap::new(),
        }
    }

    /// Add a relay
    pub fn add_relay(&mut self, relay: MevRelayInfo) {
        self.relays.insert(relay.name.clone(), relay);
    }

    /// Get relay by name
    pub fn get_relay(&self, name: &str) -> Option<&MevRelayInfo> {
        self.relays.get(name)
    }

    /// Get all active relays
    pub fn get_active_relays(&self) -> Vec<&MevRelayInfo> {
        self.relays.values().filter(|r| r.is_active).collect()
    }

    /// Remove a relay
    pub fn remove_relay(&mut self, name: &str) -> Option<MevRelayInfo> {
        self.relays.remove(name)
    }
}

impl Default for MevRelayManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mev_config() {
        let config = MevBundleConfig {
            relay_url: "https://relay.example.com".to_string(),
            auth_key: Some("test-key".to_string()),
            enabled: true,
            min_profit_wei: 1000000000000000, // 0.001 ETH
            max_gas_price: 100,               // 100 gwei
        };

        assert_eq!(config.relay_url, "https://relay.example.com");
        assert_eq!(config.auth_key, Some("test-key".to_string()));
        assert!(config.enabled);
        assert_eq!(config.min_profit_wei, 1000000000000000);
        assert_eq!(config.max_gas_price, 100);
    }

    #[test]
    fn test_mev_searcher_profitability() {
        let config = MevBundleConfig {
            relay_url: "https://relay.example.com".to_string(),
            auth_key: None,
            enabled: true,
            min_profit_wei: 1000000000000000, // 0.001 ETH
            max_gas_price: 100,               // 100 gwei
        };

        let searcher = MevSearcher::new(config).unwrap();
        assert!(searcher.is_profitable(1500000000000000)); // 0.0015 ETH
        assert!(!searcher.is_profitable(500000000000000)); // 0.0005 ETH
        assert!(searcher.is_gas_price_acceptable(50)); // 50 gwei
        assert!(!searcher.is_gas_price_acceptable(150)); // 150 gwei
    }

    #[test]
    fn test_relay_manager() {
        let mut manager = MevRelayManager::new();

        let relay = MevRelayInfo {
            name: "flashbots".to_string(),
            url: "https://relay.flashbots.net".to_string(),
            is_active: true,
            last_seen: 1234567890,
        };

        manager.add_relay(relay);
        assert!(manager.get_relay("flashbots").is_some());
        assert_eq!(manager.get_active_relays().len(), 1);

        let removed = manager.remove_relay("flashbots");
        assert!(removed.is_some());
        assert!(manager.get_relay("flashbots").is_none());
    }

    #[tokio::test]
    async fn test_mev_searcher_creation() {
        let config = MevBundleConfig {
            relay_url: "https://relay.example.com".to_string(),
            auth_key: None,
            enabled: true,
            min_profit_wei: 0,
            max_gas_price: 1000,
        };

        let searcher = MevSearcher::new(config);
        assert!(searcher.is_ok());

        let searcher = searcher.unwrap();
        assert_eq!(searcher.relay_url(), "https://relay.example.com");
        assert!(searcher.is_enabled());
    }
}
