use sniper_core::{bus::InMemoryBus, prelude::*};
use sniper_core::types::{Signal, TradePlan, ChainRef, ExecMode, GasPolicy, ExitRules};
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> eyre::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .json()
        .init();
    dotenvy::dotenv().ok();

    let bus = InMemoryBus::new(1024);

    // Signal subscriber task - listens for signals and generates trade plans
    let rx_bus = bus.clone();
    tokio::spawn(async move {
        let mut rx = rx_bus.subscribe("signals.>");
        loop {
            if let Ok(bytes) = rx.recv().await {
                if let Ok(sig) = serde_json::from_slice::<Signal>(&bytes) {
                    tracing::info!(?sig.kind, "received signal");
                    
                    // Process the signal and generate a trade plan
                    if let Some(plan) = process_signal(&sig).await {
                        // Publish the trade plan
                        let _ = rx_bus.publish("plan.created", &plan).await;
                        tracing::info!("published trade plan");
                    }
                }
            }
        }
    });

    // Demo: publisher task - simulates signal generation
    let tx_bus = bus.clone();
    tokio::spawn(async move {
        let signals = vec![
            Signal {
                source: "dex".into(),
                kind: "pair_created".into(),
                chain: ChainRef {
                    name: "ethereum".into(),
                    id: 1,
                },
                token0: Some("0xTokenA".into()),
                token1: Some("0xWETH".into()),
                extra: serde_json::json!({"pair": "0xPairAddress"}),
                seen_at_ms: 0,
            },
            Signal {
                source: "dex".into(),
                kind: "trading_enabled".into(),
                chain: ChainRef {
                    name: "bsc".into(),
                    id: 56,
                },
                token0: Some("0xTokenB".into()),
                token1: Some("0xWBNB".into()),
                extra: serde_json::json!({"token": "0xTokenAddress"}),
                seen_at_ms: 0,
            },
        ];
        
        for signal in signals {
            let _ = tx_bus.publish("signals.dex.event", &signal).await;
            tracing::info!(?signal.kind, "published demo signal");
            sleep(Duration::from_secs(2)).await;
        }
    });

    // Keep running
    loop {
        sleep(Duration::from_secs(3600)).await;
    }
}

/// Process a signal and generate a trade plan if applicable
async fn process_signal(signal: &Signal) -> Option<TradePlan> {
    match signal.kind.as_str() {
        "pair_created" => {
            tracing::info!("processing pair created signal");
            // In a real implementation, this would analyze the new pair
            // and determine if it's worth trading
            Some(TradePlan {
                chain: signal.chain.clone(),
                router: "0xRouterAddress".to_string(),
                token_in: signal.token1.clone().unwrap_or("0xWETH".to_string()),
                token_out: signal.token0.clone().unwrap_or("0xToken".to_string()),
                amount_in: 1000000000000000000, // 1 ETH/BNB
                min_out: 900000000000000000,    // 0.9 tokens (10% slippage)
                mode: ExecMode::Mempool,
                gas: GasPolicy {
                    max_fee_gwei: 50,
                    max_priority_gwei: 2,
                },
                exits: ExitRules {
                    take_profit_pct: Some(20.0),
                    stop_loss_pct: Some(10.0),
                    trailing_pct: Some(5.0),
                },
                idem_key: format!("plan_{}", signal.seen_at_ms),
            })
        },
        "trading_enabled" => {
            tracing::info!("processing trading enabled signal");
            // In a real implementation, this would analyze the token
            // and determine if it's worth trading
            Some(TradePlan {
                chain: signal.chain.clone(),
                router: "0xRouterAddress".to_string(),
                token_in: signal.token1.clone().unwrap_or("0xWETH".to_string()),
                token_out: signal.token0.clone().unwrap_or("0xToken".to_string()),
                amount_in: 500000000000000000, // 0.5 ETH/BNB
                min_out: 450000000000000000,   // 0.45 tokens (10% slippage)
                mode: ExecMode::Mempool,
                gas: GasPolicy {
                    max_fee_gwei: 40,
                    max_priority_gwei: 1,
                },
                exits: ExitRules {
                    take_profit_pct: Some(15.0),
                    stop_loss_pct: Some(7.5),
                    trailing_pct: Some(3.0),
                },
                idem_key: format!("plan_{}", signal.seen_at_ms),
            })
        },
        _ => {
            tracing::debug!("ignoring signal kind: {}", signal.kind);
            None
        }
    }
}