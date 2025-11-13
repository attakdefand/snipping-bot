//! Integration tests for all security features
//!
//! This file contains comprehensive integration tests for all security features
//! implemented in the sniper-security crate.

use anyhow::Result;
use sniper_security::{
    SecurityConfig, 
    SecuritySystem,
    penetration_testing,
    vulnerability_scanning,
    api_security,
    auth_testing
};
use std::time::Duration;

#[cfg(test)]
mod tests {
    use super::*;

    /// Test the complete security system integration
    #[tokio::test]
    async fn test_security_system_integration() -> Result<()> {
        let config = SecurityConfig::default();
        let security = SecuritySystem::new(config);
        
        // Test that all security components are properly integrated
        assert!(security.check_key_management());
        assert!(security.check_monitoring());
        assert!(security.check_risk_management());
        assert!(security.check_compliance());
        
        // Run all checks should succeed
        let result = security.run_all_checks().await?;
        assert!(result);
        
        Ok(())
    }

    /// Test penetration testing integration
    #[tokio::test]
    async fn test_penetration_testing_integration() -> Result<()> {
        let config = SecurityConfig::default();
        let security = SecuritySystem::new(config);
        
        // Run penetration testing
        let results = security.run_penetration_testing().await?;
        
        // Verify results structure
        assert!(results.duration > Duration::from_millis(0));
        assert!(!results.vector_results.is_empty());
        assert!(results.security_score <= 100);
        
        Ok(())
    }

    /// Test vulnerability scanning integration
    #[tokio::test]
    async fn test_vulnerability_scanning_integration() -> Result<()> {
        let config = SecurityConfig::default();
        let security = SecuritySystem::new(config);
        
        // Run vulnerability scanning
        let results = security.run_vulnerability_scanning().await?;
        
        // Verify results structure
        assert!(results.duration > Duration::from_millis(0));
        assert!(results.security_score <= 100);
        
        Ok(())
    }

    /// Test API security testing integration
    #[tokio::test]
    async fn test_api_security_testing_integration() -> Result<()> {
        let config = SecurityConfig::default();
        let security = SecuritySystem::new(config);
        
        // Run API security testing
        let results = security.run_api_security_testing().await?;
        
        // Verify results structure
        assert!(results.duration > Duration::from_millis(0));
        assert!(results.security_score <= 100);
        
        Ok(())
    }

    /// Test authentication and authorization testing integration
    #[tokio::test]
    async fn test_auth_testing_integration() -> Result<()> {
        let config = SecurityConfig::default();
        let security = SecuritySystem::new(config);
        
        // Run authentication and authorization testing
        let results = security.run_auth_testing().await?;
        
        // Verify results structure
        assert!(results.duration > Duration::from_millis(0));
        assert!(results.security_score <= 100);
        
        Ok(())
    }

    /// Test penetration testing module directly
    #[tokio::test]
    async fn test_penetration_testing_module() -> Result<()> {
        let config = penetration_testing::PenTestConfig::default();
        let tester = penetration_testing::PenetrationTester::new(config);
        
        // Run penetration test
        let results = tester.run_penetration_test().await?;
        
        // Verify results
        assert!(results.duration > Duration::from_millis(0));
        assert!(!results.vector_results.is_empty());
        assert!(results.security_score <= 100);
        
        Ok(())
    }

    /// Test vulnerability scanning module directly
    #[tokio::test]
    async fn test_vulnerability_scanning_module() -> Result<()> {
        let config = vulnerability_scanning::VulnerabilityScanConfig::default();
        let scanner = vulnerability_scanning::VulnerabilityScanner::new(config);
        
        // Run vulnerability scan
        let results = scanner.run_vulnerability_scan().await?;
        
        // Verify results
        assert!(results.duration > Duration::from_millis(0));
        assert!(results.security_score <= 100);
        
        Ok(())
    }

    /// Test API security testing module directly
    #[tokio::test]
    async fn test_api_security_module() -> Result<()> {
        let config = api_security::ApiSecurityConfig {
            target_endpoints: vec!["/api/v1/test".to_string()],
            auth_testing_enabled: true,
            rate_limit_testing_enabled: false,
            input_validation_testing_enabled: false,
            cors_testing_enabled: false,
            sql_injection_testing_enabled: false,
            xss_testing_enabled: false,
            test_timeout_secs: 10, // Short duration for testing
        };
        let tester = api_security::ApiSecurityTester::new(config);
        
        // Run API security test
        let results = tester.run_api_security_test().await?;
        
        // Verify results
        assert!(results.duration > Duration::from_millis(0));
        assert!(results.auth_test_results.is_some());
        assert!(results.security_score <= 100);
        
        Ok(())
    }

    /// Test authentication testing module directly
    #[tokio::test]
    async fn test_auth_testing_module() -> Result<()> {
        let config = auth_testing::AuthTestConfig {
            auth_flow_testing_enabled: true,
            authz_policy_testing_enabled: true,
            session_management_testing_enabled: false,
            privilege_escalation_testing_enabled: false,
            brute_force_protection_testing_enabled: false,
            test_timeout_secs: 10, // Short duration for testing
        };
        let mut tester = auth_testing::AuthTester::new(config);
        
        // Run auth test
        let results = tester.run_auth_test().await?;
        
        // Verify results
        assert!(results.duration > Duration::from_millis(0));
        assert!(results.auth_flow_test_results.is_some());
        assert!(results.authz_policy_test_results.is_some());
        assert!(results.security_score <= 100);
        
        Ok(())
    }

    /// Test security system with all components disabled
    #[tokio::test]
    async fn test_security_system_with_all_components_disabled() -> Result<()> {
        let config = SecurityConfig {
            key_management_enabled: false,
            monitoring_enabled: false,
            risk_management_enabled: false,
            compliance_enabled: false,
            penetration_testing_enabled: false,
            vulnerability_scanning_enabled: false,
            api_security_testing_enabled: false,
            auth_testing_enabled: false,
        };
        let security = SecuritySystem::new(config);
        
        // All checks should fail
        assert!(!security.check_key_management());
        assert!(!security.check_monitoring());
        assert!(!security.check_risk_management());
        assert!(!security.check_compliance());
        
        // Run all checks should still succeed (returns Ok(false))
        let result = security.run_all_checks().await?;
        assert!(!result);
        
        Ok(())
    }

    /// Test security system with only key management enabled
    #[tokio::test]
    async fn test_security_system_with_only_key_management_enabled() -> Result<()> {
        let config = SecurityConfig {
            key_management_enabled: true,
            monitoring_enabled: false,
            risk_management_enabled: false,
            compliance_enabled: false,
            penetration_testing_enabled: false,
            vulnerability_scanning_enabled: false,
            api_security_testing_enabled: false,
            auth_testing_enabled: false,
        };
        let security = SecuritySystem::new(config);
        
        // Only key management should pass
        assert!(security.check_key_management());
        assert!(!security.check_monitoring());
        assert!(!security.check_risk_management());
        assert!(!security.check_compliance());
        
        // Run all checks should succeed (returns Ok(true))
        let result = security.run_all_checks().await?;
        assert!(result);
        
        Ok(())
    }
}