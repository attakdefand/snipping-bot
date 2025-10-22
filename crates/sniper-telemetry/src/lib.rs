//! Telemetry module for the sniper bot.
//!
//! This module provides functionality for metrics collection, tracing, and alerting.

pub mod alerts;
pub mod metrics;
pub mod tracing;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::time::Instant;

/// Telemetry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryConfig {
    pub metrics_enabled: bool,
    pub tracing_enabled: bool,
    pub alerting_enabled: bool,
}

/// Main telemetry system
pub struct TelemetrySystem {
    metrics: Option<metrics::Metrics>,
    tracer: Option<tracing::Tracer>,
    alert_manager: Option<alerts::AlertManager>,
}

impl TelemetrySystem {
    /// Create a new telemetry system
    pub fn new(config: TelemetryConfig) -> Result<Self> {
        let metrics = if config.metrics_enabled {
            Some(metrics::Metrics::new()?)
        } else {
            None
        };

        let tracer = if config.tracing_enabled {
            Some(tracing::Tracer::new()?)
        } else {
            None
        };

        let alert_manager = if config.alerting_enabled {
            Some(alerts::AlertManager::new()?)
        } else {
            None
        };

        Ok(Self {
            metrics,
            tracer,
            alert_manager,
        })
    }

    /// Get metrics collector
    pub fn metrics(&self) -> Option<&metrics::Metrics> {
        self.metrics.as_ref()
    }

    /// Get tracer
    pub fn tracer(&self) -> Option<&tracing::Tracer> {
        self.tracer.as_ref()
    }

    /// Get alert manager
    pub fn alert_manager(&self) -> Option<&alerts::AlertManager> {
        self.alert_manager.as_ref()
    }

    /// Record a trade execution
    pub fn record_trade_execution(&self, success: bool, latency_ms: u64, gas_used: u64) {
        if let Some(metrics) = &self.metrics {
            metrics.record_trade_execution(success, latency_ms, gas_used);
        }
    }

    /// Record a signal processing event
    pub fn record_signal_processing(&self, latency_ms: u64) {
        if let Some(metrics) = &self.metrics {
            metrics.record_signal_processing(latency_ms);
        }
    }

    /// Record a risk check
    pub fn record_risk_check(&self, allowed: bool, latency_ms: u64) {
        if let Some(metrics) = &self.metrics {
            metrics.record_risk_check(allowed, latency_ms);
        }
    }

    /// Send an alert
    pub async fn send_alert(&self, message: &str, severity: alerts::AlertSeverity) -> Result<()> {
        if let Some(alert_manager) = &self.alert_manager {
            alert_manager.send_alert(message, severity).await
        } else {
            Ok(())
        }
    }
}

/// Performance timer for measuring operation latency
pub struct Timer {
    start: Instant,
}

impl Default for Timer {
    fn default() -> Self {
        Self::new()
    }
}

impl Timer {
    /// Create a new timer
    pub fn new() -> Self {
        Self {
            start: Instant::now(),
        }
    }

    /// Get elapsed time in milliseconds
    pub fn elapsed_ms(&self) -> u64 {
        self.start.elapsed().as_millis() as u64
    }
}

impl Drop for Timer {
    fn drop(&mut self) {
        // Timer automatically measures time when dropped
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_telemetry_system_creation() {
        let config = TelemetryConfig {
            metrics_enabled: true,
            tracing_enabled: true,
            alerting_enabled: true,
        };

        let telemetry = TelemetrySystem::new(config).unwrap();
        assert!(telemetry.metrics().is_some());
        assert!(telemetry.tracer().is_some());
        assert!(telemetry.alert_manager().is_some());
    }

    #[test]
    fn test_telemetry_system_disabled() {
        let config = TelemetryConfig {
            metrics_enabled: false,
            tracing_enabled: false,
            alerting_enabled: false,
        };

        let telemetry = TelemetrySystem::new(config).unwrap();
        assert!(telemetry.metrics().is_none());
        assert!(telemetry.tracer().is_none());
        assert!(telemetry.alert_manager().is_none());
    }

    #[test]
    fn test_timer() {
        let timer = Timer::new();
        std::thread::sleep(std::time::Duration::from_millis(10));
        let elapsed = timer.elapsed_ms();
        assert!(elapsed >= 10);
    }
}
