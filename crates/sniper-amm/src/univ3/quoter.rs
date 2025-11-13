//! Uniswap V3 quoter implementation
use anyhow::Result;
use sniper_core::types::TradePlan;

/// Quote for a Uniswap V3 trade
#[derive(Debug, Clone)]
pub struct Quote {
    /// The amount of token out expected
    pub amount_out: u128,
    /// The price impact of the trade
    pub price_impact: f64,
    /// The fee amount in token in
    pub fee_amount: u128,
    /// The gas estimate for the trade
    pub gas_estimate: u64,
}

/// Get a quote for a Uniswap V3 trade
///
/// # Arguments
/// * `plan` - The trade plan containing token addresses, amounts, and other parameters
///
/// # Returns
/// * `Result<Quote>` - The quote for the trade or an error
pub fn get_quote(plan: &TradePlan) -> Result<Quote> {
    // In a real implementation, this would:
    // 1. Connect to the Uniswap V3 pool
    // 2. Fetch current liquidity and tick data
    // 3. Calculate the exact output amount based on the input amount
    // 4. Calculate price impact and fees
    // 5. Estimate gas usage

    // For now, we'll return a simulated quote based on the plan
    let amount_out = plan.min_out;
    let price_impact = 0.001; // 0.1% simulated price impact
    let fee_amount = plan.amount_in * 3 / 1000; // 0.3% fee
    let gas_estimate = 150000; // Simulated gas estimate

    Ok(Quote {
        amount_out,
        price_impact,
        fee_amount,
        gas_estimate,
    })
}

/// Calculate the price impact of a trade
///
/// # Arguments
/// * `amount_in` - The amount of token in
/// * `amount_out` - The amount of token out
/// * `spot_price` - The spot price of the token pair
///
/// # Returns
/// * `f64` - The price impact as a percentage
pub fn calculate_price_impact(amount_in: u128, amount_out: u128, spot_price: f64) -> f64 {
    if spot_price == 0.0 {
        return 0.0;
    }

    let executed_price = amount_out as f64 / amount_in as f64;
    (spot_price - executed_price).abs() / spot_price * 100.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use sniper_core::types::{ChainRef, ExecMode, ExitRules, GasPolicy};

    #[test]
    fn test_get_quote() {
        let plan = TradePlan {
            chain: ChainRef {
                name: "ethereum".to_string(),
                id: 1,
            },
            router: "0xRouter".to_string(),
            token_in: "0xTokenIn".to_string(),
            token_out: "0xTokenOut".to_string(),
            amount_in: 1000000000000000000, // 1 ETH
            min_out: 900000000000000000,    // 0.9 ETH worth of tokens
            mode: ExecMode::Mempool,
            gas: GasPolicy {
                max_fee_gwei: 50,
                max_priority_gwei: 2,
            },
            exits: ExitRules {
                take_profit_pct: Some(10.0),
                stop_loss_pct: Some(5.0),
                trailing_pct: Some(2.0),
            },
            idem_key: "test-key".to_string(),
        };

        let quote = get_quote(&plan).unwrap();
        assert_eq!(quote.amount_out, 900000000000000000);
        assert_eq!(quote.price_impact, 0.001);
        assert_eq!(quote.fee_amount, 3000000000000000); // 0.3% of 1 ETH
        assert_eq!(quote.gas_estimate, 150000);
    }

    #[test]
    fn test_calculate_price_impact() {
        let price_impact = calculate_price_impact(
            1000000000000000000, // 1 ETH
            900000000000000000,  // 0.9 tokens
            0.9,                 // Spot price
        );

        // With executed price = 0.9 and spot price = 0.9, impact should be 0
        assert_eq!(price_impact, 0.0);
    }
}
