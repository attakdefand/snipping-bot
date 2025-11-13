//! Happy Path tests
//!
//! This file contains tests for the happy path testing category

#[cfg(test)]
mod tests {
    use sniper_core::types::{ChainRef, Signal, ExecMode, GasPolicy, ExitRules, TradePlan};
    use sniper_core::errors::SniperError;
    
    #[test]
    fn test_chain_ref_creation() {
        let chain = ChainRef {
            name: "ethereum".to_string(),
            id: 1,
        };
        
        assert_eq!(chain.name, "ethereum");
        assert_eq!(chain.id, 1);
    }

    #[test]
    fn test_signal_creation() {
        let chain = ChainRef {
            name: "bsc".to_string(),
            id: 56,
        };
        
        let signal = Signal {
            source: "dex".to_string(),
            kind: "pair_created".to_string(),
            chain,
            token0: Some("WETH".to_string()),
            token1: Some("USDT".to_string()),
            extra: serde_json::json!({"pair_address": "0x123"}),
            seen_at_ms: 1234567890,
        };
        
        assert_eq!(signal.source, "dex");
        assert_eq!(signal.kind, "pair_created");
        assert_eq!(signal.chain.id, 56);
    }

    #[test]
    fn test_trade_plan_creation() {
        let chain = ChainRef {
            name: "polygon".to_string(),
            id: 137,
        };
        
        let gas_policy = GasPolicy {
            max_fee_gwei: 50,
            max_priority_gwei: 2,
        };
        
        let exit_rules = ExitRules {
            take_profit_pct: Some(10.0),
            stop_loss_pct: Some(5.0),
            trailing_pct: Some(2.0),
        };
        
        let trade_plan = TradePlan {
            chain,
            router: "quickswap".to_string(),
            token_in: "USDC".to_string(),
            token_out: "WETH".to_string(),
            amount_in: 1000000000, // 1000 USDC (6 decimals)
            min_out: 500000000000000000, // 0.5 WETH (18 decimals)
            mode: ExecMode::Mempool,
            gas: gas_policy,
            exits: exit_rules,
            idem_key: "test-trade-001".to_string(),
        };
        
        assert_eq!(trade_plan.chain.id, 137);
        assert_eq!(trade_plan.mode, ExecMode::Mempool);
        assert_eq!(trade_plan.amount_in, 1000000000);
    }
    
    #[test]
    fn test_sniper_error_creation() {
        let error = SniperError::Config("Invalid configuration".to_string());
        
        match error {
            SniperError::Config(msg) => assert_eq!(msg, "Invalid configuration"),
            _ => panic!("Expected Config error"),
        }
    }
}