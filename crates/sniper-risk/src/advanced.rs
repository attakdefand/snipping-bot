//! Advanced risk controls and position sizing methods.
//! 
//! This module provides sophisticated risk management capabilities including
//! advanced position sizing, portfolio-level risk controls, and dynamic risk adjustment.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Advanced risk configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvancedRiskConfig {
    /// Enable/disable advanced risk controls
    pub enabled: bool,
    /// Position sizing method
    pub position_sizing: PositionSizingMethod,
    /// Portfolio-level risk controls
    pub portfolio_controls: PortfolioRiskControls,
    /// Dynamic risk adjustment settings
    pub dynamic_adjustment: DynamicRiskAdjustment,
}

/// Position sizing methods
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PositionSizingMethod {
    /// Fixed percentage of capital
    FixedPercentage { percentage: f64 },
    /// Volatility-adjusted position sizing
    VolatilityAdjusted { 
        target_volatility: f64, 
        max_position_size: f64 
    },
    /// Kelly criterion with cap
    KellyCriterion { 
        kelly_multiplier: f64, 
        max_position_size: f64 
    },
    /// Risk parity approach
    RiskParity { 
        target_risk_contribution: f64 
    },
}

/// Portfolio-level risk controls
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioRiskControls {
    /// Maximum portfolio exposure as percentage of capital
    pub max_portfolio_exposure_pct: f64,
    /// Maximum correlation with existing positions
    pub max_position_correlation: f64,
    /// Maximum number of concurrent positions
    pub max_concurrent_positions: usize,
    /// Sector/asset class exposure limits
    pub exposure_limits: HashMap<String, f64>,
}

/// Dynamic risk adjustment settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicRiskAdjustment {
    /// Enable/disable dynamic adjustment
    pub enabled: bool,
    /// Drawdown threshold for risk reduction
    pub drawdown_threshold_pct: f64,
    /// Risk reduction factor when threshold is breached
    pub risk_reduction_factor: f64,
    /// Recovery threshold to restore normal risk levels
    pub recovery_threshold_pct: f64,
}

/// Current portfolio state for risk calculations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioState {
    /// Current portfolio value
    pub portfolio_value: f64,
    /// Current unrealized PnL
    pub unrealized_pnl: f64,
    /// Current drawdown percentage
    pub current_drawdown_pct: f64,
    /// Current positions
    pub positions: Vec<PositionInfo>,
    /// Historical volatility data
    pub volatility_data: HashMap<String, f64>,
    /// Correlation matrix
    pub correlations: HashMap<String, HashMap<String, f64>>,
}

/// Information about a current position
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionInfo {
    /// Asset symbol
    pub symbol: String,
    /// Position size in USD
    pub size_usd: f64,
    /// Entry price
    pub entry_price: f64,
    /// Current price
    pub current_price: f64,
    /// Position PnL
    pub pnl: f64,
    /// Position volatility
    pub volatility: f64,
    /// Asset sector/classification
    pub sector: String,
}

/// Advanced risk analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvancedRiskResult {
    /// Whether the trade is allowed
    pub allowed: bool,
    /// Maximum position size allowed for this trade
    pub max_position_size: f64,
    /// Risk-adjusted position size
    pub risk_adjusted_size: f64,
    /// Reasons for the decision
    pub reasons: Vec<String>,
    /// Risk metrics for the proposed trade
    pub metrics: RiskMetrics,
}

/// Risk metrics for a proposed trade
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskMetrics {
    /// Portfolio exposure after trade
    pub portfolio_exposure_pct: f64,
    /// Correlation with existing positions
    pub position_correlation: f64,
    /// Volatility of the proposed position
    pub position_volatility: f64,
    /// Risk contribution of the position
    pub risk_contribution: f64,
    /// Dynamic risk multiplier applied
    pub risk_multiplier: f64,
}

/// Advanced risk analyzer
pub struct AdvancedRiskAnalyzer {
    config: AdvancedRiskConfig,
}

impl AdvancedRiskAnalyzer {
    /// Create a new advanced risk analyzer
    pub fn new(config: AdvancedRiskConfig) -> Self {
        Self { config }
    }
    
