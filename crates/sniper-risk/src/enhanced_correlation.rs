//! Enhanced correlation analysis module for risk management.
//!
//! This module provides sophisticated functionality for monitoring asset correlations
//! with historical price correlation analysis, volatility correlation, and market
//! regime-based correlation adjustments.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration for enhanced correlation analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedCorrelationConfig {
    /// Maximum allowed correlation between assets
    pub max_correlation: f64,
    /// Time window for correlation calculation (in hours)
    pub time_window_hours: u64,
    /// Enable/disable correlation analysis
    pub enabled: bool,
    /// Weight for historical price correlation
    pub price_correlation_weight: f64,
    /// Weight for volatility correlation
    pub volatility_correlation_weight: f64,
    /// Weight for market regime correlation
    pub regime_correlation_weight: f64,
}

impl Default for EnhancedCorrelationConfig {
    fn default() -> Self {
        Self {
            max_correlation: 0.8,
            time_window_hours: 24,
            enabled: true,
            price_correlation_weight: 0.5,
            volatility_correlation_weight: 0.3,
            regime_correlation_weight: 0.2,
        }
    }
}

/// Market regime information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MarketRegime {
    pub regime_type: MarketRegimeType,
    pub timestamp: u64,
}

/// Types of market regimes
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MarketRegimeType {
    Bull,
    Bear,
    Sideways,
    Volatile,
}

/// Historical price data for correlation analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalPriceData {
    pub asset: String,
    pub prices: Vec<f64>,
    pub timestamps: Vec<u64>,
}

/// Volatility data for correlation analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolatilityData {
    pub asset: String,
    pub volatilities: Vec<f64>,
    pub timestamps: Vec<u64>,
}

/// Enhanced correlation analyzer
pub struct EnhancedCorrelationAnalyzer {
    config: EnhancedCorrelationConfig,
    historical_prices: HashMap<String, HistoricalPriceData>,
    volatilities: HashMap<String, VolatilityData>,
    market_regimes: Vec<MarketRegime>,
}

impl EnhancedCorrelationAnalyzer {
    /// Create a new enhanced correlation analyzer
    pub fn new(config: EnhancedCorrelationConfig) -> Self {
        Self {
            config,
            historical_prices: HashMap::new(),
            volatilities: HashMap::new(),
            market_regimes: Vec::new(),
        }
    }

    /// Update historical price data for an asset
    pub fn update_historical_prices(
        &mut self,
        asset: &str,
        prices: Vec<f64>,
        timestamps: Vec<u64>,
    ) {
        self.historical_prices.insert(
            asset.to_string(),
            HistoricalPriceData {
                asset: asset.to_string(),
                prices,
                timestamps,
            },
        );
    }

    /// Update volatility data for an asset
    pub fn update_volatility_data(
        &mut self,
        asset: &str,
        volatilities: Vec<f64>,
        timestamps: Vec<u64>,
    ) {
        self.volatilities.insert(
            asset.to_string(),
            VolatilityData {
                asset: asset.to_string(),
                volatilities,
                timestamps,
            },
        );
    }

    /// Add market regime information
    pub fn add_market_regime(&mut self, regime: MarketRegime) {
        self.market_regimes.push(regime);
    }

    /// Calculate enhanced correlation between two assets
    pub fn calculate_enhanced_correlation(
        &self,
        asset_a: &str,
        asset_b: &str,
    ) -> EnhancedCorrelationResult {
        if !self.config.enabled {
            return EnhancedCorrelationResult {
                correlation: 0.0,
                price_correlation: 0.0,
                volatility_correlation: 0.0,
                regime_correlation: 0.0,
                confidence: 0.0,
                reason: "Correlation analysis disabled".to_string(),
            };
        }

        // Calculate price correlation
        let price_correlation = self.calculate_price_correlation(asset_a, asset_b);

        // Calculate volatility correlation
        let volatility_correlation = self.calculate_volatility_correlation(asset_a, asset_b);

        // Calculate regime correlation
        let regime_correlation = self.calculate_regime_correlation(asset_a, asset_b);

        // Weighted combination
        let correlation = (price_correlation * self.config.price_correlation_weight)
            + (volatility_correlation * self.config.volatility_correlation_weight)
            + (regime_correlation * self.config.regime_correlation_weight);

        let confidence = if price_correlation != 0.0
            && volatility_correlation != 0.0
            && regime_correlation != 0.0
        {
            1.0
        } else if price_correlation != 0.0
            || volatility_correlation != 0.0
            || regime_correlation != 0.0
        {
            0.7
        } else {
            0.0
        };

        EnhancedCorrelationResult {
            correlation,
            price_correlation,
            volatility_correlation,
            regime_correlation,
            confidence,
            reason: "Enhanced correlation calculated".to_string(),
        }
    }

