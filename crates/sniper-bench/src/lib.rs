//! Benchmarking tools for the sniper bot.
//!
//! This module provides functionality for performance benchmarking of various
//! components of the trading system.

pub mod stress_test;

use futures::future::join_all;
use sniper_core::types::{ChainRef, Signal, TradePlan};
use sniper_ml::{MlConfig, MlModel};
use sniper_risk;
use sniper_telemetry::{alerts, TelemetryConfig, TelemetrySystem};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Semaphore;
use tokio::time::sleep;

/// Configuration for benchmarking
#[derive(Debug, Clone)]
pub struct BenchConfig {
    /// Number of iterations to run each benchmark
    pub iterations: usize,
    /// Enable/disable detailed timing
    pub detailed_timing: bool,
    /// Number of concurrent tasks for load testing
    pub concurrent_tasks: usize,
    /// Duration for load testing in seconds
    pub load_test_duration_secs: u64,
    /// Target throughput for load testing (requests per second)
    pub target_throughput_rps: usize,
    /// Enable resource monitoring during tests
    pub enable_resource_monitoring: bool,
}

impl Default for BenchConfig {
    fn default() -> Self {
        Self {
            iterations: 1000,
            detailed_timing: false,
            concurrent_tasks: 10,
            load_test_duration_secs: 30,
            target_throughput_rps: 100,
            enable_resource_monitoring: true,
        }
    }
}

/// Benchmark results
#[derive(Debug, Clone)]
pub struct BenchResults {
    pub component: String,
    pub iterations: usize,
    pub total_duration: Duration,
    pub avg_duration: Duration,
    pub min_duration: Duration,
    pub max_duration: Duration,
    pub throughput: f64, // operations per second
    pub concurrency_level: usize,
}

/// Load test results
#[derive(Debug, Clone)]
pub struct LoadTestResults {
    pub component: String,
    pub duration: Duration,
    pub total_requests: usize,
    pub successful_requests: usize,
    pub failed_requests: usize,
    pub avg_response_time: Duration,
    pub min_response_time: Duration,
    pub max_response_time: Duration,
    pub requests_per_second: f64,
    pub concurrency_level: usize,
}

/// Main benchmarking system
pub struct Benchmarker {
    config: BenchConfig,
}

impl Benchmarker {
    /// Create a new benchmarking system
    pub fn new(config: BenchConfig) -> Self {
        Self { config }
    }

    /// Benchmark ML signal processing
    pub async fn bench_ml_signal_processing(&self) -> BenchResults {
        tracing::info!("Starting ML signal processing benchmark");

        // Create ML model
        let ml_config = MlConfig {
            model_path: "models/test_model.onnx".to_string(),
            confidence_threshold: 0.8,
            enabled: true,
        };
        let ml_model = MlModel::new(ml_config);

        // Create test signals
        let signals = self.create_test_signals();

        let mut durations = Vec::new();
        let start_time = Instant::now();

        // Run benchmark iterations
        for i in 0..self.config.iterations {
            let signal = &signals[i % signals.len()];
            let iter_start = Instant::now();

            // Process signal with ML model
            let _plan = ml_model.process_signal(signal).await;

            let iter_duration = iter_start.elapsed();
            durations.push(iter_duration);

            // Small delay to prevent overwhelming the system
            if self.config.detailed_timing {
                sleep(Duration::from_micros(100)).await;
            }
        }

        let total_duration = start_time.elapsed();
        let avg_duration = self.calculate_average(&durations);
        let min_duration = self.calculate_min(&durations);
        let max_duration = self.calculate_max(&durations);
        let throughput = self.config.iterations as f64 / total_duration.as_secs_f64();

        BenchResults {
            component: "ML Signal Processing".to_string(),
            iterations: self.config.iterations,
            total_duration,
            avg_duration,
            min_duration,
            max_duration,
            throughput,
            concurrency_level: 1,
        }
    }

