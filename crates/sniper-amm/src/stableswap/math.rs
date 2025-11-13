//! Mathematical functions for Stableswap (Curve-style) AMMs
use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Configuration for a stableswap pool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StableSwapConfig {
    /// Amplification coefficient (A)
    pub amplification_coefficient: u64,
    /// Swap fee (in basis points, e.g., 4000000 = 0.04%)
    pub fee: u64,
    /// Admin fee (in basis points, e.g., 5000000000 = 50%)
    pub admin_fee: u64,
}

/// Mathematical functions for stableswap calculations
#[derive(Debug, Clone)]
pub struct StableSwapMath {
    config: StableSwapConfig,
}

impl StableSwapMath {
    /// Create a new stableswap math instance
    pub fn new(config: StableSwapConfig) -> Self {
        Self { config }
    }

    /// Calculate the invariant (D) for a stableswap pool
    /// This is the core Stableswap equation: A * sum(x_i) * prod(x_i) + D = A * D * n^n + D^(n+1) / (n^n * prod(x_i))
    /// For simplicity, we'll use an approximation
    pub fn calculate_invariant(&self, balances: &[u128]) -> Result<u128> {
        let n = balances.len() as u128;
        if n == 0 {
            return Ok(0);
        }

        // Sum of all balances
        let sum_x = balances.iter().sum::<u128>();

        // For a simple case with equal balances, D ≈ sum_x
        // In a real implementation, this would use the full Stableswap formula
        Ok(sum_x)
    }

    /// Calculate the amount of token_out received for a given amount of token_in
    pub fn calculate_swap_amount_out(
        &self,
        _token_in_index: usize,
        _token_out_index: usize,
        amount_in: u128,
    ) -> Result<u128> {
        // Simple calculation with fee deduction
        // In a real implementation, this would use the full Stableswap formula
        let fee_amount = (amount_in * (self.config.fee as u128)) / 10000000000u128;
        let amount_in_after_fee = amount_in - fee_amount;

        // For stablecoins, we assume approximately 1:1 exchange rate
        // Minus a small fee
        Ok(amount_in_after_fee)
    }

    /// Calculate the amount of token_in needed for a given amount of token_out
    pub fn calculate_swap_amount_in(
        &self,
        _token_in_index: usize,
        _token_out_index: usize,
        amount_out: u128,
    ) -> Result<u128> {
        // Simple calculation with fee addition
        // In a real implementation, this would use the full Stableswap formula
        let amount_in_before_fee = amount_out;
        let fee_amount = (amount_in_before_fee * (self.config.fee as u128))
            / (10000000000u128 - (self.config.fee as u128));
        let amount_in = amount_in_before_fee + fee_amount;

        Ok(amount_in)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stableswap_config() {
        let config = StableSwapConfig {
            amplification_coefficient: 100,
            fee: 4000000,          // 0.04%
            admin_fee: 5000000000, // 50%
        };

        assert_eq!(config.amplification_coefficient, 100);
        assert_eq!(config.fee, 4000000);
        assert_eq!(config.admin_fee, 5000000000);
    }

    #[test]
    fn test_stableswap_math_creation() {
        let config = StableSwapConfig {
            amplification_coefficient: 100,
            fee: 4000000,
            admin_fee: 5000000000,
        };

        let math = StableSwapMath::new(config);
        assert_eq!(math.config.amplification_coefficient, 100);
    }

    #[test]
    fn test_calculate_swap_amount_out() {
        let config = StableSwapConfig {
            amplification_coefficient: 100,
            fee: 4000000,          // 0.04%
            admin_fee: 5000000000, // 50%
        };

        let math = StableSwapMath::new(config);
        let amount_out = math
            .calculate_swap_amount_out(0, 1, 1000000000000000000)
            .unwrap(); // 1 ETH

        // With 0.04% fee, we should get 0.9996 ETH
        assert_eq!(amount_out, 999600000000000000);
    }

    #[test]
    fn test_calculate_swap_amount_in() {
        let config = StableSwapConfig {
            amplification_coefficient: 100,
            fee: 4000000,          // 0.04%
            admin_fee: 5000000000, // 50%
        };

        let math = StableSwapMath::new(config);
        let amount_in = math
            .calculate_swap_amount_in(0, 1, 999600000000000000)
            .unwrap(); // 0.9996 ETH

        // We should need approximately 1 ETH to get 0.9996 ETH out
        assert!(amount_in >= 999999000000000000); // Very close to 1 ETH
        assert!(amount_in <= 1000001000000000000); // Within a small tolerance
    }
}