    /// Analyze a proposed trade with advanced risk controls
    pub fn analyze_trade(
        &self,
        proposed_asset: &str,
        proposed_size: f64,
        portfolio_state: &PortfolioState,
    ) -> AdvancedRiskResult {
        if !self.config.enabled {
            return AdvancedRiskResult {
                allowed: true,
                max_position_size: f64::INFINITY,
                risk_adjusted_size: proposed_size,
                reasons: vec!["Advanced risk controls disabled".to_string()],
                metrics: RiskMetrics {
                    portfolio_exposure_pct: 0.0,
                    position_correlation: 0.0,
                    position_volatility: 0.0,
                    risk_contribution: 0.0,
                    risk_multiplier: 1.0,
                },
            };
        }
        
        // Calculate dynamic risk multiplier based on current drawdown
        let risk_multiplier = self.calculate_dynamic_risk_multiplier(portfolio_state);
        
        // Check portfolio-level constraints
        let portfolio_exposure = self.calculate_portfolio_exposure(
            proposed_size,
            portfolio_state
        );
        
        if portfolio_exposure > self.config.portfolio_controls.max_portfolio_exposure_pct {
            return AdvancedRiskResult {
                allowed: false,
                max_position_size: 0.0,
                risk_adjusted_size: 0.0,
                reasons: vec![format!(
                    "Portfolio exposure {:.2}% exceeds maximum {:.2}%",
                    portfolio_exposure,
                    self.config.portfolio_controls.max_portfolio_exposure_pct
                )],
                metrics: RiskMetrics {
                    portfolio_exposure_pct: portfolio_exposure,
                    position_correlation: 0.0,
                    position_volatility: 0.0,
                    risk_contribution: 0.0,
                    risk_multiplier,
                },
            };
        }
        
        // Check correlation with existing positions
        let position_correlation = self.calculate_position_correlation(
            proposed_asset,
            portfolio_state
        );
        
        if position_correlation > self.config.portfolio_controls.max_position_correlation {
            return AdvancedRiskResult {
                allowed: false,
                max_position_size: 0.0,
                risk_adjusted_size: 0.0,
                reasons: vec![format!(
                    "Position correlation {:.2} exceeds maximum {:.2}",
                    position_correlation,
                    self.config.portfolio_controls.max_position_correlation
                )],
                metrics: RiskMetrics {
                    portfolio_exposure_pct: portfolio_exposure,
                    position_correlation,
                    position_volatility: 0.0,
                    risk_contribution: 0.0,
                    risk_multiplier,
                },
            };
        }
        
        // Check concurrent position limits
        if portfolio_state.positions.len() >= self.config.portfolio_controls.max_concurrent_positions {
            return AdvancedRiskResult {
                allowed: false,
                max_position_size: 0.0,
                risk_adjusted_size: 0.0,
                reasons: vec![format!(
                    "Maximum concurrent positions {} reached",
                    self.config.portfolio_controls.max_concurrent_positions
                )],
                metrics: RiskMetrics {
                    portfolio_exposure_pct: portfolio_exposure,
                    position_correlation,
                    position_volatility: 0.0,
                    risk_contribution: 0.0,
                    risk_multiplier,
                },
            };
        }
        
        // Calculate maximum position size based on sizing method
        let max_position_size = self.calculate_max_position_size(
            proposed_asset,
            portfolio_state,
            risk_multiplier
        );
        
        // Apply risk-adjusted sizing
        let risk_adjusted_size = proposed_size.min(max_position_size);
        
        AdvancedRiskResult {
            allowed: true,
            max_position_size,
            risk_adjusted_size,
            reasons: vec!["Trade approved by advanced risk controls".to_string()],
            metrics: RiskMetrics {
                portfolio_exposure_pct: portfolio_exposure,
                position_correlation,
                position_volatility: self.get_asset_volatility(proposed_asset, portfolio_state),
                risk_contribution: self.calculate_risk_contribution(
                    proposed_asset,
                    proposed_size,
                    portfolio_state
                ),
                risk_multiplier,
            },
        }
    }
    
