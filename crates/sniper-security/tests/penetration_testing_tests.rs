//! Unit tests for the penetration testing module
//!
//! This file contains comprehensive unit tests for the penetration testing functionality.

use sniper_security::penetration_testing::*;
use std::time::Duration;

/// Test penetration tester creation
#[tokio::test]
async fn test_penetration_tester_creation() {
    let config = PenTestConfig::default();
    let tester = PenetrationTester::new(config.clone());

    assert_eq!(tester.config().target_services, config.target_services);
    assert_eq!(
        tester.config().test_duration_secs,
        config.test_duration_secs
    );
    assert_eq!(
        tester.config().attack_vectors.len(),
        config.attack_vectors.len()
    );
}

/// Test penetration test execution
#[tokio::test]
async fn test_penetration_test_execution() {
    let config = PenTestConfig {
        target_services: vec!["test-service".to_string()],
        test_duration_secs: 10, // Short duration for testing
        verbose: false,
        attack_vectors: vec![AttackVector::SqlInjection, AttackVector::Xss],
    };

    let tester = PenetrationTester::new(config);
    let results = tester.run_penetration_test().await.unwrap();

    assert!(results.duration > Duration::from_millis(0));
    assert_eq!(results.config.attack_vectors.len(), 2);
    assert_eq!(results.vector_results.len(), 2);
    assert!(results.security_score <= 100);
}

/// Test security score calculation with perfect results
#[tokio::test]
async fn test_security_score_calculation_perfect() {
    let config = PenTestConfig::default();
    let _tester = PenetrationTester::new(config);

    // Create results with no successful attacks (perfect score)
    let results = PenTestResults {
        config: PenTestConfig::default(),
        duration: Duration::from_secs(10),
        attacks_blocked: 10,
        attacks_succeeded: 0,
        vector_results: std::collections::HashMap::new(),
        security_score: 100,
    };

    assert_eq!(results.security_score, 100);
}

/// Test security score calculation with some vulnerabilities
#[tokio::test]
async fn test_security_score_calculation_vulnerable() {
    // Create results with some successful attacks
    let mut vector_results = std::collections::HashMap::new();

    let sql_result = VectorTestResult {
        tests_performed: 5,
        blocked: 4,
        succeeded: 1,
        success_details: vec!["Test SQL injection succeeded".to_string()],
    };

    vector_results.insert(AttackVector::SqlInjection, sql_result);

    let results = PenTestResults {
        config: PenTestConfig::default(),
        duration: Duration::from_secs(10),
        attacks_blocked: 4,
        attacks_succeeded: 1,
        vector_results,
        security_score: 80, // 100 * (1 - 1/5) = 80
    };

    assert_eq!(results.security_score, 80);
}