    /// Calculate price correlation between two assets
    fn calculate_price_correlation(&self, asset_a: &str, asset_b: &str) -> f64 {
        let prices_a = match self.historical_prices.get(asset_a) {
            Some(data) => &data.prices,
            None => return 0.0,
        };

        let prices_b = match self.historical_prices.get(asset_b) {
            Some(data) => &data.prices,
            None => return 0.0,
        };

        if prices_a.len() != prices_b.len() || prices_a.is_empty() {
            return 0.0;
        }

        // Calculate returns
        let returns_a: Vec<f64> = prices_a.windows(2).map(|w| (w[1] - w[0]) / w[0]).collect();

        let returns_b: Vec<f64> = prices_b.windows(2).map(|w| (w[1] - w[0]) / w[0]).collect();

        if returns_a.len() != returns_b.len() || returns_a.is_empty() {
            return 0.0;
        }

        // Calculate correlation
        let mean_a: f64 = returns_a.iter().sum::<f64>() / returns_a.len() as f64;
        let mean_b: f64 = returns_b.iter().sum::<f64>() / returns_b.len() as f64;

        let numerator: f64 = returns_a
            .iter()
            .zip(returns_b.iter())
            .map(|(a, b)| (a - mean_a) * (b - mean_b))
            .sum();

        let sum_sq_a: f64 = returns_a.iter().map(|a| (a - mean_a).powi(2)).sum();

        let sum_sq_b: f64 = returns_b.iter().map(|b| (b - mean_b).powi(2)).sum();

        if sum_sq_a == 0.0 || sum_sq_b == 0.0 {
            0.0
        } else {
            numerator / (sum_sq_a * sum_sq_b).sqrt()
        }
    }

    /// Calculate volatility correlation between two assets
    fn calculate_volatility_correlation(&self, asset_a: &str, asset_b: &str) -> f64 {
        let vol_a = match self.volatilities.get(asset_a) {
            Some(data) => &data.volatilities,
            None => return 0.0,
        };

        let vol_b = match self.volatilities.get(asset_b) {
            Some(data) => &data.volatilities,
            None => return 0.0,
        };

        if vol_a.len() != vol_b.len() || vol_a.is_empty() {
            return 0.0;
        }

        // Calculate correlation of volatilities
        let mean_a: f64 = vol_a.iter().sum::<f64>() / vol_a.len() as f64;
        let mean_b: f64 = vol_b.iter().sum::<f64>() / vol_b.len() as f64;

        let numerator: f64 = vol_a
            .iter()
            .zip(vol_b.iter())
            .map(|(a, b)| (a - mean_a) * (b - mean_b))
            .sum();

        let sum_sq_a: f64 = vol_a.iter().map(|a| (a - mean_a).powi(2)).sum();

        let sum_sq_b: f64 = vol_b.iter().map(|b| (b - mean_b).powi(2)).sum();

        if sum_sq_a == 0.0 || sum_sq_b == 0.0 {
            0.0
        } else {
            numerator / (sum_sq_a * sum_sq_b).sqrt()
        }
    }

    /// Calculate regime correlation between two assets
    fn calculate_regime_correlation(&self, _asset_a: &str, _asset_b: &str) -> f64 {
        // In a real implementation, this would analyze how assets behave in different market regimes
        // For now, we'll return a simple placeholder value
        0.3
    }

    /// Update configuration
    pub fn update_config(&mut self, config: EnhancedCorrelationConfig) {
        self.config = config;
    }
}

