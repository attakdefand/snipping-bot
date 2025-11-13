//! Post-Trade Attribution Analysis Module
//!
//! This module provides functionality for analyzing the performance and impact
//! of executed trades, including profit/loss attribution, slippage analysis,
//! and market impact assessment.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

/// Configuration for post-trade attribution analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostTradeAttribConfig {
    /// Enable/disable post-trade attribution analysis
    pub enabled: bool,
    /// Slippage tolerance threshold (percentage)
    pub slippage_tolerance: f64,
    /// Market impact threshold (percentage)
    pub market_impact_threshold: f64,
    /// Time window for comparing prices (seconds)
    pub price_comparison_window: u64,
}

impl Default for PostTradeAttribConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            slippage_tolerance: 2.0, // 2% slippage tolerance
            market_impact_threshold: 1.0, // 1% market impact threshold
            price_comparison_window: 300, // 5 minutes
        }
    }
}

/// Post-trade analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostTradeAnalysis {
    /// Trade identifier
    pub trade_id: String,
    /// Trading pair
    pub pair: String,
    /// Trade side (buy/sell)
    pub side: TradeSide,
    /// Expected price at order creation
    pub expected_price: f64,
    /// Actual execution price
    pub actual_price: f64,
    /// Trade size
    pub size: f64,
    /// Slippage percentage
    pub slippage_pct: f64,
    /// Market impact percentage
    pub market_impact_pct: f64,
    /// Profit/loss in absolute terms
    pub pnl_absolute: f64,
    /// Profit/loss in percentage terms
    pub pnl_percentage: f64,
    /// Timestamp of analysis
    pub timestamp: u64,
    /// Quality score (0-100)
    pub quality_score: f64,
    /// Attribution flags
    pub attribution_flags: Vec<AttributionFlag>,
}

/// Trade side
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TradeSide {
    /// Buy order
    Buy,
    /// Sell order
    Sell,
}

/// Attribution flags for post-trade analysis
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AttributionFlag {
    /// High slippage
    HighSlippage,
    /// Significant market impact
    HighMarketImpact,
    /// Negative PnL
    NegativePnL,
    /// Large trade size
    LargeTrade,
}

/// Post-trade attribution analyzer
pub struct PostTradeAttributor {
    /// Configuration
    config: PostTradeAttribConfig,
}

impl PostTradeAttributor {
    /// Create a new post-trade attributor
    pub fn new(config: PostTradeAttribConfig) -> Self {
        Self { config }
    }

    /// Analyze a completed trade
    pub fn analyze_trade(&self, trade_data: &TradeData) -> PostTradeAnalysis {
        debug!("Analyzing trade: {}", trade_data.trade_id);

        let slippage_pct = self.calculate_slippage(
            trade_data.expected_price,
            trade_data.actual_price,
            &trade_data.side
        );

        let market_impact_pct = self.calculate_market_impact(
            trade_data.expected_price,
            trade_data.actual_price
        );

        let (pnl_absolute, pnl_percentage) = self.calculate_pnl(
            trade_data.expected_price,
            trade_data.actual_price,
            trade_data.size,
            &trade_data.side
        );

        let mut attribution_flags = Vec::new();
        let mut quality_score = 100.0;

        // Check for high slippage
        if slippage_pct.abs() > self.config.slippage_tolerance {
            attribution_flags.push(AttributionFlag::HighSlippage);
            quality_score -= 25.0;
            warn!("Trade {} has high slippage: {}%", trade_data.trade_id, slippage_pct);
        }

        // Check for high market impact
        if market_impact_pct.abs() > self.config.market_impact_threshold {
            attribution_flags.push(AttributionFlag::HighMarketImpact);
            quality_score -= 20.0;
            warn!("Trade {} has high market impact: {}%", trade_data.trade_id, market_impact_pct);
        }

        // Check for negative PnL
        if pnl_absolute < 0.0 {
            attribution_flags.push(AttributionFlag::NegativePnL);
            quality_score -= 30.0;
            warn!("Trade {} has negative PnL: ${}", trade_data.trade_id, pnl_absolute);
        }

        // Check for large trade size
        if trade_data.size > 100000.0 { // Arbitrary large trade threshold
            attribution_flags.push(AttributionFlag::LargeTrade);
            quality_score -= 10.0;
            info!("Trade {} is large: {}", trade_data.trade_id, trade_data.size);
        }

        let analysis = PostTradeAnalysis {
            trade_id: trade_data.trade_id.clone(),
            pair: trade_data.pair.clone(),
            side: trade_data.side.clone(),
            expected_price: trade_data.expected_price,
            actual_price: trade_data.actual_price,
            size: trade_data.size,
            slippage_pct,
            market_impact_pct,
            pnl_absolute,
            pnl_percentage,
            timestamp: trade_data.timestamp,
            quality_score: quality_score.max(0.0), // Ensure score doesn't go below 0
            attribution_flags,
        };

        info!("Trade {} analysis complete. Quality score: {}", trade_data.trade_id, analysis.quality_score);
        analysis
    }

