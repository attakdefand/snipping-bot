//! Security tests for the sniper bot.
//! 
//! This test file implements security checks as outlined in the DEVELOPMENT_GUIDELINES.MD
//! to ensure the snipping bot follows security best practices.

use sniper_security::{SecurityConfig, SecuritySystem};

/// Test key management security
/// 
/// This test ensures that key management follows the guidelines:
/// - Integration with HSM or Vault solutions
/// - Enforcement of least privilege access principles
#[tokio::test]
async fn test_key_management_security() {
    let config = SecurityConfig::default();
    let security = SecuritySystem::new(config);
    
    // Test that key management is properly configured
    assert!(security.check_key_management());
    println!("Key management security tests passed");
}

/// Test monitoring and audit logging
/// 
/// This test ensures that monitoring and audit logging are properly implemented:
/// - Comprehensive metrics collection
/// - Audit logging before handling real funds
#[tokio::test]
async fn test_monitoring_and_audit_logging() {
    let config = SecurityConfig::default();
    let security = SecuritySystem::new(config);
    
    // Test that monitoring is properly configured
    assert!(security.check_monitoring());
    println!("Monitoring and audit logging tests passed");
}

/// Test risk management
/// 
/// This test ensures that risk management follows the guidelines:
/// - Risk rules are properly defined and enforced
#[tokio::test]
async fn test_risk_management() {
    let config = SecurityConfig::default();
    let security = SecuritySystem::new(config);
    
    // Test that risk management is properly configured
    assert!(security.check_risk_management());
    println!("Risk management tests passed");
}

/// Test compliance constraints
/// 
/// This test ensures that compliance constraints are properly implemented:
/// - Permitted strategies are properly defined
/// - Compliance rules are enforced
#[tokio::test]
async fn test_compliance_constraints() {
    let config = SecurityConfig::default();
    let security = SecuritySystem::new(config);
    
    // Test that compliance is properly configured
    assert!(security.check_compliance());
    println!("Compliance constraint tests passed");
}

/// Test authentication and authorization
/// 
/// This test ensures that the RBAC system is properly implemented:
/// - Users can be created and assigned roles
/// - Permissions can be checked
#[tokio::test]
async fn test_authentication_authorization() {
    let config = SecurityConfig::default();
    let security = SecuritySystem::new(config);
    
    // Run authentication and authorization testing
    let results = security.run_auth_testing().await.unwrap();
    assert!(results.security_score > 80); // Should have a good security score
    println!("Authentication and authorization tests passed with security score: {}", results.security_score);
}

/// Integration test for all security components
/// 
/// This test ensures that all security components work together properly.
#[tokio::test]
async fn test_security_integration() {
    let config = SecurityConfig::default();
    let security = SecuritySystem::new(config);
    
    // Test end-to-end security flow:
    let result = security.run_all_checks().await.unwrap();
    assert!(result);
    println!("Security integration tests passed");
}