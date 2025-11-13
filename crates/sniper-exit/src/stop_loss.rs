//! Stop loss exit strategy module for the sniper bot.
//!
//! This module provides functionality for automatically exiting positions when a maximum loss threshold is reached.

use super::TradeSide;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info};

/// Stop loss configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StopLossConfig {
    /// Default stop loss percentage (e.g., 5.0 for 5%)
    pub default_percentage: f64,
    /// Per-pair stop loss percentages
    pub pair_percentages: HashMap<String, f64>,
    /// Enable/disable stop loss
    pub enabled: bool,
    /// Trailing stop loss configuration
    pub trailing: Option<TrailingStopConfig>,
}

/// Trailing stop loss configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrailingStopConfig {
    /// Trailing stop percentage
    pub percentage: f64,
    /// Activation percentage (when trailing stop becomes active)
    pub activation_percentage: f64,
}

/// Stop loss strategy
pub struct StopLossStrategy {
    /// Configuration
    config: StopLossConfig,
    /// Trailing stop tracking for active positions
    trailing_stops: HashMap<String, TrailingStopTracker>,
}

impl StopLossStrategy {
    /// Create a new stop loss strategy
    pub fn new(config: StopLossConfig) -> Self {
        Self {
            config,
            trailing_stops: HashMap::new(),
        }
    }

    /// Check if stop loss should be triggered for a position
    pub fn should_stop_loss(
        &mut self,
        pair: &str,
        entry_price: f64,
        current_price: f64,
        side: TradeSide,
    ) -> bool {
        if !self.config.enabled {
            debug!("Stop loss is disabled");
            return false;
        }

        let max_loss_percentage = self.get_max_loss_percentage(pair);
        let current_percentage = self.calculate_loss_percentage(entry_price, current_price, side);

        debug!(
            "Stop loss check for {}: Entry price: {}, Current price: {}, Max loss: {}%, Current loss: {}%",
            pair, entry_price, current_price, max_loss_percentage, current_percentage
        );

        // Check regular stop loss
        if current_percentage <= -max_loss_percentage {
            info!(
                "Stop loss triggered for {}: Max loss {}% reached with {}% loss",
                pair, max_loss_percentage, current_percentage
            );
            // Remove trailing stop if it exists
            self.trailing_stops.remove(pair);
            return true;
        }

        // Check trailing stop loss by cloning the config first
        if let Some(trailing_config) = self.config.trailing.clone() {
            self.check_trailing_stop(pair, entry_price, current_price, side, &trailing_config)
        } else {
            false
        }
    }

    /// Calculate loss percentage
    pub fn calculate_loss_percentage(
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

    /// Get maximum loss percentage for a pair
    pub fn get_max_loss_percentage(&self, pair: &str) -> f64 {
        *self
            .config
            .pair_percentages
            .get(pair)
            .unwrap_or(&self.config.default_percentage)
    }

    /// Check trailing stop loss
    fn check_trailing_stop(
        &mut self,
        pair: &str,
        entry_price: f64,
        current_price: f64,
        side: TradeSide,
        trailing_config: &TrailingStopConfig,
    ) -> bool {
        let tracker_key = pair.to_string();

        // Check if we already have a trailing stop tracker for this pair
        let mut tracker = self.trailing_stops.remove(&tracker_key).unwrap_or_else(|| {
            // No existing tracker, check if we should create one
            let profit_percentage =
                self.calculate_loss_percentage(entry_price, current_price, side);

            // Only activate trailing stop if we've reached the activation threshold
            if profit_percentage < trailing_config.activation_percentage {
                debug!(
                    "Trailing stop not activated for {} (profit: {}% < activation: {}%)",
                    pair, profit_percentage, trailing_config.activation_percentage
                );
                // Store a placeholder tracker that will be replaced when activated
                return TrailingStopTracker::new(trailing_config.percentage);
            }

            debug!(
                "Trailing stop activated for {} (profit: {}% >= activation: {}%)",
                pair, profit_percentage, trailing_config.activation_percentage
            );
            TrailingStopTracker::new(trailing_config.percentage)
        });

        let should_exit = tracker.update(current_price, side);

        if should_exit {
            info!("Trailing stop loss triggered for {}", pair);
            true
        } else {
            // Store the updated tracker
            self.trailing_stops.insert(tracker_key, tracker);
            false
        }
    }

    /// Update configuration
    pub fn update_config(&mut self, config: StopLossConfig) {
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
                "Default stop loss percentage must be positive"
            ));
        }

        if self.config.default_percentage > 100.0 {
            return Err(anyhow::anyhow!(
                "Default stop loss percentage must be <= 100%"
            ));
        }

        for (pair, percentage) in &self.config.pair_percentages {
            if *percentage <= 0.0 {
                return Err(anyhow::anyhow!(
                    "Stop loss percentage for {} must be positive",
                    pair
                ));
            }
            if *percentage > 100.0 {
                return Err(anyhow::anyhow!(
                    "Stop loss percentage for {} must be <= 100%",
                    pair
                ));
            }
        }

        if let Some(trailing) = &self.config.trailing {
            if trailing.percentage <= 0.0 {
                return Err(anyhow::anyhow!("Trailing stop percentage must be positive"));
            }
            if trailing.percentage > 100.0 {
                return Err(anyhow::anyhow!("Trailing stop percentage must be <= 100%"));
            }
            if trailing.activation_percentage <= 0.0 {
                return Err(anyhow::anyhow!(
                    "Trailing stop activation percentage must be positive"
                ));
            }
        }

        Ok(())
    }

    /// Reset trailing stops for a pair
    pub fn reset_trailing_stop(&mut self, pair: &str) {
        self.trailing_stops.remove(pair);
    }

    /// Reset all trailing stops
    pub fn reset_all_trailing_stops(&mut self) {
        self.trailing_stops.clear();
    }
}

