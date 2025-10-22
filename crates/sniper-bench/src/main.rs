use sniper_bench::{Benchmarker, BenchConfig, BenchResults};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();
    
    println!("Sniper Bot Performance Benchmarking Tool");
    println!("======================================");
    
    // Configuration
    let config = BenchConfig {
        iterations: 1000,
        detailed_timing: false,
    };
    
    let benchmarker = Benchmarker::new(config);
    
    // Run benchmarks
    println!("\nRunning ML Signal Processing Benchmark...");
    let ml_results = benchmarker.bench_ml_signal_processing().await;
    print_results(&ml_results);
    
    println!("\nRunning Risk Evaluation Benchmark...");
    let risk_results = benchmarker.bench_risk_evaluation().await;
    print_results(&risk_results);
    
    println!("\nRunning Telemetry Recording Benchmark...");
    let telemetry_results = benchmarker.bench_telemetry_recording().await;
    print_results(&telemetry_results);
    
    // Summary
    println!("\nBenchmark Summary:");
    println!("==================");
    println!("ML Signal Processing: {:.2} ops/sec", ml_results.throughput);
    println!("Risk Evaluation: {:.2} ops/sec", risk_results.throughput);
    println!("Telemetry Recording: {:.2} ops/sec", telemetry_results.throughput);
    
    Ok(())
}

fn print_results(results: &BenchResults) {
    println!("  Component: {}", results.component);
    println!("  Iterations: {}", results.iterations);
    println!("  Total Duration: {:.2?}", results.total_duration);
    println!("  Average Duration: {:.2?}", results.avg_duration);
    println!("  Min Duration: {:.2?}", results.min_duration);
    println!("  Max Duration: {:.2?}", results.max_duration);
    println!("  Throughput: {:.2} ops/sec", results.throughput);
}