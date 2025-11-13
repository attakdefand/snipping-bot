use sniper_core::types::{GasPolicy, TradePlan};
use tracing::info;

/// Gas estimation and pricing strategies
#[derive(Debug, Clone)]
pub struct GasEstimator {
    /// Base gas price in gwei
    pub base_fee_gwei: u64,
    /// Priority fee in gwei
    pub priority_fee_gwei: u64,
}

impl GasEstimator {
    /// Create a new gas estimator
    pub fn new(base_fee_gwei: u64, priority_fee_gwei: u64) -> Self {
        Self {
            base_fee_gwei,
            priority_fee_gwei,
        }
    }

    /// Estimate gas for a trade plan
    pub fn estimate_gas(&self, plan: &TradePlan) -> GasPolicy {
        info!("Estimating gas for trade on chain {}", plan.chain.name);

        // In a real implementation, this would:
        // 1. Query the blockchain for current gas prices
        // 2. Analyze the complexity of the transaction
        // 3. Apply appropriate multipliers based on urgency

        let gas_policy = GasPolicy {
            max_fee_gwei: self.base_fee_gwei + self.priority_fee_gwei,
            max_priority_gwei: self.priority_fee_gwei,
        };

        info!(
            "Gas estimate: max_fee={} gwei, priority_fee={} gwei",
            gas_policy.max_fee_gwei, gas_policy.max_priority_gwei
        );

        gas_policy
    }

    /// Adjust gas prices based on network conditions
    pub fn adjust_for_network_conditions(&mut self, network_congestion: f64) {
        // Increase gas prices during network congestion
        let multiplier = 1.0 + (network_congestion * 2.0); // Up to 3x during high congestion
        self.base_fee_gwei = (self.base_fee_gwei as f64 * multiplier) as u64;
        self.priority_fee_gwei = (self.priority_fee_gwei as f64 * multiplier) as u64;
    }
}

/// Get default gas estimator
pub fn default_gas_estimator() -> GasEstimator {
    GasEstimator::new(30, 2) // 30 gwei base fee, 2 gwei priority fee
}

#[cfg(test)]
mod tests {
    use super::*;
    use sniper_core::types::{ChainRef, ExecMode, ExitRules};

    #[test]
    fn test_gas_estimator_creation() {
        let estimator = GasEstimator::new(20, 1);
        assert_eq!(estimator.base_fee_gwei, 20);
        assert_eq!(estimator.priority_fee_gwei, 1);
    }

    #[test]
    fn test_gas_estimation() {
        let estimator = GasEstimator::new(25, 2);
        let plan = TradePlan {
            chain: ChainRef {
                name: "ethereum".to_string(),
                id: 1,
            },
            router: "0xRouterAddress".to_string(),
            token_in: "0xWETH".to_string(),
            token_out: "0xToken".to_string(),
            amount_in: 1000000000000000000, // 1 ETH
            min_out: 900000000000000000,    // 0.9 tokens
            mode: ExecMode::Mempool,
            gas: GasPolicy {
                max_fee_gwei: 50,
                max_priority_gwei: 2,
            },
            exits: ExitRules::default(),
            idem_key: "test_gas_1".to_string(),
        };

        let gas_policy = estimator.estimate_gas(&plan);
        assert_eq!(gas_policy.max_fee_gwei, 27); // 25 + 2
        assert_eq!(gas_policy.max_priority_gwei, 2);
    }

    #[test]
    fn test_network_condition_adjustment() {
        let mut estimator = GasEstimator::new(20, 1);
        estimator.adjust_for_network_conditions(0.5); // 50% congestion

        // Should increase by 2x (1.0 + 0.5 * 2.0)
        assert_eq!(estimator.base_fee_gwei, 40);
        assert_eq!(estimator.priority_fee_gwei, 2);
    }
}