    /// Calculate slippage percentage
    fn calculate_slippage(&self, expected_price: f64, actual_price: f64, side: &TradeSide) -> f64 {
        match side {
            TradeSide::Buy => ((actual_price - expected_price) / expected_price) * 100.0,
            TradeSide::Sell => ((expected_price - actual_price) / expected_price) * 100.0,
        }
    }

    /// Calculate market impact percentage
    fn calculate_market_impact(&self, expected_price: f64, actual_price: f64) -> f64 {
        ((actual_price - expected_price) / expected_price) * 100.0
    }

    /// Calculate profit/loss
    fn calculate_pnl(&self, expected_price: f64, actual_price: f64, size: f64, side: &TradeSide) -> (f64, f64) {
        let pnl_absolute = match side {
            TradeSide::Buy => (actual_price - expected_price) * size,
            TradeSide::Sell => (expected_price - actual_price) * size,
        };

        let pnl_percentage = (pnl_absolute / (expected_price * size)) * 100.0;
        (pnl_absolute, pnl_percentage)
    }

    /// Update configuration
    pub fn update_config(&mut self, config: PostTradeAttribConfig) {
        self.config = config;
    }

    /// Check if a trade analysis is considered high quality
    pub fn is_high_quality(&self, analysis: &PostTradeAnalysis) -> bool {
        if !self.config.enabled {
            return true; // If checks are disabled, consider all trades high quality
        }

        analysis.quality_score >= 70.0
    }
}

/// Trade data for analysis
#[derive(Debug, Clone)]
pub struct TradeData {
    /// Trade identifier
    pub trade_id: String,
    /// Trading pair
    pub pair: String,
    /// Trade side (buy/sell)
    pub side: TradeSide,
    /// Expected price at order creation
    pub expected_price: f64,
    /// Actual execution price
    pub actual_price: f64,
    /// Trade size
    pub size: f64,
    /// Timestamp
    pub timestamp: u64,
}