    /// Benchmark risk evaluation
    pub async fn bench_risk_evaluation(&self) -> BenchResults {
        tracing::info!("Starting risk evaluation benchmark");

        // Create test trade plans
        let plans = self.create_test_plans();

        let mut durations = Vec::new();
        let start_time = Instant::now();

        // Run benchmark iterations
        for i in 0..self.config.iterations {
            let plan = &plans[i % plans.len()];
            let iter_start = Instant::now();

            // Evaluate risk
            let _decision = sniper_risk::evaluate_trade(plan);

            let iter_duration = iter_start.elapsed();
            durations.push(iter_duration);

            // Small delay to prevent overwhelming the system
            if self.config.detailed_timing {
                sleep(Duration::from_micros(100)).await;
            }
        }

        let total_duration = start_time.elapsed();
        let avg_duration = self.calculate_average(&durations);
        let min_duration = self.calculate_min(&durations);
        let max_duration = self.calculate_max(&durations);
        let throughput = self.config.iterations as f64 / total_duration.as_secs_f64();

        BenchResults {
            component: "Risk Evaluation".to_string(),
            iterations: self.config.iterations,
            total_duration,
            avg_duration,
            min_duration,
            max_duration,
            throughput,
            concurrency_level: 1,
        }
    }

    /// Benchmark telemetry recording
    pub async fn bench_telemetry_recording(&self) -> BenchResults {
        tracing::info!("Starting telemetry recording benchmark");

        // Initialize telemetry system
        let telemetry_config = TelemetryConfig {
            metrics_enabled: true,
            tracing_enabled: true,
            alerting_enabled: true,
            alert_manager_config: Some(sniper_telemetry::alerts::AlertManagerConfig::default()),
        };
        let telemetry =
            TelemetrySystem::new(telemetry_config).expect("Failed to initialize telemetry");

        let mut durations = Vec::new();
        let start_time = Instant::now();

        // Run benchmark iterations
        for i in 0..self.config.iterations {
            let iter_start = Instant::now();

            // Record different types of telemetry data
            match i % 3 {
                0 => {
                    telemetry.record_trade_execution(true, 50, 21000);
                }
                1 => {
                    telemetry.record_signal_processing(10);
                }
                2 => {
                    telemetry.record_risk_check(true, 5);
                }
                _ => unreachable!(),
            }

            let iter_duration = iter_start.elapsed();
            durations.push(iter_duration);

            // Small delay to prevent overwhelming the system
            if self.config.detailed_timing {
                sleep(Duration::from_micros(100)).await;
            }
        }

        let total_duration = start_time.elapsed();
        let avg_duration = self.calculate_average(&durations);
        let min_duration = self.calculate_min(&durations);
        let max_duration = self.calculate_max(&durations);
        let throughput = self.config.iterations as f64 / total_duration.as_secs_f64();

        BenchResults {
            component: "Telemetry Recording".to_string(),
            iterations: self.config.iterations,
            total_duration,
            avg_duration,
            min_duration,
            max_duration,
            throughput,
            concurrency_level: 1,
        }
    }

    /// Load test ML signal processing with enhanced capabilities
    pub async fn load_test_ml_signal_processing(&self) -> LoadTestResults {
        tracing::info!(
            "Starting ML signal processing load test with {} concurrent tasks for {} seconds",
            self.config.concurrent_tasks,
            self.config.load_test_duration_secs
        );

        // Create ML model
        let ml_config = MlConfig {
            model_path: "models/test_model.onnx".to_string(),
            confidence_threshold: 0.8,
            enabled: true,
        };
        let ml_model = Arc::new(MlModel::new(ml_config));

        // Create test signals
        let signals = Arc::new(self.create_test_signals());

        // Create semaphore to limit concurrency
        let semaphore = Arc::new(Semaphore::new(self.config.concurrent_tasks));

        let start_time = Instant::now();
        let end_time = start_time + Duration::from_secs(self.config.load_test_duration_secs);

        let mut handles = vec![];
        let mut total_requests = 0;
        let mut successful_requests = 0;
        let mut failed_requests = 0;
        let mut response_times = Vec::new();

        // Calculate delay between requests to achieve target throughput
        let request_delay = if self.config.target_throughput_rps > 0 {
            Duration::from_micros(1_000_000 / self.config.target_throughput_rps as u64)
        } else {
            Duration::from_millis(10)
        };

        while Instant::now() < end_time {
            let semaphore = semaphore.clone();
            let ml_model = ml_model.clone();
            let signals = signals.clone();

            let handle = tokio::spawn(async move {
                let _permit = semaphore.acquire().await.unwrap();
                let signal = &signals[0]; // Use first signal for simplicity
                let start = Instant::now();

                // Process signal with ML model
                let result = ml_model.process_signal(signal).await;

                let duration = start.elapsed();
                match result {
                    Some(_) => (true, duration),
                    None => (false, duration),
                }
            });

            handles.push(handle);
            total_requests += 1;

            // Delay to control request rate
            sleep(request_delay).await;
        }

        // Wait for all tasks to complete
        let results = join_all(handles).await;

        for result in results {
            if let Ok((success, duration)) = result {
                response_times.push(duration);
                if success {
                    successful_requests += 1;
                } else {
                    failed_requests += 1;
                }
            }
        }

        let total_duration = start_time.elapsed();
        let avg_response_time = self.calculate_average(&response_times);
        let min_response_time = self.calculate_min(&response_times);
        let max_response_time = self.calculate_max(&response_times);
        let requests_per_second = total_requests as f64 / total_duration.as_secs_f64();

        LoadTestResults {
            component: "ML Signal Processing Load Test".to_string(),
            duration: total_duration,
            total_requests,
            successful_requests,
            failed_requests,
            avg_response_time,
            min_response_time,
            max_response_time,
            requests_per_second,
            concurrency_level: self.config.concurrent_tasks,
        }
    }

