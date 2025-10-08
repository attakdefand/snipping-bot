use sniper_core::{Signal, ChainRef};
use sniper_core::bus::InMemoryBus;
use tokio::time::Duration;

#[tokio::test]
async fn test_signal_flow() {
    let bus = InMemoryBus::new(1024);
    
    // Create a test signal
    let signal = Signal {
        source: "test".into(),
        kind: "test_event".into(),
        chain: ChainRef { name: "ethereum".into(), id: 1 },
        token0: Some("0xToken0".into()),
        token1: Some("0xToken1".into()),
        extra: serde_json::json!({"test": true}),
        seen_at_ms: 1234567890,
    };
    
    // Publish the signal
    let tx_bus = bus.clone();
    let _ = tx_bus.publish("signals.test.event", &signal).await;
    
    // Subscribe and receive the signal
    let mut rx = bus.subscribe("signals.test.event");
    
    // Check that we can receive the signal
    let bytes = tokio::time::timeout(Duration::from_secs(1), rx.recv()).await.unwrap().unwrap();
    let received_signal = serde_json::from_slice::<Signal>(&bytes).unwrap();
    
    assert_eq!(received_signal.source, "test");
    assert_eq!(received_signal.kind, "test_event");
    assert_eq!(received_signal.chain.name, "ethereum");
    assert_eq!(received_signal.chain.id, 1);
}