/// Result of enhanced correlation analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedCorrelationResult {
    /// Overall enhanced correlation value
    pub correlation: f64,
    /// Price correlation component
    pub price_correlation: f64,
    /// Volatility correlation component
    pub volatility_correlation: f64,
    /// Regime correlation component
    pub regime_correlation: f64,
    /// Confidence in the correlation value (0.0 to 1.0)
    pub confidence: f64,
    /// Reason for the result
    pub reason: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enhanced_correlation_analyzer_creation() {
        let config = EnhancedCorrelationConfig::default();
        let analyzer = EnhancedCorrelationAnalyzer::new(config);

        assert_eq!(analyzer.config.max_correlation, 0.8);
        assert_eq!(analyzer.config.time_window_hours, 24);
        assert!(analyzer.config.enabled);
    }

    #[test]
    fn test_historical_price_update() {
        let config = EnhancedCorrelationConfig::default();
        let mut analyzer = EnhancedCorrelationAnalyzer::new(config);

        let prices = vec![100.0, 105.0, 110.0, 108.0, 112.0];
        let timestamps = vec![1, 2, 3, 4, 5];

        analyzer.update_historical_prices("ETH", prices.clone(), timestamps.clone());

        let stored_data = analyzer.historical_prices.get("ETH").unwrap();
        assert_eq!(stored_data.prices, prices);
        assert_eq!(stored_data.timestamps, timestamps);
    }

    #[test]
    fn test_volatility_data_update() {
        let config = EnhancedCorrelationConfig::default();
        let mut analyzer = EnhancedCorrelationAnalyzer::new(config);

        let volatilities = vec![0.1, 0.15, 0.12, 0.18, 0.14];
        let timestamps = vec![1, 2, 3, 4, 5];

        analyzer.update_volatility_data("ETH", volatilities.clone(), timestamps.clone());

        let stored_data = analyzer.volatilities.get("ETH").unwrap();
        assert_eq!(stored_data.volatilities, volatilities);
        assert_eq!(stored_data.timestamps, timestamps);
    }

    #[test]
    fn test_market_regime_addition() {
        let config = EnhancedCorrelationConfig::default();
        let mut analyzer = EnhancedCorrelationAnalyzer::new(config);

        let regime = MarketRegime {
            regime_type: MarketRegimeType::Bull,
            timestamp: 1234567890,
        };

        analyzer.add_market_regime(regime.clone());
        assert_eq!(analyzer.market_regimes.len(), 1);
        assert_eq!(
            analyzer.market_regimes[0].regime_type,
            MarketRegimeType::Bull
        );
    }

    #[test]
    fn test_price_correlation_calculation() {
        let config = EnhancedCorrelationConfig::default();
        let mut analyzer = EnhancedCorrelationAnalyzer::new(config);

        // Perfect positive correlation
        let prices_a = vec![100.0, 110.0, 120.0, 130.0, 140.0];
        let prices_b = vec![200.0, 220.0, 240.0, 260.0, 280.0];
        let timestamps = vec![1, 2, 3, 4, 5];

        analyzer.update_historical_prices("ETH", prices_a, timestamps.clone());
        analyzer.update_historical_prices("BTC", prices_b, timestamps);

        let correlation = analyzer.calculate_price_correlation("ETH", "BTC");
        // Should be close to 1.0 for perfect positive correlation
        assert!(correlation > 0.99);
    }

    #[test]
    fn test_enhanced_correlation_calculation() {
        let config = EnhancedCorrelationConfig::default();
        let mut analyzer = EnhancedCorrelationAnalyzer::new(config);

        // Add some data
        let prices_a = vec![100.0, 110.0, 120.0, 130.0, 140.0];
        let prices_b = vec![200.0, 220.0, 240.0, 260.0, 280.0];
        let timestamps = vec![1, 2, 3, 4, 5];

        analyzer.update_historical_prices("ETH", prices_a, timestamps.clone());
        analyzer.update_historical_prices("BTC", prices_b, timestamps.clone());

        let vol_a = vec![0.1, 0.15, 0.12, 0.18, 0.14];
        let vol_b = vec![0.2, 0.25, 0.22, 0.28, 0.24];

        analyzer.update_volatility_data("ETH", vol_a, timestamps.clone());
        analyzer.update_volatility_data("BTC", vol_b, timestamps.clone());

        let result = analyzer.calculate_enhanced_correlation("ETH", "BTC");

        assert!(result.correlation >= 0.0);
        assert!(result.correlation <= 1.0);
        assert!(result.confidence > 0.0);
    }

    #[test]
    fn test_disabled_analyzer() {
        let config = EnhancedCorrelationConfig {
            enabled: false,
            ..Default::default()
        };
        let analyzer = EnhancedCorrelationAnalyzer::new(config);

        let result = analyzer.calculate_enhanced_correlation("ETH", "BTC");
        assert_eq!(result.correlation, 0.0);
        assert_eq!(result.reason, "Correlation analysis disabled");
    }
}
