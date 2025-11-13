//! Owner power monitoring implementation
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info, warn};

/// Owner power monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwnerPowerConfig {
    /// Enable/disable owner power monitoring
    pub enabled: bool,
    /// Threshold for owner balance changes that trigger alerts (percentage)
    pub balance_change_threshold_pct: f64,
    /// Threshold for owner minting capabilities that trigger alerts
    pub mint_capability_threshold: f64,
    /// Threshold for owner burn capabilities that trigger alerts
    pub burn_capability_threshold: f64,
    /// Threshold for owner pause capabilities that trigger alerts
    pub pause_capability_threshold: bool,
    /// Threshold for owner upgrade capabilities that trigger alerts
    pub upgrade_capability_threshold: bool,
    /// Time window for monitoring (in seconds)
    pub monitoring_window_seconds: u64,
    /// Minimum number of transactions to analyze
    pub min_transaction_count: usize,
}

impl Default for OwnerPowerConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            balance_change_threshold_pct: 5.0, // 5% balance change triggers alert
            mint_capability_threshold: 0.1,    // 10% of total supply can be minted
            burn_capability_threshold: 0.1,    // 10% of total supply can be burned
            pause_capability_threshold: true,  // Any pause capability triggers alert
            upgrade_capability_threshold: true, // Any upgrade capability triggers alert
            monitoring_window_seconds: 3600,   // 1 hour monitoring window
            min_transaction_count: 10,         // Analyze at least 10 transactions
        }
    }
}

/// Owner power monitoring result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwnerPowerResult {
    /// Whether the owner has excessive powers
    pub has_excessive_powers: bool,
    /// Risk score (0-100)
    pub risk_score: u32,
    /// Identified owner powers
    pub powers: Vec<OwnerPower>,
    /// Reasons for the assessment
    pub reasons: Vec<String>,
}

/// Specific owner power identified
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwnerPower {
    /// Type of power
    pub power_type: OwnerPowerType,
    /// Description of the power
    pub description: String,
    /// Risk level (1-10)
    pub risk_level: u32,
    /// Evidence for this power
    pub evidence: Vec<String>,
}

/// Types of owner powers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OwnerPowerType {
    /// Ability to mint new tokens
    Mint,
    /// Ability to burn tokens
    Burn,
    /// Ability to pause contract functions
    Pause,
    /// Ability to upgrade contract code
    Upgrade,
    /// Ability to change ownership
    OwnershipTransfer,
    /// Ability to modify fees
    FeeModification,
    /// Large balance changes
    BalanceManipulation,
    /// Other custom powers
    Custom(String),
}

/// Contract owner information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractOwner {
    /// Owner address
    pub address: String,
    /// Contract address
    pub contract_address: String,
    /// Timestamp of ownership
    pub since_timestamp: u64,
}

/// Transaction analysis data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwnerTransaction {
    /// Transaction hash
    pub tx_hash: String,
    /// Block number
    pub block_number: u64,
    /// Timestamp
    pub timestamp: u64,
    /// Function called
    pub function: String,
    /// Parameters
    pub parameters: serde_json::Value,
    /// Gas used
    pub gas_used: u64,
}

/// Token supply information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenSupply {
    /// Total supply
    pub total_supply: f64,
    /// Circulating supply
    pub circulating_supply: f64,
    /// Owner balance
    pub owner_balance: f64,
}

/// Owner power monitor
pub struct OwnerPowerMonitor {
    /// Configuration
    config: OwnerPowerConfig,
    /// Tracked contracts and their owners
    contract_owners: HashMap<String, ContractOwner>,
    /// Transaction history for analysis
    transaction_history: HashMap<String, Vec<OwnerTransaction>>,
    /// Token supply information
    token_supplies: HashMap<String, TokenSupply>,
    /// Monitoring results cache
    monitoring_cache: HashMap<String, OwnerPowerResult>,
}

impl OwnerPowerMonitor {
    /// Create a new owner power monitor
    pub fn new(config: OwnerPowerConfig) -> Self {
        Self {
            config,
            contract_owners: HashMap::new(),
            transaction_history: HashMap::new(),
            token_supplies: HashMap::new(),
            monitoring_cache: HashMap::new(),
        }
    }

