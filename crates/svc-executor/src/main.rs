use sniper_core::bus::InMemoryBus;
use sniper_core::types::{Decision, ExecMode, ExecReceipt, TradePlan};
use sniper_exec::{exec_mempool, exec_private, gas};
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> eyre::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .json()
        .init();
    dotenvy::dotenv().ok();

    let bus = InMemoryBus::new(1024);

    // Trade plan subscriber task - listens for trade plans and executes them
    let rx_bus = bus.clone();
    tokio::spawn(async move {
        let mut rx = rx_bus.subscribe("plan.created");
        loop {
            if let Ok(bytes) = rx.recv().await {
                if let Ok(plan) = serde_json::from_slice::<TradePlan>(&bytes) {
                    tracing::info!(
                        "received trade plan for {} on {}",
                        plan.token_out,
                        plan.chain.name
                    );

                    // In a real implementation, this would:
                    // 1. Send the plan to the risk service for evaluation
                    // 2. If approved, execute the trade via the appropriate execution method
                    // 3. Publish the execution result

                    // Simulate risk check
                    let decision = Decision {
                        allow: true,
                        reasons: vec!["simulation - all checks passed".to_string()],
                    };

                    if decision.allow {
                        // Execute the trade using the appropriate method based on plan.mode
                        let receipt = execute_trade(&plan).await;

                        // Publish the execution result
                        let _ = rx_bus.publish("exec.result", &receipt).await;
                        tracing::info!("executed trade: {}", receipt.tx_hash);
                    } else {
                        tracing::warn!("trade rejected by risk checks: {:?}", decision.reasons);
                    }
                }
            }
        }
    });

    // Demo: publisher task - simulates trade plan generation
    let tx_bus = bus.clone();
    tokio::spawn(async move {
        // In a real system, these would come from the strategy service
        sleep(Duration::from_secs(1)).await; // Wait a bit for subscriber to start

        // Publish some demo trade plans with different execution modes
        let plans = vec![
            TradePlan {
                chain: sniper_core::types::ChainRef {
                    name: "ethereum".into(),
                    id: 1,
                },
                router: "0xRouterAddress".to_string(),
                token_in: "0xWETH".to_string(),
                token_out: "0xTokenA".to_string(),
                amount_in: 1000000000000000000, // 1 ETH
                min_out: 900000000000000000,    // 0.9 tokens
                mode: ExecMode::Mempool,
                gas: sniper_core::types::GasPolicy {
                    max_fee_gwei: 50,
                    max_priority_gwei: 2,
                },
                exits: Default::default(),
                idem_key: "demo-mempool-1".to_string(),
            },
            TradePlan {
                chain: sniper_core::types::ChainRef {
                    name: "bsc".into(),
                    id: 56,
                },
                router: "0xRouterAddress".to_string(),
                token_in: "0xWBNB".to_string(),
                token_out: "0xTokenB".to_string(),
                amount_in: 1000000000000000000, // 1 BNB
                min_out: 900000000000000000,    // 0.9 tokens
                mode: ExecMode::Private,
                gas: sniper_core::types::GasPolicy {
                    max_fee_gwei: 50,
                    max_priority_gwei: 2,
                },
                exits: Default::default(),
                idem_key: "demo-private-1".to_string(),
            },
        ];

        for plan in plans {
            let _ = tx_bus.publish("plan.created", &plan).await;
            tracing::info!(?plan.mode, "published demo trade plan");
            sleep(Duration::from_secs(2)).await;
        }
    });

    // Keep running
    loop {
        sleep(Duration::from_secs(3600)).await;
    }
}

/// Execute a trade and return the receipt
async fn execute_trade(plan: &TradePlan) -> ExecReceipt {
    tracing::info!(
        "executing trade on {} chain using {:?} mode",
        plan.chain.name,
        plan.mode
    );

    // Use our new gas estimation
    let gas_estimator = gas::default_gas_estimator();
    let estimated_gas = gas_estimator.estimate_gas(plan);

    tracing::info!(
        "Gas estimated: max_fee={} gwei, priority_fee={} gwei",
        estimated_gas.max_fee_gwei,
        estimated_gas.max_priority_gwei
    );

    // Execute based on the mode specified in the plan
    let receipt = match plan.mode {
        ExecMode::Mempool => match exec_mempool::execute_via_mempool(plan).await {
            Ok(receipt) => receipt,
            Err(e) => {
                tracing::error!("Mempool execution failed: {}", e);
                ExecReceipt {
                    tx_hash: "0x0000000000000000000000000000000000000000000000000000000000000000"
                        .to_string(),
                    success: false,
                    block: 0,
                    gas_used: 0,
                    fees_paid_wei: 0,
                    failure_reason: Some(e.to_string()),
                }
            }
        },
        ExecMode::Private => match exec_private::execute_via_private(plan).await {
            Ok(receipt) => receipt,
            Err(e) => {
                tracing::error!("Private execution failed: {}", e);
                ExecReceipt {
                    tx_hash: "0x0000000000000000000000000000000000000000000000000000000000000000"
                        .to_string(),
                    success: false,
                    block: 0,
                    gas_used: 0,
                    fees_paid_wei: 0,
                    failure_reason: Some(e.to_string()),
                }
            }
        },
        ExecMode::Bundle => {
            // For now, fall back to mempool execution for bundle mode
            match exec_mempool::execute_via_mempool(plan).await {
                Ok(receipt) => receipt,
                Err(e) => {
                    tracing::error!("Bundle execution failed: {}", e);
                    ExecReceipt {
                        tx_hash:
                            "0x0000000000000000000000000000000000000000000000000000000000000000"
                                .to_string(),
                        success: false,
                        block: 0,
                        gas_used: 0,
                        fees_paid_wei: 0,
                        failure_reason: Some(e.to_string()),
                    }
                }
            }
        }
    };

    receipt
}

#[cfg(test)]
mod tests {
    use super::*;
    use sniper_core::types::{ChainRef, ExitRules, GasPolicy};

    #[tokio::test]
    async fn test_execute_trade_mempool() {
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
            idem_key: "test_mempool_1".to_string(),
        };

        let receipt = execute_trade(&plan).await;
        assert!(receipt.tx_hash.starts_with("0x"));
        // Note: In a real test, we would mock the execution and verify specific outcomes
    }
}
