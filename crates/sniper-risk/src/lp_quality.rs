//! Liquidity Provider Quality Assessment Module
//!
//! This module provides functionality for assessing the quality of liquidity providers
//! in decentralized exchanges, helping to identify potential risks such as rug pulls,
//! sandwich attacks, and other malicious behaviors.

// use anyhow::Result; // Unused import
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info, warn};

/// Configuration for LP quality assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LpQualityConfig {
    /// Minimum liquidity threshold in USD
    pub min_liquidity_threshold: f64,
    /// Maximum price impact tolerance (percentage)
    pub max_price_impact_tolerance: f64,
    /// Time window for monitoring LP behavior (seconds)
    pub monitoring_window_seconds: u64,
    /// Threshold for frequent LP changes (number of changes)
    pub frequent_lp_change_threshold: usize,
    /// Enable/disable LP quality checks
    pub enabled: bool,
}

impl Default for LpQualityConfig {
    fn default() -> Self {
        Self {
            min_liquidity_threshold: 1000.0,  // $1000 minimum
            max_price_impact_tolerance: 5.0,  // 5% price impact
            monitoring_window_seconds: 3600,  // 1 hour
            frequent_lp_change_threshold: 10, // 10 changes
            enabled: true,
        }
    }
}

/// Liquidity provider quality metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LpQualityMetrics {
    /// Liquidity provider address
    pub lp_address: String,
    /// Total liquidity provided in USD
    pub total_liquidity: f64,
    /// Number of transactions in monitoring window
    pub transaction_count: usize,
    /// Average price impact of trades
    pub avg_price_impact: f64,
    /// Number of times LP position changed
    pub lp_changes: usize,
    /// Timestamp of last update
    pub last_updated: u64,
    /// Quality score (0-100)
    pub quality_score: f64,
    /// Risk flags
    pub risk_flags: Vec<RiskFlag>,
}

/// Risk flags for LP quality assessment
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RiskFlag {
    /// Low liquidity
    LowLiquidity,
    /// High price impact
    HighPriceImpact,
    /// Frequent LP changes
    FrequentLpChanges,
    /// New LP (less than 24 hours)
    NewLp,
    /// Unknown LP
    UnknownLp,
}

/// Liquidity provider quality assessment
pub struct LpQualityAssessor {
    /// Configuration
    config: LpQualityConfig,
    /// Quality metrics for tracked LPs
    metrics: HashMap<String, LpQualityMetrics>,
}

impl LpQualityAssessor {
    /// Create a new LP quality assessor
    pub fn new(config: LpQualityConfig) -> Self {
        Self {
            config,
            metrics: HashMap::new(),
        }
    }

    /// Assess the quality of a liquidity provider
    pub fn assess_lp_quality(
        &mut self,
        lp_address: &str,
        liquidity_data: &LiquidityData,
    ) -> LpQualityMetrics {
        debug!("Assessing LP quality for: {}", lp_address);

        let mut risk_flags = Vec::new();
        let mut quality_score: f64 = 100.0;

        // Check liquidity threshold
        if liquidity_data.total_liquidity < self.config.min_liquidity_threshold {
            risk_flags.push(RiskFlag::LowLiquidity);
            quality_score -= 31.0; // Increased from 30.0
            warn!(
                "LP {} has low liquidity: ${}",
                lp_address, liquidity_data.total_liquidity
            );
        }

        // Check price impact
        if liquidity_data.avg_price_impact > self.config.max_price_impact_tolerance {
            risk_flags.push(RiskFlag::HighPriceImpact);
            quality_score -= 36.0; // Increased from 35.0
            warn!(
                "LP {} has high price impact: {}%",
                lp_address, liquidity_data.avg_price_impact
            );
        }

        // Check LP changes
        if liquidity_data.lp_changes > self.config.frequent_lp_change_threshold {
            risk_flags.push(RiskFlag::FrequentLpChanges);
            quality_score -= 36.0; // Increased from 35.0
            warn!(
                "LP {} has frequent changes: {}",
                lp_address, liquidity_data.lp_changes
            );
        }

        // Check if new LP
        if liquidity_data.is_new_lp {
            risk_flags.push(RiskFlag::NewLp);
            quality_score -= 15.0;
            info!("LP {} is new", lp_address);
        }

        let metrics = LpQualityMetrics {
            lp_address: lp_address.to_string(),
            total_liquidity: liquidity_data.total_liquidity,
            transaction_count: liquidity_data.transaction_count,
            avg_price_impact: liquidity_data.avg_price_impact,
            lp_changes: liquidity_data.lp_changes,
            last_updated: liquidity_data.timestamp,
            quality_score: quality_score.max(0.0_f64), // Ensure score doesn't go below 0
            risk_flags,
        };

        // Store metrics
        self.metrics.insert(lp_address.to_string(), metrics.clone());

        info!("LP {} quality score: {}", lp_address, metrics.quality_score);
        metrics
    }

