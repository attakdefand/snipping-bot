//! CEI (Checks-Effects-Interactions) Validation Tests
//!
//! This file contains tests for validating that smart contracts follow the
//! Checks-Effects-Interactions pattern to prevent reentrancy vulnerabilities.

use sniper_validation::cei::{CeiValidator, CeiViolation};

#[test]
fn test_valid_function_follows_cei() {
    let validator = CeiValidator::new();
    
    // This function follows CEI pattern:
    // 1. Checks first (require statements)
    // 2. Effects second (state changes)
    // 3. Interactions last (external calls)
    let function_code = r#"
        function safeTransfer(address to, uint256 amount) {
            // Checks
            require(to != address(0), "Invalid address");
            require(amount <= balances[msg.sender], "Insufficient balance");
            
            // Effects
            balances[msg.sender] -= amount;
            balances[to] += amount;
            
            // Interactions
            emit Transfer(msg.sender, to, amount);
        }
    "#;
    
    let result = validator.validate_function(function_code).unwrap();
    assert!(result.is_valid, "Valid CEI function should pass validation");
    assert_eq!(result.violations.len(), 0, "Valid CEI function should have no violations");
}

#[test]
fn test_external_call_before_state_change_violation() {
    let validator = CeiValidator::new();
    
    // This function violates CEI by making an external call before state changes
    let function_code = r#"
        function unsafeTransfer(address to, uint256 amount) {
            // Interaction first (VIOLATION)
            ERC20(token).transfer(to, amount);
            
            // Effects second (should be first)
            balances[msg.sender] -= amount;
            balances[to] += amount;
        }
    "#;
    
    let result = validator.validate_function(function_code).unwrap();
    assert!(!result.is_valid, "Function with CEI violation should fail validation");
    assert!(result.violations.contains(&CeiViolation::ExternalCallBeforeStateChange));
}

#[test]
fn test_multiple_external_calls_violation() {
    let validator = CeiValidator::new();
    
    // This function makes multiple external calls
    let function_code = r#"
        function multiCall(address to, uint256 amount) {
            require(amount > 0);
            
            // Effects
            balances[msg.sender] -= amount;
            balances[to] += amount;
            
            // Multiple interactions (POTENTIAL VIOLATION)
            ERC20(token1).transfer(to, amount);
            ERC20(token2).transfer(to, amount);
        }
    "#;
    
    let result = validator.validate_function(function_code).unwrap();
    assert!(!result.is_valid, "Function with multiple external calls should fail validation");
    assert!(result.violations.contains(&CeiViolation::MultipleExternalCalls));
}

#[test]
fn test_missing_validation_violation() {
    let validator = CeiValidator::new();
    
    // This function is missing validation checks
    let function_code = r#"
        function noValidation(address to, uint256 amount) {
            // No checks (VIOLATION)
            
            // Effects
            balances[msg.sender] -= amount;
            balances[to] += amount;
            
            // Interaction
            ERC20(token).transfer(to, amount);
        }
    "#;
    
    let result = validator.validate_function(function_code).unwrap();
    assert!(!result.is_valid, "Function with missing validation should fail validation");
    assert!(result.violations.contains(&CeiViolation::MissingValidation));
}

#[test]
fn test_proper_cei_with_external_call_at_end() {
    let validator = CeiValidator::new();
    
    // This function properly follows CEI with external call at the end
    let function_code = r#"
        function properTransfer(address to, uint256 amount) {
            // Checks first
            require(to != address(0), "Invalid address");
            require(amount > 0, "Amount must be positive");
            require(amount <= balances[msg.sender], "Insufficient balance");
            
            // Effects second
            balances[msg.sender] -= amount;
            balances[to] += amount;
            
            // Interaction last (CORRECT)
            bool success = ERC20(token).transfer(to, amount);
            require(success, "Transfer failed");
        }
    "#;
    
    let result = validator.validate_function(function_code).unwrap();
    assert!(result.is_valid, "Proper CEI function should pass validation");
    assert_eq!(result.violations.len(), 0, "Proper CEI function should have no violations");
}