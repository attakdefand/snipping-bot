//! Trading limits enforcement implementation
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info, warn};

/// Trading limits configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingLimitsConfig {
    /// Enable/disable trading limits enforcement
    pub enabled: bool,
    /// Maximum position size per trade (in USD)
    pub max_position_size_usd: f64,
    /// Maximum position size per trade (as percentage of portfolio)
    pub max_position_size_pct: f64,
    /// Maximum daily trading volume (in USD)
    pub max_daily_volume_usd: f64,
    /// Maximum number of trades per day
    pub max_trades_per_day: usize,
    /// Maximum loss per day (in USD)
    pub max_daily_loss_usd: f64,
    /// Maximum loss per day (as percentage of portfolio)
    pub max_daily_loss_pct: f64,
    /// Maximum exposure to a single asset (as percentage of portfolio)
    pub max_asset_exposure_pct: f64,
    /// Maximum exposure to a single sector (as percentage of portfolio)
    pub max_sector_exposure_pct: f64,
    /// Time window for limit calculations (in seconds)
    pub time_window_seconds: u64,
}

impl Default for TradingLimitsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_position_size_usd: 10000.0, // $10,000 max per trade
            max_position_size_pct: 5.0,     // 5% of portfolio max per trade
            max_daily_volume_usd: 100000.0, // $100,000 max daily volume
            max_trades_per_day: 50,         // Max 50 trades per day
            max_daily_loss_usd: 5000.0,     // $5,000 max daily loss
            max_daily_loss_pct: 2.0,        // 2% of portfolio max daily loss
            max_asset_exposure_pct: 10.0,   // 10% max exposure to single asset
            max_sector_exposure_pct: 20.0,  // 20% max exposure to single sector
            time_window_seconds: 86400,     // 24 hours
        }
    }
}

/// Trading activity record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingActivity {
    /// Trade identifier
    pub trade_id: String,
    /// Asset symbol
    pub asset: String,
    /// Sector/classification
    pub sector: String,
    /// Trade size in USD
    pub size_usd: f64,
    /// Trade profit/loss
    pub pnl_usd: f64,
    /// Timestamp
    pub timestamp: u64,
}

/// Current position information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionInfo {
    /// Asset symbol
    pub asset: String,
    /// Sector/classification
    pub sector: String,
    /// Position size in USD
    pub size_usd: f64,
    /// Entry price
    pub entry_price: f64,
    /// Current price
    pub current_price: f64,
}

/// Current portfolio state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioState {
    /// Current portfolio value in USD
    pub portfolio_value_usd: f64,
    /// Current positions
    pub positions: Vec<PositionInfo>,
    /// Historical trading activity
    pub trading_history: Vec<TradingActivity>,
}

/// Trading limit check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LimitCheckResult {
    /// Whether the trade is allowed
    pub allowed: bool,
    /// Reasons for the decision
    pub reasons: Vec<String>,
    /// Current usage statistics
    pub usage_stats: UsageStats,
}

/// Usage statistics for current limits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageStats {
    /// Current daily volume
    pub daily_volume_usd: f64,
    /// Current daily trade count
    pub daily_trades: usize,
    /// Current daily loss
    pub daily_loss_usd: f64,
    /// Current exposure to proposed asset
    pub asset_exposure_pct: f64,
    /// Current exposure to proposed sector
    pub sector_exposure_pct: f64,
}

/// Trading limits enforcer
pub struct TradingLimitsEnforcer {
    /// Configuration
    config: TradingLimitsConfig,
    /// Current portfolio state
    portfolio_state: PortfolioState,
    /// Daily usage tracking
    daily_usage: DailyUsage,
}

/// Daily usage tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyUsage {
    /// Daily trading volume
    pub volume_usd: f64,
    /// Daily trade count
    pub trade_count: usize,
    /// Daily losses
    pub losses_usd: f64,
    /// Last reset timestamp
    pub last_reset: u64,
}