    /// Load test risk evaluation with enhanced capabilities
    pub async fn load_test_risk_evaluation(&self) -> LoadTestResults {
        tracing::info!(
            "Starting risk evaluation load test with {} concurrent tasks for {} seconds",
            self.config.concurrent_tasks,
            self.config.load_test_duration_secs
        );

        // Create test trade plans
        let plans = Arc::new(self.create_test_plans());

        // Create semaphore to limit concurrency
        let semaphore = Arc::new(Semaphore::new(self.config.concurrent_tasks));

        let start_time = Instant::now();
        let end_time = start_time + Duration::from_secs(self.config.load_test_duration_secs);

        let mut handles = vec![];
        let mut total_requests = 0;
        let mut successful_requests = 0;
        let mut failed_requests = 0;
        let mut response_times = Vec::new();

        // Calculate delay between requests to achieve target throughput
        let request_delay = if self.config.target_throughput_rps > 0 {
            Duration::from_micros(1_000_000 / self.config.target_throughput_rps as u64)
        } else {
            Duration::from_millis(10)
        };

        while Instant::now() < end_time {
            let semaphore = semaphore.clone();
            let plans = plans.clone();

            let handle = tokio::spawn(async move {
                let _permit = semaphore.acquire().await.unwrap();
                let plan = &plans[0]; // Use first plan for simplicity
                let start = Instant::now();

                // Risk evaluation is synchronous, so we just call it
                let result = std::panic::catch_unwind(|| sniper_risk::evaluate_trade(plan));

                let duration = start.elapsed();
                match result {
                    Ok(_) => (true, duration),
                    Err(_) => (false, duration),
                }
            });

            handles.push(handle);
            total_requests += 1;

            // Delay to control request rate
            sleep(request_delay).await;
        }

        // Wait for all tasks to complete
        let results = join_all(handles).await;

        for result in results {
            if let Ok((success, duration)) = result {
                response_times.push(duration);
                if success {
                    successful_requests += 1;
                } else {
                    failed_requests += 1;
                }
            }
        }

        let total_duration = start_time.elapsed();
        let avg_response_time = self.calculate_average(&response_times);
        let min_response_time = self.calculate_min(&response_times);
        let max_response_time = self.calculate_max(&response_times);
        let requests_per_second = total_requests as f64 / total_duration.as_secs_f64();

        LoadTestResults {
            component: "Risk Evaluation Load Test".to_string(),
            duration: total_duration,
            total_requests,
            successful_requests,
            failed_requests,
            avg_response_time,
            min_response_time,
            max_response_time,
            requests_per_second,
            concurrency_level: self.config.concurrent_tasks,
        }
    }