    /// Monitor a contract for owner powers
    ///
    /// # Arguments
    /// * `contract_address` - Address of the contract to monitor
    ///
    /// # Returns
    /// * `Result<OwnerPowerResult>` - Monitoring result
    pub fn monitor_contract(&mut self, contract_address: &str) -> Result<OwnerPowerResult> {
        debug!("Monitoring contract for owner powers: {}", contract_address);

        if !self.config.enabled {
            return Ok(OwnerPowerResult {
                has_excessive_powers: false,
                risk_score: 0,
                powers: vec![],
                reasons: vec!["Owner power monitoring disabled".to_string()],
            });
        }

        // Check cache first
        if let Some(cached_result) = self.monitoring_cache.get(contract_address) {
            let cache_age = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs()
                - cached_result
                    .powers
                    .first()
                    .map(|p| p.risk_level as u64)
                    .unwrap_or(0);

            // Use cached result if it's less than 5 minutes old
            if cache_age < 300 {
                return Ok(cached_result.clone());
            }
        }

        let mut powers = Vec::new();
        let mut reasons = Vec::new();
        let mut risk_score: u32 = 0;

        // Check for minting capabilities
        if let Some(mint_power) = self.check_minting_capability(contract_address)? {
            powers.push(mint_power.clone());
            reasons.push(mint_power.description.clone());
            risk_score += mint_power.risk_level * 10;
        }

        // Check for burning capabilities
        if let Some(burn_power) = self.check_burning_capability(contract_address)? {
            powers.push(burn_power.clone());
            reasons.push(burn_power.description.clone());
            risk_score += burn_power.risk_level * 10;
        }

        // Check for pause capabilities
        if let Some(pause_power) = self.check_pause_capability(contract_address)? {
            powers.push(pause_power.clone());
            reasons.push(pause_power.description.clone());
            risk_score += pause_power.risk_level * 10;
        }

        // Check for upgrade capabilities
        if let Some(upgrade_power) = self.check_upgrade_capability(contract_address)? {
            powers.push(upgrade_power.clone());
            reasons.push(upgrade_power.description.clone());
            risk_score += upgrade_power.risk_level * 10;
        }

        // Check for ownership transfer capabilities
        if let Some(ownership_power) = self.check_ownership_transfer_capability(contract_address)? {
            powers.push(ownership_power.clone());
            reasons.push(ownership_power.description.clone());
            risk_score += ownership_power.risk_level * 10;
        }

        // Check for fee modification capabilities
        if let Some(fee_power) = self.check_fee_modification_capability(contract_address)? {
            powers.push(fee_power.clone());
            reasons.push(fee_power.description.clone());
            risk_score += fee_power.risk_level * 10;
        }

        // Check for balance manipulation
        if let Some(balance_power) = self.check_balance_manipulation(contract_address)? {
            powers.push(balance_power.clone());
            reasons.push(balance_power.description.clone());
            risk_score += balance_power.risk_level * 10;
        }

        let has_excessive_powers = risk_score >= 50; // Threshold for excessive powers

        let result = OwnerPowerResult {
            has_excessive_powers,
            risk_score: risk_score.min(100),
            powers,
            reasons,
        };

        // Cache the result
        self.monitoring_cache
            .insert(contract_address.to_string(), result.clone());

        if has_excessive_powers {
            warn!(
                "Contract {} has excessive owner powers with risk score {}",
                contract_address, risk_score
            );
        } else {
            info!(
                "Contract {} analyzed, no excessive owner powers detected (risk score: {})",
                contract_address, risk_score
            );
        }

        Ok(result)
    }

