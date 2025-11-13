//! Take profit exit strategy module for the sniper bot.
//!
//! This module provides functionality for automatically exiting positions when a target profit level is reached.

use super::TradeSide;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info};

/// Take profit configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TakeProfitConfig {
    /// Default take profit percentage (e.g., 10.0 for 10%)
    pub default_percentage: f64,
    /// Per-pair take profit percentages
    pub pair_percentages: HashMap<String, f64>,
    /// Enable/disable take profit
    pub enabled: bool,
}

/// Take profit strategy
pub struct TakeProfitStrategy {
    /// Configuration
    config: TakeProfitConfig,
}

impl TakeProfitStrategy {
    /// Create a new take profit strategy
    pub fn new(config: TakeProfitConfig) -> Self {
        Self { config }
    }

    /// Check if take profit should be triggered for a position
    pub fn should_take_profit(
        &self,
        pair: &str,
        entry_price: f64,
        current_price: f64,
        side: TradeSide,
    ) -> bool {
        if !self.config.enabled {
            debug!("Take profit is disabled");
            return false;
        }

        let target_percentage = self.get_target_percentage(pair);
        let current_percentage = self.calculate_profit_percentage(entry_price, current_price, side);

        debug!(
            "Take profit check for {}: Entry price: {}, Current price: {}, Target: {}%, Current: {}%",
            pair, entry_price, current_price, target_percentage, current_percentage
        );

        if current_percentage >= target_percentage {
            info!(
                "Take profit triggered for {}: Target {}% reached with {}% profit",
                pair, target_percentage, current_percentage
            );
            true
        } else {
            false
        }
    }

    /// Calculate profit percentage
    pub fn calculate_profit_percentage(
        &self,
        entry_price: f64,
        current_price: f64,
        side: TradeSide,
    ) -> f64 {
        match side {
            TradeSide::Long => ((current_price - entry_price) / entry_price) * 100.0,
            TradeSide::Short => ((entry_price - current_price) / entry_price) * 100.0,
        }
    }

    /// Get target percentage for a pair
    pub fn get_target_percentage(&self, pair: &str) -> f64 {
        *self
            .config
            .pair_percentages
            .get(pair)
            .unwrap_or(&self.config.default_percentage)
    }

    /// Update configuration
    pub fn update_config(&mut self, config: TakeProfitConfig) {
        self.config = config;
    }

    /// Add or update pair percentage
    pub fn set_pair_percentage(&mut self, pair: String, percentage: f64) {
        self.config.pair_percentages.insert(pair, percentage);
    }

    /// Remove pair percentage (will use default)
    pub fn remove_pair_percentage(&mut self, pair: &str) {
        self.config.pair_percentages.remove(pair);
    }