impl TradeData {
    /// Create new trade data
    pub fn new(
        trade_id: String,
        pair: String,
        side: TradeSide,
        expected_price: f64,
        actual_price: f64,
        size: f64,
        timestamp: u64,
    ) -> Self {
        Self {
            trade_id,
            pair,
            side,
            expected_price,
            actual_price,
            size,
            timestamp,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_post_trade_attributor_creation() {
        let config = PostTradeAttribConfig::default();
        let attributor = PostTradeAttributor::new(config);
        
        assert_eq!(attributor.config.slippage_tolerance, 2.0);
        assert_eq!(attributor.config.market_impact_threshold, 1.0);
    }

    #[test]
    fn test_trade_data_creation() {
        let data = TradeData::new(
            "trade_123".to_string(),
            "ETH/USDT".to_string(),
            TradeSide::Buy,
            3000.0,
            3010.0,
            1.0,
            1234567890,
        );
        
        assert_eq!(data.trade_id, "trade_123");
        assert_eq!(data.pair, "ETH/USDT");
        assert_eq!(data.side, TradeSide::Buy);
        assert_eq!(data.expected_price, 3000.0);
        assert_eq!(data.actual_price, 3010.0);
        assert_eq!(data.size, 1.0);
        assert_eq!(data.timestamp, 1234567890);
    }

    #[test]
    fn test_slippage_calculation() {
        let config = PostTradeAttribConfig::default();
        let attributor = PostTradeAttributor::new(config);
        
        // Buy trade with positive slippage (worse price)
        let slippage = attributor.calculate_slippage(3000.0, 3010.0, &TradeSide::Buy);
        assert_eq!(slippage, (10.0 / 3000.0) * 100.0);
        
        // Buy trade with negative slippage (better price)
        let slippage = attributor.calculate_slippage(3000.0, 2990.0, &TradeSide::Buy);
        assert_eq!(slippage, (-10.0 / 3000.0) * 100.0);
        
        // Sell trade with positive slippage (better price)
        let slippage = attributor.calculate_slippage(3000.0, 3010.0, &TradeSide::Sell);
        assert_eq!(slippage, (-10.0 / 3000.0) * 100.0);
        
        // Sell trade with negative slippage (worse price)
        let slippage = attributor.calculate_slippage(3000.0, 2990.0, &TradeSide::Sell);
        assert_eq!(slippage, (10.0 / 3000.0) * 100.0);
    }

    #[test]
    fn test_market_impact_calculation() {
        let config = PostTradeAttribConfig::default();
        let attributor = PostTradeAttributor::new(config);
        
        let impact = attributor.calculate_market_impact(3000.0, 3010.0);
        assert_eq!(impact, (10.0 / 3000.0) * 100.0);
        
        let impact = attributor.calculate_market_impact(3000.0, 2990.0);
        assert_eq!(impact, (-10.0 / 3000.0) * 100.0);
    }

    #[test]
    fn test_pnl_calculation() {
        let config = PostTradeAttribConfig::default();
        let attributor = PostTradeAttributor::new(config);
        
        // Buy trade with positive outcome
        let (pnl_abs, pnl_pct) = attributor.calculate_pnl(3000.0, 3010.0, 1.0, &TradeSide::Buy);
        assert_eq!(pnl_abs, 10.0);
        assert_eq!(pnl_pct, (10.0 / 3000.0) * 100.0);
        
        // Buy trade with negative outcome
        let (pnl_abs, pnl_pct) = attributor.calculate_pnl(3000.0, 2990.0, 1.0, &TradeSide::Buy);
        assert_eq!(pnl_abs, -10.0);
        assert_eq!(pnl_pct, (-10.0 / 3000.0) * 100.0);
        
        // Sell trade with positive outcome
        let (pnl_abs, pnl_pct) = attributor.calculate_pnl(3000.0, 2990.0, 1.0, &TradeSide::Sell);
        assert_eq!(pnl_abs, 10.0);
        assert_eq!(pnl_pct, (10.0 / 3000.0) * 100.0);
        
        // Sell trade with negative outcome
        let (pnl_abs, pnl_pct) = attributor.calculate_pnl(3000.0, 3010.0, 1.0, &TradeSide::Sell);
        assert_eq!(pnl_abs, -10.0);
        assert_eq!(pnl_pct, (-10.0 / 3000.0) * 100.0);
    }

    #[test]
    fn test_successful_trade_analysis() {
        let config = PostTradeAttribConfig::default();
        let attributor = PostTradeAttributor::new(config);
        
        let trade_data = TradeData::new(
            "trade_123".to_string(),
            "ETH/USDT".to_string(),
            TradeSide::Buy,
            3000.0,
            2995.0, // Better price (negative slippage)
            1.0,
            1234567890,
        );
        
        let analysis = attributor.analyze_trade(&trade_data);
        
        assert_eq!(analysis.trade_id, "trade_123");
        assert_eq!(analysis.pair, "ETH/USDT");
        assert_eq!(analysis.side, TradeSide::Buy);
        assert_eq!(analysis.expected_price, 3000.0);
        assert_eq!(analysis.actual_price, 2995.0);
        assert_eq!(analysis.size, 1.0);
        assert!(analysis.slippage_pct < 0.0); // Negative slippage (good)
        assert!(analysis.pnl_absolute > 0.0); // Positive PnL (good)
        assert!(analysis.quality_score >= 70.0); // High quality
        assert!(attributor.is_high_quality(&analysis));
        assert!(analysis.attribution_flags.is_empty()); // No flags for good trade
    }

    #[test]
    fn test_high_slippage_trade_analysis() {
        let config = PostTradeAttribConfig::default();
        let attributor = PostTradeAttributor::new(config);
        
        let trade_data = TradeData::new(
            "trade_123".to_string(),
            "ETH/USDT".to_string(),
            TradeSide::Buy,
            3000.0,
            3100.0, // Much worse price (high slippage)
            1.0,
            1234567890,
        );
        
        let analysis = attributor.analyze_trade(&trade_data);
        
        assert!(analysis.slippage_pct > 2.0); // Above tolerance
        assert!(analysis.quality_score < 70.0); // Low quality
        assert!(!attributor.is_high_quality(&analysis));
        assert!(analysis.attribution_flags.contains(&AttributionFlag::HighSlippage));
    }

    #[test]
    fn test_negative_pnl_trade_analysis() {
        let config = PostTradeAttribConfig::default();
        let attributor = PostTradeAttributor::new(config);
        
        let trade_data = TradeData::new(
            "trade_123".to_string(),
            "ETH/USDT".to_string(),
            TradeSide::Buy,
            3000.0,
            3010.0, // Worse price (negative PnL)
            1.0,
            1234567890,
        );
        
        let analysis = attributor.analyze_trade(&trade_data);
        
        assert!(analysis.pnl_absolute < 0.0); // Negative PnL
        assert!(analysis.quality_score < 70.0); // Low quality
        assert!(!attributor.is_high_quality(&analysis));
        assert!(analysis.attribution_flags.contains(&AttributionFlag::NegativePnL));
    }

    #[test]
    fn test_large_trade_analysis() {
        let config = PostTradeAttribConfig::default();
        let attributor = PostTradeAttributor::new(config);
        
        let trade_data = TradeData::new(
            "trade_123".to_string(),
            "ETH/USDT".to_string(),
            TradeSide::Buy,
            3000.0,
            3005.0, // Slight slippage
            150000.0, // Large trade size
            1234567890,
        );
        
        let analysis = attributor.analyze_trade(&trade_data);
        
        assert!(analysis.size > 100000.0); // Large trade
        assert!(analysis.quality_score >= 70.0); // Still high quality
        assert!(attributor.is_high_quality(&analysis));
        assert!(analysis.attribution_flags.contains(&AttributionFlag::LargeTrade));
    }

    #[test]
    fn test_config_update() {
        let config = PostTradeAttribConfig::default();
        let mut attributor = PostTradeAttributor::new(config);
        
        let new_config = PostTradeAttribConfig {
            enabled: false,
            slippage_tolerance: 5.0,
            market_impact_threshold: 3.0,
            price_comparison_window: 600,
        };
        
        attributor.update_config(new_config.clone());
        assert_eq!(attributor.config.slippage_tolerance, 5.0);
        assert_eq!(attributor.config.market_impact_threshold, 3.0);
        assert_eq!(attributor.config.price_comparison_window, 600);
        assert_eq!(attributor.config.enabled, false);
    }

    #[test]
    fn test_disabled_checks() {
        let mut config = PostTradeAttribConfig::default();
        config.enabled = false;
        let attributor = PostTradeAttributor::new(config);
        
        let trade_data = TradeData::new(
            "trade_123".to_string(),
            "ETH/USDT".to_string(),
            TradeSide::Buy,
            3000.0,
            3100.0, // High slippage trade
            1.0,
            1234567890,
        );
        
        let analysis = attributor.analyze_trade(&trade_data);
        // Even a bad trade should be considered high quality when checks are disabled
        assert!(attributor.is_high_quality(&analysis));
    }
}