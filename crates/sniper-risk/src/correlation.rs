//! Correlation analysis module for risk management.
//!
//! This module provides functionality for monitoring asset correlations
//! to prevent overexposure and manage portfolio risk.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration for correlation analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelationConfig {
    /// Maximum allowed correlation between assets
    pub max_correlation: f64,
    /// Time window for correlation calculation (in hours)
    pub time_window_hours: u64,
    /// Enable/disable correlation analysis
    pub enabled: bool,
}

/// Asset correlation data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetCorrelation {
    pub asset_a: String,
    pub asset_b: String,
    pub correlation: f64,
    pub timestamp: u64,
}

/// Correlation analysis system
pub struct CorrelationAnalyzer {
    config: CorrelationConfig,
    correlations: HashMap<String, f64>,
}

impl CorrelationAnalyzer {
    /// Create a new correlation analyzer
    pub fn new(config: CorrelationConfig) -> Self {
        Self {
            config,
            correlations: HashMap::new(),
        }
    }

    /// Update correlation data for a pair of assets
    pub fn update_correlation(&mut self, asset_a: &str, asset_b: &str, correlation: f64) {
        if !self.config.enabled {
            return;
        }

        let key = format!("{}-{}", asset_a, asset_b);
        self.correlations.insert(key, correlation);
    }

    /// Check if a trade would violate correlation limits
    pub fn check_correlation_risk(&self, token_in: &str, token_out: &str) -> CorrelationRiskResult {
        if !self.config.enabled {
            return CorrelationRiskResult {
                allowed: true,
                reason: "Correlation analysis disabled".to_string(),
                correlation: 0.0,
            };
        }

        let key = format!("{}-{}", token_in, token_out);
        if let Some(correlation) = self.correlations.get(&key) {
            if *correlation > self.config.max_correlation {
                CorrelationRiskResult {
                    allowed: false,
                    reason: format!(
                        "Correlation {:.2} exceeds maximum {:.2}",
                        correlation, self.config.max_correlation
                    ),
                    correlation: *correlation,
                }
            } else {
                CorrelationRiskResult {
                    allowed: true,
                    reason: format!("Correlation {:.2} within limits", correlation),
                    correlation: *correlation,
                }
            }
        } else {
            // No correlation data available, allow the trade
            CorrelationRiskResult {
                allowed: true,
                reason: "No correlation data available".to_string(),
                correlation: 0.0,
            }
        }
    }

    /// Get all current correlations
    pub fn get_correlations(&self) -> &HashMap<String, f64> {
        &self.correlations
    }
}

/// Result of a correlation risk check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelationRiskResult {
    /// Whether the trade is allowed
    pub allowed: bool,
    /// Reason for the decision
    pub reason: String,
    /// The correlation value that was checked
    pub correlation: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_correlation_analyzer() {
        let config = CorrelationConfig {
            max_correlation: 0.8,
            time_window_hours: 24,
            enabled: true,
        };

        let mut analyzer = CorrelationAnalyzer::new(config);

        // Update correlation data
        analyzer.update_correlation("ETH", "BTC", 0.75);
        analyzer.update_correlation("ETH", "LINK", 0.9);

        // Check correlation risk
        let result1 = analyzer.check_correlation_risk("ETH", "BTC");
        assert!(result1.allowed);
        assert_eq!(result1.correlation, 0.75);

        let result2 = analyzer.check_correlation_risk("ETH", "LINK");
        assert!(!result2.allowed);
        assert_eq!(result2.correlation, 0.9);

        let result3 = analyzer.check_correlation_risk("ETH", "UNI");
        assert!(result3.allowed);
        assert_eq!(result3.correlation, 0.0);
    }

    #[test]
    fn test_correlation_analyzer_disabled() {
        let config = CorrelationConfig {
            max_correlation: 0.8,
            time_window_hours: 24,
            enabled: false,
        };

        let mut analyzer = CorrelationAnalyzer::new(config);
        analyzer.update_correlation("ETH", "BTC", 0.9);

        let result = analyzer.check_correlation_risk("ETH", "BTC");
        assert!(result.allowed);
        assert_eq!(result.reason, "Correlation analysis disabled");
    }
}
