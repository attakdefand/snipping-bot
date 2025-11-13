//! Load testing integration tests

use sniper_bench::{BenchConfig, Benchmarker};
use std::time::Duration;

#[tokio::test]
async fn test_load_testing_ml_signal_processing() {
    let config = BenchConfig {
        iterations: 100,
        detailed_timing: false,
        concurrent_tasks: 5,
        load_test_duration_secs: 2, // Short duration for testing
        target_throughput_rps: 50,
        enable_resource_monitoring: false,
    };

    let benchmarker = Benchmarker::new(config);
    let results = benchmarker.load_test_ml_signal_processing().await;

    assert_eq!(results.component, "ML Signal Processing Load Test");
    assert!(results.duration > Duration::from_nanos(0));
    assert!(results.total_requests > 0);
    assert!(results.requests_per_second > 0.0);
    assert_eq!(results.concurrency_level, 5);
}

#[tokio::test]
async fn test_load_testing_risk_evaluation() {
    let config = BenchConfig {
        iterations: 100,
        detailed_timing: false,
        concurrent_tasks: 5,
        load_test_duration_secs: 2, // Short duration for testing
        target_throughput_rps: 50,
        enable_resource_monitoring: false,
    };

    let benchmarker = Benchmarker::new(config);
    let results = benchmarker.load_test_risk_evaluation().await;

    assert_eq!(results.component, "Risk Evaluation Load Test");
    assert!(results.duration > Duration::from_nanos(0));
    assert!(results.total_requests > 0);
    assert!(results.requests_per_second > 0.0);
    assert_eq!(results.concurrency_level, 5);
}

#[tokio::test]
async fn test_load_testing_telemetry_recording() {
    let config = BenchConfig {
        iterations: 100,
        detailed_timing: false,
        concurrent_tasks: 5,
        load_test_duration_secs: 2, // Short duration for testing
        target_throughput_rps: 50,
        enable_resource_monitoring: false,
    };

    let benchmarker = Benchmarker::new(config);
    let results = benchmarker.load_test_telemetry_recording().await;

    assert_eq!(results.component, "Telemetry Recording Load Test");
    assert!(results.duration > Duration::from_nanos(0));
    assert!(results.total_requests > 0);
    assert!(results.requests_per_second > 0.0);
    assert_eq!(results.concurrency_level, 5);
    assert_eq!(results.failed_requests, 0); // All telemetry recording should succeed
}

#[tokio::test]
async fn test_comprehensive_load_test() {
    let config = BenchConfig {
        iterations: 50,
        detailed_timing: false,
        concurrent_tasks: 3,
        load_test_duration_secs: 1, // Short duration for testing
        target_throughput_rps: 30,
        enable_resource_monitoring: false,
    };

    let benchmarker = Benchmarker::new(config);
    let results = benchmarker.run_comprehensive_load_test().await;

    // Should have results for ML, Risk, and Telemetry
    assert_eq!(results.len(), 3);

    // Check ML results
    assert_eq!(results[0].component, "ML Signal Processing Load Test");
    assert!(results[0].duration > Duration::from_nanos(0));
    assert!(results[0].total_requests > 0);

    // Check Risk results
    assert_eq!(results[1].component, "Risk Evaluation Load Test");
    assert!(results[1].duration > Duration::from_nanos(0));
    assert!(results[1].total_requests > 0);

    // Check Telemetry results
    assert_eq!(results[2].component, "Telemetry Recording Load Test");
    assert!(results[2].duration > Duration::from_nanos(0));
    assert!(results[2].total_requests > 0);
    assert_eq!(results[2].failed_requests, 0);
}
