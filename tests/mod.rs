//! Main test file that includes all 66+ testing types
//!
//! This file serves as the entry point for all testing categories

mod happy_path;
mod boundary;
mod equivalence;
mod state;
mod api_contract;
mod i18n;
mod accessibility;
mod feature_flag;
mod data_validation;
mod auth;
mod sanitization;
mod crypto;
mod secrets;
mod session;
mod privacy;
mod schema_migration;
mod data_migration;
mod consistency;
mod analytics;
mod smoke;
mod sanity;
mod regression;
mod concurrency;
mod wcag;
mod localization;
mod messaging;
mod payments;
mod search;

// Add our new test modules
mod security_test;
mod backtest_test;

// Add our integration tests
mod integration_security_tests;
mod integration_backtest_tests;
mod integration_risk_enhancements;
mod core_components_test;