impl TradingLimitsEnforcer {
    /// Create a new trading limits enforcer
    pub fn new(config: TradingLimitsConfig, portfolio_state: PortfolioState) -> Self {
        Self {
            config,
            portfolio_state,
            daily_usage: DailyUsage {
                volume_usd: 0.0,
                trade_count: 0,
                losses_usd: 0.0,
                last_reset: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            },
        }
    }

    /// Check if a proposed trade is within limits
    ///
    /// # Arguments
    /// * `trade_id` - Unique identifier for the trade
    /// * `asset` - Asset symbol to trade
    /// * `sector` - Sector/classification of the asset
    /// * `size_usd` - Size of the trade in USD
    /// * `expected_pnl` - Expected profit/loss of the trade
    ///
    /// # Returns
    /// * `Result<LimitCheckResult>` - Limit check result
    pub fn check_trade_limits(
        &mut self,
        trade_id: &str,
        asset: &str,
        sector: &str,
        size_usd: f64,
        expected_pnl: f64,
    ) -> Result<LimitCheckResult> {
        debug!("Checking trading limits for trade: {}", trade_id);

        if !self.config.enabled {
            return Ok(LimitCheckResult {
                allowed: true,
                reasons: vec!["Trading limits enforcement disabled".to_string()],
                usage_stats: self.calculate_usage_stats(asset, sector, size_usd)?,
            });
        }

        // Reset daily usage if needed
        self.reset_daily_usage_if_needed()?;

        let mut reasons = Vec::new();
        let usage_stats = self.calculate_usage_stats(asset, sector, size_usd)?;

        // Check position size limits
        if size_usd > self.config.max_position_size_usd {
            reasons.push(format!(
                "Trade size ${:.2} exceeds maximum ${:.2}",
                size_usd, self.config.max_position_size_usd
            ));
        }

        let position_size_pct = (size_usd / self.portfolio_state.portfolio_value_usd) * 100.0;
        if position_size_pct > self.config.max_position_size_pct {
            reasons.push(format!(
                "Trade size {:.2}% exceeds maximum {:.2}% of portfolio",
                position_size_pct, self.config.max_position_size_pct
            ));
        }

        // Check daily volume limit
        if usage_stats.daily_volume_usd > self.config.max_daily_volume_usd {
            reasons.push(format!(
                "Daily volume ${:.2} exceeds maximum ${:.2}",
                usage_stats.daily_volume_usd, self.config.max_daily_volume_usd
            ));
        }

        // Check daily trade count limit
        if usage_stats.daily_trades > self.config.max_trades_per_day {
            reasons.push(format!(
                "Daily trade count {} exceeds maximum {}",
                usage_stats.daily_trades, self.config.max_trades_per_day
            ));
        }

        // Check daily loss limit
        let projected_daily_loss = usage_stats.daily_loss_usd + expected_pnl.min(0.0).abs();
        if projected_daily_loss > self.config.max_daily_loss_usd {
            reasons.push(format!(
                "Projected daily loss ${:.2} exceeds maximum ${:.2}",
                projected_daily_loss, self.config.max_daily_loss_usd
            ));
        }

        let projected_daily_loss_pct =
            (projected_daily_loss / self.portfolio_state.portfolio_value_usd) * 100.0;
        if projected_daily_loss_pct > self.config.max_daily_loss_pct {
            reasons.push(format!(
                "Projected daily loss {:.2}% exceeds maximum {:.2}% of portfolio",
                projected_daily_loss_pct, self.config.max_daily_loss_pct
            ));
        }

        // Check asset exposure limit
        if usage_stats.asset_exposure_pct > self.config.max_asset_exposure_pct {
            reasons.push(format!(
                "Asset exposure {:.2}% exceeds maximum {:.2}%",
                usage_stats.asset_exposure_pct, self.config.max_asset_exposure_pct
            ));
        }

        // Check sector exposure limit
        if usage_stats.sector_exposure_pct > self.config.max_sector_exposure_pct {
            reasons.push(format!(
                "Sector exposure {:.2}% exceeds maximum {:.2}%",
                usage_stats.sector_exposure_pct, self.config.max_sector_exposure_pct
            ));
        }

        let allowed = reasons.is_empty();

        if allowed {
            info!("Trade {} approved within limits", trade_id);
        } else {
            warn!(
                "Trade {} rejected due to limit violations: {:?}",
                trade_id, reasons
            );
        }

        Ok(LimitCheckResult {
            allowed,
            reasons,
            usage_stats,
        })
    }