    /// Get quality metrics for an LP
    pub fn get_lp_metrics(&self, lp_address: &str) -> Option<&LpQualityMetrics> {
        self.metrics.get(lp_address)
    }

    /// Check if an LP is considered high quality
    pub fn is_high_quality(&self, lp_address: &str) -> bool {
        if !self.config.enabled {
            return true; // If checks are disabled, consider all LPs high quality
        }

        match self.metrics.get(lp_address) {
            Some(metrics) => metrics.quality_score >= 70.0,
            None => false, // Unknown LPs are considered low quality
        }
    }

    /// Update configuration
    pub fn update_config(&mut self, config: LpQualityConfig) {
        self.config = config;
    }

    /// Get all tracked LP metrics
    pub fn get_all_metrics(&self) -> &HashMap<String, LpQualityMetrics> {
        &self.metrics
    }

    /// Clear metrics for an LP
    pub fn clear_lp_metrics(&mut self, lp_address: &str) {
        self.metrics.remove(lp_address);
    }

    /// Clear all metrics
    pub fn clear_all_metrics(&mut self) {
        self.metrics.clear();
    }
}

/// Liquidity data for assessment
#[derive(Debug, Clone)]
pub struct LiquidityData {
    /// Total liquidity in USD
    pub total_liquidity: f64,
    /// Number of transactions in monitoring window
    pub transaction_count: usize,
    /// Average price impact of trades
    pub avg_price_impact: f64,
    /// Number of times LP position changed
    pub lp_changes: usize,
    /// Whether this is a new LP
    pub is_new_lp: bool,
    /// Timestamp
    pub timestamp: u64,
}

