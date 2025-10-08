use sniper_core::{bus::InMemoryBus, prelude::*};
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> eyre::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .json()
        .init();
    dotenvy::dotenv().ok();

    let bus = InMemoryBus::new(1024);

    // Demo: publisher task
    let tx_bus = bus.clone();
    tokio::spawn(async move {
        loop {
            let sig = Signal {
                source: "dex".into(),
                kind: "pair_created".into(),
                chain: ChainRef {
                    name: "ethereum".into(),
                    id: 1,
                },
                token0: None,
                token1: None,
                extra: serde_json::json!({"demo":true}),
                seen_at_ms: 0,
            };
            let _ = tx_bus.publish("signals.dex.pair_created", &sig).await;
            sleep(Duration::from_secs(5)).await;
        }
    });

    // Demo: subscriber task
    let mut rx = bus.subscribe("signals.>");
    tokio::spawn(async move {
        loop {
            if let Ok(bytes) = rx.recv().await {
                if let Ok(sig) = serde_json::from_slice::<Signal>(&bytes) {
                    tracing::info!(?sig.kind, "received signal");
                }
            }
        }
    });

    // Keep running; a real service would expose HTTP/gRPC, healthz, etc.
    loop {
        sleep(Duration::from_secs(3600)).await;
    }
}
