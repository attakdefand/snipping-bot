//! Enhanced stress testing integration tests

use sniper_backtest::ChaosScenario;
use sniper_bench::stress_test::{ResourceMonitoringConfig, StressTestConfig, StressTester};
use std::time::Duration;

#[tokio::test]
async fn test_enhanced_stress_tester_creation() {
    let resource_config = ResourceMonitoringConfig {
        monitor_cpu: true,
        monitor_memory: true,
        monitor_disk_io: false,
        monitor_network_io: false,
        sampling_interval_ms: 500,
    };

    let config = StressTestConfig {
        concurrent_runs: 3,
        run_duration_secs: 1,
        signals_per_run: 10,
        enable_chaos: false,
        chaos_scenarios: vec![],
        high_load_conditions: vec![],
        resource_monitoring: resource_config,
    };

    let tester = StressTester::new(config.clone());

    assert_eq!(tester.config.concurrent_runs, config.concurrent_runs);
    assert_eq!(tester.config.run_duration_secs, config.run_duration_secs);
    assert!(tester.config.resource_monitoring.monitor_cpu);
    assert!(tester.config.resource_monitoring.monitor_memory);
    assert_eq!(tester.config.resource_monitoring.sampling_interval_ms, 500);
}

#[tokio::test]
async fn test_enhanced_stress_test_execution() {
    let config = StressTestConfig {
        concurrent_runs: 2,   // Reduced for testing
        run_duration_secs: 1, // Short duration for testing
        signals_per_run: 10,
        enable_chaos: false, // Disable chaos for basic test
        chaos_scenarios: vec![],
        high_load_conditions: vec![],
        resource_monitoring: ResourceMonitoringConfig::default(),
    };

    let tester = StressTester::new(config);
    let results = tester.run_stress_test().await;

    assert_eq!(results.config.concurrent_runs, 2);
    assert_eq!(results.successful_runs, 2); // All should succeed
    assert_eq!(results.failed_runs, 0); // None should fail
    assert!(results.duration > Duration::from_nanos(0));
    assert!(results.avg_execution_time > Duration::from_nanos(0));
    assert!(results.resource_metrics.avg_cpu_pct > 0.0);
}

#[tokio::test]
async fn test_enhanced_chaos_scenarios() {
    let chaos_scenario = ChaosScenario::NetworkLatency {
        base_latency_ms: 50,
        additional_latency_ms: 100,
        affected_percentage: 0.5,
    };

    let config = StressTestConfig {
        concurrent_runs: 1,   // Reduced for testing
        run_duration_secs: 1, // Short duration for testing
        signals_per_run: 10,
        enable_chaos: true,
        chaos_scenarios: vec![chaos_scenario],
        high_load_conditions: vec![],
        resource_monitoring: ResourceMonitoringConfig::default(),
    };

    let tester = StressTester::new(config);
    let results = tester.run_stress_test().await;

    assert!(results.chaos_results.is_some());
    if let Some(chaos_results) = results.chaos_results {
        assert_eq!(chaos_results.len(), 1);
    }
}

#[tokio::test]
async fn test_resource_monitoring_config() {
    let resource_config = ResourceMonitoringConfig::default();

    assert!(resource_config.monitor_cpu);
    assert!(resource_config.monitor_memory);
    assert!(resource_config.monitor_disk_io);
    assert!(resource_config.monitor_network_io);
    assert_eq!(resource_config.sampling_interval_ms, 1000);
}