impl LiquidityData {
    /// Create new liquidity data
    pub fn new(
        total_liquidity: f64,
        transaction_count: usize,
        avg_price_impact: f64,
        lp_changes: usize,
        is_new_lp: bool,
        timestamp: u64,
    ) -> Self {
        Self {
            total_liquidity,
            transaction_count,
            avg_price_impact,
            lp_changes,
            is_new_lp,
            timestamp,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lp_quality_assessor_creation() {
        let config = LpQualityConfig::default();
        let assessor = LpQualityAssessor::new(config);

        assert!(assessor.metrics.is_empty());
    }

    #[test]
    fn test_liquidity_data_creation() {
        let data = LiquidityData::new(5000.0, 100, 2.5, 5, false, 1234567890);

        assert_eq!(data.total_liquidity, 5000.0);
        assert_eq!(data.transaction_count, 100);
        assert_eq!(data.avg_price_impact, 2.5);
        assert_eq!(data.lp_changes, 5);
        assert!(!data.is_new_lp);
        assert_eq!(data.timestamp, 1234567890);
    }

    #[test]
    fn test_high_quality_lp() {
        let config = LpQualityConfig::default();
        let mut assessor = LpQualityAssessor::new(config);

        let liquidity_data = LiquidityData::new(5000.0, 100, 1.0, 2, false, 1234567890);
        let metrics = assessor.assess_lp_quality(
            "0x1234567890123456789012345678901234567890",
            &liquidity_data,
        );

        assert!(metrics.quality_score >= 70.0);
        assert!(assessor.is_high_quality("0x1234567890123456789012345678901234567890"));
        assert!(metrics.risk_flags.is_empty());
    }

    #[test]
    fn test_low_liquidity_lp() {
        let config = LpQualityConfig::default();
        let mut assessor = LpQualityAssessor::new(config);

        let liquidity_data = LiquidityData::new(500.0, 100, 1.0, 2, false, 1234567890); // Below threshold
        let metrics = assessor.assess_lp_quality(
            "0x1234567890123456789012345678901234567890",
            &liquidity_data,
        );

        assert!(metrics.quality_score < 70.0);
        assert!(!assessor.is_high_quality("0x1234567890123456789012345678901234567890"));
        assert!(metrics.risk_flags.contains(&RiskFlag::LowLiquidity));
    }

    #[test]
    fn test_high_price_impact_lp() {
        let config = LpQualityConfig::default();
        let mut assessor = LpQualityAssessor::new(config);

        let liquidity_data = LiquidityData::new(5000.0, 100, 10.0, 2, false, 1234567890); // Above threshold
        let metrics = assessor.assess_lp_quality(
            "0x1234567890123456789012345678901234567890",
            &liquidity_data,
        );

        assert!(metrics.quality_score < 70.0);
        assert!(!assessor.is_high_quality("0x1234567890123456789012345678901234567890"));
        assert!(metrics.risk_flags.contains(&RiskFlag::HighPriceImpact));
    }

    #[test]
    fn test_frequent_lp_changes() {
        let config = LpQualityConfig::default();
        let mut assessor = LpQualityAssessor::new(config);

        let liquidity_data = LiquidityData::new(5000.0, 100, 1.0, 15, false, 1234567890); // Above threshold
        let metrics = assessor.assess_lp_quality(
            "0x1234567890123456789012345678901234567890",
            &liquidity_data,
        );

        assert!(metrics.quality_score < 70.0);
        assert!(!assessor.is_high_quality("0x1234567890123456789012345678901234567890"));
        assert!(metrics.risk_flags.contains(&RiskFlag::FrequentLpChanges));
    }

    #[test]
    fn test_new_lp() {
        let config = LpQualityConfig::default();
        let mut assessor = LpQualityAssessor::new(config);

        let liquidity_data = LiquidityData::new(5000.0, 100, 1.0, 2, true, 1234567890); // New LP
        let metrics = assessor.assess_lp_quality(
            "0x1234567890123456789012345678901234567890",
            &liquidity_data,
        );

        assert!(metrics.quality_score >= 70.0); // Still high quality but with a flag
        assert!(assessor.is_high_quality("0x1234567890123456789012345678901234567890"));
        assert!(metrics.risk_flags.contains(&RiskFlag::NewLp));
    }

    #[test]
    fn test_multiple_risk_factors() {
        let config = LpQualityConfig::default();
        let mut assessor = LpQualityAssessor::new(config);

        let liquidity_data = LiquidityData::new(500.0, 100, 10.0, 15, true, 1234567890); // Multiple issues
        let metrics = assessor.assess_lp_quality(
            "0x1234567890123456789012345678901234567890",
            &liquidity_data,
        );

        assert!(metrics.quality_score < 70.0);
        assert!(!assessor.is_high_quality("0x1234567890123456789012345678901234567890"));
        assert!(metrics.risk_flags.contains(&RiskFlag::LowLiquidity));
        assert!(metrics.risk_flags.contains(&RiskFlag::HighPriceImpact));
        assert!(metrics.risk_flags.contains(&RiskFlag::FrequentLpChanges));
        assert!(metrics.risk_flags.contains(&RiskFlag::NewLp));
    }

    #[test]
    fn test_config_update() {
        let config = LpQualityConfig::default();
        let mut assessor = LpQualityAssessor::new(config);

        let new_config = LpQualityConfig {
            min_liquidity_threshold: 2000.0,
            max_price_impact_tolerance: 3.0,
            monitoring_window_seconds: 7200,
            frequent_lp_change_threshold: 5,
            enabled: false,
        };

        assessor.update_config(new_config.clone());
        assert_eq!(assessor.config.min_liquidity_threshold, 2000.0);
        assert_eq!(assessor.config.max_price_impact_tolerance, 3.0);
        assert_eq!(assessor.config.monitoring_window_seconds, 7200);
        assert_eq!(assessor.config.frequent_lp_change_threshold, 5);
        assert!(!assessor.config.enabled);
    }

    #[test]
    fn test_metrics_management() {
        let config = LpQualityConfig::default();
        let mut assessor = LpQualityAssessor::new(config);

        let liquidity_data = LiquidityData::new(5000.0, 100, 1.0, 2, false, 1234567890);
        let metrics = assessor.assess_lp_quality(
            "0x1234567890123456789012345678901234567890",
            &liquidity_data,
        );

        // Check we can retrieve metrics
        let retrieved = assessor.get_lp_metrics("0x1234567890123456789012345678901234567890");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().quality_score, metrics.quality_score);

        // Check we can get all metrics
        let all_metrics = assessor.get_all_metrics();
        assert_eq!(all_metrics.len(), 1);

        // Test clearing metrics
        assessor.clear_lp_metrics("0x1234567890123456789012345678901234567890");
        assert!(assessor
            .get_lp_metrics("0x1234567890123456789012345678901234567890")
            .is_none());

        // Test clearing all metrics
        assessor.assess_lp_quality(
            "0x1234567890123456789012345678901234567890",
            &liquidity_data,
        );
        assessor.assess_lp_quality(
            "0xABCDEF123456789012345678901234567890ABCD",
            &liquidity_data,
        );
        assert_eq!(assessor.get_all_metrics().len(), 2);

        assessor.clear_all_metrics();
        assert_eq!(assessor.get_all_metrics().len(), 0);
    }

    #[test]
    fn test_disabled_checks() {
        let config = LpQualityConfig {
            enabled: false,
            ..Default::default()
        };
        let assessor = LpQualityAssessor::new(config);

        // Even a low quality LP should be considered high quality when checks are disabled
        assert!(assessor.is_high_quality("0x1234567890123456789012345678901234567890"));
    }

    #[test]
    fn test_unknown_lp() {
        let config = LpQualityConfig::default();
        let assessor = LpQualityAssessor::new(config);

        // Unknown LPs should be considered low quality
        assert!(!assessor.is_high_quality("0x1234567890123456789012345678901234567890"));
    }
}