    /// Load test telemetry recording with enhanced capabilities
    pub async fn load_test_telemetry_recording(&self) -> LoadTestResults {
        tracing::info!(
            "Starting telemetry recording load test with {} concurrent tasks for {} seconds",
            self.config.concurrent_tasks,
            self.config.load_test_duration_secs
        );

        // Initialize telemetry system
        let telemetry_config = TelemetryConfig {
            metrics_enabled: true,
            tracing_enabled: true,
            alerting_enabled: true,
            alert_manager_config: Some(sniper_telemetry::alerts::AlertManagerConfig::default()),
        };
        let telemetry = Arc::new(
            TelemetrySystem::new(telemetry_config).expect("Failed to initialize telemetry"),
        );

        // Create semaphore to limit concurrency
        let semaphore = Arc::new(Semaphore::new(self.config.concurrent_tasks));

        let start_time = Instant::now();
        let end_time = start_time + Duration::from_secs(self.config.load_test_duration_secs);

        let mut handles = vec![];
        let mut total_requests = 0;

        // Calculate delay between requests to achieve target throughput
        let request_delay = if self.config.target_throughput_rps > 0 {
            Duration::from_micros(1_000_000 / self.config.target_throughput_rps as u64)
        } else {
            Duration::from_millis(10)
        };

        while Instant::now() < end_time {
            let semaphore = semaphore.clone();
            let telemetry = telemetry.clone();

            let handle = tokio::spawn(async move {
                let _permit = semaphore.acquire().await.unwrap();
                let start = Instant::now();

                // Record different types of telemetry data
                telemetry.record_trade_execution(true, 50, 21000);
                telemetry.record_signal_processing(10);
                telemetry.record_risk_check(true, 5);

                start.elapsed()
            });

            handles.push(handle);
            total_requests += 1;

            // Delay to control request rate
            sleep(request_delay).await;
        }

        // Wait for all tasks to complete
        let results = join_all(handles).await;

        let mut response_times = Vec::new();
        for result in results {
            if let Ok(duration) = result {
                response_times.push(duration);
            }
        }

        let total_duration = start_time.elapsed();
        let avg_response_time = self.calculate_average(&response_times);
        let min_response_time = self.calculate_min(&response_times);
        let max_response_time = self.calculate_max(&response_times);
        let requests_per_second = total_requests as f64 / total_duration.as_secs_f64();
        let successful_requests = total_requests; // All should succeed
        let failed_requests = 0;

        LoadTestResults {
            component: "Telemetry Recording Load Test".to_string(),
            duration: total_duration,
            total_requests,
            successful_requests,
            failed_requests,
            avg_response_time,
            min_response_time,
            max_response_time,
            requests_per_second,
            concurrency_level: self.config.concurrent_tasks,
        }
    }

    /// Run comprehensive load testing across all components
    pub async fn run_comprehensive_load_test(&self) -> Vec<LoadTestResults> {
        tracing::info!("Starting comprehensive load test across all components");

        let mut results = Vec::new();

        // Run ML signal processing load test
        let ml_results = self.load_test_ml_signal_processing().await;
        results.push(ml_results);

        // Run risk evaluation load test
        let risk_results = self.load_test_risk_evaluation().await;
        results.push(risk_results);

        // Run telemetry recording load test
        let telemetry_results = self.load_test_telemetry_recording().await;
        results.push(telemetry_results);

        results
    }

    /// Benchmark trade execution latency
    pub async fn bench_trade_execution_latency(&self) -> BenchResults {
        tracing::info!("Starting trade execution latency benchmark");

        // Create test trade plans
        let plans = self.create_test_plans();

        let mut durations = Vec::new();
        let start_time = Instant::now();

        // Run benchmark iterations
        for i in 0..self.config.iterations {
            let plan = &plans[i % plans.len()];
            let iter_start = Instant::now();

            // Simulate trade execution
            // In a real implementation, this would actually execute trades
            let _result = self.simulate_trade_execution(plan).await;

            let iter_duration = iter_start.elapsed();
            durations.push(iter_duration);

            // Small delay to prevent overwhelming the system
            if self.config.detailed_timing {
                sleep(Duration::from_micros(100)).await;
            }
        }

        let total_duration = start_time.elapsed();
        let avg_duration = self.calculate_average(&durations);
        let min_duration = self.calculate_min(&durations);
        let max_duration = self.calculate_max(&durations);
        let throughput = self.config.iterations as f64 / total_duration.as_secs_f64();

        BenchResults {
            component: "Trade Execution Latency".to_string(),
            iterations: self.config.iterations,
            total_duration,
            avg_duration,
            min_duration,
            max_duration,
            throughput,
            concurrency_level: 1,
        }
    }

    /// Simulate trade execution for benchmarking
    async fn simulate_trade_execution(&self, _plan: &TradePlan) -> bool {
        // Simulate some work for trade execution
        sleep(Duration::from_micros(500)).await;
        true // Simulate successful execution
    }

