//! Integration test for the enhanced features implemented in Phase 2.
//! 
//! This test demonstrates the CEX integration, policy engine, and telemetry systems
//! working together as specified in the PRODUCT_ROADMAP.MD.

use sniper_cex::{Client, ExchangeId, Symbol, OrderSide, OrderType};
use sniper_policy::{GeoPolicy, VenuePolicy, KycPolicy, CompositePolicy, PolicyVerdict, UserContext, GeoRegion, VenueId, KycStatus};
use sniper_telemetry::{TelemetrySystem, TelemetryConfig, metrics, tracing, alerts};

#[tokio::test]
async fn test_cex_integration() {
    // Test CEX client creation
    let client = Client::new(
        ExchangeId("binance".to_string()),
        "api_key".to_string(),
        "api_secret".to_string(),
        "https://api.binance.com".to_string(),
        "wss://stream.binance.com:9443".to_string(),
    );
    
    assert_eq!(client.exchange_id().0, "binance");
    assert_eq!(client.rest_endpoint(), "https://api.binance.com");
    assert_eq!(client.ws_endpoint(), "wss://stream.binance.com:9443");
    
    // Test symbol creation
    let symbol = Symbol("BTC/USDT".to_string());
    assert_eq!(symbol.0, "BTC/USDT");
    
    // Test order side
    let buy = OrderSide::Buy;
    let sell = OrderSide::Sell;
    
    match buy {
        OrderSide::Buy => assert!(true),
        OrderSide::Sell => assert!(false),
    }
    
    match sell {
        OrderSide::Buy => assert!(false),
        OrderSide::Sell => assert!(true),
    }
}

#[test]
fn test_policy_engine() {
    // Test geographic policy
    let geo_policy = GeoPolicy::new(
        vec![GeoRegion("US".to_string()), GeoRegion("CA".to_string())],
        vec![GeoRegion("CN".to_string()), GeoRegion("RU".to_string())],
    );
    
    let us_context = UserContext {
        user_id: "user1".to_string(),
        ip_address: Some("1.2.3.4".to_string()),
        geo_region: Some(GeoRegion("US".to_string())),
        kyc_status: KycStatus::Verified,
        venue_id: VenueId("binance".to_string()),
    };
    
    let cn_context = UserContext {
        user_id: "user2".to_string(),
        ip_address: Some("5.6.7.8".to_string()),
        geo_region: Some(GeoRegion("CN".to_string())),
        kyc_status: KycStatus::Verified,
        venue_id: VenueId("binance".to_string()),
    };
    
    let geo_verdict_us = geo_policy.evaluate(&us_context);
    assert!(geo_verdict_us.allowed);
    assert!(geo_verdict_us.reasons.is_empty());
    
    let geo_verdict_cn = geo_policy.evaluate(&cn_context);
    assert!(!geo_verdict_cn.allowed);
    assert_eq!(geo_verdict_cn.reasons, vec!["Region CN is blocked"]);
    
    // Test venue policy
    let venue_policy = VenuePolicy::new(
        vec![VenueId("binance".to_string()), VenueId("coinbase".to_string())],
        vec![VenueId("kucoin".to_string())],
    );
    
    let binance_context = UserContext {
        user_id: "user3".to_string(),
        ip_address: Some("9.10.11.12".to_string()),
        geo_region: Some(GeoRegion("US".to_string())),
        kyc_status: KycStatus::Verified,
        venue_id: VenueId("binance".to_string()),
    };
    
    let kucoin_context = UserContext {
        user_id: "user4".to_string(),
        ip_address: Some("13.14.15.16".to_string()),
        geo_region: Some(GeoRegion("US".to_string())),
        kyc_status: KycStatus::Verified,
        venue_id: VenueId("kucoin".to_string()),
    };
    
    let venue_verdict_binance = venue_policy.evaluate(&binance_context);
    assert!(venue_verdict_binance.allowed);
    assert!(venue_verdict_binance.reasons.is_empty());
    
    let venue_verdict_kucoin = venue_policy.evaluate(&kucoin_context);
    assert!(!venue_verdict_kucoin.allowed);
    assert_eq!(venue_verdict_kucoin.reasons, vec!["Venue kucoin is blocked"]);
    
    // Test KYC policy
    let kyc_policy = KycPolicy::new(vec![VenueId("binance".to_string())]);
    
    let verified_context = UserContext {
        user_id: "user5".to_string(),
        ip_address: Some("17.18.19.20".to_string()),
        geo_region: Some(GeoRegion("US".to_string())),
        kyc_status: KycStatus::Verified,
        venue_id: VenueId("binance".to_string()),
    };
    
    let pending_context = UserContext {
        user_id: "user6".to_string(),
        ip_address: Some("21.22.23.24".to_string()),
        geo_region: Some(GeoRegion("US".to_string())),
        kyc_status: KycStatus::Pending,
        venue_id: VenueId("binance".to_string()),
    };
    
    let kyc_verdict_verified = kyc_policy.evaluate(&verified_context);
    assert!(kyc_verdict_verified.allowed);
    assert_eq!(kyc_verdict_verified.reasons, vec!["KYC verified"]);
    
    let kyc_verdict_pending = kyc_policy.evaluate(&pending_context);
    assert!(!kyc_verdict_pending.allowed);
    assert_eq!(kyc_verdict_pending.reasons, vec!["KYC verification pending"]);
    
    // Test composite policy
    let mut composite = CompositePolicy::new();
    composite.add_engine(Box::new(geo_policy));
    composite.add_engine(Box::new(venue_policy));
    composite.add_engine(Box::new(kyc_policy));
    
    let composite_verdict = composite.evaluate(&us_context);
    assert!(composite_verdict.allowed);
}