    /// Check for minting capabilities
    fn check_minting_capability(&self, contract_address: &str) -> Result<Option<OwnerPower>> {
        // In a real implementation, this would analyze the contract's ABI and code
        // For this implementation, we'll simulate with a simple approach

        // Simulate checking minting capability
        let can_mint = true; // Simulate that the contract can mint
        let mint_amount = 5000.0; // Simulate mint amount
        let total_supply = self
            .token_supplies
            .get(contract_address)
            .map(|s| s.total_supply)
            .unwrap_or(100000.0); // Default to 100,000 total supply

        let mint_percentage = if total_supply > 0.0 {
            (mint_amount / total_supply) * 100.0
        } else {
            0.0
        };

        if can_mint && mint_percentage > self.config.mint_capability_threshold * 100.0 {
            Ok(Some(OwnerPower {
                power_type: OwnerPowerType::Mint,
                description: format!(
                    "Owner can mint {:.2}% of total supply ({:.2} tokens)",
                    mint_percentage, mint_amount
                ),
                risk_level: ((mint_percentage / (self.config.mint_capability_threshold * 100.0))
                    * 10.0) as u32,
                evidence: vec![
                    "Mint function detected in contract ABI".to_string(),
                    format!("Maximum mint amount: {:.2} tokens", mint_amount),
                ],
            }))
        } else {
            Ok(None)
        }
    }

    /// Check for burning capabilities
    fn check_burning_capability(&self, contract_address: &str) -> Result<Option<OwnerPower>> {
        // In a real implementation, this would analyze the contract's ABI and code
        // For this implementation, we'll simulate with a simple approach

        // Simulate checking burning capability
        let can_burn = true; // Simulate that the contract can burn
        let burn_amount = 5000.0; // Simulate burn amount
        let total_supply = self
            .token_supplies
            .get(contract_address)
            .map(|s| s.total_supply)
            .unwrap_or(100000.0); // Default to 100,000 total supply

        let burn_percentage = if total_supply > 0.0 {
            (burn_amount / total_supply) * 100.0
        } else {
            0.0
        };

        if can_burn && burn_percentage > self.config.burn_capability_threshold * 100.0 {
            Ok(Some(OwnerPower {
                power_type: OwnerPowerType::Burn,
                description: format!(
                    "Owner can burn {:.2}% of total supply ({:.2} tokens)",
                    burn_percentage, burn_amount
                ),
                risk_level: ((burn_percentage / (self.config.burn_capability_threshold * 100.0))
                    * 10.0) as u32,
                evidence: vec![
                    "Burn function detected in contract ABI".to_string(),
                    format!("Maximum burn amount: {:.2} tokens", burn_amount),
                ],
            }))
        } else {
            Ok(None)
        }
    }

    /// Check for pause capabilities
    fn check_pause_capability(&self, _contract_address: &str) -> Result<Option<OwnerPower>> {
        // In a real implementation, this would analyze the contract's ABI and code
        // For this implementation, we'll simulate with a simple approach

        // Simulate checking pause capability
        let can_pause = true; // Simulate that the contract can pause

        if can_pause && self.config.pause_capability_threshold {
            Ok(Some(OwnerPower {
                power_type: OwnerPowerType::Pause,
                description: "Owner can pause contract functions".to_string(),
                risk_level: 8, // High risk
                evidence: vec![
                    "Pause function detected in contract ABI".to_string(),
                    "Only owner can call pause function".to_string(),
                ],
            }))
        } else {
            Ok(None)
        }
    }

    /// Check for upgrade capabilities
    fn check_upgrade_capability(&self, _contract_address: &str) -> Result<Option<OwnerPower>> {
        // In a real implementation, this would analyze the contract's ABI and code
        // For this implementation, we'll simulate with a simple approach

        // Simulate checking upgrade capability
        let can_upgrade = true; // Simulate that the contract can be upgraded

        if can_upgrade && self.config.upgrade_capability_threshold {
            Ok(Some(OwnerPower {
                power_type: OwnerPowerType::Upgrade,
                description: "Owner can upgrade contract code".to_string(),
                risk_level: 9, // Very high risk
                evidence: vec![
                    "Upgrade function detected in contract ABI".to_string(),
                    "Proxy pattern detected in contract".to_string(),
                ],
            }))
        } else {
            Ok(None)
        }
    }