    /// Create test signals for benchmarking
    fn create_test_signals(&self) -> Vec<Signal> {
        vec![
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
        ]
    }

    /// Create test trade plans for benchmarking
    fn create_test_plans(&self) -> Vec<TradePlan> {
        vec![
            TradePlan {
                chain: ChainRef {
                    name: "ethereum".into(),
                    id: 1,
                },
                router: "0xRouterAddress".to_string(),
                token_in: "0xWETH".to_string(),
                token_out: "0xTokenA".to_string(),
                amount_in: 1000000000000000000, // 1 ETH
                min_out: 900000000000000000,    // 0.9 tokens (10% slippage)
                mode: sniper_core::types::ExecMode::Mempool,
                gas: sniper_core::types::GasPolicy {
                    max_fee_gwei: 50,
                    max_priority_gwei: 2,
                },
                exits: sniper_core::types::ExitRules {
                    take_profit_pct: Some(20.0),
                    stop_loss_pct: Some(10.0),
                    trailing_pct: Some(5.0),
                },
                idem_key: "test-plan-1".to_string(),
            },
            TradePlan {
                chain: ChainRef {
                    name: "bsc".into(),
                    id: 56,
                },
                router: "0xRouterAddress".to_string(),
                token_in: "0xWBNB".to_string(),
                token_out: "0xTokenB".to_string(),
                amount_in: 500000000000000000, // 0.5 BNB
                min_out: 450000000000000000,   // 0.45 tokens (10% slippage)
                mode: sniper_core::types::ExecMode::Mempool,
                gas: sniper_core::types::GasPolicy {
                    max_fee_gwei: 10,
                    max_priority_gwei: 1,
                },
                exits: sniper_core::types::ExitRules {
                    take_profit_pct: Some(15.0),
                    stop_loss_pct: Some(7.5),
                    trailing_pct: Some(3.0),
                },
                idem_key: "test-plan-2".to_string(),
            },
        ]
    }

    /// Calculate average duration
    fn calculate_average(&self, durations: &[Duration]) -> Duration {
        if durations.is_empty() {
            return Duration::from_nanos(0);
        }

        let total_nanos: u128 = durations.iter().map(|d| d.as_nanos()).sum();
        Duration::from_nanos((total_nanos / durations.len() as u128) as u64)
    }

    /// Calculate minimum duration
    fn calculate_min(&self, durations: &[Duration]) -> Duration {
        *durations.iter().min().unwrap_or(&Duration::from_nanos(0))
    }

    /// Calculate maximum duration
    fn calculate_max(&self, durations: &[Duration]) -> Duration {
        *durations.iter().max().unwrap_or(&Duration::from_nanos(0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_benchmarker_creation() {
        let config = BenchConfig {
            iterations: 10,
            detailed_timing: false,
            concurrent_tasks: 5,
            load_test_duration_secs: 10,
            target_throughput_rps: 100,
            enable_resource_monitoring: true,
        };

        let benchmarker = Benchmarker::new(config);
        assert_eq!(benchmarker.config.iterations, 10);
    }

    #[tokio::test]
    async fn test_ml_benchmark() {
        let config = BenchConfig {
            iterations: 5,
            detailed_timing: false,
            concurrent_tasks: 5,
            load_test_duration_secs: 10,
            target_throughput_rps: 100,
            enable_resource_monitoring: true,
        };

        let benchmarker = Benchmarker::new(config);
        let results = benchmarker.bench_ml_signal_processing().await;

        assert_eq!(results.component, "ML Signal Processing");
        assert_eq!(results.iterations, 5);
        assert!(results.total_duration > Duration::from_nanos(0));
    }

    #[tokio::test]
    async fn test_load_test_ml() {
        let config = BenchConfig {
            iterations: 100,
            detailed_timing: false,
            concurrent_tasks: 3,
            load_test_duration_secs: 2, // Short duration for testing
            target_throughput_rps: 50,
            enable_resource_monitoring: false,
        };

        let benchmarker = Benchmarker::new(config);
        let results = benchmarker.load_test_ml_signal_processing().await;

        assert_eq!(results.component, "ML Signal Processing Load Test");
        assert!(results.duration > Duration::from_nanos(0));
        assert!(results.total_requests > 0);
    }
}
