//! Benchmarking integration tests

use sniper_bench::{BenchConfig, Benchmarker};
use std::time::Duration;

#[tokio::test]
async fn test_benchmarking_ml_signal_processing() {
    let config = BenchConfig {
        iterations: 5,
        detailed_timing: false,
        concurrent_tasks: 5,
        load_test_duration_secs: 10,
        target_throughput_rps: 100,
        enable_resource_monitoring: false,
    };

    let benchmarker = Benchmarker::new(config);
    let results = benchmarker.bench_ml_signal_processing().await;

    assert_eq!(results.component, "ML Signal Processing");
    assert_eq!(results.iterations, 5);
    assert!(results.total_duration > Duration::from_nanos(0));
    assert!(results.throughput > 0.0);
}

#[tokio::test]
async fn test_benchmarking_risk_evaluation() {
    let config = BenchConfig {
        iterations: 5,
        detailed_timing: false,
        concurrent_tasks: 5,
        load_test_duration_secs: 10,
        target_throughput_rps: 100,
        enable_resource_monitoring: false,
    };

    let benchmarker = Benchmarker::new(config);
    let results = benchmarker.bench_risk_evaluation().await;

    assert_eq!(results.component, "Risk Evaluation");
    assert_eq!(results.iterations, 5);
    assert!(results.total_duration > Duration::from_nanos(0));
    assert!(results.throughput > 0.0);
}

#[tokio::test]
async fn test_benchmarking_telemetry_recording() {
    let config = BenchConfig {
        iterations: 5,
        detailed_timing: false,
        concurrent_tasks: 5,
        load_test_duration_secs: 10,
        target_throughput_rps: 100,
        enable_resource_monitoring: false,
    };

    let benchmarker = Benchmarker::new(config);
    let results = benchmarker.bench_telemetry_recording().await;

    assert_eq!(results.component, "Telemetry Recording");
    assert_eq!(results.iterations, 5);
    assert!(results.total_duration > Duration::from_nanos(0));
    assert!(results.throughput > 0.0);
}

#[tokio::test]
async fn test_benchmarking_trade_execution_latency() {
    let config = BenchConfig {
        iterations: 5,
        detailed_timing: false,
        concurrent_tasks: 5,
        load_test_duration_secs: 10,
        target_throughput_rps: 100,
        enable_resource_monitoring: false,
    };

    let benchmarker = Benchmarker::new(config);
    let results = benchmarker.bench_trade_execution_latency().await;

    assert_eq!(results.component, "Trade Execution Latency");
    assert_eq!(results.iterations, 5);
    assert!(results.total_duration > Duration::from_nanos(0));
    assert!(results.throughput > 0.0);
    assert!(results.avg_duration > Duration::from_nanos(0));
    assert!(results.min_duration > Duration::from_nanos(0));
    assert!(results.max_duration > Duration::from_nanos(0));
}
