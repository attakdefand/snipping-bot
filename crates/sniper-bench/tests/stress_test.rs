//! Stress testing integration tests

use sniper_backtest::ChaosScenario;
use sniper_bench::stress_test::{StressTestConfig, StressTester};

#[tokio::test]
async fn test_stress_tester_creation() {
    let mut config = StressTestConfig::default();
    config.concurrent_runs = 3;
    config.run_duration_secs = 1;
    config.signals_per_run = 10;
    config.enable_chaos = false;

    let tester = StressTester::new(config.clone());

    assert_eq!(tester.config.concurrent_runs, config.concurrent_runs);
    assert_eq!(tester.config.run_duration_secs, config.run_duration_secs);
}

#[tokio::test]
async fn test_stress_test_execution() {
    let mut config = StressTestConfig::default();
    config.concurrent_runs = 3; // Reduced for testing
    config.run_duration_secs = 1; // Short duration for testing
    config.signals_per_run = 10;
    config.enable_chaos = false; // Disable chaos for basic test
    config.chaos_scenarios = vec![];
    config.high_load_conditions = vec![];

    let tester = StressTester::new(config);
    let results = tester.run_stress_test().await;

    assert_eq!(results.config.concurrent_runs, 3);
    assert_eq!(results.successful_runs, 3); // All should succeed
    assert_eq!(results.failed_runs, 0); // None should fail
    assert!(results.duration > std::time::Duration::from_nanos(0));
}

#[tokio::test]
async fn test_chaos_scenarios() {
    let chaos_scenario = ChaosScenario::NetworkLatency {
        base_latency_ms: 50,
        additional_latency_ms: 100,
        affected_percentage: 0.5,
    };

    let mut config = StressTestConfig::default();
    config.concurrent_runs = 1; // Reduced for testing
    config.run_duration_secs = 1; // Short duration for testing
    config.signals_per_run = 10;
    config.enable_chaos = true;
    config.chaos_scenarios = vec![chaos_scenario];
    config.high_load_conditions = vec![];

    let tester = StressTester::new(config);
    let results = tester.run_stress_test().await;

    assert!(results.chaos_results.is_some());
    if let Some(chaos_results) = results.chaos_results {
        assert_eq!(chaos_results.len(), 1);
    }
}