#[test]
fn test_telemetry_system() {
    // Test telemetry system creation
    let config = TelemetryConfig {
        metrics_enabled: true,
        tracing_enabled: true,
        alerting_enabled: true,
    };
    
    let telemetry = TelemetrySystem::new(config).unwrap();
    assert!(telemetry.metrics().is_some());
    assert!(telemetry.tracer().is_some());
    assert!(telemetry.alert_manager().is_some());
    
    // Test metrics recording
    if let Some(metrics) = telemetry.metrics() {
        metrics.record_trade_execution(true, 100, 21000);
        metrics.record_trade_execution(false, 150, 0);
        metrics.record_signal_processing(50);
        metrics.record_risk_check(true, 25);
        
        let snapshot = metrics.snapshot();
        assert_eq!(snapshot.successful_trades, 1);
        assert_eq!(snapshot.failed_trades, 1);
        assert_eq!(snapshot.avg_trade_latency_ms, 125);
        assert_eq!(snapshot.total_gas_used, 21000);
        assert_eq!(snapshot.signals_processed, 1);
        assert_eq!(snapshot.avg_signal_latency_ms, 50);
        assert_eq!(snapshot.risk_checks_allowed, 1);
        assert_eq!(snapshot.avg_risk_check_latency_ms, 25);
    }
    
    // Test tracing
    if let Some(tracer) = telemetry.tracer() {
        let mut span = tracer.start_span("test_operation");
        tracer.add_attribute(&mut span, "test_key", "test_value");
        let ended_span = tracer.end_span(span);
        
        assert_eq!(ended_span.name, "test_operation");
        assert!(!ended_span.id.is_empty());
        assert!(ended_span.end_time.is_some());
        assert_eq!(ended_span.attributes.get("test_key"), Some(&"test_value".to_string()));
    }
}

#[tokio::test]
async fn test_integration() {
    // Create all components
    let client = Client::new(
        ExchangeId("binance".to_string()),
        "api_key".to_string(),
        "api_secret".to_string(),
        "https://api.binance.com".to_string(),
        "wss://stream.binance.com:9443".to_string(),
    );
    
    let mut composite = CompositePolicy::new();
    composite.add_engine(Box::new(GeoPolicy::new(vec![], vec![])));
    composite.add_engine(Box::new(VenuePolicy::new(vec![], vec![])));
    composite.add_engine(Box::new(KycPolicy::new(vec![])));
    
    let config = TelemetryConfig {
        metrics_enabled: true,
        tracing_enabled: true,
        alerting_enabled: true,
    };
    
    let telemetry = TelemetrySystem::new(config).unwrap();
    
    // Test that all components work together
    let context = UserContext {
        user_id: "user1".to_string(),
        ip_address: Some("1.2.3.4".to_string()),
        geo_region: Some(GeoRegion("US".to_string())),
        kyc_status: KycStatus::Verified,
        venue_id: VenueId("binance".to_string()),
    };
    
    // Policy evaluation
    let verdict = composite.evaluate(&context);
    assert!(verdict.allowed);
    
    // Telemetry recording
    if let Some(metrics) = telemetry.metrics() {
        metrics.record_trade_execution(true, 100, 21000);
        let snapshot = metrics.snapshot();
        assert_eq!(snapshot.successful_trades, 1);
    }
    
    // Alert sending
    if let Some(alert_manager) = telemetry.alert_manager() {
        let result = alert_manager.send_alert("Test alert", alerts::AlertSeverity::Info).await;
        assert!(result.is_ok());
    }
    
    println!("All integration tests passed!");
}