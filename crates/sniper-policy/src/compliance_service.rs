//! Compliance service for the sniper bot application
//!
//! This module provides a high-level compliance service that can be used
//! throughout the application to ensure compliance with trading regulations.

use crate::compliance::{
    AuditTrailIntegrityResult, ComplianceCheckResult, ComplianceConfig, CompliancePolicy,
    ComplianceReport, DataRetentionVerificationResult,
};
use crate::compliance_monitor::ComplianceMonitor;
// use std::collections::HashMap; // Unused import
use std::sync::{Arc, Mutex};
use tracing::{error, info};

/// Compliance service for the application
pub struct ComplianceService {
    /// The compliance policy engine
    policy: Arc<Mutex<CompliancePolicy>>,
    /// Compliance monitoring service
    monitor: ComplianceMonitor,
}

impl ComplianceService {
    /// Create a new compliance service
    pub fn new(config: ComplianceConfig) -> Self {
        let policy = CompliancePolicy::new(config);
        let monitor = ComplianceMonitor::new(policy.clone(), 3600); // Check every hour by default

        Self {
            policy: Arc::new(Mutex::new(policy)),
            monitor,
        }
    }

    /// Start the compliance monitoring service
    pub fn start_monitoring(&mut self) {
        self.monitor.start();
        info!("Compliance monitoring service started");
    }

    /// Stop the compliance monitoring service
    pub fn stop_monitoring(&mut self) {
        self.monitor.stop();
        info!("Compliance monitoring service stopped");
    }

    /// Check if a trade is compliant with jurisdiction rules
    pub fn check_jurisdiction_compliance(
        &self,
        jurisdiction: &str,
        activity: &str,
    ) -> Option<ComplianceCheckResult> {
        if let Ok(policy) = self.policy.lock() {
            Some(policy.check_jurisdiction_compliance(jurisdiction, activity))
        } else {
            error!("Failed to acquire compliance policy lock for jurisdiction check");
            None
        }
    }

    /// Check if a trade is compliant with exchange TOS
    pub fn check_exchange_tos_compliance(
        &self,
        exchange: &str,
        daily_volume: f64,
        trade_count: u32,
        trading_pair: &str,
    ) -> Option<ComplianceCheckResult> {
        if let Ok(policy) = self.policy.lock() {
            Some(policy.check_exchange_tos_compliance(
                exchange,
                daily_volume,
                trade_count,
                trading_pair,
            ))
        } else {
            error!("Failed to acquire compliance policy lock for exchange TOS check");
            None
        }
    }

    /// Check if data retention policies are being followed
    pub fn check_data_retention_compliance(&self) -> Option<ComplianceCheckResult> {
        if let Ok(policy) = self.policy.lock() {
            Some(policy.check_data_retention_compliance())
        } else {
            error!("Failed to acquire compliance policy lock for data retention check");
            None
        }
    }

    /// Log an audit event
    pub fn log_audit_event(
        &self,
        event_type: &str,
        actor: &str,
        description: &str,
        metadata: std::collections::HashMap<String, String>,
    ) {
        if let Ok(mut policy) = self.policy.lock() {
            policy.log_audit_event(event_type, actor, description, metadata);
        } else {
            error!("Failed to acquire compliance policy lock for audit logging");
        }
    }

    /// Verify audit trail integrity
    pub fn verify_audit_trail_integrity(&self) -> Option<AuditTrailIntegrityResult> {
        if let Ok(policy) = self.policy.lock() {
            Some(policy.verify_audit_trail_integrity())
        } else {
            error!("Failed to acquire compliance policy lock for audit trail integrity check");
            None
        }
    }

    /// Verify data retention compliance
    pub fn verify_data_retention_compliance(&self) -> Option<DataRetentionVerificationResult> {
        if let Ok(policy) = self.policy.lock() {
            Some(policy.verify_data_retention_compliance())
        } else {
            error!("Failed to acquire compliance policy lock for data retention verification");
            None
        }
    }

    /// Generate a comprehensive compliance report
    pub fn generate_compliance_report(&self) -> Option<ComplianceReport> {
        if let Ok(policy) = self.policy.lock() {
            Some(policy.generate_comprehensive_compliance_report())
        } else {
            error!("Failed to acquire compliance policy lock for compliance report generation");
            None
        }
    }

    /// Send compliance report to configured recipients
    pub fn send_compliance_report(&self) -> bool {
        if let Ok(policy) = self.policy.lock() {
            policy.send_compliance_report().is_ok()
        } else {
            error!("Failed to acquire compliance policy lock for sending compliance report");
            false
        }
    }

    /// Perform all compliance checks
    pub fn perform_all_compliance_checks(&self) -> Option<Vec<ComplianceCheckResult>> {
        if let Ok(policy) = self.policy.lock() {
            Some(policy.perform_all_compliance_checks())
        } else {
            error!("Failed to acquire compliance policy lock for performing all compliance checks");
            None
        }
    }

    /// Get a reference to the compliance policy for advanced operations
    pub fn policy(&self) -> Arc<Mutex<CompliancePolicy>> {
        Arc::clone(&self.policy)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compliance_service_creation() {
        let config = ComplianceConfig::default();
        let service = ComplianceService::new(config);

        assert!(service.policy().lock().is_ok());
    }

    #[test]
    fn test_compliance_service_checks() {
        let config = ComplianceConfig::default();
        let service = ComplianceService::new(config);

        // Test jurisdiction compliance check
        let jurisdiction_result = service.check_jurisdiction_compliance("US", "spot_trading");
        assert!(jurisdiction_result.is_some());

        // Test exchange TOS compliance check
        let exchange_result =
            service.check_exchange_tos_compliance("binance", 10000.0, 100, "ETH/USDT");
        assert!(exchange_result.is_some());

        // Test data retention compliance check
        let retention_result = service.check_data_retention_compliance();
        assert!(retention_result.is_some());

        // Test audit trail integrity verification
        let audit_integrity_result = service.verify_audit_trail_integrity();
        assert!(audit_integrity_result.is_some());

        // Test data retention verification
        let data_retention_result = service.verify_data_retention_compliance();
        assert!(data_retention_result.is_some());

        // Test compliance report generation
        let report = service.generate_compliance_report();
        assert!(report.is_some());

        // Test performing all compliance checks
        let all_checks = service.perform_all_compliance_checks();
        assert!(all_checks.is_some());
    }

    #[test]
    fn test_compliance_service_audit_logging() {
        let config = ComplianceConfig::default();
        let service = ComplianceService::new(config);

        let mut metadata = std::collections::HashMap::new();
        metadata.insert("test_key".to_string(), "test_value".to_string());

        // This should not panic
        service.log_audit_event(
            "TEST_EVENT",
            "test_user",
            "Test event description",
            metadata,
        );
    }

    #[test]
    fn test_compliance_service_report_sending() {
        let config = ComplianceConfig::default();
        let service = ComplianceService::new(config);

        // This should not panic and should return a boolean
        let result = service.send_compliance_report();
        assert!(result); // Should be true since reporting is mocked
    }
}
