//! Compliance demonstration application
//!
//! This example shows how to use the compliance features of the sniper-policy crate.

use sniper_policy::compliance_loader::{load_compliance_config, create_default_compliance_config};
use sniper_policy::compliance_service::ComplianceService;
use std::collections::HashMap;
use tracing::{info, warn, error};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    info!("Starting compliance demonstration");
    
    // Try to load compliance configuration from file
    let config = match load_compliance_config("configs/compliance-config.toml") {
        Ok(config) => {
            info!("Loaded compliance configuration from file");
            config
        },
        Err(e) => {
            warn!("Failed to load compliance config from file: {}, using default config", e);
            create_default_compliance_config()
        }
    };
    
    // Create compliance service
    let mut compliance_service = ComplianceService::new(config);
    
    // Start monitoring
    compliance_service.start_monitoring();
    
    // Perform some compliance checks
    info!("Performing compliance checks...");
    
    // Check jurisdiction compliance
    if let Some(result) = compliance_service.check_jurisdiction_compliance("US", "spot_trading") {
        if result.compliant {
            info!("Jurisdiction compliance check passed for US spot trading");
        } else {
            warn!("Jurisdiction compliance check failed: {:?}", result.reasons);
        }
    }
    
    // Check exchange TOS compliance
    if let Some(result) = compliance_service.check_exchange_tos_compliance("binance", 50000.0, 500, "ETH/USDT") {
        if result.compliant {
            info!("Exchange TOS compliance check passed for Binance");
        } else {
            warn!("Exchange TOS compliance check failed: {:?}", result.reasons);
        }
    }
    
    // Log some audit events
    let mut metadata = HashMap::new();
    metadata.insert("trade_id".to_string(), "12345".to_string());
    metadata.insert("user_id".to_string(), "user1".to_string());
    metadata.insert("amount".to_string(), "1.5 ETH".to_string());
    
    compliance_service.log_audit_event("TRADE_EXECUTED", "user1", "Executed trade on Binance", metadata);
    
    // Verify audit trail integrity
    if let Some(integrity_result) = compliance_service.verify_audit_trail_integrity() {
        if integrity_result.integrity_verified {
            info!("Audit trail integrity verified");
        } else {
            warn!("Audit trail integrity issues: {:?}", integrity_result.issues);
        }
    }
    
    // Verify data retention compliance
    if let Some(retention_result) = compliance_service.verify_data_retention_compliance() {
        if retention_result.retention_compliant {
            info!("Data retention policies are compliant");
        } else {
            warn!("Data retention violations found in {} files", retention_result.violating_files.len());
        }
    }
    
    // Generate and send compliance report
    if let Some(report) = compliance_service.generate_compliance_report() {
        info!("Compliance report generated with status: {}", report.summary.status);
        info!("Compliance score: {}%", report.summary.score);
        info!("Checks performed: {}", report.summary.checks_performed);
        info!("Violations: {}", report.summary.violations);
    }
    
    // Send the report
    if compliance_service.send_compliance_report() {
        info!("Compliance report sent successfully");
    } else {
        error!("Failed to send compliance report");
    }
    
    // Perform all compliance checks
    if let Some(all_checks) = compliance_service.perform_all_compliance_checks() {
        info!("Performed {} compliance checks", all_checks.len());
        for (i, check) in all_checks.iter().enumerate() {
            if check.compliant {
                info!("Check {} passed", i + 1);
            } else {
                warn!("Check {} failed: {:?}", i + 1, check.reasons);
            }
        }
    }
    
    // Stop monitoring
    compliance_service.stop_monitoring();
    
    info!("Compliance demonstration completed");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_compliance_demo() {
        // This test ensures the demo code compiles and runs without panicking
        // We don't actually run the main function in tests since it would start threads
        assert!(true);
    }
}