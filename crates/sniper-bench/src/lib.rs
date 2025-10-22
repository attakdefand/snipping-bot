//! Benchmarking tools for the sniper bot.
//! 
//! This module provides functionality for performance benchmarking of various
//! components of the trading system.

use sniper_core::types::{Signal, TradePlan, ChainRef};
use sniper_ml::{MlConfig, MlModel};
use sniper_risk;
use sniper_telemetry::{TelemetrySystem, TelemetryConfig};
use std::time::{Instant, Duration};
use tokio::time::sleep;

/// Configuration for benchmarking
#[derive(Debug, Clone)]
pub struct BenchConfig {
    /// Number of iterations to run each benchmark
    pub iterations: usize,
    /// Enable/disable detailed timing
    pub detailed_timing: bool,
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
}

/// Main benchmarking system
pub struct Benchmarker {
    config: BenchConfig,
}

impl Benchmarker {
    /// Create a new benchmarking system
    pub fn new(config: BenchConfig) -> Self {
        Self {
            config,
        }
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
        };
        let telemetry = TelemetrySystem::new(telemetry_config).expect("Failed to initialize telemetry");
        
        let mut durations = Vec::new();
        let start_time = Instant::now();
        
        // Run benchmark iterations
        for i in 0..self.config.iterations {
            let iter_start = Instant::now();
            
            // Record different types of telemetry data
            match i % 3 {
                0 => {
                    telemetry.record_trade_execution(true, 50, 21000);
                },
                1 => {
                    telemetry.record_signal_processing(10);
                },
                2 => {
                    telemetry.record_risk_check(true, 5);
                },
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
        }
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
        };
        
        let benchmarker = Benchmarker::new(config);
        assert_eq!(benchmarker.config.iterations, 10);
    }
    
    #[tokio::test]
    async fn test_ml_benchmark() {
        let config = BenchConfig {
            iterations: 5,
            detailed_timing: false,
        };
        
        let benchmarker = Benchmarker::new(config);
        let results = benchmarker.bench_ml_signal_processing().await;
        
        assert_eq!(results.component, "ML Signal Processing");
        assert_eq!(results.iterations, 5);
        assert!(results.total_duration > Duration::from_nanos(0));
    }
}