    /// Record a completed trade
    ///
    /// # Arguments
    /// * `activity` - Trading activity record
    pub fn record_trade(&mut self, activity: TradingActivity) -> Result<()> {
        // Update daily usage
        self.daily_usage.volume_usd += activity.size_usd;
        self.daily_usage.trade_count += 1;
        if activity.pnl_usd < 0.0 {
            self.daily_usage.losses_usd += activity.pnl_usd.abs();
        }

        // Add to trading history
        self.portfolio_state.trading_history.push(activity);

        // Keep only recent history within the time window
        self.prune_old_history()?;

        Ok(())
    }

    /// Update portfolio state
    ///
    /// # Arguments
    /// * `new_state` - New portfolio state
    pub fn update_portfolio_state(&mut self, new_state: PortfolioState) {
        self.portfolio_state = new_state;
    }

    /// Calculate current usage statistics
    fn calculate_usage_stats(
        &self,
        asset: &str,
        sector: &str,
        trade_size_usd: f64,
    ) -> Result<UsageStats> {
        // Calculate daily volume including proposed trade
        let daily_volume_usd = self.daily_usage.volume_usd + trade_size_usd;

        // Calculate daily trades including proposed trade
        let daily_trades = self.daily_usage.trade_count + 1;

        // Calculate daily losses (assuming worst case for proposed trade)
        let daily_loss_usd = self.daily_usage.losses_usd;

        // Calculate asset exposure including proposed trade
        let current_asset_exposure: f64 = self
            .portfolio_state
            .positions
            .iter()
            .filter(|p| p.asset == asset)
            .map(|p| p.size_usd)
            .sum();

        let total_asset_exposure = current_asset_exposure + trade_size_usd;
        let asset_exposure_pct =
            (total_asset_exposure / self.portfolio_state.portfolio_value_usd) * 100.0;

        // Calculate sector exposure including proposed trade
        let current_sector_exposure: f64 = self
            .portfolio_state
            .positions
            .iter()
            .filter(|p| p.sector == sector)
            .map(|p| p.size_usd)
            .sum();

        let total_sector_exposure = current_sector_exposure + trade_size_usd;
        let sector_exposure_pct =
            (total_sector_exposure / self.portfolio_state.portfolio_value_usd) * 100.0;

        Ok(UsageStats {
            daily_volume_usd,
            daily_trades,
            daily_loss_usd,
            asset_exposure_pct,
            sector_exposure_pct,
        })
    }

    /// Reset daily usage if needed
    fn reset_daily_usage_if_needed(&mut self) -> Result<()> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        if now - self.daily_usage.last_reset > self.config.time_window_seconds {
            self.daily_usage.volume_usd = 0.0;
            self.daily_usage.trade_count = 0;
            self.daily_usage.losses_usd = 0.0;
            self.daily_usage.last_reset = now;

            // Prune old history
            self.prune_old_history()?;
        }

