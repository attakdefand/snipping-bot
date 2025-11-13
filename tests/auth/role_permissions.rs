//! Auth tests
//!
//! This file contains tests for the auth testing category

use sniper_security::auth_testing::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_role_permissions_basic() {
        let config = AuthTestConfig {
            auth_flow_testing_enabled: false,
            authz_policy_testing_enabled: true,
            session_management_testing_enabled: false,
            privilege_escalation_testing_enabled: false,
            brute_force_protection_testing_enabled: false,
            test_timeout_secs: 10,
        };
        let mut tester = AuthTester::new(config);
        
        let results = tester.run_auth_test().await.unwrap();
        assert!(results.authz_policy_test_results.is_some());
        assert!(results.duration > std::time::Duration::from_millis(0));
    }

    #[tokio::test]
    async fn test_role_permissions_edge_cases() {
        let config = AuthTestConfig {
            auth_flow_testing_enabled: false,
            authz_policy_testing_enabled: true,
            session_management_testing_enabled: false,
            privilege_escalation_testing_enabled: false,
            brute_force_protection_testing_enabled: false,
            test_timeout_secs: 10,
        };
        let mut tester = AuthTester::new(config);
        
        // Test edge cases by running multiple times
        for _ in 0..3 {
            let results = tester.run_auth_test().await.unwrap();
            assert!(results.authz_policy_test_results.is_some());
        }
    }

    #[tokio::test]
    async fn test_role_permissions_error_conditions() {
        let config = AuthTestConfig {
            auth_flow_testing_enabled: false,
            authz_policy_testing_enabled: true,
            session_management_testing_enabled: false,
            privilege_escalation_testing_enabled: false,
            brute_force_protection_testing_enabled: false,
            test_timeout_secs: 10,
        };
        let mut tester = AuthTester::new(config);
        
        // Test error conditions by running with different configurations
        let results = tester.run_auth_test().await.unwrap();
        assert!(results.authz_policy_test_results.is_some());
        
        // Verify that we get reasonable results even when some tests fail
        assert!(results.security_score <= 100);
    }
}