    /// Validate configuration
    pub fn validate_config(&self) -> Result<()> {
        if self.config.default_percentage <= 0.0 {
            return Err(anyhow::anyhow!(
                "Default take profit percentage must be positive"
            ));
        }

        for (pair, percentage) in &self.config.pair_percentages {
            if *percentage <= 0.0 {
                return Err(anyhow::anyhow!(
                    "Take profit percentage for {} must be positive",
                    pair
                ));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_take_profit_strategy_creation() {
        let config = TakeProfitConfig {
            default_percentage: 10.0,
            pair_percentages: HashMap::new(),
            enabled: true,
        };

        let strategy = TakeProfitStrategy::new(config.clone());
        assert_eq!(strategy.config.default_percentage, 10.0);
        assert!(strategy.config.enabled);
    }

    #[test]
    fn test_profit_percentage_calculation() {
        let config = TakeProfitConfig {
            default_percentage: 10.0,
            pair_percentages: HashMap::new(),
            enabled: true,
        };
        let strategy = TakeProfitStrategy::new(config);

        // Test long position profit
        let profit_pct = strategy.calculate_profit_percentage(100.0, 110.0, TradeSide::Long);
        assert_eq!(profit_pct, 10.0);

        // Test long position loss
        let profit_pct = strategy.calculate_profit_percentage(100.0, 90.0, TradeSide::Long);
        assert_eq!(profit_pct, -10.0);

        // Test short position profit
        let profit_pct = strategy.calculate_profit_percentage(100.0, 90.0, TradeSide::Short);
        assert_eq!(profit_pct, 10.0);

        // Test short position loss
        let profit_pct = strategy.calculate_profit_percentage(100.0, 110.0, TradeSide::Short);
        assert_eq!(profit_pct, -10.0);
    }

    #[test]
    fn test_target_percentage() {
        let mut pair_percentages = HashMap::new();
        pair_percentages.insert("ETH/USDT".to_string(), 15.0);

        let config = TakeProfitConfig {
            default_percentage: 10.0,
            pair_percentages,
            enabled: true,
        };
        let strategy = TakeProfitStrategy::new(config);

        // Test pair-specific percentage
        assert_eq!(strategy.get_target_percentage("ETH/USDT"), 15.0);

        // Test default percentage
        assert_eq!(strategy.get_target_percentage("BTC/USDT"), 10.0);
    }

    #[test]
    fn test_should_take_profit() {
        let mut pair_percentages = HashMap::new();
        pair_percentages.insert("ETH/USDT".to_string(), 5.0);

        let config = TakeProfitConfig {
            default_percentage: 10.0,
            pair_percentages,
            enabled: true,
        };
        let strategy = TakeProfitStrategy::new(config);

        // Test long position - should trigger
        assert!(strategy.should_take_profit("BTC/USDT", 100.0, 115.0, TradeSide::Long));

        // Test long position - should not trigger
        assert!(!strategy.should_take_profit("BTC/USDT", 100.0, 105.0, TradeSide::Long));

        // Test short position - should trigger
        assert!(strategy.should_take_profit("BTC/USDT", 100.0, 85.0, TradeSide::Short));

        // Test short position - should not trigger
        assert!(!strategy.should_take_profit("BTC/USDT", 100.0, 95.0, TradeSide::Short));

        // Test pair-specific percentage
        assert!(strategy.should_take_profit("ETH/USDT", 100.0, 106.0, TradeSide::Long));
        assert!(!strategy.should_take_profit("ETH/USDT", 100.0, 104.0, TradeSide::Long));
    }

    #[test]
    fn test_config_validation() {
        // Valid config
        let config = TakeProfitConfig {
            default_percentage: 10.0,
            pair_percentages: HashMap::new(),
            enabled: true,
        };
        let strategy = TakeProfitStrategy::new(config);
        assert!(strategy.validate_config().is_ok());

        // Invalid config - negative default percentage
        let config = TakeProfitConfig {
            default_percentage: -5.0,
            pair_percentages: HashMap::new(),
            enabled: true,
        };
        let strategy = TakeProfitStrategy::new(config);
        assert!(strategy.validate_config().is_err());

        // Invalid config - negative pair percentage
        let mut pair_percentages = HashMap::new();
        pair_percentages.insert("ETH/USDT".to_string(), -5.0);

        let config = TakeProfitConfig {
            default_percentage: 10.0,
            pair_percentages,
            enabled: true,
        };
        let strategy = TakeProfitStrategy::new(config);
        assert!(strategy.validate_config().is_err());
    }

    #[test]
    fn test_config_management() {
        let config = TakeProfitConfig {
            default_percentage: 10.0,
            pair_percentages: HashMap::new(),
            enabled: true,
        };
        let mut strategy = TakeProfitStrategy::new(config);

        // Test adding pair percentage
        strategy.set_pair_percentage("ETH/USDT".to_string(), 15.0);
        assert_eq!(strategy.get_target_percentage("ETH/USDT"), 15.0);

        // Test removing pair percentage
        strategy.remove_pair_percentage("ETH/USDT");
        assert_eq!(strategy.get_target_percentage("ETH/USDT"), 10.0);

        // Test updating config
        let new_config = TakeProfitConfig {
            default_percentage: 20.0,
            pair_percentages: HashMap::new(),
            enabled: true,
        };
        strategy.update_config(new_config);
        assert_eq!(strategy.config.default_percentage, 20.0);
    }
}