        Ok(())
    }

    /// Prune old trading history
    fn prune_old_history(&mut self) -> Result<()> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let cutoff_time = now - self.config.time_window_seconds;

        self.portfolio_state
            .trading_history
            .retain(|activity| activity.timestamp > cutoff_time);

        Ok(())
    }

    /// Get current daily usage
    pub fn get_daily_usage(&self) -> &DailyUsage {
        &self.daily_usage
    }

    /// Update configuration
    ///
    /// # Arguments
    /// * `config` - New configuration
    pub fn update_config(&mut self, config: TradingLimitsConfig) {
        self.config = config;
    }

    /// Reset all limits (for testing or special circumstances)
    pub fn reset_limits(&mut self) {
        self.daily_usage.volume_usd = 0.0;
        self.daily_usage.trade_count = 0;
        self.daily_usage.losses_usd = 0.0;
        self.daily_usage.last_reset = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
    }
}

/// Advanced trading limits enforcer with adaptive limits
pub struct AdvancedTradingLimitsEnforcer {
    /// Base trading limits enforcer
    base_enforcer: TradingLimitsEnforcer,
    /// Performance history for adaptive limits
    performance_history: Vec<PerformanceRecord>,
    /// Adaptive adjustment factors
    adjustment_factors: HashMap<String, f64>,
}

/// Performance record for adaptive limits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceRecord {
    /// Time period identifier
    pub period: String,
    /// Profit/loss for the period
    pub pnl: f64,
    /// Volatility during the period
    pub volatility: f64,
    /// Number of trades
    pub trade_count: usize,
    /// Timestamp
    pub timestamp: u64,
}

impl AdvancedTradingLimitsEnforcer {
    /// Create a new advanced trading limits enforcer
    pub fn new(base_enforcer: TradingLimitsEnforcer) -> Self {
        Self {
            base_enforcer,
            performance_history: Vec::new(),
            adjustment_factors: HashMap::new(),
        }
    }

    /// Check trade limits with adaptive adjustments
    ///
    /// # Arguments
    /// * `trade_id` - Unique identifier for the trade
    /// * `asset` - Asset symbol to trade
    /// * `sector` - Sector/classification of the asset
    /// * `size_usd` - Size of the trade in USD
    /// * `expected_pnl` - Expected profit/loss of the trade
    ///
    /// # Returns
    /// * `Result<LimitCheckResult>` - Enhanced limit check result
    pub fn check_trade_limits_adaptive(
        &mut self,
        trade_id: &str,
        asset: &str,
        sector: &str,
        size_usd: f64,
        expected_pnl: f64,
    ) -> Result<LimitCheckResult> {
        // Get base limit check
        let mut result = self.base_enforcer.check_trade_limits(
            trade_id,
            asset,
            sector,
            size_usd,
            expected_pnl,
        )?;

        // Apply adaptive adjustments
        if let Some(adjusted_size) = self.calculate_adaptive_position_size(asset, size_usd) {
            if adjusted_size < size_usd {
                result.allowed = false;
                result.reasons.push(format!(
                    "Adaptive limit reduced position size from ${:.2} to ${:.2}",
                    size_usd, adjusted_size
                ));
            }
        }

        Ok(result)
    }

    /// Calculate adaptive position size based on performance
    fn calculate_adaptive_position_size(&self, _asset: &str, base_size: f64) -> Option<f64> {
        // In a real implementation, this would use ML models and performance data
        // For this implementation, we'll simulate with a simple approach

        // Calculate performance-based adjustment factor
        let mut adjustment_factor = 1.0;

        if !self.performance_history.is_empty() {
            let avg_pnl: f64 = self.performance_history.iter().map(|p| p.pnl).sum::<f64>()
                / self.performance_history.len() as f64;

            // If performance is good, increase limits slightly
            if avg_pnl > 100.0 {
                adjustment_factor = 1.1;
            }
            // If performance is poor, decrease limits
            else if avg_pnl < -100.0 {
                adjustment_factor = 0.8;
            }
        }

        // Apply asset-specific adjustment if available
        // For this implementation, we'll just use a default

        Some(base_size * adjustment_factor)
    }

