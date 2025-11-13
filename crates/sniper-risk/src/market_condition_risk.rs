//! Market condition-based dynamic risk adjustment module.
//!
//! This module provides functionality for adjusting risk parameters based on
//! various market conditions including volatility, trend analysis, and liquidity.

use serde::{Deserialize, Serialize};
// use tracing::{debug, info}; // Unused imports

/// Configuration for market condition-based risk adjustment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketConditionRiskConfig {
    /// Enable/disable market condition-based risk adjustment
    pub enabled: bool,
    /// Base risk multiplier when conditions are normal
    pub base_risk_multiplier: f64,
    /// Risk multiplier adjustment for high volatility
    pub high_volatility_multiplier: f64,
    /// Risk multiplier adjustment for low volatility
    pub low_volatility_multiplier: f64,
    /// Risk multiplier adjustment for bull markets
    pub bull_market_multiplier: f64,
    /// Risk multiplier adjustment for bear markets
    pub bear_market_multiplier: f64,
    /// Risk multiplier adjustment for sideways markets
    pub sideways_market_multiplier: f64,
    /// Threshold for high volatility (as multiple of normal)
    pub high_volatility_threshold: f64,
    /// Threshold for low volatility (as multiple of normal)
    pub low_volatility_threshold: f64,
}

impl Default for MarketConditionRiskConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            base_risk_multiplier: 1.0,
            high_volatility_multiplier: 0.7,
            low_volatility_multiplier: 1.2,
            bull_market_multiplier: 1.1,
            bear_market_multiplier: 0.8,
            sideways_market_multiplier: 0.9,
            high_volatility_threshold: 1.5,
            low_volatility_threshold: 0.7,
        }
    }
}

/// Market conditions data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketConditions {
    /// Current market volatility
    pub volatility: f64,
    /// Normal market volatility for comparison
    pub normal_volatility: f64,
    /// Current market trend
    pub trend: MarketTrend,
    /// Current market liquidity
    pub liquidity: f64,
    /// Normal market liquidity for comparison
    pub normal_liquidity: f64,
    /// Timestamp of the data
    pub timestamp: u64,
}

/// Market trend types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MarketTrend {
    Bull,
    Bear,
    Sideways,
}

/// Market condition-based risk adjuster
pub struct MarketConditionRiskAdjuster {
    config: MarketConditionRiskConfig,
    market_conditions_history: Vec<MarketConditions>,
}

impl MarketConditionRiskAdjuster {
    /// Create a new market condition risk adjuster
    pub fn new(config: MarketConditionRiskConfig) -> Self {
        Self {
            config,
            market_conditions_history: Vec::new(),
        }
    }

    /// Update market conditions data
    pub fn update_market_conditions(&mut self, conditions: MarketConditions) {
        self.market_conditions_history.push(conditions);

        // Keep only recent data (last 100 entries)
        if self.market_conditions_history.len() > 100 {
            self.market_conditions_history
                .drain(0..self.market_conditions_history.len() - 100);
        }
    }