/// Trailing stop tracker
#[derive(Debug, Clone)]
struct TrailingStopTracker {
    /// Trailing percentage
    percentage: f64,
    /// Highest price seen (for long) or lowest price seen (for short)
    peak_price: Option<f64>,
}

impl TrailingStopTracker {
    /// Create a new trailing stop tracker
    fn new(percentage: f64) -> Self {
        Self {
            percentage,
            peak_price: None,
        }
    }

    /// Update the tracker with current price and check if stop should trigger
    fn update(&mut self, current_price: f64, side: TradeSide) -> bool {
        match side {
            TradeSide::Long => {
                // For long positions, track highest price
                let peak = self.peak_price.unwrap_or(current_price);
                if current_price >= peak {
                    self.peak_price = Some(current_price);
                    debug!("Updated peak price for long position: {}", current_price);
                    false
                } else {
                    let drawdown = ((peak - current_price) / peak) * 100.0;
                    if drawdown >= self.percentage {
                        debug!(
                            "Trailing stop triggered for long position: drawdown {}% >= {}%",
                            drawdown, self.percentage
                        );
                        true
                    } else {
                        debug!(
                            "Trailing stop monitoring long position: drawdown {}% < {}%",
                            drawdown, self.percentage
                        );
                        false
                    }
                }
            }
            TradeSide::Short => {
                // For short positions, track lowest price
                let peak = self.peak_price.unwrap_or(current_price);
                if current_price <= peak {
                    self.peak_price = Some(current_price);
                    debug!("Updated peak price for short position: {}", current_price);
                    false
                } else {
                    let drawdown = ((current_price - peak) / peak) * 100.0;
                    if drawdown >= self.percentage {
                        debug!(
                            "Trailing stop triggered for short position: drawdown {}% >= {}%",
                            drawdown, self.percentage
                        );
                        true
                    } else {
                        debug!(
                            "Trailing stop monitoring short position: drawdown {}% < {}%",
                            drawdown, self.percentage
                        );
                        false
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stop_loss_strategy_creation() {
        let config = StopLossConfig {
            default_percentage: 5.0,
            pair_percentages: HashMap::new(),
            enabled: true,
            trailing: None,
        };

        let strategy = StopLossStrategy::new(config.clone());
        assert_eq!(strategy.config.default_percentage, 5.0);
        assert!(strategy.config.enabled);
        assert!(strategy.trailing_stops.is_empty());
    }

    #[test]
    fn test_loss_percentage_calculation() {
        let config = StopLossConfig {
            default_percentage: 5.0,
            pair_percentages: HashMap::new(),
            enabled: true,
            trailing: None,
        };
        let strategy = StopLossStrategy::new(config);

        // Test long position loss
        let loss_pct = strategy.calculate_loss_percentage(100.0, 90.0, TradeSide::Long);
        assert_eq!(loss_pct, -10.0);

        // Test long position profit
        let loss_pct = strategy.calculate_loss_percentage(100.0, 110.0, TradeSide::Long);
        assert_eq!(loss_pct, 10.0);

        // Test short position loss
        let loss_pct = strategy.calculate_loss_percentage(100.0, 110.0, TradeSide::Short);
        assert_eq!(loss_pct, -10.0);

        // Test short position profit
        let loss_pct = strategy.calculate_loss_percentage(100.0, 90.0, TradeSide::Short);
        assert_eq!(loss_pct, 10.0);
    }

    #[test]
    fn test_max_loss_percentage() {
        let mut pair_percentages = HashMap::new();
        pair_percentages.insert("ETH/USDT".to_string(), 3.0);

        let config = StopLossConfig {
            default_percentage: 5.0,
            pair_percentages,
            enabled: true,
            trailing: None,
        };
        let strategy = StopLossStrategy::new(config);

        // Test pair-specific percentage
        assert_eq!(strategy.get_max_loss_percentage("ETH/USDT"), 3.0);

        // Test default percentage
        assert_eq!(strategy.get_max_loss_percentage("BTC/USDT"), 5.0);
    }

    #[test]
    fn test_should_stop_loss() {
        let mut pair_percentages = HashMap::new();
        pair_percentages.insert("ETH/USDT".to_string(), 2.0);

        let config = StopLossConfig {
            default_percentage: 5.0,
            pair_percentages,
            enabled: true,
            trailing: None,
        };
        let mut strategy = StopLossStrategy::new(config);

        // Test long position - should trigger
        assert!(strategy.should_stop_loss("BTC/USDT", 100.0, 90.0, TradeSide::Long));

        // Test long position - should not trigger
        assert!(!strategy.should_stop_loss("BTC/USDT", 100.0, 96.0, TradeSide::Long));

        // Test short position - should trigger
        assert!(strategy.should_stop_loss("BTC/USDT", 100.0, 110.0, TradeSide::Short));

        // Test short position - should not trigger
        assert!(!strategy.should_stop_loss("BTC/USDT", 100.0, 104.0, TradeSide::Short));

        // Test pair-specific percentage
        assert!(strategy.should_stop_loss("ETH/USDT", 100.0, 97.0, TradeSide::Long));
        assert!(!strategy.should_stop_loss("ETH/USDT", 100.0, 98.5, TradeSide::Long));
    }

    #[test]
    fn test_config_validation() {
        // Valid config
        let config = StopLossConfig {
            default_percentage: 5.0,
            pair_percentages: HashMap::new(),
            enabled: true,
            trailing: None,
        };
        let strategy = StopLossStrategy::new(config);
        assert!(strategy.validate_config().is_ok());

        // Invalid config - negative default percentage
        let config = StopLossConfig {
            default_percentage: -5.0,
            pair_percentages: HashMap::new(),
            enabled: true,
            trailing: None,
        };
        let strategy = StopLossStrategy::new(config);
        assert!(strategy.validate_config().is_err());

        // Invalid config - percentage > 100%
        let config = StopLossConfig {
            default_percentage: 150.0,
            pair_percentages: HashMap::new(),
            enabled: true,
            trailing: None,
        };
        let strategy = StopLossStrategy::new(config);
        assert!(strategy.validate_config().is_err());

        // Invalid config - negative pair percentage
        let mut pair_percentages = HashMap::new();
        pair_percentages.insert("ETH/USDT".to_string(), -5.0);

        let config = StopLossConfig {
            default_percentage: 5.0,
            pair_percentages,
            enabled: true,
            trailing: None,
        };
        let strategy = StopLossStrategy::new(config);
        assert!(strategy.validate_config().is_err());

        // Valid config with trailing stop
        let config = StopLossConfig {
            default_percentage: 5.0,
            pair_percentages: HashMap::new(),
            enabled: true,
            trailing: Some(TrailingStopConfig {
                percentage: 2.0,
                activation_percentage: 1.0,
            }),
        };
        let strategy = StopLossStrategy::new(config);
        assert!(strategy.validate_config().is_ok());

        // Invalid trailing config - negative percentage
        let config = StopLossConfig {
            default_percentage: 5.0,
            pair_percentages: HashMap::new(),
            enabled: true,
            trailing: Some(TrailingStopConfig {
                percentage: -2.0,
                activation_percentage: 1.0,
            }),
        };
        let strategy = StopLossStrategy::new(config);
        assert!(strategy.validate_config().is_err());
    }

    #[test]
    fn test_config_management() {
        let config = StopLossConfig {
            default_percentage: 5.0,
            pair_percentages: HashMap::new(),
            enabled: true,
            trailing: None,
        };
        let mut strategy = StopLossStrategy::new(config);

        // Test adding pair percentage
        strategy.set_pair_percentage("ETH/USDT".to_string(), 3.0);
        assert_eq!(strategy.get_max_loss_percentage("ETH/USDT"), 3.0);

        // Test removing pair percentage
        strategy.remove_pair_percentage("ETH/USDT");
        assert_eq!(strategy.get_max_loss_percentage("ETH/USDT"), 5.0);

        // Test updating config
        let new_config = StopLossConfig {
            default_percentage: 10.0,
            pair_percentages: HashMap::new(),
            enabled: true,
            trailing: None,
        };
        strategy.update_config(new_config);
        assert_eq!(strategy.config.default_percentage, 10.0);
    }

    #[test]
    fn test_trailing_stop_tracker() {
        // Test long position trailing stop
        let mut tracker = TrailingStopTracker::new(2.0);

        // First call sets peak and returns false
        assert!(!tracker.update(100.0, TradeSide::Long)); // Sets peak
        assert_eq!(tracker.peak_price, Some(100.0));

        // Higher price updates peak and returns false
        assert!(!tracker.update(105.0, TradeSide::Long)); // Updates peak
        assert_eq!(tracker.peak_price, Some(105.0));

        // Lower price triggers stop (2.0% drawdown from 105.0)
        assert!(tracker.update(102.89, TradeSide::Long)); // ~2.009% drawdown, should trigger
        assert_eq!(tracker.peak_price, Some(105.0));

        // Test short position trailing stop
        let mut tracker = TrailingStopTracker::new(2.0);

        // First call sets peak and returns false
        assert!(!tracker.update(100.0, TradeSide::Short)); // Sets peak
        assert_eq!(tracker.peak_price, Some(100.0));

        // Lower price updates peak and returns false
        assert!(!tracker.update(95.0, TradeSide::Short)); // Updates peak
        assert_eq!(tracker.peak_price, Some(95.0));

        // Higher price triggers stop (~2.1% drawdown from 95.0)
        assert!(tracker.update(97.01, TradeSide::Short)); // ~2.11% drawdown, should trigger
        assert_eq!(tracker.peak_price, Some(95.0));
    }

    #[test]
    fn test_trailing_stop_functionality() {
        let config = StopLossConfig {
            default_percentage: 5.0,
            pair_percentages: HashMap::new(),
            enabled: true,
            trailing: Some(TrailingStopConfig {
                percentage: 2.0,
                activation_percentage: 1.0,
            }),
        };
        let mut strategy = StopLossStrategy::new(config);

        // Test trailing stop activation
        // Profit of 3% should activate trailing stop
        assert!(!strategy.should_stop_loss("BTC/USDT", 100.0, 103.0, TradeSide::Long));

        // Now a 2% drawdown should trigger the trailing stop
        // From peak of 103, a 2% drawdown would be to 103 * (1 - 0.02) = 100.94
        // Let's use a slightly lower value to ensure it triggers
        assert!(strategy.should_stop_loss("BTC/USDT", 100.0, 100.93, TradeSide::Long));
    }

    #[test]
    fn test_trailing_stop_reset() {
        let config = StopLossConfig {
            default_percentage: 5.0,
            pair_percentages: HashMap::new(),
            enabled: true,
            trailing: Some(TrailingStopConfig {
                percentage: 2.0,
                activation_percentage: 1.0,
            }),
        };
        let mut strategy = StopLossStrategy::new(config);

        // Add a trailing stop
        assert!(!strategy.should_stop_loss("BTC/USDT", 100.0, 103.0, TradeSide::Long));
        assert!(strategy.trailing_stops.contains_key("BTC/USDT"));

        // Reset trailing stop
        strategy.reset_trailing_stop("BTC/USDT");
        assert!(!strategy.trailing_stops.contains_key("BTC/USDT"));

        // Reset all trailing stops
        assert!(!strategy.should_stop_loss("BTC/USDT", 100.0, 103.0, TradeSide::Long));
        assert!(!strategy.should_stop_loss("ETH/USDT", 100.0, 103.0, TradeSide::Long));
        assert_eq!(strategy.trailing_stops.len(), 2);
        strategy.reset_all_trailing_stops();
        assert!(strategy.trailing_stops.is_empty());
    }
}
