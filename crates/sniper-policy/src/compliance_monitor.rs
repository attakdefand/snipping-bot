//! Compliance monitoring service for continuous compliance checking
//!
//! This module provides a service that can run compliance checks at regular intervals
//! and generate reports as needed.

use crate::compliance::{
    AuditTrailIntegrityResult, CompliancePolicy, DataRetentionVerificationResult,
};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tracing::{error, info, warn};

/// Compliance monitoring service
pub struct ComplianceMonitor {
    /// The compliance policy engine
    policy: Arc<Mutex<CompliancePolicy>>,
    /// Monitoring interval in seconds
    interval_seconds: u64,
    /// Whether the monitor is running
    running: bool,
}

impl ComplianceMonitor {
    /// Create a new compliance monitor
    pub fn new(policy: CompliancePolicy, interval_seconds: u64) -> Self {
        Self {
            policy: Arc::new(Mutex::new(policy)),
            interval_seconds,
            running: false,
        }
    }

    /// Start the compliance monitoring service
    pub fn start(&mut self) {
        if self.running {
            warn!("Compliance monitor is already running");
            return;
        }

        self.running = true;
        let policy = Arc::clone(&self.policy);
        let interval = self.interval_seconds;

        info!(
            "Starting compliance monitoring service with {} second interval",
            interval
        );

        thread::spawn(move || {
            loop {
                // Perform compliance checks
                if let Ok(policy_guard) = policy.lock() {
                    Self::perform_compliance_checks(&policy_guard);
                } else {
                    error!("Failed to acquire compliance policy lock");
                }

                // Sleep for the specified interval
                thread::sleep(Duration::from_secs(interval));
            }
        });
    }

    /// Stop the compliance monitoring service
    pub fn stop(&mut self) {
        self.running = false;
        info!("Compliance monitoring service stopped");
    }

    /// Perform all compliance checks
    fn perform_compliance_checks(policy: &CompliancePolicy) {
        info!("Performing compliance checks...");

        // Check audit trail integrity
        let audit_result = policy.verify_audit_trail_integrity();
        Self::handle_audit_integrity_result(&audit_result);

        // Check data retention compliance
        let retention_result = policy.verify_data_retention_compliance();
        Self::handle_data_retention_result(&retention_result);

        // Generate and send compliance report if needed
        if let Err(e) = policy.send_compliance_report() {
            error!("Failed to send compliance report: {}", e);
        }

        info!("Compliance checks completed");
    }

    /// Handle audit trail integrity verification result
    fn handle_audit_integrity_result(result: &AuditTrailIntegrityResult) {
        if result.integrity_verified {
            info!("Audit trail integrity verified");
        } else {
            warn!("Audit trail integrity issues found: {:?}", result.issues);
        }
    }

    /// Handle data retention verification result
    fn handle_data_retention_result(result: &DataRetentionVerificationResult) {
        if result.retention_compliant {
            info!("Data retention policies are compliant");
        } else {
            warn!(
                "Data retention violations found in {} files",
                result.violating_files.len()
            );
            for file in &result.violating_files {
                warn!("  - {}", file);
            }
        }
    }

    /// Get a reference to the compliance policy
    pub fn policy(&self) -> Arc<Mutex<CompliancePolicy>> {
        Arc::clone(&self.policy)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compliance::ComplianceConfig;

    #[test]
    fn test_compliance_monitor_creation() {
        let config = ComplianceConfig::default();
        let policy = CompliancePolicy::new(config);
        let monitor = ComplianceMonitor::new(policy, 60);

        assert!(!monitor.running);
        assert_eq!(monitor.interval_seconds, 60);
    }

    #[test]
    fn test_compliance_monitor_start_stop() {
        let config = ComplianceConfig::default();
        let policy = CompliancePolicy::new(config);
        let mut monitor = ComplianceMonitor::new(policy, 1);

        monitor.start();
        assert!(monitor.running);

        monitor.stop();
        assert!(!monitor.running);
    }
}