    /// Calculate dynamic risk multiplier based on market conditions
    pub fn calculate_risk_multiplier(
        &self,
        portfolio_drawdown: f64,
        drawdown_threshold: f64,
    ) -> MarketRiskMultiplierResult {
        if !self.config.enabled {
            return MarketRiskMultiplierResult {
                multiplier: self.config.base_risk_multiplier,
                components: vec![],
                reason: "Market condition risk adjustment disabled".to_string(),
            };
        }

        // Get the most recent market conditions
        let current_conditions = match self.market_conditions_history.last() {
            Some(conditions) => conditions,
            None => {
                return MarketRiskMultiplierResult {
                    multiplier: self.config.base_risk_multiplier,
                    components: vec![],
                    reason: "No market conditions data available".to_string(),
                };
            }
        };

        let mut components = Vec::new();
        let mut total_multiplier = self.config.base_risk_multiplier;

        // Apply drawdown-based adjustment (existing functionality)
        if portfolio_drawdown >= drawdown_threshold {
            let drawdown_multiplier = 0.5; // Hardcoded for now, could be configurable
            total_multiplier *= drawdown_multiplier;
            components.push(RiskMultiplierComponent {
                factor: "PortfolioDrawdown".to_string(),
                value: drawdown_multiplier,
                description: format!(
                    "Portfolio drawdown {:.2}% exceeds threshold {:.2}%",
                    portfolio_drawdown, drawdown_threshold
                ),
            });
        }

        // Apply volatility-based adjustment
        let volatility_ratio = current_conditions.volatility / current_conditions.normal_volatility;
        let volatility_multiplier = if volatility_ratio >= self.config.high_volatility_threshold {
            components.push(RiskMultiplierComponent {
                factor: "HighVolatility".to_string(),
                value: self.config.high_volatility_multiplier,
                description: format!("High volatility {:.2}x normal detected", volatility_ratio),
            });
            self.config.high_volatility_multiplier
        } else if volatility_ratio <= self.config.low_volatility_threshold {
            components.push(RiskMultiplierComponent {
                factor: "LowVolatility".to_string(),
                value: self.config.low_volatility_multiplier,
                description: format!("Low volatility {:.2}x normal detected", volatility_ratio),
            });
            self.config.low_volatility_multiplier
        } else {
            components.push(RiskMultiplierComponent {
                factor: "NormalVolatility".to_string(),
                value: 1.0,
                description: format!("Normal volatility {:.2}x normal", volatility_ratio),
            });
            1.0
        };
        total_multiplier *= volatility_multiplier;

        // Apply trend-based adjustment
        let trend_multiplier = match current_conditions.trend {
            MarketTrend::Bull => {
                components.push(RiskMultiplierComponent {
                    factor: "BullMarket".to_string(),
                    value: self.config.bull_market_multiplier,
                    description: "Bull market conditions detected".to_string(),
                });
                self.config.bull_market_multiplier
            }
            MarketTrend::Bear => {
                components.push(RiskMultiplierComponent {
                    factor: "BearMarket".to_string(),
                    value: self.config.bear_market_multiplier,
                    description: "Bear market conditions detected".to_string(),
                });
                self.config.bear_market_multiplier
            }
            MarketTrend::Sideways => {
                components.push(RiskMultiplierComponent {
                    factor: "SidewaysMarket".to_string(),
                    value: self.config.sideways_market_multiplier,
                    description: "Sideways market conditions detected".to_string(),
                });
                self.config.sideways_market_multiplier
            }
        };
        total_multiplier *= trend_multiplier;

        // Apply liquidity-based adjustment (simplified)
        let liquidity_ratio = current_conditions.liquidity / current_conditions.normal_liquidity;
        let liquidity_multiplier = if liquidity_ratio < 0.5 {
            let multiplier = 0.8;
            components.push(RiskMultiplierComponent {
                factor: "LowLiquidity".to_string(),
                value: multiplier,
                description: format!("Low liquidity {:.2}x normal detected", liquidity_ratio),
            });
            multiplier
        } else if liquidity_ratio > 2.0 {
            let multiplier = 1.1;
            components.push(RiskMultiplierComponent {
                factor: "HighLiquidity".to_string(),
                value: multiplier,
                description: format!("High liquidity {:.2}x normal detected", liquidity_ratio),
            });
            multiplier
        } else {
            components.push(RiskMultiplierComponent {
                factor: "NormalLiquidity".to_string(),
                value: 1.0,
                description: format!("Normal liquidity {:.2}x normal", liquidity_ratio),
            });
            1.0
        };
        total_multiplier *= liquidity_multiplier;

        // Ensure multiplier doesn't go too low
        total_multiplier = total_multiplier.max(0.1);

        MarketRiskMultiplierResult {
            multiplier: total_multiplier,
            components,
            reason: "Market condition-based risk multiplier calculated".to_string(),
        }
    }

    /// Get recent market conditions history
    pub fn get_market_conditions_history(&self) -> &Vec<MarketConditions> {
        &self.market_conditions_history
    }

    /// Update configuration
    pub fn update_config(&mut self, config: MarketConditionRiskConfig) {
        self.config = config;
    }
}

/// Risk multiplier component for detailed analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskMultiplierComponent {
    /// Factor name
    pub factor: String,
    /// Multiplier value
    pub value: f64,
    /// Description of why this factor was applied
    pub description: String,
}

