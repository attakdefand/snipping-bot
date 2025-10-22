//! Input validation module.
//!
//! This module provides functionality for validating inputs to prevent common
//! vulnerabilities like injection attacks, buffer overflows, etc.

use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Input validation rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRules {
    /// Maximum length for string inputs
    pub max_length: Option<usize>,
    /// Minimum length for string inputs
    pub min_length: Option<usize>,
    /// Regular expression pattern for validation
    pub pattern: Option<String>,
    /// Whether the input is required
    pub required: bool,
    /// List of allowed values
    pub allowed_values: Option<Vec<String>>,
}

/// Input validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Whether the input is valid
    pub is_valid: bool,
    /// Error message if validation failed
    pub error_message: Option<String>,
}

/// Input validator
pub struct InputValidator;

impl InputValidator {
    /// Create a new input validator
    pub fn new() -> Self {
        Self
    }

    /// Validate a string input against rules
    pub fn validate_string(
        &self,
        input: &str,
        rules: &ValidationRules,
    ) -> Result<ValidationResult> {
        // Check if required
        if rules.required && input.is_empty() {
            return Ok(ValidationResult {
                is_valid: false,
                error_message: Some("Input is required".to_string()),
            });
        }

        // Check minimum length
        if let Some(min_length) = rules.min_length {
            if input.len() < min_length {
                return Ok(ValidationResult {
                    is_valid: false,
                    error_message: Some(format!(
                        "Input must be at least {} characters",
                        min_length
                    )),
                });
            }
        }

        // Check maximum length
        if let Some(max_length) = rules.max_length {
            if input.len() > max_length {
                return Ok(ValidationResult {
                    is_valid: false,
                    error_message: Some(format!(
                        "Input must be no more than {} characters",
                        max_length
                    )),
                });
            }
        }

        // Check pattern
        if let Some(pattern) = &rules.pattern {
            let regex = regex::Regex::new(pattern)?;
            if !regex.is_match(input) {
                return Ok(ValidationResult {
                    is_valid: false,
                    error_message: Some("Input does not match required pattern".to_string()),
                });
            }
        }

        // Check allowed values
        if let Some(allowed_values) = &rules.allowed_values {
            if !allowed_values.is_empty() && !allowed_values.contains(&input.to_string()) {
                return Ok(ValidationResult {
                    is_valid: false,
                    error_message: Some("Input is not in the list of allowed values".to_string()),
                });
            }
        }

        Ok(ValidationResult {
            is_valid: true,
            error_message: None,
        })
    }

    /// Validate an address
    pub fn validate_address(&self, address: &str) -> Result<ValidationResult> {
        // Basic Ethereum address validation
        let rules = ValidationRules {
            max_length: Some(42),
            min_length: Some(42),
            pattern: Some(r"^0x[a-fA-F0-9]{40}$".to_string()),
            required: true,
            allowed_values: None,
        };

        self.validate_string(address, &rules)
    }

    /// Validate an amount
    pub fn validate_amount(&self, amount: &str) -> Result<ValidationResult> {
        // Basic numeric validation
        let rules = ValidationRules {
            max_length: Some(78), // 78 digits for uint256
            min_length: Some(1),
            pattern: Some(r"^[0-9]+$".to_string()),
            required: true,
            allowed_values: None,
        };

        self.validate_string(amount, &rules)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_validator_creation() {
        let _validator = InputValidator::new();
        assert!(true, "Input validator created successfully");
    }

    #[test]
    fn test_valid_address() {
        let validator = InputValidator::new();
        let result = validator
            .validate_address("0x742d35Cc6634C0532925a3b8D4C9db96C4b4d8b6")
            .unwrap();
        assert!(result.is_valid);
    }

    #[test]
    fn test_invalid_address_length() {
        let validator = InputValidator::new();
        let result = validator
            .validate_address("0x742d35Cc6634C0532925a3b8D4C9db96C4b4d8")
            .unwrap();
        assert!(!result.is_valid);
        assert!(result.error_message.is_some());
    }

    #[test]
    fn test_invalid_address_pattern() {
        let validator = InputValidator::new();
        let result = validator
            .validate_address("0x742d35Cc6634C0532925a3b8D4C9db96C4b4d8bG")
            .unwrap();
        assert!(!result.is_valid);
        assert!(result.error_message.is_some());
    }

    #[test]
    fn test_valid_amount() {
        let validator = InputValidator::new();
        let result = validator.validate_amount("1000000000000000000").unwrap();
        assert!(result.is_valid);
    }

    #[test]
    fn test_invalid_amount() {
        let validator = InputValidator::new();
        let result = validator.validate_amount("abc").unwrap();
        assert!(!result.is_valid);
        assert!(result.error_message.is_some());
    }

    #[test]
    fn test_empty_required_input() {
        let validator = InputValidator::new();
        let rules = ValidationRules {
            max_length: None,
            min_length: None,
            pattern: None,
            required: true,
            allowed_values: None,
        };

        let result = validator.validate_string("", &rules).unwrap();
        assert!(!result.is_valid);
        assert_eq!(result.error_message, Some("Input is required".to_string()));
    }
}
