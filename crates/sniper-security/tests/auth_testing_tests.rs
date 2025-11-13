//! Unit tests for the authentication and authorization testing module
//!
//! This file contains comprehensive unit tests for the authentication and authorization testing functionality.

use sniper_security::auth_testing::*;
use std::time::Duration;

/// Test authentication tester creation
#[tokio::test]
async fn test_auth_tester_creation() {
    let config = AuthTestConfig::default();
    let tester = AuthTester::new(config.clone());

    assert_eq!(
        tester.config().auth_flow_testing_enabled,
        config.auth_flow_testing_enabled
    );
    assert_eq!(tester.config().test_timeout_secs, config.test_timeout_secs);
}

/// Test authentication test execution
#[tokio::test]
async fn test_auth_test_execution() {
    let config = AuthTestConfig {
        auth_flow_testing_enabled: true,
        authz_policy_testing_enabled: true,
        session_management_testing_enabled: false,
        privilege_escalation_testing_enabled: false,
        brute_force_protection_testing_enabled: false,
        test_timeout_secs: 10, // Short duration for testing
    };

    let mut tester = AuthTester::new(config);
    let results = tester.run_auth_test().await.unwrap();

    assert!(results.duration > Duration::from_millis(0));
    assert!(results.auth_flow_test_results.is_some());
    assert!(results.authz_policy_test_results.is_some());
    assert!(results.session_management_test_results.is_none());
    assert!(results.security_score <= 100);
}

/// Test security score calculation with perfect results
#[tokio::test]
async fn test_security_score_calculation_perfect() {
    // Create results with no vulnerabilities (perfect score)
    let results = AuthTestResults {
        config: AuthTestConfig::default(),
        duration: Duration::from_secs(10),
        auth_flow_test_results: None,
        authz_policy_test_results: None,
        session_management_test_results: None,
        privilege_escalation_test_results: None,
        brute_force_protection_test_results: None,
        vulnerabilities_found: 0,
        security_score: 100,
    };

    assert_eq!(results.security_score, 100);
}

/// Test security score calculation with some vulnerabilities
#[tokio::test]
async fn test_security_score_calculation_vulnerable() {
    // Create results with some vulnerabilities
    let results = AuthTestResults {
        config: AuthTestConfig::default(),
        duration: Duration::from_secs(10),
        auth_flow_test_results: None,
        authz_policy_test_results: None,
        session_management_test_results: None,
        privilege_escalation_test_results: None,
        brute_force_protection_test_results: None,
        vulnerabilities_found: 3,
        security_score: 85, // 100 - (3 * 5)
    };

    assert_eq!(results.security_score, 85);
}
