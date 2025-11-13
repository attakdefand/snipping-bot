//! Compliance configuration loader
//!
//! This module provides functionality to load compliance configuration from TOML files.

use crate::compliance::{
    AuditTrailConfig, ComplianceConfig, DataRetentionEnforcement, ExchangeTosRules,
    JurisdictionRules, ReportingConfig,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use tracing::info;

/// Compliance configuration structure for TOML parsing
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct TomlComplianceConfig {
    #[serde(default)]
    compliance: ComplianceSection,
    #[serde(default)]
    jurisdictions: JurisdictionsSection,
    #[serde(default)]
    exchange_tos: ExchangeTosSection,
    #[serde(default)]
    data_retention: DataRetentionSection,
    #[serde(default)]
    audit_trail: AuditTrailSection,
    #[serde(default)]
    reporting: ReportingSection,
    #[serde(default)]
    monitoring: MonitoringSection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ComplianceSection {
    #[serde(default = "default_enabled")]
    enabled: bool,
}

impl Default for ComplianceSection {
    fn default() -> Self {
        Self {
            enabled: default_enabled(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct JurisdictionsSection {
    #[serde(default)]
    default_allowed_activities: Vec<String>,
    #[serde(default)]
    default_prohibited_activities: Vec<String>,
    #[serde(flatten)]
    specific_jurisdictions: HashMap<String, JurisdictionConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct JurisdictionConfig {
    #[serde(default)]
    allowed_activities: Vec<String>,
    #[serde(default)]
    prohibited_activities: Vec<String>,
    #[serde(default = "default_reporting_frequency")]
    reporting_frequency_days: u32,
    #[serde(default = "default_compliance_officer")]
    compliance_officer: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct ExchangeTosSection {
    #[serde(default = "default_max_daily_volume")]
    default_max_daily_volume_usd: f64,
    #[serde(default = "default_max_trades_per_day")]
    default_max_trades_per_day: u32,
    #[serde(default)]
    default_prohibited_pairs: Vec<String>,
    #[serde(default = "default_cooldown_period")]
    default_cooldown_period_seconds: u64,
    #[serde(flatten)]
    specific_exchanges: HashMap<String, ExchangeTosConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ExchangeTosConfig {
    #[serde(default = "default_max_daily_volume")]
    max_daily_volume_usd: f64,
    #[serde(default = "default_max_trades_per_day")]
    max_trades_per_day: u32,
    #[serde(default)]
    prohibited_pairs: Vec<String>,
    #[serde(default = "default_cooldown_period")]
    cooldown_period_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DataRetentionSection {
    #[serde(default = "default_retention_days")]
    days: u32,
    #[serde(default = "default_enabled")]
    enforcement_enabled: bool,
    #[serde(default)]
    storage_paths: Vec<String>,
    #[serde(default = "default_verification_frequency")]
    verification_frequency_hours: u32,
    #[serde(default = "default_enabled")]
    alert_on_violations: bool,
}

impl Default for DataRetentionSection {
    fn default() -> Self {
        Self {
            days: default_retention_days(),
            enforcement_enabled: default_enabled(),
            storage_paths: Vec::new(),
            verification_frequency_hours: default_verification_frequency(),
            alert_on_violations: default_enabled(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AuditTrailSection {
    #[serde(default = "default_enabled")]
    enabled: bool,
    #[serde(default = "default_log_level")]
    log_level: String,
    #[serde(default = "default_retention_days")]
    retention_days: u32,
    #[serde(default = "default_enabled")]
    encrypt_logs: bool,
}

impl Default for AuditTrailSection {
    fn default() -> Self {
        Self {
            enabled: default_enabled(),
            log_level: default_log_level(),
            retention_days: default_retention_days(),
            encrypt_logs: default_enabled(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ReportingSection {
    #[serde(default = "default_enabled")]
    enabled: bool,
    #[serde(default = "default_reporting_frequency_hours")]
    frequency_hours: u32,
    #[serde(default)]
    recipients: Vec<String>,
    #[serde(default = "default_report_format")]
    format: String,
    #[serde(default = "default_enabled")]
    detailed_findings: bool,
}

impl Default for ReportingSection {
    fn default() -> Self {
        Self {
            enabled: default_enabled(),
            frequency_hours: default_reporting_frequency_hours(),
            recipients: Vec::new(),
            format: default_report_format(),
            detailed_findings: default_enabled(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MonitoringSection {
    #[serde(default = "default_monitoring_interval")]
    interval_seconds: u64,
}

impl Default for MonitoringSection {
    fn default() -> Self {
        Self {
            interval_seconds: default_monitoring_interval(),
        }
    }
}

// Default value functions
fn default_enabled() -> bool {
    true
}
fn default_reporting_frequency() -> u32 {
    30
}
fn default_compliance_officer() -> String {
    "compliance@firm.com".to_string()
}
fn default_max_daily_volume() -> f64 {
    100000.0
}
fn default_max_trades_per_day() -> u32 {
    1000
}
fn default_cooldown_period() -> u64 {
    60
}
fn default_retention_days() -> u32 {
    365
}
fn default_verification_frequency() -> u32 {
    1
}
fn default_log_level() -> String {
    "INFO".to_string()
}
fn default_reporting_frequency_hours() -> u32 {
    24
}
fn default_report_format() -> String {
    "JSON".to_string()
}
fn default_monitoring_interval() -> u64 {
    3600
}

/// Load compliance configuration from a TOML file
pub fn load_compliance_config(file_path: &str) -> Result<ComplianceConfig, String> {
    info!("Loading compliance configuration from {}", file_path);

    // Read the file contents
    let contents = fs::read_to_string(file_path)
        .map_err(|e| format!("Failed to read compliance config file: {}", e))?;

    // Parse the TOML
    let toml_config: TomlComplianceConfig = toml::from_str(&contents)
        .map_err(|e| format!("Failed to parse compliance config TOML: {}", e))?;

    // Convert to our internal ComplianceConfig structure
    let config = convert_toml_config(toml_config);

    info!("Successfully loaded compliance configuration");
    Ok(config)
}

/// Convert TOML configuration to internal ComplianceConfig
fn convert_toml_config(toml_config: TomlComplianceConfig) -> ComplianceConfig {
    let mut jurisdiction_rules = HashMap::new();

    // Convert jurisdiction rules
    for (jurisdiction, rules) in toml_config.jurisdictions.specific_jurisdictions {
        jurisdiction_rules.insert(
            jurisdiction,
            JurisdictionRules {
                allowed_activities: rules.allowed_activities,
                prohibited_activities: rules.prohibited_activities,
                reporting_frequency_days: rules.reporting_frequency_days,
                compliance_officer: rules.compliance_officer,
            },
        );
    }

    let mut exchange_tos_rules = HashMap::new();

    // Convert exchange TOS rules
    for (exchange, rules) in toml_config.exchange_tos.specific_exchanges {
        exchange_tos_rules.insert(
            exchange,
            ExchangeTosRules {
                max_daily_volume_usd: rules.max_daily_volume_usd,
                max_trades_per_day: rules.max_trades_per_day,
                prohibited_pairs: rules.prohibited_pairs,
                cooldown_period_seconds: rules.cooldown_period_seconds,
            },
        );
    }

    ComplianceConfig {
        enabled: toml_config.compliance.enabled,
        jurisdiction_rules,
        data_retention_days: toml_config.data_retention.days,
        exchange_tos_rules,
        audit_trail_config: AuditTrailConfig {
            enabled: toml_config.audit_trail.enabled,
            log_level: toml_config.audit_trail.log_level,
            retention_days: toml_config.audit_trail.retention_days,
            encrypt_logs: toml_config.audit_trail.encrypt_logs,
        },
        reporting_config: ReportingConfig {
            enabled: toml_config.reporting.enabled,
            frequency_hours: toml_config.reporting.frequency_hours,
            recipients: toml_config.reporting.recipients,
            format: toml_config.reporting.format,
            detailed_findings: toml_config.reporting.detailed_findings,
        },
        data_retention_enforcement: DataRetentionEnforcement {
            enabled: toml_config.data_retention.enforcement_enabled,
            storage_paths: toml_config.data_retention.storage_paths,
            verification_frequency_hours: toml_config.data_retention.verification_frequency_hours,
            alert_on_violations: toml_config.data_retention.alert_on_violations,
        },
    }
}

/// Create a default compliance configuration
pub fn create_default_compliance_config() -> ComplianceConfig {
    info!("Creating default compliance configuration");

    let mut jurisdiction_rules = HashMap::new();
    jurisdiction_rules.insert(
        "US".to_string(),
        JurisdictionRules {
            allowed_activities: vec!["spot_trading".to_string(), "limit_orders".to_string()],
            prohibited_activities: vec![
                "margin_trading".to_string(),
                "short_selling".to_string(),
                "leverage_trading".to_string(),
            ],
            reporting_frequency_days: 30,
            compliance_officer: "compliance@firm.com".to_string(),
        },
    );

    jurisdiction_rules.insert(
        "EU".to_string(),
        JurisdictionRules {
            allowed_activities: vec!["spot_trading".to_string(), "limit_orders".to_string()],
            prohibited_activities: vec!["margin_trading".to_string(), "short_selling".to_string()],
            reporting_frequency_days: 15,
            compliance_officer: "compliance-eu@firm.com".to_string(),
        },
    );

    let mut exchange_tos_rules = HashMap::new();
    exchange_tos_rules.insert(
        "binance".to_string(),
        ExchangeTosRules {
            max_daily_volume_usd: 100000.0,
            max_trades_per_day: 1000,
            prohibited_pairs: vec!["ETH/BTC".to_string()],
            cooldown_period_seconds: 60,
        },
    );

    exchange_tos_rules.insert(
        "coinbase".to_string(),
        ExchangeTosRules {
            max_daily_volume_usd: 50000.0,
            max_trades_per_day: 500,
            prohibited_pairs: vec![],
            cooldown_period_seconds: 30,
        },
    );

    ComplianceConfig {
        enabled: true,
        jurisdiction_rules,
        data_retention_days: 365,
        exchange_tos_rules,
        audit_trail_config: AuditTrailConfig {
            enabled: true,
            log_level: "INFO".to_string(),
            retention_days: 365,
            encrypt_logs: true,
        },
        reporting_config: ReportingConfig {
            enabled: true,
            frequency_hours: 24,
            recipients: vec![
                "compliance@firm.com".to_string(),
                "audit@firm.com".to_string(),
            ],
            format: "JSON".to_string(),
            detailed_findings: true,
        },
        data_retention_enforcement: DataRetentionEnforcement {
            enabled: true,
            storage_paths: vec![
                "/var/log/snipping-bot".to_string(),
                "/var/data/snipping-bot".to_string(),
            ],
            verification_frequency_hours: 1,
            alert_on_violations: true,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_compliance_config() {
        let config = create_default_compliance_config();
        assert!(config.enabled);
        assert_eq!(config.jurisdiction_rules.len(), 2);
        assert_eq!(config.exchange_tos_rules.len(), 2);
        assert!(config.audit_trail_config.enabled);
        assert!(config.reporting_config.enabled);
        assert!(config.data_retention_enforcement.enabled);
    }

    #[test]
    fn test_convert_toml_config() {
        let toml_config = TomlComplianceConfig::default();
        let config = convert_toml_config(toml_config);

        assert!(config.enabled);
        assert_eq!(config.data_retention_days, 365);
        assert!(config.audit_trail_config.enabled);
        assert!(config.reporting_config.enabled);
        assert!(config.data_retention_enforcement.enabled);
    }
}
