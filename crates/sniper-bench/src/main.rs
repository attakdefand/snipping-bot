use sniper_bench::{BenchConfig, BenchResults, Benchmarker, LoadTestResults};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt().with_env_filter("info").init();

    println!("Sniper Bot Performance Benchmarking Tool");
    println!("======================================");

    // Configuration for benchmarks
    let bench_config = BenchConfig {
        iterations: 1000,
        detailed_timing: false,
        concurrent_tasks: 10,
        load_test_duration_secs: 30,
        target_throughput_rps: 100,
        enable_resource_monitoring: true,
    };

    let benchmarker = Benchmarker::new(bench_config);

    // Run benchmarks
    println!("\nRunning ML Signal Processing Benchmark...");
    let ml_results = benchmarker.bench_ml_signal_processing().await;
    print_bench_results(&ml_results);

    println!("\nRunning Risk Evaluation Benchmark...");
    let risk_results = benchmarker.bench_risk_evaluation().await;
    print_bench_results(&risk_results);

    println!("\nRunning Telemetry Recording Benchmark...");
    let telemetry_results = benchmarker.bench_telemetry_recording().await;
    print_bench_results(&telemetry_results);

    println!("\nRunning Trade Execution Latency Benchmark...");
    let execution_results = benchmarker.bench_trade_execution_latency().await;
    print_bench_results(&execution_results);

    // Run load tests
    println!("\nRunning ML Signal Processing Load Test...");
    let ml_load_results = benchmarker.load_test_ml_signal_processing().await;
    print_load_test_results(&ml_load_results);

    println!("\nRunning Risk Evaluation Load Test...");
    let risk_load_results = benchmarker.load_test_risk_evaluation().await;
    print_load_test_results(&risk_load_results);

    println!("\nRunning Telemetry Recording Load Test...");
    let telemetry_load_results = benchmarker.load_test_telemetry_recording().await;
    print_load_test_results(&telemetry_load_results);

    // Run comprehensive load test
    println!("\nRunning Comprehensive Load Test...");
    let comprehensive_results = benchmarker.run_comprehensive_load_test().await;
    for result in &comprehensive_results {
        print_load_test_results(result);
    }

    // Summary
    println!("\nBenchmark Summary:");
    println!("==================");
    println!("ML Signal Processing: {:.2} ops/sec", ml_results.throughput);
    println!("Risk Evaluation: {:.2} ops/sec", risk_results.throughput);
    println!(
        "Telemetry Recording: {:.2} ops/sec",
        telemetry_results.throughput
    );
    println!(
        "Trade Execution Latency: {:.2} ops/sec",
        execution_results.throughput
    );

    println!("\nLoad Test Summary:");
    println!("==================");
    println!(
        "ML Signal Processing: {:.2} req/sec ({} concurrent)",
        ml_load_results.requests_per_second, ml_load_results.concurrency_level
    );
    println!(
        "Risk Evaluation: {:.2} req/sec ({} concurrent)",
        risk_load_results.requests_per_second, risk_load_results.concurrency_level
    );
    println!(
        "Telemetry Recording: {:.2} req/sec ({} concurrent)",
        telemetry_load_results.requests_per_second, telemetry_load_results.concurrency_level
    );

    Ok(())
}

fn print_bench_results(results: &BenchResults) {
    println!("  Component: {}", results.component);
    println!("  Iterations: {}", results.iterations);
    println!("  Total Duration: {:.2?}", results.total_duration);
    println!("  Average Duration: {:.2?}", results.avg_duration);
    println!("  Min Duration: {:.2?}", results.min_duration);
    println!("  Max Duration: {:.2?}", results.max_duration);
    println!("  Throughput: {:.2} ops/sec", results.throughput);
    println!("  Concurrency Level: {}", results.concurrency_level);
}

fn print_load_test_results(results: &LoadTestResults) {
    println!("  Component: {}", results.component);
    println!("  Duration: {:.2?}", results.duration);
    println!("  Total Requests: {}", results.total_requests);
    println!("  Successful Requests: {}", results.successful_requests);
    println!("  Failed Requests: {}", results.failed_requests);
    println!("  Average Response Time: {:.2?}", results.avg_response_time);
    println!("  Min Response Time: {:.2?}", results.min_response_time);
    println!("  Max Response Time: {:.2?}", results.max_response_time);
    println!("  Requests Per Second: {:.2}", results.requests_per_second);
    println!("  Concurrency Level: {}", results.concurrency_level);
}
