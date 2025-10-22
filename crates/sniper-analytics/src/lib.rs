//! Analytics module for the sniper bot.
//! 
//! This module provides functionality for real-time performance visualization,
//! portfolio analytics, and detailed performance attribution.

use sniper_core::types::TradePlan;
use sniper_telemetry::metrics::{Metrics, MetricsSnapshot};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Configuration for analytics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsConfig {
    /// Enable/disable analytics collection
    pub enabled: bool,
    /// Metrics collection interval in seconds
    pub collection_interval_secs: u64,
}

/// Analytics data for a trade
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeAnalytics {
    pub plan_id: String,
    pub timestamp: u64,
    pub chain: String,
    pub token_in: String,
    pub token_out: String,
    pub amount_in: u64,
    pub predicted_return: f64,
    pub actual_return: Option<f64>,
    pub execution_latency_ms: u64,
    pub gas_used: u64,
    pub risk_score: f64,
}

/// Portfolio analytics data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioAnalytics {
    pub timestamp: u64,
    pub total_value: f64,
    pub asset_allocation: HashMap<String, f64>,
    pub risk_exposure: f64,
    pub correlation_matrix: HashMap<String, HashMap<String, f64>>,
    pub sector_allocation: HashMap<String, f64>,
    pub geographic_exposure: HashMap<String, f64>,
    pub concentration_risk: f64,
    pub volatility: f64,
    pub value_at_risk: f64,
}

/// Main analytics system
pub struct AnalyticsSystem {
    config: AnalyticsConfig,
    metrics: Arc<MetricsSnapshot>,
    trade_data: Arc<RwLock<Vec<TradeAnalytics>>>,
    portfolio_data: Arc<RwLock<Option<PortfolioAnalytics>>>,
}

impl AnalyticsSystem {
    /// Create a new analytics system
    pub fn new(config: AnalyticsConfig, metrics: Arc<MetricsSnapshot>) -> Self {
        Self {
            config,
            metrics,
            trade_data: Arc::new(RwLock::new(Vec::new())),
            portfolio_data: Arc::new(RwLock::new(None)),
        }
    }
    
    /// Record trade analytics data
    pub async fn record_trade(&self, analytics: TradeAnalytics) {
        if !self.config.enabled {
            return;
        }
        
        tracing::debug!("Recording trade analytics for plan: {}", analytics.plan_id);
        
        let mut trade_data = self.trade_data.write().await;
        trade_data.push(analytics);
        
        // Keep only the last 1000 trades to prevent memory issues
        let len = trade_data.len();
        if len > 1000 {
            trade_data.drain(0..len-1000);
        }
    }
    
    /// Record portfolio analytics data
    pub async fn record_portfolio(&self, analytics: PortfolioAnalytics) {
        if !self.config.enabled {
            return;
        }
        
        tracing::debug!("Recording portfolio analytics");
        
        let mut portfolio_data = self.portfolio_data.write().await;
        *portfolio_data = Some(analytics);
    }
    
    /// Get recent trade analytics
    pub async fn get_recent_trades(&self, count: usize) -> Vec<TradeAnalytics> {
        let trade_data = self.trade_data.read().await;
        let start = if trade_data.len() > count {
            trade_data.len() - count
        } else {
            0
        };
        trade_data[start..].to_vec()
    }
    
    /// Get current portfolio analytics
    pub async fn get_portfolio(&self) -> Option<PortfolioAnalytics> {
        let portfolio_data = self.portfolio_data.read().await;
        portfolio_data.clone()
    }
    
    /// Calculate risk-adjusted returns
    pub async fn calculate_risk_adjusted_returns(&self) -> f64 {
        let trade_data = self.trade_data.read().await;
        
        if trade_data.is_empty() {
            return 0.0;
        }
        
        // Calculate average return
        let total_return: f64 = trade_data.iter()
            .filter_map(|trade| trade.actual_return)
            .sum();
        let avg_return = total_return / trade_data.len() as f64;
        
        // Calculate standard deviation of returns
        let returns: Vec<f64> = trade_data.iter()
            .filter_map(|trade| trade.actual_return)
            .collect();
        
        if returns.is_empty() {
            return 0.0;
        }
        
        let mean_return: f64 = returns.iter().sum::<f64>() / returns.len() as f64;
        let variance: f64 = returns.iter().map(|r| (r - mean_return).powi(2)).sum::<f64>() / returns.len() as f64;
        let std_dev = variance.sqrt();
        
        // Calculate Sharpe ratio (simplified with risk-free rate = 0)
        if std_dev > 0.0 {
            avg_return / std_dev
        } else {
            0.0
        }
    }
    
    /// Calculate Value at Risk (VaR) at given confidence level
    pub async fn calculate_value_at_risk(&self, confidence_level: f64) -> f64 {
        let trade_data = self.trade_data.read().await;
        
        if trade_data.is_empty() {
            return 0.0;
        }
        
        // Collect returns
        let mut returns: Vec<f64> = trade_data.iter()
            .filter_map(|trade| trade.actual_return)
            .collect();
        
        if returns.is_empty() {
            return 0.0;
        }
        
        // Sort returns
        returns.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        // Calculate VaR using historical simulation
        let index = ((1.0 - confidence_level) * returns.len() as f64) as usize;
        if index < returns.len() {
            returns[index].abs() // Return absolute value as VaR is typically expressed as a positive value
        } else {
            0.0
        }
    }
    