    /// Calculate dynamic risk multiplier based on current drawdown
    fn calculate_dynamic_risk_multiplier(&self, portfolio_state: &PortfolioState) -> f64 {
        if !self.config.dynamic_adjustment.enabled {
            return 1.0;
        }
        
        let drawdown = portfolio_state.current_drawdown_pct;
        
        if drawdown >= self.config.dynamic_adjustment.drawdown_threshold_pct {
            self.config.dynamic_adjustment.risk_reduction_factor
        } else if drawdown <= self.config.dynamic_adjustment.recovery_threshold_pct {
            1.0 // Full risk restored
        } else {
            // Linear interpolation between recovery and reduction thresholds
            let range = self.config.dynamic_adjustment.drawdown_threshold_pct - 
                       self.config.dynamic_adjustment.recovery_threshold_pct;
            let progress = (drawdown - self.config.dynamic_adjustment.recovery_threshold_pct) / range;
            1.0 - (progress * (1.0 - self.config.dynamic_adjustment.risk_reduction_factor))
        }
    }
    
    /// Calculate portfolio exposure after proposed trade
    fn calculate_portfolio_exposure(&self, proposed_size: f64, portfolio_state: &PortfolioState) -> f64 {
        let current_exposure: f64 = portfolio_state.positions.iter()
            .map(|p| p.size_usd)
            .sum();
        let total_exposure = current_exposure + proposed_size;
        (total_exposure / portfolio_state.portfolio_value) * 100.0
    }
    
    /// Calculate correlation with existing positions
    fn calculate_position_correlation(&self, asset: &str, portfolio_state: &PortfolioState) -> f64 {
        if portfolio_state.positions.is_empty() {
            return 0.0;
        }
        
        let mut correlations = Vec::new();
        for position in &portfolio_state.positions {
            if let Some(asset_correlations) = portfolio_state.correlations.get(asset) {
                if let Some(correlation) = asset_correlations.get(&position.symbol) {
                    correlations.push(*correlation);
                }
            }
        }
        
        if correlations.is_empty() {
            0.0
        } else {
            correlations.iter().sum::<f64>() / correlations.len() as f64
        }
    }
    
    /// Calculate maximum position size based on sizing method
    fn calculate_max_position_size(
        &self,
        asset: &str,
        portfolio_state: &PortfolioState,
        risk_multiplier: f64,
    ) -> f64 {
        match &self.config.position_sizing {
            PositionSizingMethod::FixedPercentage { percentage } => {
                portfolio_state.portfolio_value * percentage / 100.0 * risk_multiplier
            },
            PositionSizingMethod::VolatilityAdjusted { 
                target_volatility, 
                max_position_size 
            } => {
                let asset_volatility = self.get_asset_volatility(asset, portfolio_state);
                if asset_volatility > 0.0 {
                    let size = (target_volatility / asset_volatility) * portfolio_state.portfolio_value * risk_multiplier;
                    size.min(portfolio_state.portfolio_value * max_position_size / 100.0)
                } else {
                    portfolio_state.portfolio_value * max_position_size / 100.0 * risk_multiplier
                }
            },
            PositionSizingMethod::KellyCriterion { 
                kelly_multiplier, 
                max_position_size 
            } => {
                // Simplified Kelly - in practice this would use actual edge and odds
                let kelly_fraction = 0.1; // Placeholder
                let size = kelly_fraction * kelly_multiplier * portfolio_state.portfolio_value * risk_multiplier;
                size.min(portfolio_state.portfolio_value * max_position_size / 100.0)
            },
            PositionSizingMethod::RiskParity { 
                target_risk_contribution 
            } => {
                // Simplified risk parity - in practice this would be more complex
                portfolio_state.portfolio_value * target_risk_contribution / 100.0 * risk_multiplier
            },
        }
    }
    
    /// Get volatility for an asset
    fn get_asset_volatility(&self, asset: &str, portfolio_state: &PortfolioState) -> f64 {
        *portfolio_state.volatility_data.get(asset).unwrap_or(&0.0)
    }
    