    /// Record performance for adaptive adjustments
    ///
    /// # Arguments
    /// * `record` - Performance record
    pub fn record_performance(&mut self, record: PerformanceRecord) {
        self.performance_history.push(record);

        // Keep only recent performance data (last 30 records)
        if self.performance_history.len() > 30 {
            self.performance_history
                .drain(0..self.performance_history.len() - 30);
        }
    }

    /// Update adjustment factor for an asset
    ///
    /// # Arguments
    /// * `asset` - Asset symbol
    /// * `factor` - Adjustment factor
    pub fn update_adjustment_factor(&mut self, asset: &str, factor: f64) {
        self.adjustment_factors.insert(asset.to_string(), factor);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trading_limits_config() {
        let config = TradingLimitsConfig::default();
        assert!(config.enabled);
        assert_eq!(config.max_position_size_usd, 10000.0);
        assert_eq!(config.max_position_size_pct, 5.0);
        assert_eq!(config.max_daily_volume_usd, 100000.0);
        assert_eq!(config.max_trades_per_day, 50);
        assert_eq!(config.max_daily_loss_usd, 5000.0);
        assert_eq!(config.max_daily_loss_pct, 2.0);
        assert_eq!(config.max_asset_exposure_pct, 10.0);
        assert_eq!(config.max_sector_exposure_pct, 20.0);
        assert_eq!(config.time_window_seconds, 86400);
    }

    #[test]
    fn test_trading_limits_enforcer_creation() {
        let config = TradingLimitsConfig::default();
        let portfolio_state = PortfolioState {
            portfolio_value_usd: 100000.0,
            positions: vec![],
            trading_history: vec![],
        };

        let enforcer = TradingLimitsEnforcer::new(config, portfolio_state);
        assert_eq!(enforcer.portfolio_state.portfolio_value_usd, 100000.0);
        assert_eq!(enforcer.daily_usage.volume_usd, 0.0);
    }

    #[test]
    fn test_disabled_limits_enforcement() {
        let config = TradingLimitsConfig {
            enabled: false,
            ..Default::default()
        };

        let portfolio_state = PortfolioState {
            portfolio_value_usd: 100000.0,
            positions: vec![],
            trading_history: vec![],
        };

        let mut enforcer = TradingLimitsEnforcer::new(config, portfolio_state);
        let result = enforcer
            .check_trade_limits("test-trade", "ETH", "crypto", 15000.0, 100.0)
            .unwrap();

        assert!(result.allowed);
        assert_eq!(
            result.reasons,
            vec!["Trading limits enforcement disabled".to_string()]
        );
    }

    #[test]
    fn test_position_size_limits() {
        let config = TradingLimitsConfig::default();
        let portfolio_state = PortfolioState {
            portfolio_value_usd: 100000.0,
            positions: vec![],
            trading_history: vec![],
        };

        let mut enforcer = TradingLimitsEnforcer::new(config, portfolio_state);

        // Test exceeding USD limit
        let result = enforcer
            .check_trade_limits("test-trade-1", "ETH", "crypto", 15000.0, 100.0)
            .unwrap();
        assert!(!result.allowed);
        assert!(result
            .reasons
            .iter()
            .any(|r| r.contains("exceeds maximum $10000.00")));

        // Test exceeding percentage limit (5% of $100,000 = $5,000)
        let result = enforcer
            .check_trade_limits("test-trade-2", "ETH", "crypto", 6000.0, 100.0)
            .unwrap();
        assert!(!result.allowed);
        assert!(result
            .reasons
            .iter()
            .any(|r| r.contains("exceeds maximum 5.00% of portfolio")));
    }

    #[test]
    fn test_daily_limits() {
        let config = TradingLimitsConfig::default();
        let portfolio_state = PortfolioState {
            portfolio_value_usd: 100000.0,
            positions: vec![],
            trading_history: vec![],
        };

        let mut enforcer = TradingLimitsEnforcer::new(config, portfolio_state);

        // Simulate high daily usage
        enforcer.daily_usage.volume_usd = 95000.0;
        enforcer.daily_usage.trade_count = 45;
        enforcer.daily_usage.losses_usd = 4000.0;

        // Test exceeding daily volume with a trade
        let result = enforcer
            .check_trade_limits("test-trade", "ETH", "crypto", 10000.0, -100.0)
            .unwrap();
        assert!(!result.allowed);
        assert!(result.reasons.iter().any(|r| r.contains("Daily volume")));

        // Test exceeding daily trade count
        // Reset losses to avoid hitting that limit first
        enforcer.daily_usage.losses_usd = 0.0;
        // Set trade count to the limit, so adding one more will exceed it
        enforcer.daily_usage.trade_count = 50;
        let result = enforcer
            .check_trade_limits("test-trade", "ETH", "crypto", 1000.0, -100.0)
            .unwrap();
        assert!(!result.allowed);
        assert!(result
            .reasons
            .iter()
            .any(|r| r.contains("Daily trade count")));
    }

    #[test]
    fn test_exposure_limits() {
        let config = TradingLimitsConfig::default();
        let portfolio_state = PortfolioState {
            portfolio_value_usd: 100000.0,
            positions: vec![PositionInfo {
                asset: "ETH".to_string(),
                sector: "crypto".to_string(),
                size_usd: 8000.0,
                entry_price: 2000.0,
                current_price: 2100.0,
            }],
            trading_history: vec![],
        };

        let mut enforcer = TradingLimitsEnforcer::new(config, portfolio_state);

        // Test exceeding asset exposure (8000 + 3000 = 11000, which is 11% of 100000)
        let result = enforcer
            .check_trade_limits("test-trade", "ETH", "crypto", 3000.0, 100.0)
            .unwrap();
        assert!(!result.allowed);
        assert!(result.reasons.iter().any(|r| r.contains("Asset exposure")));

        // Test exceeding sector exposure
        let result = enforcer
            .check_trade_limits("test-trade", "BTC", "crypto", 15000.0, 100.0)
            .unwrap();
        assert!(!result.allowed);
        assert!(result.reasons.iter().any(|r| r.contains("Sector exposure")));
    }

    #[test]
    fn test_trade_recording() {
        let config = TradingLimitsConfig::default();
        let portfolio_state = PortfolioState {
            portfolio_value_usd: 100000.0,
            positions: vec![],
            trading_history: vec![],
        };

        let mut enforcer = TradingLimitsEnforcer::new(config, portfolio_state);

        let activity = TradingActivity {
            trade_id: "test-trade".to_string(),
            asset: "ETH".to_string(),
            sector: "crypto".to_string(),
            size_usd: 1000.0,
            pnl_usd: 50.0,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };

        enforcer.record_trade(activity).unwrap();
        assert_eq!(enforcer.daily_usage.volume_usd, 1000.0);
        assert_eq!(enforcer.daily_usage.trade_count, 1);
        assert_eq!(enforcer.daily_usage.losses_usd, 0.0);
        assert_eq!(enforcer.portfolio_state.trading_history.len(), 1);
    }

    #[test]
    fn test_advanced_trading_limits_enforcer() {
        let config = TradingLimitsConfig::default();
        let portfolio_state = PortfolioState {
            portfolio_value_usd: 100000.0,
            positions: vec![],
            trading_history: vec![],
        };

        let base_enforcer = TradingLimitsEnforcer::new(config, portfolio_state);
        let mut advanced_enforcer = AdvancedTradingLimitsEnforcer::new(base_enforcer);

        let record = PerformanceRecord {
            period: "2023-01".to_string(),
            pnl: 500.0,
            volatility: 0.02,
            trade_count: 20,
            timestamp: 1234567890,
        };

        advanced_enforcer.record_performance(record);
        assert_eq!(advanced_enforcer.performance_history.len(), 1);
    }
}
