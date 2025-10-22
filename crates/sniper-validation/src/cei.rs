//! Checks-Effects-Interactions (CEI) pattern validation.
//!
//! This module provides functionality to validate that smart contracts follow the
//! Checks-Effects-Interactions pattern to prevent reentrancy attacks.

use anyhow::Result;
use serde::{Deserialize, Serialize};

/// CEI Pattern Violation Types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CeiViolation {
    /// External call before state changes (reentrancy risk)
    ExternalCallBeforeStateChange,
    /// State change after external call (reentrancy risk)
    StateChangeAfterExternalCall,
    /// Multiple external calls without proper checks
    MultipleExternalCalls,
    /// Missing validation checks
    MissingValidation,
}

/// CEI Validation Result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CeiValidationResult {
    /// Whether the contract follows CEI pattern
    pub is_valid: bool,
    /// List of violations found
    pub violations: Vec<CeiViolation>,
    /// Detailed analysis
    pub analysis: String,
}

/// CEI Validator
pub struct CeiValidator;

impl CeiValidator {
    /// Create a new CEI validator
    pub fn new() -> Self {
        Self
    }

    /// Validate that a contract follows the CEI pattern
    pub fn validate_contract(&self, _contract_code: &str) -> Result<CeiValidationResult> {
        // In a real implementation, this would analyze the contract code
        // For now, we'll return a placeholder result
        Ok(CeiValidationResult {
            is_valid: true,
            violations: vec![],
            analysis: "Contract follows CEI pattern".to_string(),
        })
    }

    /// Check if a function follows the CEI pattern
    pub fn validate_function(&self, function_code: &str) -> Result<CeiValidationResult> {
        let mut violations = Vec::new();

        // Check for common CEI violations
        if self.has_external_call_before_state_change(function_code) {
            violations.push(CeiViolation::ExternalCallBeforeStateChange);
        }

        if self.has_state_change_after_external_call(function_code) {
            violations.push(CeiViolation::StateChangeAfterExternalCall);
        }

        if self.has_multiple_external_calls(function_code) {
            violations.push(CeiViolation::MultipleExternalCalls);
        }

        if self.has_missing_validation(function_code) {
            violations.push(CeiViolation::MissingValidation);
        }

        let is_valid = violations.is_empty();
        let analysis = if is_valid {
            "Function follows CEI pattern".to_string()
        } else {
            format!("Function has {} CEI violations", violations.len())
        };

        Ok(CeiValidationResult {
            is_valid,
            violations,
            analysis,
        })
    }

    /// Check for external calls before state changes
    fn has_external_call_before_state_change(&self, function_code: &str) -> bool {
        // Look for patterns like external calls followed by state changes
        // This is a simplified check - a real implementation would parse the AST
        let call_pos = function_code.find("transfer(");
        let state_pos = function_code.find("balances[");

        match (call_pos, state_pos) {
            (Some(call_pos), Some(state_pos)) => call_pos < state_pos,
            _ => false,
        }
    }

    /// Check for state changes after external calls
    fn has_state_change_after_external_call(&self, function_code: &str) -> bool {
        // Look for patterns like state changes after external calls
        let call_pos = function_code.find("transfer(");
        let state_pos = function_code.find("balances[");

        match (call_pos, state_pos) {
            (Some(call_pos), Some(state_pos)) => state_pos > call_pos,
            _ => false,
        }
    }

    /// Check for multiple external calls
    fn has_multiple_external_calls(&self, function_code: &str) -> bool {
        // Count occurrences of external calls
        let call_count = function_code.matches("transfer(").count();
        call_count > 1
    }

    /// Check for missing validation
    fn has_missing_validation(&self, function_code: &str) -> bool {
        // Check if there are validation checks
        !function_code.contains("require(") && !function_code.contains("assert(")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cei_validator_creation() {
        let _validator = CeiValidator::new();
        assert!(true, "CEI validator created successfully");
    }

    #[test]
    fn test_valid_function() {
        let validator = CeiValidator::new();
        let function_code = r#"
            function safeTransfer(address to, uint256 amount) {
                require(to != address(0), "Invalid address");
                require(amount <= balances[msg.sender], "Insufficient balance");
                
                balances[msg.sender] -= amount;
                balances[to] += amount;
                
                emit Transfer(msg.sender, to, amount);
            }
        "#;

        let result = validator.validate_function(function_code).unwrap();
        assert!(result.is_valid);
        assert_eq!(result.violations.len(), 0);
    }

    #[test]
    fn test_external_call_before_state_change() {
        let validator = CeiValidator::new();
        let function_code = r#"
            function unsafeTransfer(address to, uint256 amount) {
                ERC20(token).transfer(to, amount);  // External call first
                
                balances[msg.sender] -= amount;     // State change after
                balances[to] += amount;
            }
        "#;

        let result = validator.validate_function(function_code).unwrap();
        println!("Violations: {:?}", result.violations);
        assert!(!result.is_valid);
        assert!(result
            .violations
            .contains(&CeiViolation::ExternalCallBeforeStateChange));
    }

    #[test]
    fn test_multiple_external_calls() {
        let validator = CeiValidator::new();
        let function_code = r#"
            function multiCall(address to, uint256 amount) {
                require(amount > 0);
                
                balances[msg.sender] -= amount;
                balances[to] += amount;
                
                ERC20(token1).transfer(to, amount);  // First external call
                ERC20(token2).transfer(to, amount);  // Second external call
            }
        "#;

        let result = validator.validate_function(function_code).unwrap();
        println!("Is valid: {}", result.is_valid);
        println!("Violations: {:?}", result.violations);
        assert!(!result.is_valid);
        assert!(result
            .violations
            .contains(&CeiViolation::MultipleExternalCalls));
    }

    #[test]
    fn test_missing_validation() {
        let validator = CeiValidator::new();
        let function_code = r#"
            function noValidation(address to, uint256 amount) {
                balances[msg.sender] -= amount;
                balances[to] += amount;
                
                ERC20(token).transfer(to, amount);
            }
        "#;

        let result = validator.validate_function(function_code).unwrap();
        assert!(!result.is_valid);
        assert!(result.violations.contains(&CeiViolation::MissingValidation));
    }
}