    /// Calculate risk contribution of a position
    fn calculate_risk_contribution(
        &self,
        asset: &str,
        size: f64,
        portfolio_state: &PortfolioState,
    ) -> f64 {
        let asset_volatility = self.get_asset_volatility(asset, portfolio_state);
        let position_risk = size * asset_volatility;
        
        if portfolio_state.positions.is_empty() {
            return position_risk;
        }
        
        let total_portfolio_risk: f64 = portfolio_state.positions.iter()
            .map(|p| p.size_usd * p.volatility)
            .sum();
            
        if total_portfolio_risk > 0.0 {
            (position_risk / total_portfolio_risk) * 100.0
        } else {
            0.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_advanced_risk_analyzer() {
        let config = AdvancedRiskConfig {
            enabled: true,
            position_sizing: PositionSizingMethod::FixedPercentage { percentage: 2.0 },
            portfolio_controls: PortfolioRiskControls {
                max_portfolio_exposure_pct: 25.0,
                max_position_correlation: 0.7,
                max_concurrent_positions: 10,
                exposure_limits: HashMap::new(),
            },
            dynamic_adjustment: DynamicRiskAdjustment {
                enabled: true,
                drawdown_threshold_pct: 5.0,
                risk_reduction_factor: 0.5,
                recovery_threshold_pct: 1.0,
            },
        };
        
        let analyzer = AdvancedRiskAnalyzer::new(config);
        
        let portfolio_state = PortfolioState {
            portfolio_value: 100000.0,
            unrealized_pnl: 0.0,
            current_drawdown_pct: 2.0,
            positions: vec![],
            volatility_data: HashMap::new(),
            correlations: HashMap::new(),
        };
        
        let result = analyzer.analyze_trade("ETH", 1000.0, &portfolio_state);
        
        assert!(result.allowed);
        assert!(result.risk_adjusted_size > 0.0);
        // With 2% drawdown and thresholds at 1% and 5%, multiplier should be between 0.5 and 1.0
        assert!(result.metrics.risk_multiplier >= 0.5 && result.metrics.risk_multiplier <= 1.0);
    }
    
    #[test]
    fn test_dynamic_risk_multiplier() {
        let config = AdvancedRiskConfig {
            enabled: true,
            position_sizing: PositionSizingMethod::FixedPercentage { percentage: 2.0 },
            portfolio_controls: PortfolioRiskControls {
                max_portfolio_exposure_pct: 25.0,
                max_position_correlation: 0.7,
                max_concurrent_positions: 10,
                exposure_limits: HashMap::new(),
            },
            dynamic_adjustment: DynamicRiskAdjustment {
                enabled: true,
                drawdown_threshold_pct: 5.0,
                risk_reduction_factor: 0.5,
                recovery_threshold_pct: 1.0,
            },
        };
        
        let analyzer = AdvancedRiskAnalyzer::new(config);
        
        let mut portfolio_state = PortfolioState {
            portfolio_value: 100000.0,
            unrealized_pnl: 0.0,
            current_drawdown_pct: 0.0,
            positions: vec![],
            volatility_data: HashMap::new(),
            correlations: HashMap::new(),
        };
        
        // Test normal conditions
        assert_eq!(analyzer.calculate_dynamic_risk_multiplier(&portfolio_state), 1.0);
        
        // Test at drawdown threshold
        portfolio_state.current_drawdown_pct = 5.0;
        assert_eq!(analyzer.calculate_dynamic_risk_multiplier(&portfolio_state), 0.5);
        
        // Test at recovery threshold
        portfolio_state.current_drawdown_pct = 1.0;
        assert_eq!(analyzer.calculate_dynamic_risk_multiplier(&portfolio_state), 1.0);
        
        // Test midpoint (3% drawdown between 1% recovery and 5% threshold)
        portfolio_state.current_drawdown_pct = 3.0;
        let multiplier = analyzer.calculate_dynamic_risk_multiplier(&portfolio_state);
        // Linear interpolation: (3-1)/(5-1) = 0.5 progress, so multiplier = 1.0 - (0.5 * (1.0 - 0.5)) = 0.75
        assert!((multiplier - 0.75).abs() < 0.001);
    }
}