//! Metrics collection module for the sniper bot.
//!
//! This module provides functionality for collecting and exporting metrics.

use anyhow::Result;
use prometheus::{Histogram, HistogramOpts, IntCounter, Registry};
use serde::{Deserialize, Serialize};
// use std::convert::TryInto; // Unused import
use std::sync::atomic::{AtomicU64, Ordering};

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

    // Prometheus metrics
    prometheus_registry: Registry,
    pub successful_trades_counter: IntCounter,
    pub failed_trades_counter: IntCounter,
    pub trade_latency_histogram: Histogram,
    pub gas_used_counter: IntCounter,
    pub signals_processed_counter: IntCounter,
    pub signal_latency_histogram: Histogram,
    pub risk_checks_allowed_counter: IntCounter,
    pub risk_checks_denied_counter: IntCounter,
    pub risk_check_latency_histogram: Histogram,
}

impl Metrics {
    /// Create a new metrics collector
    pub fn new() -> Result<Self> {
        let registry = Registry::new();

        // Create Prometheus metrics
        let successful_trades_counter = IntCounter::new(
            "sniper_successful_trades_total",
            "Total number of successful trades",
        )?;
        let failed_trades_counter = IntCounter::new(
            "sniper_failed_trades_total",
            "Total number of failed trades",
        )?;
        let trade_latency_histogram = Histogram::with_opts(
            HistogramOpts::new(
                "sniper_trade_latency_seconds",
                "Trade execution latency in seconds",
            )
            .buckets(vec![
                0.0005, 0.001, 0.002, 0.004, 0.008, 0.016, 0.032, 0.064, 0.128, 0.256, 0.512,
                1.024, 2.048, 4.096, 8.192, 16.384, 32.768, 65.536, 131.072, 262.144,
            ]),
        )?;
        let gas_used_counter =
            IntCounter::new("sniper_gas_used_total", "Total gas used for all trades")?;
        let signals_processed_counter = IntCounter::new(
            "sniper_signals_processed_total",
            "Total number of signals processed",
        )?;
        let signal_latency_histogram = Histogram::with_opts(
            HistogramOpts::new(
                "sniper_signal_latency_seconds",
                "Signal processing latency in seconds",
            )
            .buckets(vec![
                0.0001, 0.0002, 0.0004, 0.0008, 0.0016, 0.0032, 0.0064, 0.0128, 0.0256, 0.0512,
                0.1024, 0.2048, 0.4096, 0.8192, 1.6384, 3.2768, 6.5536, 13.1072, 26.2144, 52.4288,
            ]),
        )?;
        let risk_checks_allowed_counter = IntCounter::new(
            "sniper_risk_checks_allowed_total",
            "Total number of allowed risk checks",
        )?;
        let risk_checks_denied_counter = IntCounter::new(
            "sniper_risk_checks_denied_total",
            "Total number of denied risk checks",
        )?;
        let risk_check_latency_histogram = Histogram::with_opts(
            HistogramOpts::new(
                "sniper_risk_check_latency_seconds",
                "Risk check latency in seconds",
            )
            .buckets(vec![
                0.00005, 0.0001, 0.0002, 0.0004, 0.0008, 0.0016, 0.0032, 0.0064, 0.0128, 0.0256,
                0.0512, 0.1024, 0.2048, 0.4096, 0.8192, 1.6384, 3.2768, 6.5536, 13.1072, 26.2144,
            ]),
        )?;

        // Register metrics
        registry.register(Box::new(successful_trades_counter.clone()))?;
        registry.register(Box::new(failed_trades_counter.clone()))?;
        registry.register(Box::new(trade_latency_histogram.clone()))?;
        registry.register(Box::new(gas_used_counter.clone()))?;
        registry.register(Box::new(signals_processed_counter.clone()))?;
        registry.register(Box::new(signal_latency_histogram.clone()))?;
        registry.register(Box::new(risk_checks_allowed_counter.clone()))?;
        registry.register(Box::new(risk_checks_denied_counter.clone()))?;
        registry.register(Box::new(risk_check_latency_histogram.clone()))?;

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
            prometheus_registry: registry,
            successful_trades_counter,
            failed_trades_counter,
            trade_latency_histogram,
            gas_used_counter,
            signals_processed_counter,
            signal_latency_histogram,
            risk_checks_allowed_counter,
            risk_checks_denied_counter,
            risk_check_latency_histogram,
        })
    }

    /// Record a trade execution
    pub fn record_trade_execution(&self, success: bool, latency_ms: u64, gas_used: u64) {
        if success {
            self.successful_trades.fetch_add(1, Ordering::Relaxed);
            self.successful_trades_counter.inc();
        } else {
            self.failed_trades.fetch_add(1, Ordering::Relaxed);
            self.failed_trades_counter.inc();
        }

        self.total_trade_latency_ms
            .fetch_add(latency_ms, Ordering::Relaxed);
        self.trade_latency_histogram
            .observe(latency_ms as f64 / 1000.0); // Convert ms to seconds

        self.total_gas_used.fetch_add(gas_used, Ordering::Relaxed);
        self.gas_used_counter.inc_by(gas_used);
    }

    /// Record a signal processing event
    pub fn record_signal_processing(&self, latency_ms: u64) {
        self.signals_processed.fetch_add(1, Ordering::Relaxed);
        self.signals_processed_counter.inc();

        self.total_signal_latency_ms
            .fetch_add(latency_ms, Ordering::Relaxed);
        self.signal_latency_histogram
            .observe(latency_ms as f64 / 1000.0); // Convert ms to seconds
    }

    /// Record a risk check
    pub fn record_risk_check(&self, allowed: bool, latency_ms: u64) {
        if allowed {
            self.risk_checks_allowed.fetch_add(1, Ordering::Relaxed);
            self.risk_checks_allowed_counter.inc();
        } else {
            self.risk_checks_denied.fetch_add(1, Ordering::Relaxed);
            self.risk_checks_denied_counter.inc();
        }

        self.total_risk_check_latency_ms
            .fetch_add(latency_ms, Ordering::Relaxed);
        self.risk_check_latency_histogram
            .observe(latency_ms as f64 / 1000.0); // Convert ms to seconds
    }

    /// Get current metrics snapshot
    pub fn snapshot(&self) -> MetricsSnapshot {
        MetricsSnapshot {
            successful_trades: self.successful_trades.load(Ordering::Relaxed),
            failed_trades: self.failed_trades.load(Ordering::Relaxed),
            avg_trade_latency_ms: self.calculate_average(
                self.total_trade_latency_ms.load(Ordering::Relaxed),
                self.successful_trades.load(Ordering::Relaxed)
                    + self.failed_trades.load(Ordering::Relaxed),
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
                self.risk_checks_allowed.load(Ordering::Relaxed)
                    + self.risk_checks_denied.load(Ordering::Relaxed),
            ),
        }
    }

    /// Get Prometheus registry for exposing metrics
    pub fn registry(&self) -> &Registry {
        &self.prometheus_registry
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