    /// Check for ownership transfer capabilities
    fn check_ownership_transfer_capability(
        &self,
        contract_address: &str,
    ) -> Result<Option<OwnerPower>> {
        // Check if we have owner information
        if let Some(owner_info) = self.contract_owners.get(contract_address) {
            // Check if ownership is recent (less than 30 days old)
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();

            let ownership_age_days = (now - owner_info.since_timestamp) / 86400;

            if ownership_age_days < 30 {
                Ok(Some(OwnerPower {
                    power_type: OwnerPowerType::OwnershipTransfer,
                    description: format!(
                        "Recent ownership transfer ({} days ago)",
                        ownership_age_days
                    ),
                    risk_level: 6, // Medium risk
                    evidence: vec![
                        format!("Owner address: {}", owner_info.address),
                        format!("Ownership since: {}", owner_info.since_timestamp),
                    ],
                }))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    /// Check for fee modification capabilities
    fn check_fee_modification_capability(
        &self,
        _contract_address: &str,
    ) -> Result<Option<OwnerPower>> {
        // In a real implementation, this would analyze the contract's ABI and code
        // For this implementation, we'll simulate with a simple approach

        // Simulate checking fee modification capability
        let can_modify_fees = true; // Simulate that the contract can modify fees
        let max_fee_change = 5.0; // Simulate 5% maximum fee change

        if can_modify_fees && max_fee_change > 2.0 {
            // Alert if fee can be changed by more than 2%
            Ok(Some(OwnerPower {
                power_type: OwnerPowerType::FeeModification,
                description: format!("Owner can modify fees by up to {:.1}%", max_fee_change),
                risk_level: 5, // Medium risk
                evidence: vec![
                    "SetFee function detected in contract ABI".to_string(),
                    format!("Maximum fee change: {:.1}%", max_fee_change),
                ],
            }))
        } else {
            Ok(None)
        }
    }

    /// Check for balance manipulation
    fn check_balance_manipulation(&self, contract_address: &str) -> Result<Option<OwnerPower>> {
        if let Some(tx_history) = self.transaction_history.get(contract_address) {
            if tx_history.len() < self.config.min_transaction_count {
                return Ok(None);
            }

            // Check for large balance changes in owner transactions
            let large_changes = tx_history
                .iter()
                .filter(|tx| {
                    tx.function == "transfer"
                        || tx.function == "transferFrom"
                        || tx.function == "mint"
                        || tx.function == "burn"
                })
                .count();

            let change_percentage = (large_changes as f64 / tx_history.len() as f64) * 100.0;

            if change_percentage > self.config.balance_change_threshold_pct {
                Ok(Some(OwnerPower {
                    power_type: OwnerPowerType::BalanceManipulation,
                    description: format!(
                        "High percentage of owner transactions involve balance changes ({:.1}%)",
                        change_percentage
                    ),
                    risk_level: ((change_percentage / self.config.balance_change_threshold_pct)
                        * 10.0) as u32,
                    evidence: vec![
                        format!(
                            "{} out of {} transactions involve balance changes",
                            large_changes,
                            tx_history.len()
                        ),
                        "Owner frequently interacts with token balances".to_string(),
                    ],
                }))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    /// Register a contract owner
    ///
    /// # Arguments
    /// * `contract_address` - Address of the contract
    /// * `owner` - Contract owner information
    pub fn register_contract_owner(&mut self, contract_address: &str, owner: ContractOwner) {
        self.contract_owners
            .insert(contract_address.to_string(), owner);
    }

    /// Add transaction data for a contract
    ///
    /// # Arguments
    /// * `contract_address` - Address of the contract
    /// * `transaction` - Transaction data to add
    pub fn add_transaction_data(&mut self, contract_address: &str, transaction: OwnerTransaction) {
        self.transaction_history
            .entry(contract_address.to_string())
            .or_default()
            .push(transaction);

        // Keep only recent data within the monitoring window
        if let Some(data) = self.transaction_history.get_mut(contract_address) {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();

            data.retain(|tx| now - tx.timestamp <= self.config.monitoring_window_seconds);
        }
    }

    /// Update token supply information
    ///
    /// # Arguments
    /// * `contract_address` - Address of the contract
    /// * `supply` - Token supply information
    pub fn update_token_supply(&mut self, contract_address: &str, supply: TokenSupply) {
        self.token_supplies
            .insert(contract_address.to_string(), supply);
    }

    /// Clear cached monitoring results
    pub fn clear_cache(&mut self) {
        self.monitoring_cache.clear();
    }

    /// Update configuration
    ///
    /// # Arguments
    /// * `config` - New configuration
    pub fn update_config(&mut self, config: OwnerPowerConfig) {
        self.config = config;
    }

    /// Get monitoring result for a contract from cache
    ///
    /// # Arguments
    /// * `contract_address` - Address of the contract
    ///
    /// # Returns
    /// * `Option<&OwnerPowerResult>` - Cached monitoring result or None
    pub fn get_cached_result(&self, contract_address: &str) -> Option<&OwnerPowerResult> {
        self.monitoring_cache.get(contract_address)
    }
}

/// Advanced owner power monitor with machine learning capabilities
pub struct AdvancedOwnerPowerMonitor {
    /// Base owner power monitor
    base_monitor: OwnerPowerMonitor,
    /// Historical monitoring results for learning
    historical_results: Vec<MonitoringOutcome>,
    /// Learning rate for model updates
    learning_rate: f64,
}

/// Monitoring outcome for learning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringOutcome {
    /// Contract address
    pub contract_address: String,
    /// Risk score
    pub risk_score: u32,
    /// Actual outcome (confirmed_risk, false_positive, etc.)
    pub actual_outcome: String,
    /// Timestamp
    pub timestamp: u64,
}

impl AdvancedOwnerPowerMonitor {
    /// Create a new advanced owner power monitor
    pub fn new(base_monitor: OwnerPowerMonitor) -> Self {
        Self {
            base_monitor,
            historical_results: Vec::new(),
            learning_rate: 0.01,
        }
    }

    /// Monitor a contract with learning capabilities
    ///
    /// # Arguments
    /// * `contract_address` - Address of the contract to monitor
    ///
    /// # Returns
    /// * `Result<OwnerPowerResult>` - Enhanced monitoring result
    pub fn monitor_contract_with_learning(
        &mut self,
        contract_address: &str,
    ) -> Result<OwnerPowerResult> {
        // Get base monitoring result
        let mut result = self.base_monitor.monitor_contract(contract_address)?;

        // Apply learning adjustments
        if let Some(adjusted_risk_score) = self.adjust_risk_score(contract_address, &result) {
            result.risk_score = adjusted_risk_score;
            result.has_excessive_powers = adjusted_risk_score >= 50;
        }

        Ok(result)
    }

    /// Adjust risk score based on historical data
    fn adjust_risk_score(&self, _contract_address: &str, result: &OwnerPowerResult) -> Option<u32> {
        // In a real implementation, this would use ML models
        // For this implementation, we'll simulate with a simple approach

        let mut adjustment: i32 = 0;

        // If we have historical data, adjust based on patterns
        if !self.historical_results.is_empty() {
            let correct_detections = self
                .historical_results
                .iter()
                .filter(|d| d.actual_outcome == "confirmed_risk")
                .count();

            let total_detections = self.historical_results.len();
            let accuracy = correct_detections as f64 / total_detections as f64;

            // If accuracy is low, reduce risk score
            if accuracy < 0.7 {
                adjustment -= 20;
            } else if accuracy > 0.9 {
                adjustment += 10;
            }
        }

        let adjusted_risk_score = (result.risk_score as i32 + adjustment).clamp(0, 100) as u32;
        Some(adjusted_risk_score)
    }

    /// Record monitoring outcome for learning
    ///
    /// # Arguments
    /// * `outcome` - Monitoring outcome data
    pub fn record_monitoring_outcome(&mut self, outcome: MonitoringOutcome) {
        self.historical_results.push(outcome);

        // Keep only recent data (last 1000 monitoring results)
        if self.historical_results.len() > 1000 {
            self.historical_results
                .drain(0..self.historical_results.len() - 1000);
        }
    }

    /// Update learning rate
    ///
    /// # Arguments
    /// * `rate` - New learning rate
    pub fn update_learning_rate(&mut self, rate: f64) {
        self.learning_rate = rate;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_owner_power_config() {
        let config = OwnerPowerConfig::default();
        assert!(config.enabled);
        assert_eq!(config.balance_change_threshold_pct, 5.0);
        assert_eq!(config.mint_capability_threshold, 0.1);
        assert_eq!(config.burn_capability_threshold, 0.1);
        assert!(config.pause_capability_threshold);
        assert!(config.upgrade_capability_threshold);
        assert_eq!(config.monitoring_window_seconds, 3600);
        assert_eq!(config.min_transaction_count, 10);
    }

    #[test]
    fn test_owner_power_monitor_creation() {
        let config = OwnerPowerConfig::default();
        let monitor = OwnerPowerMonitor::new(config);
        assert!(monitor.contract_owners.is_empty());
        assert!(monitor.transaction_history.is_empty());
        assert!(monitor.token_supplies.is_empty());
        assert!(monitor.monitoring_cache.is_empty());
    }

    #[test]
    fn test_disabled_owner_power_monitoring() {
        let config = OwnerPowerConfig {
            enabled: false,
            ..Default::default()
        };
        let mut monitor = OwnerPowerMonitor::new(config);

        let result = monitor
            .monitor_contract("0x1234567890123456789012345678901234567890")
            .unwrap();
        assert!(!result.has_excessive_powers);
        assert_eq!(result.risk_score, 0);
        assert_eq!(
            result.reasons,
            vec!["Owner power monitoring disabled".to_string()]
        );
        assert!(result.powers.is_empty());
    }

    #[test]
    fn test_contract_owner_registration() {
        let config = OwnerPowerConfig::default();
        let mut monitor = OwnerPowerMonitor::new(config);

        let owner = ContractOwner {
            address: "0xOwner".to_string(),
            contract_address: "0xContract".to_string(),
            since_timestamp: 1234567890,
        };

        monitor.register_contract_owner("0xContract", owner);
        assert!(monitor.contract_owners.contains_key("0xContract"));
    }

    #[test]
    fn test_transaction_data_management() {
        let config = OwnerPowerConfig::default();
        let mut monitor = OwnerPowerMonitor::new(config);

        let transaction = OwnerTransaction {
            tx_hash: "0xabc".to_string(),
            block_number: 123456,
            timestamp: 1234567890,
            function: "transfer".to_string(),
            parameters: serde_json::json!({"to": "0xRecipient", "amount": "100"}),
            gas_used: 50000,
        };

        monitor.add_transaction_data("0xContract", transaction);
        assert!(monitor.transaction_history.contains_key("0xContract"));
    }

    #[test]
    fn test_token_supply_update() {
        let config = OwnerPowerConfig::default();
        let mut monitor = OwnerPowerMonitor::new(config);

        let supply = TokenSupply {
            total_supply: 100000.0,
            circulating_supply: 80000.0,
            owner_balance: 10000.0,
        };

        monitor.update_token_supply("0xContract", supply);
        assert!(monitor.token_supplies.contains_key("0xContract"));
    }

    #[test]
    fn test_owner_power_types() {
        let mint_power = OwnerPower {
            power_type: OwnerPowerType::Mint,
            description: "Can mint tokens".to_string(),
            risk_level: 7,
            evidence: vec!["Mint function in ABI".to_string()],
        };

        assert!(matches!(mint_power.power_type, OwnerPowerType::Mint));
        assert_eq!(mint_power.description, "Can mint tokens");
        assert_eq!(mint_power.risk_level, 7);
        assert_eq!(mint_power.evidence.len(), 1);
    }

    #[test]
    fn test_owner_power_result_creation() {
        let result = OwnerPowerResult {
            has_excessive_powers: true,
            risk_score: 80,
            powers: vec![],
            reasons: vec!["Test reason".to_string()],
        };

        assert!(result.has_excessive_powers);
        assert_eq!(result.risk_score, 80);
        assert!(result.powers.is_empty());
        assert_eq!(result.reasons.len(), 1);
    }

    #[test]
    fn test_advanced_owner_power_monitor() {
        let config = OwnerPowerConfig::default();
        let base_monitor = OwnerPowerMonitor::new(config);
        let mut advanced_monitor = AdvancedOwnerPowerMonitor::new(base_monitor);

        let outcome = MonitoringOutcome {
            contract_address: "0x1234567890123456789012345678901234567890".to_string(),
            risk_score: 80,
            actual_outcome: "confirmed_risk".to_string(),
            timestamp: 1234567890,
        };

        advanced_monitor.record_monitoring_outcome(outcome);
        assert_eq!(advanced_monitor.historical_results.len(), 1);
    }
}
