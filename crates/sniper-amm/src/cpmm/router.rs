//! CPMM router implementation for Uniswap V2-style AMMs
use super::math::{CpmmConfig, CpmmMath};
use anyhow::Result;
use sniper_core::types::{ExecReceipt, TradePlan};
use tracing::{debug, info};

/// CPMM router for executing trades on Uniswap V2-style AMMs
#[derive(Debug, Clone)]
pub struct CpmmRouter {
    /// Configuration for the CPMM pool
    config: CpmmConfig,
    /// Mathematical functions for CPMM calculations
    math: CpmmMath,
}

impl CpmmRouter {
    /// Create a new CPMM router
    pub fn new(config: CpmmConfig) -> Self {
        let math = CpmmMath::new(config.clone());
        Self { config, math }
    }

    /// Get a quote for a trade on a CPMM pool
    pub fn get_quote(&self, reserve_in: u128, reserve_out: u128, plan: &TradePlan) -> Result<u128> {
        info!(
            "Getting quote for CPMM trade on chain {} with fee {}",
            plan.chain.name, self.config.fee
        );

        let amount_out =
            self.math
                .calculate_swap_amount_out(reserve_in, reserve_out, plan.amount_in)?;

        debug!("Quote: {} -> {}", plan.amount_in, amount_out);
        Ok(amount_out)
    }

    /// Execute a trade on a CPMM pool
    pub async fn execute_trade(&self, _plan: &TradePlan) -> Result<ExecReceipt> {
        info!("Executing CPMM trade with fee {}", self.config.fee);

        // In a real implementation, this would interact with the blockchain
        // For now, we'll simulate a successful trade

        // Simulate some processing time
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let receipt = ExecReceipt {
            tx_hash: "0xcpmm_placeholder".to_string(),
            success: true,
            block: 12345678,
            gas_used: 120000, // CPMM trades typically use moderate gas
            fees_paid_wei: 2400000000000000, // 0.0024 ETH in wei
            failure_reason: None,
        };

        info!("CPMM trade executed: {}", receipt.tx_hash);
        Ok(receipt)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sniper_core::types::{ChainRef, ExecMode, ExitRules, GasPolicy};

    #[test]
    fn test_cpmm_router_creation() {
        let config = CpmmConfig {
            fee: 3000000, // 0.3%
        };

        let router = CpmmRouter::new(config);
        assert_eq!(router.config.fee, 3000000);
    }

    #[tokio::test]
    async fn test_cpmm_trade_execution() {
        let config = CpmmConfig {
            fee: 3000000, // 0.3%
        };

        let router = CpmmRouter::new(config);

        let plan = TradePlan {
            chain: ChainRef {
                name: "ethereum".to_string(),
                id: 1,
            },
            router: "0xCpmmRouter".to_string(),
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
        assert_eq!(receipt.gas_used, 120000);
        assert_eq!(receipt.tx_hash, "0xcpmm_placeholder");
    }
}