/// Result of market risk multiplier calculation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketRiskMultiplierResult {
    /// Calculated risk multiplier
    pub multiplier: f64,
    /// Components that contributed to the multiplier
    pub components: Vec<RiskMultiplierComponent>,
    /// Reason for the result
    pub reason: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_market_condition_risk_adjuster_creation() {
        let config = MarketConditionRiskConfig::default();
        let adjuster = MarketConditionRiskAdjuster::new(config);

        assert!(adjuster.market_conditions_history.is_empty());
    }

    #[test]
    fn test_market_conditions_update() {
        let config = MarketConditionRiskConfig::default();
        let mut adjuster = MarketConditionRiskAdjuster::new(config);

        let conditions = MarketConditions {
            volatility: 0.15,
            normal_volatility: 0.1,
            trend: MarketTrend::Bull,
            liquidity: 1000000.0,
            normal_liquidity: 800000.0,
            timestamp: 1234567890,
        };

        adjuster.update_market_conditions(conditions.clone());
        assert_eq!(adjuster.market_conditions_history.len(), 1);
        assert_eq!(adjuster.market_conditions_history[0].volatility, 0.15);
    }

    #[test]
    fn test_risk_multiplier_calculation_normal_conditions() {
        let config = MarketConditionRiskConfig::default();
        let mut adjuster = MarketConditionRiskAdjuster::new(config);

        let conditions = MarketConditions {
            volatility: 0.1, // Normal volatility
            normal_volatility: 0.1,
            trend: MarketTrend::Sideways,
            liquidity: 800000.0, // Normal liquidity
            normal_liquidity: 800000.0,
            timestamp: 1234567890,
        };

        adjuster.update_market_conditions(conditions);

        let result = adjuster.calculate_risk_multiplier(0.0, 5.0); // No drawdown
        assert_eq!(result.multiplier, 0.9); // Base (1.0) * Sideways (0.9)
        assert_eq!(result.components.len(), 3); // Drawdown component (1.0) + Sideways component (0.9) + Normal liquidity (1.0)
    }

    #[test]
    fn test_risk_multiplier_calculation_high_volatility() {
        let config = MarketConditionRiskConfig::default();
        let mut adjuster = MarketConditionRiskAdjuster::new(config);

        let conditions = MarketConditions {
            volatility: 0.2, // High volatility (2x normal)
            normal_volatility: 0.1,
            trend: MarketTrend::Bull,
            liquidity: 800000.0,
            normal_liquidity: 800000.0,
            timestamp: 1234567890,
        };

        adjuster.update_market_conditions(conditions);

        let result = adjuster.calculate_risk_multiplier(0.0, 5.0);
        // Base (1.0) * High Volatility (0.7) * Bull (1.1) = 0.77
        assert!((result.multiplier - 0.77).abs() < 0.001);
        assert!(result
            .components
            .iter()
            .any(|c| c.factor == "HighVolatility"));
    }

    #[test]
    fn test_risk_multiplier_calculation_drawdown() {
        let config = MarketConditionRiskConfig::default();
        let mut adjuster = MarketConditionRiskAdjuster::new(config);

        let conditions = MarketConditions {
            volatility: 0.1,
            normal_volatility: 0.1,
            trend: MarketTrend::Sideways,
            liquidity: 800000.0,
            normal_liquidity: 800000.0,
            timestamp: 1234567890,
        };

        adjuster.update_market_conditions(conditions);

        let result = adjuster.calculate_risk_multiplier(6.0, 5.0); // 6% drawdown exceeds 5% threshold
                                                                   // Drawdown (0.5) * Base (1.0) * Sideways (0.9) = 0.45
        assert!((result.multiplier - 0.45).abs() < 0.001);
        assert!(result
            .components
            .iter()
            .any(|c| c.factor == "PortfolioDrawdown"));
    }

    #[test]
    fn test_disabled_adjuster() {
        let config = MarketConditionRiskConfig {
            enabled: false,
            ..Default::default()
        };
        let adjuster = MarketConditionRiskAdjuster::new(config);

        let result = adjuster.calculate_risk_multiplier(0.0, 5.0);
        assert_eq!(result.multiplier, 1.0); // Base multiplier
        assert_eq!(result.reason, "Market condition risk adjustment disabled");
    }

    #[test]
    fn test_no_market_data() {
        let config = MarketConditionRiskConfig::default();
        let adjuster = MarketConditionRiskAdjuster::new(config);

        let result = adjuster.calculate_risk_multiplier(0.0, 5.0);
        assert_eq!(result.multiplier, 1.0); // Base multiplier
        assert_eq!(result.reason, "No market conditions data available");
        assert!(result.components.is_empty());
    }
}
