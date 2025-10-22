use sniper_core::bus::InMemoryBus;
use sniper_core::types::{Decision, ExecReceipt, TradePlan};
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
                        // Execute the trade
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
    let _tx_bus = bus.clone();
    tokio::spawn(async move {
        // In a real system, these would come from the strategy service
        sleep(Duration::from_secs(1)).await; // Wait a bit for subscriber to start
    });

    // Keep running
    loop {
        sleep(Duration::from_secs(3600)).await;
    }
}

/// Execute a trade and return the receipt
async fn execute_trade(plan: &TradePlan) -> ExecReceipt {
    tracing::info!("executing trade on {} chain", plan.chain.name);

    // In a real implementation, this would:
    // 1. Connect to the appropriate blockchain
    // 2. Build the transaction
    // 3. Sign the transaction (using the keys service)
    // 4. Submit the transaction via the selected execution mode
    // 5. Wait for confirmation and return the receipt

    // Simulate execution
    ExecReceipt {
        tx_hash: format!("0x{}", hex::encode(plan.idem_key.as_bytes())),
        success: true,
        block: 12345678,
        gas_used: 150000,
        fees_paid_wei: 3150000000000000, // 0.00315 ETH
        failure_reason: None,
    }
}
