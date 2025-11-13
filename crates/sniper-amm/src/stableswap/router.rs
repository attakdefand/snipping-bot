//! Stableswap router implementation for Curve-style AMMs
use super::math::{StableSwapConfig, StableSwapMath};
use anyhow::Result;
use sniper_core::types::{ExecReceipt, TradePlan};
use tracing::{debug, info};

/// Stableswap router for executing trades on Curve-style AMMs
#[derive(Debug, Clone)]
pub struct StableSwapRouter {
    /// Configuration for the stableswap pool
    config: StableSwapConfig,
    /// Mathematical functions for stableswap calculations
    math: StableSwapMath,
}

impl StableSwapRouter {
    /// Create a new stableswap router
    pub fn new(config: StableSwapConfig) -> Self {
        let math = StableSwapMath::new(config.clone());
        Self { config, math }
    }

    /// Get a quote for a trade on a stableswap pool
    pub fn get_quote(&self, plan: &TradePlan) -> Result<u128> {
        info!(
            "Getting quote for stableswap trade on chain {} with fee {}",
            plan.chain.name, self.config.fee
        );

        // For stableswap, we need to know which tokens are being traded
        // In a real implementation, we would look up the pool based on the tokens
        // For this implementation, we'll use the math module to calculate the output

        let amount_out = self.math.calculate_swap_amount_out(
            0, // token_in_index (would be determined from tokens)
            1, // token_out_index (would be determined from tokens)
            plan.amount_in,
        )?;

        debug!("Quote: {} -> {}", plan.amount_in, amount_out);
        Ok(amount_out)
    }

    /// Execute a trade on a stableswap pool
    pub async fn execute_trade(&self, plan: &TradePlan) -> Result<ExecReceipt> {
        info!("Executing stableswap trade with fee {}", self.config.fee);

        // In a real implementation, this would interact with the blockchain
        // For now, we'll simulate a successful trade

        // Calculate the expected output
        let _amount_out = self.get_quote(plan)?;

        // Simulate some processing time
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let receipt = ExecReceipt {
            tx_hash: format!("0xstable_{}", plan.idem_key),
            success: true,
            block: 12345678,
            gas_used: 150000, // Stableswap trades typically use more gas
            fees_paid_wei: 3000000000000000, // 0.003 ETH in wei
            failure_reason: None,
        };

        info!("Stableswap trade executed: {}", receipt.tx_hash);
        Ok(receipt)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sniper_core::types::{ChainRef, ExecMode, ExitRules, GasPolicy};

    #[test]
    fn test_stableswap_router_creation() {
        let config = StableSwapConfig {
            amplification_coefficient: 100,
            fee: 4000000,          // 0.04%
            admin_fee: 5000000000, // 50%
        };

        let router = StableSwapRouter::new(config);
        assert_eq!(router.config.amplification_coefficient, 100);
        assert_eq!(router.config.fee, 4000000);
        assert_eq!(router.config.admin_fee, 5000000000);
    }

    #[tokio::test]
    async fn test_stableswap_trade_execution() {
        let config = StableSwapConfig {
            amplification_coefficient: 100,
            fee: 4000000,          // 0.04%
            admin_fee: 5000000000, // 50%
        };

        let router = StableSwapRouter::new(config);

        let plan = TradePlan {
            chain: ChainRef {
                name: "ethereum".to_string(),
                id: 1,
            },
            router: "0xStableSwapRouter".to_string(),
            token_in: "0xTokenIn".to_string(),
            token_out: "0xTokenOut".to_string(),
            amount_in: 1000000000000000000, // 1 ETH
            min_out: 990000000000000000,    // 0.99 ETH worth of tokens (0.1% slippage)
            mode: ExecMode::Mempool,
            gas: GasPolicy {
                max_fee_gwei: 50,
                max_priority_gwei: 2,
            },
            exits: ExitRules::default(),
            idem_key: "test-key".to_string(),
        };

        let receipt = router.execute_trade(&plan).await.unwrap();
        assert!(receipt.success);
        assert_eq!(receipt.gas_used, 150000);
        assert!(receipt.tx_hash.starts_with("0xstable_"));
    }
}