    /// Calculate correlation matrix for assets
    pub async fn calculate_correlation_matrix(&self) -> HashMap<String, HashMap<String, f64>> {
        let trade_data = self.trade_data.read().await;
        let mut correlation_matrix = HashMap::new();
        
        if trade_data.is_empty() {
            return correlation_matrix;
        }
        
        // In a real implementation, this would calculate actual correlations
        // between different assets based on their return histories
        
        // Placeholder implementation with simulated correlations
        let mut eth_correlations = HashMap::new();
        eth_correlations.insert("ETH".to_string(), 1.0);
        eth_correlations.insert("BTC".to_string(), 0.7);
        eth_correlations.insert("LINK".to_string(), 0.5);
        correlation_matrix.insert("ETH".to_string(), eth_correlations);
        
        let mut btc_correlations = HashMap::new();
        btc_correlations.insert("ETH".to_string(), 0.7);
        btc_correlations.insert("BTC".to_string(), 1.0);
        btc_correlations.insert("LINK".to_string(), 0.3);
        correlation_matrix.insert("BTC".to_string(), btc_correlations);
        
        correlation_matrix
    }
    
    /// Calculate performance metrics
    pub async fn calculate_performance_metrics(&self) -> HashMap<String, f64> {
        let mut metrics = HashMap::new();
        
        let trade_data = self.trade_data.read().await;
        
        if trade_data.is_empty() {
            return metrics;
        }
        
        // Calculate win rate
        let successful_trades = trade_data.iter()
            .filter(|trade| trade.actual_return.unwrap_or(0.0) > 0.0)
            .count();
        let win_rate = successful_trades as f64 / trade_data.len() as f64;
        metrics.insert("win_rate".to_string(), win_rate);
        
        // Calculate average return
        let total_return: f64 = trade_data.iter()
            .filter_map(|trade| trade.actual_return)
            .sum();
        let avg_return = if successful_trades > 0 {
            total_return / successful_trades as f64
        } else {
            0.0
        };
        metrics.insert("avg_return".to_string(), avg_return);
        
        // Calculate average execution latency
        let total_latency: u64 = trade_data.iter()
            .map(|trade| trade.execution_latency_ms)
            .sum();
        let avg_latency = total_latency as f64 / trade_data.len() as f64;
        metrics.insert("avg_latency_ms".to_string(), avg_latency);
        
        // Calculate Sharpe ratio (simplified)
        let returns: Vec<f64> = trade_data.iter()
            .filter_map(|trade| trade.actual_return)
            .collect();
        if !returns.is_empty() {
            let mean_return: f64 = returns.iter().sum::<f64>() / returns.len() as f64;
            let variance: f64 = returns.iter().map(|r| (r - mean_return).powi(2)).sum::<f64>() / returns.len() as f64;
            let std_dev = variance.sqrt();
            let sharpe_ratio = if std_dev > 0.0 { mean_return / std_dev } else { 0.0 };
            metrics.insert("sharpe_ratio".to_string(), sharpe_ratio);
        }
        
        // Calculate maximum drawdown
        let mut peak_value = 1.0;
        let mut max_drawdown = 0.0;
        let mut current_value = 1.0;
        
        for trade in trade_data.iter() {
            if let Some(return_pct) = trade.actual_return {
                current_value *= 1.0 + return_pct;
                if current_value > peak_value {
                    peak_value = current_value;
                }
                let drawdown = (peak_value - current_value) / peak_value;
                if drawdown > max_drawdown {
                    max_drawdown = drawdown;
                }
            }
        }
        metrics.insert("max_drawdown".to_string(), max_drawdown);
        
        // Calculate profit factor
        let gross_profits: f64 = trade_data.iter()
            .filter_map(|trade| trade.actual_return)
            .filter(|&r| r > 0.0)
            .sum::<f64>();
        let gross_losses: f64 = trade_data.iter()
            .filter_map(|trade| trade.actual_return)
            .filter(|&r| r < 0.0)
            .sum::<f64>().abs();
        let profit_factor = if gross_losses > 0.0 { gross_profits / gross_losses } else { f64::INFINITY };
        metrics.insert("profit_factor".to_string(), profit_factor);
        
        metrics
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sniper_core::types::ChainRef;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[tokio::test]
    async fn test_analytics_system() {
        let config = AnalyticsConfig {
            enabled: true,
            collection_interval_secs: 60,
        };
        
        let metrics = Arc::new(Metrics::new().unwrap().snapshot());
        let analytics = AnalyticsSystem::new(config, metrics);
        
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
            
        let trade_analytics = TradeAnalytics {
            plan_id: "test_plan".to_string(),
            timestamp,
            chain: "ethereum".to_string(),
            token_in: "0xWETH".to_string(),
            token_out: "0xToken".to_string(),
            amount_in: 1000000000000000000,
            predicted_return: 0.15,
            actual_return: Some(0.12),
            execution_latency_ms: 50,
            gas_used: 150000,
            risk_score: 0.3,
        };
        
        analytics.record_trade(trade_analytics).await;
        
        let recent_trades = analytics.get_recent_trades(10).await;
        assert_eq!(recent_trades.len(), 1);
        
        let performance_metrics = analytics.calculate_performance_metrics().await;
        assert!(performance_metrics.contains_key("win_rate"));
        assert!(performance_metrics.contains_key("avg_return"));
        assert!(performance_metrics.contains_key("avg_latency_ms"));
        
        let risk_adjusted_returns = analytics.calculate_risk_adjusted_returns().await;
        // Risk-adjusted returns can be any value including negative
        
        let var = analytics.calculate_value_at_risk(0.95).await;
        // VaR can be any non-negative value
        
        let correlation_matrix = analytics.calculate_correlation_matrix().await;
        assert!(!correlation_matrix.is_empty());
    }
}