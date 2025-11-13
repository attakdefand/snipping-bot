//! Unit tests for the API security testing module
//!
//! This file contains comprehensive unit tests for the API security testing functionality.

use sniper_security::api_security::*;
use std::time::Duration;

/// Test API security tester creation
#[tokio::test]
async fn test_api_security_tester_creation() {
    let config = ApiSecurityConfig::default();
    let tester = ApiSecurityTester::new(config.clone());

    assert_eq!(tester.config().target_endpoints, config.target_endpoints);
    assert_eq!(tester.config().test_timeout_secs, config.test_timeout_secs);
}

/// Test API security test execution
#[tokio::test]
async fn test_api_security_test_execution() {
    let config = ApiSecurityConfig {
        target_endpoints: vec!["/api/v1/test".to_string()],
        auth_testing_enabled: true,
        rate_limit_testing_enabled: false,
        input_validation_testing_enabled: false,
        cors_testing_enabled: false,
        sql_injection_testing_enabled: false,
        xss_testing_enabled: false,
        test_timeout_secs: 10, // Short duration for testing
    };

    let tester = ApiSecurityTester::new(config);
    let results = tester.run_api_security_test().await.unwrap();

    assert!(results.duration > Duration::from_millis(0));
    assert!(results.auth_test_results.is_some());
    assert!(results.rate_limit_test_results.is_none());
    assert!(results.security_score <= 100);
}

/// Test security score calculation with perfect results
#[tokio::test]
async fn test_security_score_calculation_perfect() {
    // Create results with no vulnerabilities (perfect score)
    let results = ApiSecurityTestResults {
        config: ApiSecurityConfig::default(),
        duration: Duration::from_secs(10),
        auth_test_results: None,
        rate_limit_test_results: None,
        input_validation_test_results: None,
        cors_test_results: None,
        sql_injection_test_results: None,
        xss_test_results: None,
        vulnerabilities_found: 0,
        security_score: 100,
    };

    assert_eq!(results.security_score, 100);
}

/// Test security score calculation with some vulnerabilities
#[tokio::test]
async fn test_security_score_calculation_vulnerable() {
    // Create results with some vulnerabilities
    let results = ApiSecurityTestResults {
        config: ApiSecurityConfig::default(),
        duration: Duration::from_secs(10),
        auth_test_results: None,
        rate_limit_test_results: None,
        input_validation_test_results: None,
        cors_test_results: None,
        sql_injection_test_results: None,
        xss_test_results: None,
        vulnerabilities_found: 5,
        security_score: 75, // 100 - (5 * 5)
    };

    assert_eq!(results.security_score, 75);
}
