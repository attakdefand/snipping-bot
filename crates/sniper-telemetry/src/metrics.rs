//! Metrics collection module for the sniper bot.
//! 
//! This module provides functionality for collecting and exporting metrics.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;

/// Metrics collector for the sniper bot
pub struct Metrics {
    // Trade execution metrics
    successful_trades: AtomicU64,
    failed_trades: AtomicU64,
    total_trade_latency_ms: AtomicU64,
    total_gas_used: AtomicU64,
    
    // Signal processing metrics
    signals_processed: AtomicU64,
    total_signal_latency_ms: AtomicU64,
    
    // Risk check metrics
    risk_checks_allowed: AtomicU64,
    risk_checks_denied: AtomicU64,
    total_risk_check_latency_ms: AtomicU64,
}

impl Metrics {
    /// Create a new metrics collector
    pub fn new() -> Result<Self> {
        Ok(Self {
            successful_trades: AtomicU64::new(0),
            failed_trades: AtomicU64::new(0),
            total_trade_latency_ms: AtomicU64::new(0),
            total_gas_used: AtomicU64::new(0),
            signals_processed: AtomicU64::new(0),
            total_signal_latency_ms: AtomicU64::new(0),
            risk_checks_allowed: AtomicU64::new(0),
            risk_checks_denied: AtomicU64::new(0),
            total_risk_check_latency_ms: AtomicU64::new(0),
        })
    }
    
    /// Record a trade execution
    pub fn record_trade_execution(&self, success: bool, latency_ms: u64, gas_used: u64) {
        if success {
            self.successful_trades.fetch_add(1, Ordering::Relaxed);
        } else {
            self.failed_trades.fetch_add(1, Ordering::Relaxed);
        }
        self.total_trade_latency_ms.fetch_add(latency_ms, Ordering::Relaxed);
        self.total_gas_used.fetch_add(gas_used, Ordering::Relaxed);
    }
    
    /// Record a signal processing event
    pub fn record_signal_processing(&self, latency_ms: u64) {
        self.signals_processed.fetch_add(1, Ordering::Relaxed);
        self.total_signal_latency_ms.fetch_add(latency_ms, Ordering::Relaxed);
    }
    
    /// Record a risk check
    pub fn record_risk_check(&self, allowed: bool, latency_ms: u64) {
        if allowed {
            self.risk_checks_allowed.fetch_add(1, Ordering::Relaxed);
        } else {
            self.risk_checks_denied.fetch_add(1, Ordering::Relaxed);
        }
        self.total_risk_check_latency_ms.fetch_add(latency_ms, Ordering::Relaxed);
    }
    
    /// Get current metrics snapshot
    pub fn snapshot(&self) -> MetricsSnapshot {
        MetricsSnapshot {
            successful_trades: self.successful_trades.load(Ordering::Relaxed),
            failed_trades: self.failed_trades.load(Ordering::Relaxed),
            avg_trade_latency_ms: self.calculate_average(
                self.total_trade_latency_ms.load(Ordering::Relaxed),
                self.successful_trades.load(Ordering::Relaxed) + self.failed_trades.load(Ordering::Relaxed),
            ),
            total_gas_used: self.total_gas_used.load(Ordering::Relaxed),
            signals_processed: self.signals_processed.load(Ordering::Relaxed),
            avg_signal_latency_ms: self.calculate_average(
                self.total_signal_latency_ms.load(Ordering::Relaxed),
                self.signals_processed.load(Ordering::Relaxed),
            ),
            risk_checks_allowed: self.risk_checks_allowed.load(Ordering::Relaxed),
            risk_checks_denied: self.risk_checks_denied.load(Ordering::Relaxed),
            avg_risk_check_latency_ms: self.calculate_average(
                self.total_risk_check_latency_ms.load(Ordering::Relaxed),
                self.risk_checks_allowed.load(Ordering::Relaxed) + self.risk_checks_denied.load(Ordering::Relaxed),
            ),
        }
    }
    
    /// Calculate average value, handling division by zero
    fn calculate_average(&self, total: u64, count: u64) -> u64 {
        if count == 0 {
            0
        } else {
            total / count
        }
    }
}

/// Snapshot of current metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSnapshot {
    pub successful_trades: u64,
    pub failed_trades: u64,
    pub avg_trade_latency_ms: u64,
    pub total_gas_used: u64,
    pub signals_processed: u64,
    pub avg_signal_latency_ms: u64,
    pub risk_checks_allowed: u64,
    pub risk_checks_denied: u64,
    pub avg_risk_check_latency_ms: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_creation() {
        let metrics = Metrics::new().unwrap();
        let snapshot = metrics.snapshot();
        
        assert_eq!(snapshot.successful_trades, 0);
        assert_eq!(snapshot.failed_trades, 0);
        assert_eq!(snapshot.avg_trade_latency_ms, 0);
    }
    
    #[test]
    fn test_record_trade_execution() {
        let metrics = Metrics::new().unwrap();
        metrics.record_trade_execution(true, 100, 21000);
        metrics.record_trade_execution(false, 150, 0);
        
        let snapshot = metrics.snapshot();
        assert_eq!(snapshot.successful_trades, 1);
        assert_eq!(snapshot.failed_trades, 1);
        assert_eq!(snapshot.avg_trade_latency_ms, 125);
        assert_eq!(snapshot.total_gas_used, 21000);
    }
    
    #[test]
    fn test_record_signal_processing() {
        let metrics = Metrics::new().unwrap();
        metrics.record_signal_processing(50);
        metrics.record_signal_processing(75);
        
        let snapshot = metrics.snapshot();
        assert_eq!(snapshot.signals_processed, 2);
        assert_eq!(snapshot.avg_signal_latency_ms, 62);
    }
}