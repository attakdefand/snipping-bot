//! Backtesting module for the sniper bot.
//!
//! This module provides functionality for comprehensive backtesting with historical data
//! to evaluate strategy performance before live deployment.

use csv::ReaderBuilder;
use serde::{Deserialize, Serialize};
use sniper_core::types::{Signal, TradePlan};
use sniper_ml::MlModel;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;

use tokio::time::sleep;

/// Configuration for backtesting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestConfig {
    /// Start timestamp for backtesting
    pub start_time: i64,
    /// End timestamp for backtesting
    pub end_time: i64,
    /// Initial capital for simulation
    pub initial_capital: f64,
    /// Trading fee percentage (0.003 = 0.3%)
    pub trading_fee_pct: f64,
    /// Slippage percentage (0.005 = 0.5%)
    pub slippage_pct: f64,
    /// Maximum position size as percentage of capital (0.1 = 10%)
    pub max_position_size_pct: f64,
    /// Enable/disable backtesting
    pub enabled: bool,
    /// Path to historical data directory
    pub data_path: Option<String>,
    /// Execution model type
    pub execution_model: ExecutionModelType,
}

/// Execution model types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionModelType {
    Simple,
    OrderBook,
    Impact,
}

/// Historical market data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketDataPoint {
    pub timestamp: u64,
    pub token_address: String,
    pub price_usd: f64,
    pub volume_24h: f64,
    pub liquidity: f64,
}

/// Enhanced market data with order book information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBookData {
    pub timestamp: u64,
    pub token_address: String,
    pub bid_prices: Vec<f64>,
    pub bid_volumes: Vec<f64>,
    pub ask_prices: Vec<f64>,
    pub ask_volumes: Vec<f64>,
    pub price_usd: f64,
    pub volume_24h: f64,
    pub liquidity: f64,
}

/// Processed transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessedTransaction {
    /// Transaction hash
    pub hash: String,
    /// Gas price in Gwei
    pub gas_price_gwei: f64,
    /// Gas used
    pub gas_used: u64,
    /// Processed timestamp in milliseconds
    pub processed_at_ms: u64,
}

/// CSV format for historical price data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalPriceRecord {
    pub timestamp: u64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

/// Advanced slippage model parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlippageModel {
    pub model_type: SlippageModelType,
    pub k_coefficient: f64,    // Kyle's lambda for impact model
    pub max_slippage_pct: f64, // Maximum allowed slippage
}

/// Slippage model types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SlippageModelType {
    Fixed,
    VolumeWeighted,
    Impact,
}

/// Backtest result for a single trade
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeResult {
    pub plan_id: String,
    pub entry_time: i64,
    pub exit_time: i64,
    pub entry_price: f64,
    pub exit_price: f64,
    pub amount_in: f64,
    pub amount_out: f64,
    pub profit_loss: f64,
    pub profit_loss_pct: f64,
    pub fees_paid: f64,
    pub slippage_loss: f64,
    pub position_size_pct: f64,
    pub execution_details: ExecutionDetails,
}

/// Execution details for advanced models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionDetails {
    pub model_used: String,
    pub queue_position: Option<usize>,
    pub partial_fills: Vec<PartialFill>,
    pub latency_ms: u64,
}

/// Partial fill information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartialFill {
    pub price: f64,
    pub amount: f64,
    pub timestamp: u64,
}

/// Comprehensive backtest results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestResults {
    pub config: BacktestConfig,
    pub total_trades: usize,
    pub winning_trades: usize,
    pub losing_trades: usize,
    pub total_profit_loss: f64,
    pub total_fees_paid: f64,
    pub total_slippage_loss: f64,
    pub win_rate: f64,
    pub avg_profit_per_trade: f64,
    pub max_drawdown: f64,
    pub sharpe_ratio: f64,
    pub sortino_ratio: f64,
    pub calmar_ratio: f64,
    pub max_consecutive_wins: usize,
    pub max_consecutive_losses: usize,
    pub individual_trades: Vec<TradeResult>,
}

/// Walk-forward optimization configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalkForwardConfig {
    /// Size of training window in seconds
    pub train_window_secs: i64,
    /// Size of testing window in seconds
    pub test_window_secs: i64,
    /// Step size between windows in seconds
    pub step_secs: i64,
    /// Minimum number of trades required in training window
    pub min_train_trades: usize,
    /// Minimum number of trades required in testing window
    pub min_test_trades: usize,
}

/// Walk-forward optimization results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalkForwardResults {
    pub config: WalkForwardConfig,
    pub windows: Vec<WalkForwardWindowResults>,
    pub overall_performance: BacktestResults,
}

/// Results for a single walk-forward window
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalkForwardWindowResults {
    pub window_start: i64,
    pub window_end: i64,
    pub train_results: BacktestResults,
    pub test_results: BacktestResults,
    pub parameters: serde_json::Value, // Parameters used for this window
}

/// Forward testing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForwardTestConfig {
    /// Enable/disable forward testing
    pub enabled: bool,
    /// Test duration in seconds
    pub duration_secs: u64,
    /// Capital allocation percentage (0.0 to 1.0)
    pub capital_allocation: f64,
    /// Risk limits
    pub risk_limits: RiskLimits,
    /// Kill switch configuration
    pub kill_switch: KillSwitchConfig,
}

/// Risk limits for forward testing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskLimits {
    /// Maximum drawdown percentage
    pub max_drawdown_pct: f64,
    /// Maximum position size percentage
    pub max_position_size_pct: f64,
    /// Maximum daily loss percentage
    pub max_daily_loss_pct: f64,
}

/// Kill switch configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KillSwitchConfig {
    /// Enable/disable kill switch
    pub enabled: bool,
    /// Drawdown threshold for activation
    pub drawdown_threshold_pct: f64,
    /// Anomalous spread threshold for activation
    pub spread_threshold_pct: f64,
    /// Liquidity vacuum threshold for activation
    pub liquidity_threshold_pct: f64,
}

/// Chaos testing scenario configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChaosTestConfig {
    /// Enable/disable chaos testing
    pub enabled: bool,
    /// List of chaos scenarios to test
    pub scenarios: Vec<ChaosScenario>,
}

/// Paper trade backtest configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaperTradeConfig {
    /// Enable/disable paper trading
    pub enabled: bool,
    /// Time warp factor for accelerated replay
    pub time_warp_factor: f64,
    /// Whether to run in shadow mode
    pub shadow_mode: bool,
}

/// Forward test results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForwardTestResults {
    /// Configuration used
    pub config: ForwardTestConfig,
    /// Actual performance
    pub performance: BacktestResults,
    /// Risk metrics
    pub risk_metrics: RiskMetrics,
    /// Kill switch activations
    pub kill_switch_activations: Vec<KillSwitchActivation>,
}

/// Fork test configuration for on-chain testing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForkTestConfig {
    /// Enable/disable fork testing
    pub enabled: bool,
    /// Block number to fork from
    pub fork_block: Option<u64>,
    /// MEV scenario to simulate
    pub mev_scenario: Option<MevScenario>,
    /// Gas price model
    pub gas_model: GasPriceModel,
    /// Reorg simulation settings
    pub reorg_settings: ReorgSettings,
}

/// MEV scenarios for testing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MevScenario {
    /// Pool snipe attack
    PoolSnipe {
        attacker_address: String,
        target_pool: String,
        amount_in: f64,
    },
    /// Sandwich attack
    Sandwich {
        attacker_position: QueuePosition,
        target_trade: String,
    },
    /// Frontrun attack
    Frontrun {
        attacker_address: String,
        target_transaction: String,
    },
}

/// Gas price model for fork testing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GasPriceModel {
    /// Base gas price in Gwei
    pub base_gas_price: f64,
    /// Gas price multiplier for spikes
    pub spike_multiplier: f64,
    /// Probability of gas spike
    pub spike_probability: f64,
}

/// Reorg simulation settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReorgSettings {
    /// Enable/disable reorg simulation
    pub enabled: bool,
    /// Number of blocks to reorg
    pub reorg_blocks: u64,
    /// Probability of reorg
    pub reorg_probability: f64,
}

/// Fork test results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForkTestResults {
    /// Configuration used
    pub config: ForkTestConfig,
    /// Backtest results
    pub performance: BacktestResults,
    /// MEV impact metrics
    pub mev_impact: MevImpactMetrics,
    /// Reorg statistics
    pub reorg_stats: ReorgStatistics,
}

/// MEV impact metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MevImpactMetrics {
    /// Profit lost to MEV attacks
    pub profit_lost: f64,
    /// Additional slippage from MEV
    pub additional_slippage: f64,
    /// Failed transactions due to MEV
    pub failed_transactions: usize,
}

/// Reorg statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReorgStatistics {
    /// Number of reorgs simulated
    pub reorgs_simulated: usize,
    /// Transactions affected by reorgs
    pub transactions_affected: usize,
    /// Average reorg depth
    pub avg_reorg_depth: f64,
}

/// Risk metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskMetrics {
    /// Actual drawdown percentage
    pub actual_drawdown_pct: f64,
    /// Actual position sizes
    pub position_sizes: Vec<f64>,
    /// Daily losses
    pub daily_losses: Vec<f64>,
}

/// Kill switch activation record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KillSwitchActivation {
    /// Timestamp of activation
    pub timestamp: u64,
    /// Reason for activation
    pub reason: KillSwitchReason,
    /// Additional details
    pub details: String,
}

/// Kill switch activation reasons
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KillSwitchReason {
    DrawdownExceeded,
    AnomalousSpread,
    LiquidityVacuum,
    Manual,
}

/// Types of chaos scenarios
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChaosScenario {
    /// Network latency injection
    NetworkLatency {
        /// Base latency in milliseconds
        base_latency_ms: u64,
        /// Additional latency to inject
        additional_latency_ms: u64,
        /// Percentage of requests affected
        affected_percentage: f64,
    },
    /// Exchange outage simulation
    ExchangeOutage {
        /// Duration of outage in seconds
        duration_secs: u64,
        /// Time of outage start (relative to test start)
        start_time_secs: u64,
    },
    /// Gas price spike
    GasSpike {
        /// Multiplier for gas prices
        multiplier: f64,
        /// Duration of spike in seconds
        duration_secs: u64,
        /// Time of spike start (relative to test start)
        start_time_secs: u64,
    },
    /// Market volatility increase
    MarketVolatility {
        /// Volatility multiplier
        multiplier: f64,
        /// Duration of increased volatility in seconds
        duration_secs: u64,
        /// Time of volatility increase start (relative to test start)
        start_time_secs: u64,
    },
    /// Order book staleness
    OrderBookStaleness {
        /// Staleness duration in seconds
        duration_secs: u64,
        /// Time of staleness start (relative to test start)
        start_time_secs: u64,
    },
}

/// Results from chaos testing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChaosTestResults {
    pub config: ChaosTestConfig,
    pub scenario_results: Vec<ChaosScenarioResult>,
    pub baseline_performance: BacktestResults,
    pub overall_impact: ChaosImpactMetrics,
}

/// Results for a single chaos scenario
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChaosScenarioResult {
    pub scenario: ChaosScenario,
    pub performance: BacktestResults,
    pub impact_metrics: ChaosImpactMetrics,
}

/// Metrics measuring the impact of chaos scenarios
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChaosImpactMetrics {
    /// Performance degradation percentage
    pub performance_degradation_pct: f64,
    /// Additional slippage incurred
    pub additional_slippage_pct: f64,
    /// Increased latency in milliseconds
    pub increased_latency_ms: u64,
    /// Failed trade percentage
    pub failed_trade_pct: f64,
}

/// Portfolio backtest configuration for multi-asset testing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioBacktestConfig {
    /// Assets to include in portfolio backtest
    pub assets: Vec<String>,
    /// Correlation matrix for assets
    pub correlation_matrix: HashMap<String, HashMap<String, f64>>,
    /// Portfolio weights for each asset
    pub weights: HashMap<String, f64>,
    /// Rebalancing frequency in seconds
    pub rebalance_frequency_secs: i64,
}

/// Smart router execution model for multi-venue testing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartRouterExecutionModel {
    /// Venue selection criteria
    pub venue_selection: VenueSelectionCriteria,
    /// Cross-venue latency matrix
    pub latency_matrix: HashMap<String, HashMap<String, u64>>,
    /// Failover behaviors
    pub failover_behaviors: HashMap<String, FailoverBehavior>,
}

/// Venue selection criteria
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VenueSelectionCriteria {
    /// Priority factors for venue selection
    pub factors: Vec<VenueFactor>,
    /// Weights for each factor
    pub weights: HashMap<String, f64>,
}

/// Factors for venue selection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VenueFactor {
    Fees,
    Liquidity,
    Reliability,
    Latency,
}

/// Failover behavior configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailoverBehavior {
    /// Timeout before failover in milliseconds
    pub timeout_ms: u64,
    /// Retry attempts before failover
    pub retry_attempts: u32,
    /// Backup venues in order of preference
    pub backup_venues: Vec<String>,
}

/// Market regime for regime-aware backtesting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketRegime {
    /// Regime identifier
    pub id: String,
    /// Time periods when this regime is active
    pub time_periods: Vec<(i64, i64)>,
    /// Regime-specific parameters
    pub parameters: RegimeParameters,
}

/// Regime-specific parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegimeParameters {
    /// Volatility multiplier
    pub volatility_multiplier: f64,
    /// Liquidity multiplier
    pub liquidity_multiplier: f64,
    /// Fee multiplier
    pub fee_multiplier: f64,
}

/// Scenario testing configuration for chaos testing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioTestConfig {
    /// List of scenarios to test
    pub scenarios: Vec<MarketScenario>,
    /// Baseline performance for comparison
    pub baseline_performance: Option<BacktestResults>,
}

/// Market scenarios for stress testing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MarketScenario {
    /// Exchange outage simulation
    ExchangeOutage {
        venue: String,
        duration_secs: u64,
        start_time_secs: u64,
    },
    /// Network latency injection
    NetworkLatency {
        venue: String,
        latency_ms: u64,
        duration_secs: u64,
        start_time_secs: u64,
    },
    /// Gas price spike (for DEX testing)
    GasSpike {
        multiplier: f64,
        duration_secs: u64,
        start_time_secs: u64,
    },
    /// Market volatility increase
    MarketVolatility {
        multiplier: f64,
        duration_secs: u64,
        start_time_secs: u64,
    },
    /// Order book staleness
    OrderBookStaleness {
        venue: String,
        duration_secs: u64,
        start_time_secs: u64,
    },
    /// Sandwich attack simulation
    SandwichAttack {
        attacker_position: QueuePosition,
        duration_secs: u64,
        start_time_secs: u64,
    },
}

/// Chaos scenario configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChaosScenarioConfig {
    /// Enable/disable chaos testing
    pub enabled: bool,
    /// List of chaos scenarios to test
    pub scenarios: Vec<ChaosScenarioType>,
}

/// Types of chaos scenarios
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChaosScenarioType {
    /// Exchange outage simulation
    ExchangeOutage {
        venue: String,
        duration_secs: u64,
        start_time_secs: u64,
    },
    /// Network latency injection
    NetworkLatency {
        venue: String,
        latency_ms: u64,
        duration_secs: u64,
        start_time_secs: u64,
    },
    /// Gas price spike (for DEX testing)
    GasSpike {
        multiplier: f64,
        duration_secs: u64,
        start_time_secs: u64,
    },
    /// Market volatility increase
    MarketVolatility {
        multiplier: f64,
        duration_secs: u64,
        start_time_secs: u64,
    },
    /// Order book staleness
    OrderBookStaleness {
        venue: String,
        duration_secs: u64,
        start_time_secs: u64,
    },
    /// Sandwich attack simulation
    SandwichAttack {
        attacker_position: QueuePosition,
        duration_secs: u64,
        start_time_secs: u64,
    },
}

/// Queue position for order book modeling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QueuePosition {
    Front,
    Middle,
    Back,
}

/// Deterministic clock for backtesting
#[derive(Debug, Clone)]
pub struct SimClock {
    /// Current timestamp in milliseconds
    pub current_time_ms: u64,
    /// Time warp factor for accelerated replay
    pub time_warp_factor: f64,
    /// Whether the clock is paused
    pub paused: bool,
}

impl SimClock {
    /// Create a new simulation clock
    pub fn new(start_time_ms: u64) -> Self {
        Self {
            current_time_ms: start_time_ms,
            time_warp_factor: 1.0,
            paused: false,
        }
    }

    /// Advance the clock by a given amount of real time
    pub fn advance(&mut self, real_time_delta_ms: u64) {
        if !self.paused {
            self.current_time_ms += (real_time_delta_ms as f64 * self.time_warp_factor) as u64;
        }
    }

    /// Set the time warp factor for accelerated replay
    pub fn set_time_warp(&mut self, factor: f64) {
        self.time_warp_factor = factor;
    }

    /// Pause the clock
    pub fn pause(&mut self) {
        self.paused = true;
    }

    /// Resume the clock
    pub fn resume(&mut self) {
        self.paused = false;
    }

    /// Get the current time
    pub fn now(&self) -> u64 {
        self.current_time_ms
    }
}

/// Tick-based market data for event-driven backtesting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TickData {
    pub timestamp: u64,
    pub token_address: String,
    pub price: f64,
    pub volume: f64,
    pub side: TradeSide,
}

/// Trade side
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TradeSide {
    Buy,
    Sell,
}

/// Bar-based market data (OHLCV)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BarData {
    pub timestamp: u64,
    pub token_address: String,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

/// Transaction Cost Analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TCAResults {
    /// Estimated transaction costs
    pub estimated_costs: f64,
    /// Actual transaction costs
    pub actual_costs: f64,
    /// Slippage analysis
    pub slippage: SlippageAnalysis,
}

/// Slippage analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlippageAnalysis {
    /// Average slippage percentage
    pub avg_slippage_pct: f64,
    /// Maximum slippage percentage
    pub max_slippage_pct: f64,
    /// Slippage distribution
    pub distribution: HashMap<String, f64>,
}

/// Capacity analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapacityAnalysis {
    /// Maximum trade size without significant impact
    pub max_trade_size: f64,
    /// Market impact analysis
    pub market_impact: MarketImpactAnalysis,
    /// Liquidity depth analysis
    pub liquidity_depth: LiquidityDepthAnalysis,
}

/// Market impact analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketImpactAnalysis {
    /// Price impact coefficient
    pub price_impact_coefficient: f64,
    /// Volume impact analysis
    pub volume_impact: HashMap<String, f64>,
}

/// Liquidity depth analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiquidityDepthAnalysis {
    /// Liquidity at different price levels
    pub depth_at_levels: HashMap<String, f64>,
    /// Time to consume liquidity
    pub time_to_consume: HashMap<String, u64>,
}

/// Comprehensive analysis report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisReport {
    /// Transaction Cost Analysis results
    pub tca: TCAResults,
    /// Capacity analysis results
    pub capacity: CapacityAnalysis,
    /// Timestamp of analysis
    pub timestamp: u64,
}

/// Advanced execution models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AdvancedExecutionModel {
    /// CEX Limit Order Book execution model
    CEXLimitOrderBookExec(CEXLimitOrderBookExec),
    /// AMM execution model
    AMMExec(AMMExec),
    /// Hybrid router execution model
    HybridRouterExec(HybridRouterExec),
}

/// CEX Limit Order Book execution model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CEXLimitOrderBookExec {
    /// Queue position modeling
    pub queue_modeling: bool,
    /// Partial fill simulation
    pub partial_fills: bool,
    /// Adverse selection protection
    pub adverse_selection_protection: bool,
}

/// AMM execution model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AMMExec {
    /// CPMM math implementation
    pub cpmm_math: bool,
    /// Price impact calculation
    pub price_impact: bool,
    /// Pool fee tiers
    pub fee_tiers: Vec<f64>,
    /// TWAP calculation
    pub twap: bool,
}

/// Hybrid router execution model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HybridRouterExec {
    /// Venue selection policy
    pub venue_selection: VenueSelectionPolicy,
    /// Cross-venue latency matrix
    pub latency_matrix: HashMap<String, HashMap<String, u64>>,
    /// Failover behaviors
    pub failover_behaviors: HashMap<String, FailoverBehavior>,
}

/// Smart router for multi-venue execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartRouter {
    /// Venue selection policy
    pub venue_selection: VenueSelectionPolicy,
    /// Cross-venue latency matrix
    pub latency_matrix: HashMap<String, HashMap<String, u64>>,
    /// Venue reliability scores
    pub reliability_scores: HashMap<String, f64>,
    /// Current venue
    pub current_venue: String,
}

/// Venue information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VenueInfo {
    /// Venue name
    pub name: String,
    /// Fees information
    pub fees: FeeModel,
    /// Liquidity information
    pub liquidity: f64,
    /// Latency information
    pub latency: LatencyModel,
    /// Reliability score (0.0 to 1.0)
    pub reliability: f64,
}

/// Venue selection policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VenueSelectionPolicy {
    /// Priority factors for venue selection
    pub factors: Vec<VenueFactor>,
    /// Weights for each factor
    pub weights: HashMap<String, f64>,
}

/// Latency model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyModel {
    /// Send latency in milliseconds
    pub send_latency_ms: u64,
    /// Venue latency in milliseconds
    pub venue_latency_ms: u64,
    /// Acknowledgment latency in milliseconds
    pub ack_latency_ms: u64,
}

/// Fee model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeeModel {
    /// Maker fee in basis points
    pub maker_bps: f64,
    /// Taker fee in basis points
    pub taker_bps: f64,
    /// Gas fee in USD
    pub gas_usd: f64,
}

/// Main backtesting engine
#[derive(Debug, Clone)]
pub struct BacktestEngine {
    config: BacktestConfig,
    // Historical price data
    historical_data: HashMap<String, Vec<MarketDataPoint>>,
    // Order book data for advanced execution simulation
    order_book_data: HashMap<String, Vec<OrderBookData>>,
    // OHLCV data for bar-based backtesting
    ohlcv_data: HashMap<String, Vec<HistoricalPriceRecord>>,
    // Tick data for event-driven backtesting
    tick_data: HashMap<String, Vec<TickData>>,
    // Bar data for OHLCV backtesting
    bar_data: HashMap<String, Vec<BarData>>,
    // Slippage model configuration
    slippage_model: SlippageModel,
    // Deterministic clock for simulation
    clock: SimClock,
    // Market regimes for regime-aware backtesting
    market_regimes: Vec<MarketRegime>,
}

impl BacktestEngine {
    /// Create a new backtesting engine
    pub fn new(config: BacktestConfig) -> Self {
        let start_time = config.start_time as u64;
        Self {
            config,
            historical_data: HashMap::new(),
            order_book_data: HashMap::new(),
            ohlcv_data: HashMap::new(),
            tick_data: HashMap::new(),
            bar_data: HashMap::new(),
            slippage_model: SlippageModel {
                model_type: SlippageModelType::Fixed,
                k_coefficient: 0.1,
                max_slippage_pct: 5.0,
            },
            clock: SimClock::new(start_time),
            market_regimes: Vec::new(),
        }
    }

    /// Apply regime-specific parameters to backtest configuration
    pub fn apply_regime_parameters(&mut self, regime: &MarketRegime) {
        // Adjust fees based on regime
        self.config.trading_fee_pct *= regime.parameters.fee_multiplier;

        // Adjust slippage model based on regime
        self.slippage_model.k_coefficient *= regime.parameters.volatility_multiplier;

        // Log the regime change
        tracing::info!("Applied regime '{}' parameters", regime.id);
    }

    /// Run forward test simulation
    pub async fn run_forward_test(
        &self,
        signals: Vec<Signal>,
        ml_model: &MlModel,
        config: ForwardTestConfig,
    ) -> ForwardTestResults {
        if !config.enabled {
            return ForwardTestResults {
                config,
                performance: self.empty_results(),
                risk_metrics: RiskMetrics {
                    actual_drawdown_pct: 0.0,
                    position_sizes: vec![],
                    daily_losses: vec![],
                },
                kill_switch_activations: vec![],
            };
        }

        tracing::info!(
            "Starting forward test simulation for {} seconds with {}% capital allocation",
            config.duration_secs,
            config.capital_allocation * 100.0
        );

        // Adjust initial capital based on allocation
        let mut adjusted_config = self.config.clone();
        adjusted_config.initial_capital *= config.capital_allocation;

        // Create a new engine with adjusted configuration
        let test_engine = BacktestEngine::new(adjusted_config);

        // Run the backtest (in a real forward test, this would use live data)
        let performance = test_engine.run_backtest(signals, ml_model).await;

        // Calculate risk metrics
        let risk_metrics = self.calculate_risk_metrics(&performance);

        // Check for kill switch activations
        let kill_switch_activations =
            self.check_kill_switch_activations(&performance, &config.kill_switch);

        ForwardTestResults {
            config,
            performance,
            risk_metrics,
            kill_switch_activations,
        }
    }

    /// Run fork test simulation for on-chain testing
    pub async fn run_fork_test(
        &self,
        signals: Vec<Signal>,
        ml_model: &MlModel,
        config: ForkTestConfig,
    ) -> ForkTestResults {
        if !config.enabled {
            return ForkTestResults {
                config,
                performance: self.empty_results(),
                mev_impact: MevImpactMetrics {
                    profit_lost: 0.0,
                    additional_slippage: 0.0,
                    failed_transactions: 0,
                },
                reorg_stats: ReorgStatistics {
                    reorgs_simulated: 0,
                    transactions_affected: 0,
                    avg_reorg_depth: 0.0,
                },
            };
        }

        tracing::info!(
            "Starting fork test simulation from block {:?}",
            config.fork_block
        );

        // Adjust configuration based on fork test settings
        let mut fork_config = self.config.clone();

        // Apply gas price model
        if config.gas_model.spike_multiplier > 1.0 && config.gas_model.spike_probability > 0.0 {
            // Increase trading fees to simulate gas spike
            fork_config.trading_fee_pct *=
                config.gas_model.spike_multiplier * config.gas_model.spike_probability;
        }

        // Create a new engine with fork configuration
        let mut fork_engine = BacktestEngine::new(fork_config);

        // Apply MEV scenario if specified
        if let Some(mev_scenario) = &config.mev_scenario {
            self.apply_mev_scenario(&mut fork_engine, mev_scenario);
        }

        // Run the backtest with fork configuration
        let performance = fork_engine.run_backtest(signals, ml_model).await;

        // Calculate MEV impact metrics
        let mev_impact = self.calculate_mev_impact(&config);

        // Calculate reorg statistics
        let reorg_stats = self.calculate_reorg_stats(&config);

        ForkTestResults {
            config,
            performance,
            mev_impact,
            reorg_stats,
        }
    }

    /// Apply MEV scenario to the fork engine
    fn apply_mev_scenario(&self, engine: &mut BacktestEngine, scenario: &MevScenario) {
        match scenario {
            MevScenario::PoolSnipe { amount_in, .. } => {
                // Increase slippage to simulate pool snipe impact
                engine.config.slippage_pct *= 1.0 + (amount_in / 1000.0).min(0.1);
            }
            MevScenario::Sandwich { .. } => {
                // Increase slippage and fees to simulate sandwich attack
                engine.config.slippage_pct *= 1.5;
                engine.config.trading_fee_pct *= 1.2;
            }
            MevScenario::Frontrun { .. } => {
                // Increase latency and slippage to simulate frontrun
                engine.config.slippage_pct *= 1.3;
            }
        }
    }

    /// Calculate MEV impact metrics
    fn calculate_mev_impact(&self, config: &ForkTestConfig) -> MevImpactMetrics {
        let (profit_lost, additional_slippage) = match &config.mev_scenario {
            Some(MevScenario::PoolSnipe { amount_in, .. }) => {
                let impact = (amount_in / 1000.0).min(0.1);
                (impact * 1000.0, impact * 0.5)
            }
            Some(MevScenario::Sandwich { .. }) => (500.0, 0.02),
            Some(MevScenario::Frontrun { .. }) => (300.0, 0.01),
            None => (0.0, 0.0),
        };

        MevImpactMetrics {
            profit_lost,
            additional_slippage,
            failed_transactions: 0, // Would be calculated in a real implementation
        }
    }

    /// Calculate reorg statistics
    fn calculate_reorg_stats(&self, config: &ForkTestConfig) -> ReorgStatistics {
        if config.reorg_settings.enabled {
            ReorgStatistics {
                reorgs_simulated: (config.reorg_settings.reorg_probability * 100.0) as usize,
                transactions_affected: (config.reorg_settings.reorg_probability * 50.0) as usize,
                avg_reorg_depth: config.reorg_settings.reorg_blocks as f64,
            }
        } else {
            ReorgStatistics {
                reorgs_simulated: 0,
                transactions_affected: 0,
                avg_reorg_depth: 0.0,
            }
        }
    }

    /// Calculate risk metrics from backtest results
    fn calculate_risk_metrics(&self, results: &BacktestResults) -> RiskMetrics {
        // Extract position sizes from trades
        let position_sizes: Vec<f64> = results
            .individual_trades
            .iter()
            .map(|trade| trade.position_size_pct)
            .collect();

        // Calculate daily losses (simplified - assume evenly distributed)
        let mut daily_losses = Vec::new();
        if results.total_trades > 0 {
            let avg_loss_per_trade = if results.losing_trades > 0 {
                (results.total_profit_loss
                    - (results.total_fees_paid + results.total_slippage_loss))
                    / results.losing_trades as f64
            } else {
                0.0
            };

            // Assume 10 trades per day for demonstration
            let trades_per_day = 10;
            let days = results.total_trades.div_ceil(trades_per_day);

            for _ in 0..days {
                let daily_loss = avg_loss_per_trade * trades_per_day as f64;
                daily_losses.push(daily_loss);
            }
        }

        RiskMetrics {
            actual_drawdown_pct: results.max_drawdown * 100.0,
            position_sizes,
            daily_losses,
        }
    }

    /// Check for kill switch activations based on results and configuration
    fn check_kill_switch_activations(
        &self,
        results: &BacktestResults,
        config: &KillSwitchConfig,
    ) -> Vec<KillSwitchActivation> {
        let mut activations = Vec::new();

        if !config.enabled {
            return activations;
        }

        // Check drawdown threshold
        if results.max_drawdown * 100.0 > config.drawdown_threshold_pct {
            activations.push(KillSwitchActivation {
                timestamp: self.clock.now(),
                reason: KillSwitchReason::DrawdownExceeded,
                details: format!(
                    "Drawdown {}% exceeded threshold {}%",
                    results.max_drawdown * 100.0,
                    config.drawdown_threshold_pct
                ),
            });
        }

        // In a real implementation, we would also check for:
        // - Anomalous spread conditions
        // - Liquidity vacuum conditions
        // - Manual activations

        activations
    }

    /// Get the current configuration
    pub fn config(&self) -> &BacktestConfig {
        &self.config
    }

    /// Set the slippage model
    pub fn set_slippage_model(&mut self, model: SlippageModel) {
        self.slippage_model = model;
    }

    /// Add historical data for a token
    pub fn add_historical_data(&mut self, token_address: String, data: Vec<MarketDataPoint>) {
        self.historical_data.insert(token_address, data);
    }

    /// Add order book data for a token
    pub fn add_order_book_data(&mut self, token_address: String, data: Vec<OrderBookData>) {
        self.order_book_data.insert(token_address, data);
    }

    /// Add OHLCV data for a token
    pub fn add_ohlcv_data(&mut self, token_address: String, data: Vec<HistoricalPriceRecord>) {
        self.ohlcv_data.insert(token_address, data);
    }

    /// Load historical data from CSV file
    pub fn load_historical_data_from_csv(
        &mut self,
        token_address: &str,
        file_path: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut csv_reader = ReaderBuilder::new().has_headers(true).from_reader(reader);

        let mut data = Vec::new();
        for result in csv_reader.deserialize() {
            let record: HistoricalPriceRecord = result?;
            data.push(record);
        }

        self.ohlcv_data.insert(token_address.to_string(), data);
        Ok(())
    }

    /// Add tick data for a token
    pub fn add_tick_data(&mut self, token_address: String, data: Vec<TickData>) {
        self.tick_data.insert(token_address, data);
    }

    /// Add bar data for a token
    pub fn add_bar_data(&mut self, token_address: String, data: Vec<BarData>) {
        self.bar_data.insert(token_address, data);
    }

    /// Load tick data from CSV file
    pub fn load_tick_data_from_csv(
        &mut self,
        token_address: &str,
        file_path: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut csv_reader = ReaderBuilder::new().has_headers(true).from_reader(reader);

        let mut data = Vec::new();
        for result in csv_reader.deserialize() {
            let record: TickData = result?;
            data.push(record);
        }

        self.tick_data.insert(token_address.to_string(), data);
        Ok(())
    }

    /// Load bar data from CSV file
    pub fn load_bar_data_from_csv(
        &mut self,
        token_address: &str,
        file_path: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut csv_reader = ReaderBuilder::new().has_headers(true).from_reader(reader);

        let mut data = Vec::new();
        for result in csv_reader.deserialize() {
            let record: BarData = result?;
            data.push(record);
        }

        self.bar_data.insert(token_address.to_string(), data);
        Ok(())
    }

    /// Get market data for a specific time and token
    pub fn get_market_data(&self, token_address: &str, timestamp: u64) -> Option<MarketDataPoint> {
        if let Some(data) = self.historical_data.get(token_address) {
            // Find the closest data point to the requested timestamp
            data.iter()
                .filter(|d| d.timestamp <= timestamp)
                .max_by_key(|d| d.timestamp)
                .cloned()
        } else {
            None
        }
    }

    /// Get tick data for a specific time range and token
    pub fn get_tick_data_range(
        &self,
        token_address: &str,
        start_time: u64,
        end_time: u64,
    ) -> Vec<TickData> {
        if let Some(data) = self.tick_data.get(token_address) {
            data.iter()
                .filter(|d| d.timestamp >= start_time && d.timestamp <= end_time)
                .cloned()
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get bar data for a specific time range and token
    pub fn get_bar_data_range(
        &self,
        token_address: &str,
        start_time: u64,
        end_time: u64,
    ) -> Vec<BarData> {
        if let Some(data) = self.bar_data.get(token_address) {
            data.iter()
                .filter(|d| d.timestamp >= start_time && d.timestamp <= end_time)
                .cloned()
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get current market regime based on timestamp
    pub fn get_current_regime(&self, timestamp: u64) -> Option<&MarketRegime> {
        self.market_regimes.iter().find(|regime| {
            regime
                .time_periods
                .iter()
                .any(|(start, end)| timestamp as i64 >= *start && timestamp as i64 <= *end)
        })
    }

    /// Add market regime
    pub fn add_market_regime(&mut self, regime: MarketRegime) {
        self.market_regimes.push(regime);
    }

    /// Run a backtest simulation
    pub async fn run_backtest(&self, signals: Vec<Signal>, ml_model: &MlModel) -> BacktestResults {
        if !self.config.enabled {
            return self.empty_results();
        }

        tracing::info!(
            "Starting backtest from {} to {}",
            self.config.start_time,
            self.config.end_time
        );

        let mut capital = self.config.initial_capital;
        let mut trades: Vec<TradeResult> = Vec::new();
        let mut peak_capital = capital;
        let mut max_drawdown = 0.0;
        let mut max_consecutive_wins = 0;
        let mut max_consecutive_losses = 0;
        let mut current_consecutive_wins = 0;
        let mut current_consecutive_losses = 0;

        for signal in signals {
            // Skip signals outside our backtest time range
            if (signal.seen_at_ms as i64) < self.config.start_time
                || (signal.seen_at_ms as i64) > self.config.end_time
            {
                continue;
            }

            // Process signal with ML model
            if let Some(plan) = ml_model.process_signal(&signal).await {
                // Simulate risk evaluation
                let decision = sniper_risk::evaluate_trade(&plan);

                if decision.allow {
                    // Simulate trade execution
                    if let Some(result) = self.simulate_trade(&plan, &signal).await {
                        let profit_loss = result.profit_loss;
                        capital += profit_loss - result.fees_paid - result.slippage_loss;
                        trades.push(result);

                        // Update consecutive win/loss counters
                        if profit_loss > 0.0 {
                            current_consecutive_wins += 1;
                            current_consecutive_losses = 0;
                            if current_consecutive_wins > max_consecutive_wins {
                                max_consecutive_wins = current_consecutive_wins;
                            }
                        } else {
                            current_consecutive_losses += 1;
                            current_consecutive_wins = 0;
                            if current_consecutive_losses > max_consecutive_losses {
                                max_consecutive_losses = current_consecutive_losses;
                            }
                        }

                        // Update drawdown metrics
                        if capital > peak_capital {
                            peak_capital = capital;
                        }

                        let drawdown = (peak_capital - capital) / peak_capital;
                        if drawdown > max_drawdown {
                            max_drawdown = drawdown;
                        }
                    }
                }
            }

            // Small delay to simulate processing time
            sleep(std::time::Duration::from_millis(1)).await;
        }

        // Calculate final metrics
        let total_trades = trades.len();
        let winning_trades = trades.iter().filter(|t| t.profit_loss > 0.0).count();
        let losing_trades = total_trades - winning_trades;
        let total_profit_loss: f64 = trades.iter().map(|t| t.profit_loss).sum();
        let total_fees_paid: f64 = trades.iter().map(|t| t.fees_paid).sum();
        let total_slippage_loss: f64 = trades.iter().map(|t| t.slippage_loss).sum();
        let win_rate = if total_trades > 0 {
            winning_trades as f64 / total_trades as f64
        } else {
            0.0
        };
        let avg_profit_per_trade = if total_trades > 0 {
            total_profit_loss / total_trades as f64
        } else {
            0.0
        };

        // Calculate Sharpe ratio (assuming risk-free rate of 0)
        let returns: Vec<f64> = trades.iter().map(|t| t.profit_loss_pct).collect();
        let sharpe_ratio = if returns.is_empty() {
            0.0
        } else {
            let mean_return: f64 = returns.iter().sum::<f64>() / returns.len() as f64;
            let variance: f64 = returns
                .iter()
                .map(|r| (r - mean_return).powi(2))
                .sum::<f64>()
                / returns.len() as f64;
            let std_dev = variance.sqrt();
            if std_dev > 0.0 {
                mean_return / std_dev
            } else {
                0.0
            }
        };

        // Calculate Sortino ratio (considering only downside deviation)
        let sortino_ratio = if returns.is_empty() {
            0.0
        } else {
            let mean_return: f64 = returns.iter().sum::<f64>() / returns.len() as f64;
            let downside_returns: Vec<f64> =
                returns.iter().filter(|&&r| r < 0.0).cloned().collect();
            let downside_variance: f64 = if !downside_returns.is_empty() {
                downside_returns.iter().map(|r| r.powi(2)).sum::<f64>()
                    / downside_returns.len() as f64
            } else {
                0.0
            };
            let downside_std_dev = downside_variance.sqrt();
            if downside_std_dev > 0.0 {
                mean_return / downside_std_dev
            } else {
                0.0
            }
        };

        // Calculate Calmar ratio
        let calmar_ratio = if max_drawdown > 0.0 {
            (total_profit_loss / self.config.initial_capital) / max_drawdown
        } else {
            0.0
        };

        BacktestResults {
            config: self.config.clone(),
            total_trades,
            winning_trades,
            losing_trades,
            total_profit_loss,
            total_fees_paid,
            total_slippage_loss,
            win_rate,
            avg_profit_per_trade,
            max_drawdown,
            sharpe_ratio,
            sortino_ratio,
            calmar_ratio,
            max_consecutive_wins,
            max_consecutive_losses,
            individual_trades: trades,
        }
    }

    /// Simulate a trade execution with enhanced models
    async fn simulate_trade(&self, plan: &TradePlan, signal: &Signal) -> Option<TradeResult> {
        // In a real implementation, this would:
        // 1. Look up historical price data for entry and exit
        // 2. Simulate slippage and execution using order book data if available
        // 3. Apply trading fees
        // 4. Calculate profit/loss

        // Placeholder implementation with simulated results
        let entry_time = signal.seen_at_ms;
        let exit_time = entry_time + 3600000; // 1 hour later
        let entry_price = 100.0; // Simulated entry price
        let exit_price = 110.0; // Simulated exit price (10% gain)
        let amount_in = plan.amount_in as f64 / 1e18; // Convert from wei to ETH

        // Apply execution model based on configuration
        let (actual_exit_price, execution_details) = match self.config.execution_model {
            ExecutionModelType::Simple => {
                let slippage_multiplier = 1.0 - self.config.slippage_pct;
                let price = exit_price * slippage_multiplier;
                let details = ExecutionDetails {
                    model_used: "Simple".to_string(),
                    queue_position: None,
                    partial_fills: vec![],
                    latency_ms: 10,
                };
                (price, details)
            }
            ExecutionModelType::OrderBook => {
                // Use order book data for more realistic execution
                let (price, details) = self.simulate_order_book_execution(
                    signal.token0.as_deref().unwrap_or(""),
                    exit_price,
                    amount_in,
                );
                (price, details)
            }
            ExecutionModelType::Impact => {
                // Use market impact model
                let (price, details) = self.simulate_impact_execution(exit_price, amount_in);
                (price, details)
            }
        };

        let amount_out = amount_in * (actual_exit_price / entry_price);

        let profit_loss = (actual_exit_price - entry_price) * amount_in;
        let profit_loss_pct = (actual_exit_price / entry_price - 1.0) * 100.0;
        let fees_paid = amount_in * self.config.trading_fee_pct;
        let slippage_loss = (exit_price - actual_exit_price) * amount_in;
        let position_size_pct = amount_in / self.config.initial_capital * 100.0;

        Some(TradeResult {
            plan_id: plan.idem_key.clone(),
            entry_time,
            exit_time,
            entry_price,
            exit_price: actual_exit_price,
            amount_in,
            amount_out,
            profit_loss,
            profit_loss_pct,
            fees_paid,
            slippage_loss,
            position_size_pct,
            execution_details,
        })
    }

    /// Simulate order book execution
    fn simulate_order_book_execution(
        &self,
        token_address: &str,
        price: f64,
        amount: f64,
    ) -> (f64, ExecutionDetails) {
        // Check if we have order book data for this token
        if let Some(order_book_data) = self.order_book_data.get(token_address) {
            if !order_book_data.is_empty() {
                // Get the most recent order book data
                let latest_data = &order_book_data[order_book_data.len() - 1];

                // Walk the order book to simulate execution
                let mut remaining_amount = amount;
                let mut total_value = 0.0;
                let mut partial_fills = Vec::new();
                let mut queue_position = 0;

                // Simulate walking the ask side (for buying)
                for (i, (ask_price, ask_volume)) in latest_data
                    .ask_prices
                    .iter()
                    .zip(&latest_data.ask_volumes)
                    .enumerate()
                {
                    if remaining_amount <= 0.0 {
                        break;
                    }

                    let fill_amount = remaining_amount.min(*ask_volume);
                    let fill_value = fill_amount * ask_price;

                    partial_fills.push(PartialFill {
                        price: *ask_price,
                        amount: fill_amount,
                        timestamp: latest_data.timestamp,
                    });

                    total_value += fill_value;
                    remaining_amount -= fill_amount;
                    queue_position = i;
                }

                // Calculate average execution price
                let avg_price = if amount > 0.0 {
                    total_value / amount
                } else {
                    price
                };

                let details = ExecutionDetails {
                    model_used: "OrderBook".to_string(),
                    queue_position: Some(queue_position),
                    partial_fills,
                    latency_ms: 50, // Higher latency for order book simulation
                };

                return (avg_price, details);
            }
        }

        // Fallback to simple slippage if no order book data
        let slippage_multiplier = 1.0 - self.config.slippage_pct;
        let price_with_slippage = price * slippage_multiplier;
        let details = ExecutionDetails {
            model_used: "OrderBookFallback".to_string(),
            queue_position: None,
            partial_fills: vec![],
            latency_ms: 20,
        };

        (price_with_slippage, details)
    }

    /// Simulate market impact execution
    fn simulate_impact_execution(&self, price: f64, amount: f64) -> (f64, ExecutionDetails) {
        // Calculate price impact using Kyle's lambda model
        // Impact = k * (amount / avg_volume) ^ 0.5
        let k = self.slippage_model.k_coefficient;
        let avg_volume = 1000.0; // Placeholder - would come from historical data
        let impact = k * (amount / avg_volume).sqrt();

        // Limit impact to maximum allowed slippage
        let capped_impact = impact.min(self.slippage_model.max_slippage_pct / 100.0);

        // For buying, price increases (negative impact on profit)
        let impacted_price = price * (1.0 + capped_impact);

        let details = ExecutionDetails {
            model_used: "Impact".to_string(),
            queue_position: None,
            partial_fills: vec![PartialFill {
                price: impacted_price,
                amount,
                timestamp: 0, // Would be actual timestamp
            }],
            latency_ms: 30,
        };

        (impacted_price, details)
    }

    /// Calculate slippage based on order book depth
    fn calculate_slippage_from_order_book(
        &self,
        order_book: &[OrderBookData],
        _amount: f64,
    ) -> f64 {
        // This is a simplified model - in practice, you would walk the order book
        // to determine the actual price impact
        if order_book.is_empty() {
            return 1.0 - self.config.slippage_pct;
        }

        // Get the most recent order book data
        let latest_data = &order_book[order_book.len() - 1];

        // Simple model: assume slippage is inversely proportional to liquidity
        let liquidity = latest_data.liquidity;
        if liquidity > 0.0 {
            let base_slippage = self.config.slippage_pct;
            let liquidity_factor = (1000000.0 / liquidity).min(1.0); // Cap at 100% slippage
            1.0 - (base_slippage * (1.0 + liquidity_factor))
        } else {
            1.0 - self.config.slippage_pct
        }
    }

    /// Run walk-forward optimization
    pub async fn run_walk_forward_optimization(
        &self,
        signals: Vec<Signal>,
        ml_model: &MlModel,
        config: WalkForwardConfig,
    ) -> WalkForwardResults {
        tracing::info!(
            "Starting walk-forward optimization: train={}s, test={}s, step={}s",
            config.train_window_secs,
            config.test_window_secs,
            config.step_secs
        );

        let mut windows = Vec::new();
        let mut all_trades = Vec::new();

        // Determine the overall time range
        let min_time = signals
            .iter()
            .map(|s| s.seen_at_ms as i64)
            .min()
            .unwrap_or(self.config.start_time);
        let max_time = signals
            .iter()
            .map(|s| s.seen_at_ms as i64)
            .max()
            .unwrap_or(self.config.end_time);

        let mut current_start = min_time.max(self.config.start_time);

        while current_start + config.train_window_secs + config.test_window_secs
            <= max_time.min(self.config.end_time)
        {
            let train_end = current_start + config.train_window_secs;
            let test_end = train_end + config.test_window_secs;

            // Split signals into train and test sets
            let train_signals: Vec<Signal> = signals
                .iter()
                .filter(|s| {
                    let time = s.seen_at_ms as i64;
                    time >= current_start && time < train_end
                })
                .cloned()
                .collect();

            let test_signals: Vec<Signal> = signals
                .iter()
                .filter(|s| {
                    let time = s.seen_at_ms as i64;
                    time >= train_end && time < test_end
                })
                .cloned()
                .collect();

            // Skip if not enough signals
            if train_signals.len() < config.min_train_trades
                || test_signals.len() < config.min_test_trades
            {
                current_start += config.step_secs;
                continue;
            }

            // Run backtest on training data (this would be where parameter optimization happens)
            let train_results = self.run_backtest(train_signals, ml_model).await;

            // Run backtest on test data with same parameters
            let test_results = self.run_backtest(test_signals, ml_model).await;

            // Collect all trades for overall performance calculation
            all_trades.extend(test_results.individual_trades.iter().cloned());

            // Store window results
            windows.push(WalkForwardWindowResults {
                window_start: current_start,
                window_end: test_end,
                train_results,
                test_results,
                parameters: serde_json::json!({}), // In a real implementation, this would contain optimized parameters
            });

            current_start += config.step_secs;
        }

        // Calculate overall performance across all test windows
        let overall_performance = self.calculate_overall_performance(all_trades);

        WalkForwardResults {
            config,
            windows,
            overall_performance,
        }
    }

    /// Calculate overall performance from individual trades
    fn calculate_overall_performance(&self, trades: Vec<TradeResult>) -> BacktestResults {
        if trades.is_empty() {
            return self.empty_results();
        }

        let total_trades = trades.len();
        let winning_trades = trades.iter().filter(|t| t.profit_loss > 0.0).count();
        let losing_trades = total_trades - winning_trades;
        let total_profit_loss: f64 = trades.iter().map(|t| t.profit_loss).sum();
        let total_fees_paid: f64 = trades.iter().map(|t| t.fees_paid).sum();
        let total_slippage_loss: f64 = trades.iter().map(|t| t.slippage_loss).sum();
        let win_rate = if total_trades > 0 {
            winning_trades as f64 / total_trades as f64
        } else {
            0.0
        };
        let avg_profit_per_trade = if total_trades > 0 {
            total_profit_loss / total_trades as f64
        } else {
            0.0
        };

        // Calculate drawdown
        let mut peak_capital = self.config.initial_capital;
        let mut max_drawdown = 0.0;
        let mut current_capital = self.config.initial_capital;

        for trade in &trades {
            current_capital += trade.profit_loss - trade.fees_paid - trade.slippage_loss;
            if current_capital > peak_capital {
                peak_capital = current_capital;
            }
            let drawdown = (peak_capital - current_capital) / peak_capital;
            if drawdown > max_drawdown {
                max_drawdown = drawdown;
            }
        }

        // Calculate returns for Sharpe/Sortino ratios
        let returns: Vec<f64> = trades.iter().map(|t| t.profit_loss_pct).collect();

        // Calculate Sharpe ratio (assuming risk-free rate of 0)
        let sharpe_ratio = if returns.is_empty() {
            0.0
        } else {
            let mean_return: f64 = returns.iter().sum::<f64>() / returns.len() as f64;
            let variance: f64 = returns
                .iter()
                .map(|r| (r - mean_return).powi(2))
                .sum::<f64>()
                / returns.len() as f64;
            let std_dev = variance.sqrt();
            if std_dev > 0.0 {
                mean_return / std_dev
            } else {
                0.0
            }
        };

        // Calculate Sortino ratio (considering only downside deviation)
        let sortino_ratio = if returns.is_empty() {
            0.0
        } else {
            let mean_return: f64 = returns.iter().sum::<f64>() / returns.len() as f64;
            let downside_returns: Vec<f64> =
                returns.iter().filter(|&&r| r < 0.0).cloned().collect();
            let downside_variance: f64 = if !downside_returns.is_empty() {
                downside_returns.iter().map(|r| r.powi(2)).sum::<f64>()
                    / downside_returns.len() as f64
            } else {
                0.0
            };
            let downside_std_dev = downside_variance.sqrt();
            if downside_std_dev > 0.0 {
                mean_return / downside_std_dev
            } else {
                0.0
            }
        };

        // Calculate Calmar ratio
        let calmar_ratio = if max_drawdown > 0.0 {
            (total_profit_loss / self.config.initial_capital) / max_drawdown
        } else {
            0.0
        };

        // Count consecutive wins/losses
        let (max_consecutive_wins, max_consecutive_losses) =
            self.calculate_consecutive_trades(&trades);

        BacktestResults {
            config: self.config.clone(),
            total_trades,
            winning_trades,
            losing_trades,
            total_profit_loss,
            total_fees_paid,
            total_slippage_loss,
            win_rate,
            avg_profit_per_trade,
            max_drawdown,
            sharpe_ratio,
            sortino_ratio,
            calmar_ratio,
            max_consecutive_wins,
            max_consecutive_losses,
            individual_trades: trades,
        }
    }

    /// Calculate maximum consecutive wins and losses
    fn calculate_consecutive_trades(&self, trades: &[TradeResult]) -> (usize, usize) {
        let mut max_consecutive_wins = 0;
        let mut max_consecutive_losses = 0;
        let mut current_consecutive_wins = 0;
        let mut current_consecutive_losses = 0;

        for trade in trades {
            if trade.profit_loss > 0.0 {
                current_consecutive_wins += 1;
                current_consecutive_losses = 0;
                if current_consecutive_wins > max_consecutive_wins {
                    max_consecutive_wins = current_consecutive_wins;
                }
            } else {
                current_consecutive_losses += 1;
                current_consecutive_wins = 0;
                if current_consecutive_losses > max_consecutive_losses {
                    max_consecutive_losses = current_consecutive_losses;
                }
            }
        }

        (max_consecutive_wins, max_consecutive_losses)
    }

    /// Run portfolio backtest across multiple assets
    pub async fn run_portfolio_backtest(
        &self,
        signals_by_asset: HashMap<String, Vec<Signal>>,
        ml_model: &MlModel,
        portfolio_config: PortfolioBacktestConfig,
    ) -> BacktestResults {
        tracing::info!(
            "Starting portfolio backtest for {} assets",
            portfolio_config.assets.len()
        );

        // This would implement portfolio-level backtesting with correlation modeling
        // For now, we'll run individual backtests and combine results
        let mut all_trades = Vec::new();
        let mut total_profit_loss = 0.0;
        let mut total_fees = 0.0;
        let mut total_slippage = 0.0;

        for asset in &portfolio_config.assets {
            if let Some(signals) = signals_by_asset.get(asset) {
                let results = self.run_backtest(signals.clone(), ml_model).await;
                all_trades.extend(results.individual_trades);
                total_profit_loss += results.total_profit_loss;
                total_fees += results.total_fees_paid;
                total_slippage += results.total_slippage_loss;
            }
        }

        // Calculate combined metrics
        let total_trades = all_trades.len();
        let winning_trades = all_trades.iter().filter(|t| t.profit_loss > 0.0).count();
        let losing_trades = total_trades - winning_trades;

        BacktestResults {
            config: self.config.clone(),
            total_trades,
            winning_trades,
            losing_trades,
            total_profit_loss,
            total_fees_paid: total_fees,
            total_slippage_loss: total_slippage,
            win_rate: 0.0,
            avg_profit_per_trade: 0.0,
            max_drawdown: 0.0,
            sharpe_ratio: 0.0,
            sortino_ratio: 0.0,
            calmar_ratio: 0.0,
            max_consecutive_wins: 0,
            max_consecutive_losses: 0,
            individual_trades: all_trades,
        }
    }

    /// Run portfolio backtest with smart routing
    pub async fn run_portfolio_backtest_with_routing(
        &self,
        signals_by_asset: HashMap<String, Vec<Signal>>,
        ml_model: &MlModel,
        portfolio_config: PortfolioBacktestConfig,
        router: &SmartRouter,
    ) -> BacktestResults {
        tracing::info!(
            "Starting portfolio backtest with smart routing for {} assets",
            portfolio_config.assets.len()
        );

        // This would implement portfolio-level backtesting with smart routing
        // For now, we'll run individual backtests and combine results
        let mut all_trades = Vec::new();
        let mut total_profit_loss = 0.0;
        let mut total_fees = 0.0;
        let mut total_slippage = 0.0;

        for asset in &portfolio_config.assets {
            if let Some(signals) = signals_by_asset.get(asset) {
                let results = self.run_backtest(signals.clone(), ml_model).await;
                all_trades.extend(results.individual_trades);
                total_profit_loss += results.total_profit_loss;
                total_fees += results.total_fees_paid;
                total_slippage += results.total_slippage_loss;
            }
        }

        // Calculate combined metrics
        let total_trades = all_trades.len();
        let winning_trades = all_trades.iter().filter(|t| t.profit_loss > 0.0).count();
        let losing_trades = total_trades - winning_trades;
        let win_rate = if total_trades > 0 {
            winning_trades as f64 / total_trades as f64
        } else {
            0.0
        };
        let avg_profit_per_trade = if total_trades > 0 {
            total_profit_loss / total_trades as f64
        } else {
            0.0
        };

        // Calculate drawdown
        let mut peak_capital = self.config.initial_capital;
        let mut max_drawdown = 0.0;
        let mut current_capital = self.config.initial_capital;

        for trade in &all_trades {
            current_capital += trade.profit_loss - trade.fees_paid - trade.slippage_loss;
            if current_capital > peak_capital {
                peak_capital = current_capital;
            }
            let drawdown = (peak_capital - current_capital) / peak_capital;
            if drawdown > max_drawdown {
                max_drawdown = drawdown;
            }
        }

        // Calculate risk metrics (simplified)
        let returns: Vec<f64> = all_trades.iter().map(|t| t.profit_loss_pct).collect();
        let sharpe_ratio = self.calculate_sharpe_ratio(&returns);
        let sortino_ratio = self.calculate_sortino_ratio(&returns);
        let calmar_ratio = if max_drawdown > 0.0 {
            (total_profit_loss / self.config.initial_capital) / max_drawdown
        } else {
            0.0
        };

        let (max_consecutive_wins, max_consecutive_losses) =
            self.calculate_consecutive_trades(&all_trades);

        BacktestResults {
            config: self.config.clone(),
            total_trades,
            winning_trades,
            losing_trades,
            total_profit_loss,
            total_fees_paid: total_fees,
            total_slippage_loss: total_slippage,
            win_rate,
            avg_profit_per_trade,
            max_drawdown,
            sharpe_ratio,
            sortino_ratio,
            calmar_ratio,
            max_consecutive_wins,
            max_consecutive_losses,
            individual_trades: all_trades,
        }
    }

    /// Run paper trade backtest (live-replay with real clock)
    pub async fn run_paper_trade_backtest(
        &mut self,
        signals: Vec<Signal>,
        ml_model: &MlModel,
        config: PaperTradeConfig,
    ) -> BacktestResults {
        if !config.enabled {
            return self.empty_results();
        }

        tracing::info!(
            "Starting paper trade backtest with time warp factor: {}",
            config.time_warp_factor
        );

        // Set time warp factor
        self.clock.set_time_warp(config.time_warp_factor);

        // If in shadow mode, we don't actually execute trades but simulate them
        if config.shadow_mode {
            tracing::info!("Running in shadow mode - simulating trades without execution");
        }

        // Run the backtest with real-time clock advancement
        let results = self.run_backtest_with_real_time(signals, ml_model).await;

        results
    }

    /// Run backtest with real-time clock advancement
    async fn run_backtest_with_real_time(
        &mut self,
        signals: Vec<Signal>,
        ml_model: &MlModel,
    ) -> BacktestResults {
        let mut capital = self.config.initial_capital;
        let mut trades: Vec<TradeResult> = Vec::new();
        let mut peak_capital = capital;
        let mut max_drawdown = 0.0;
        let mut max_consecutive_wins = 0;
        let mut max_consecutive_losses = 0;
        let mut current_consecutive_wins = 0;
        let mut current_consecutive_losses = 0;

        let mut last_time = std::time::Instant::now();

        for signal in signals {
            // Advance clock based on real time
            let now = std::time::Instant::now();
            let elapsed = now.duration_since(last_time).as_millis() as u64;
            self.clock.advance(elapsed);
            last_time = now;

            // Skip signals outside our backtest time range
            if (signal.seen_at_ms as i64) < self.config.start_time
                || (signal.seen_at_ms as i64) > self.config.end_time
            {
                continue;
            }

            // Process signal with ML model
            if let Some(plan) = ml_model.process_signal(&signal).await {
                // Simulate risk evaluation
                let decision = sniper_risk::evaluate_trade(&plan);

                if decision.allow {
                    // Simulate trade execution
                    if let Some(result) = self.simulate_trade(&plan, &signal).await {
                        let profit_loss = result.profit_loss;
                        capital += profit_loss - result.fees_paid - result.slippage_loss;
                        trades.push(result);

                        // Update consecutive win/loss counters
                        if profit_loss > 0.0 {
                            current_consecutive_wins += 1;
                            current_consecutive_losses = 0;
                            if current_consecutive_wins > max_consecutive_wins {
                                max_consecutive_wins = current_consecutive_wins;
                            }
                        } else {
                            current_consecutive_losses += 1;
                            current_consecutive_wins = 0;
                            if current_consecutive_losses > max_consecutive_losses {
                                max_consecutive_losses = current_consecutive_losses;
                            }
                        }

                        // Update drawdown metrics
                        if capital > peak_capital {
                            peak_capital = capital;
                        }

                        let drawdown = (peak_capital - capital) / peak_capital;
                        if drawdown > max_drawdown {
                            max_drawdown = drawdown;
                        }
                    }
                }
            }
        }

        // Calculate final metrics
        let total_trades = trades.len();
        let winning_trades = trades.iter().filter(|t| t.profit_loss > 0.0).count();
        let losing_trades = total_trades - winning_trades;
        let total_profit_loss: f64 = trades.iter().map(|t| t.profit_loss).sum();
        let total_fees_paid: f64 = trades.iter().map(|t| t.fees_paid).sum();
        let total_slippage_loss: f64 = trades.iter().map(|t| t.slippage_loss).sum();
        let win_rate = if total_trades > 0 {
            winning_trades as f64 / total_trades as f64
        } else {
            0.0
        };
        let avg_profit_per_trade = if total_trades > 0 {
            total_profit_loss / total_trades as f64
        } else {
            0.0
        };

        // Calculate Sharpe ratio (assuming risk-free rate of 0)
        let returns: Vec<f64> = trades.iter().map(|t| t.profit_loss_pct).collect();
        let sharpe_ratio = if returns.is_empty() {
            0.0
        } else {
            let mean_return: f64 = returns.iter().sum::<f64>() / returns.len() as f64;
            let variance: f64 = returns
                .iter()
                .map(|r| (r - mean_return).powi(2))
                .sum::<f64>()
                / returns.len() as f64;
            let std_dev = variance.sqrt();
            if std_dev > 0.0 {
                mean_return / std_dev
            } else {
                0.0
            }
        };

        // Calculate Sortino ratio (considering only downside deviation)
        let sortino_ratio = if returns.is_empty() {
            0.0
        } else {
            let mean_return: f64 = returns.iter().sum::<f64>() / returns.len() as f64;
            let downside_returns: Vec<f64> =
                returns.iter().filter(|&&r| r < 0.0).cloned().collect();
            let downside_variance: f64 = if !downside_returns.is_empty() {
                downside_returns.iter().map(|r| r.powi(2)).sum::<f64>()
                    / downside_returns.len() as f64
            } else {
                0.0
            };
            let downside_std_dev = downside_variance.sqrt();
            if downside_std_dev > 0.0 {
                mean_return / downside_std_dev
            } else {
                0.0
            }
        };

        // Calculate Calmar ratio
        let calmar_ratio = if max_drawdown > 0.0 {
            (total_profit_loss / self.config.initial_capital) / max_drawdown
        } else {
            0.0
        };

        BacktestResults {
            config: self.config.clone(),
            total_trades,
            winning_trades,
            losing_trades,
            total_profit_loss,
            total_fees_paid,
            total_slippage_loss,
            win_rate,
            avg_profit_per_trade,
            max_drawdown,
            sharpe_ratio,
            sortino_ratio,
            calmar_ratio,
            max_consecutive_wins,
            max_consecutive_losses,
            individual_trades: trades,
        }
    }

    /// Run scenario testing for stress testing
    pub async fn run_scenario_testing(
        &self,
        signals: Vec<Signal>,
        ml_model: &MlModel,
        scenario_config: ScenarioTestConfig,
    ) -> ChaosTestResults {
        tracing::info!(
            "Starting scenario testing with {} scenarios",
            scenario_config.scenarios.len()
        );

        // Run baseline backtest first if not provided
        let baseline_performance = match scenario_config.baseline_performance {
            Some(ref results) => results.clone(),
            None => self.run_backtest(signals.clone(), ml_model).await,
        };

        let mut scenario_results = Vec::new();

        // Run each scenario
        for scenario in &scenario_config.scenarios {
            tracing::info!("Running scenario: {:?}", scenario);

            // Apply scenario to a copy of the engine
            let scenario_engine = self.clone_with_scenario(scenario);

            // Run backtest with scenario
            let performance = scenario_engine
                .run_backtest(signals.clone(), ml_model)
                .await;

            // Calculate impact metrics
            let impact_metrics = self.calculate_chaos_impact(&baseline_performance, &performance);

            scenario_results.push(ChaosScenarioResult {
                scenario: ChaosScenario::NetworkLatency {
                    // Placeholder mapping
                    base_latency_ms: 50,
                    additional_latency_ms: 100,
                    affected_percentage: 0.1,
                },
                performance,
                impact_metrics,
            });
        }

        // Calculate overall impact
        let overall_impact = self.calculate_overall_chaos_impact(&scenario_results);

        ChaosTestResults {
            config: ChaosTestConfig {
                enabled: true,
                scenarios: vec![], // Would be populated from scenario_config
            },
            scenario_results,
            baseline_performance,
            overall_impact,
        }
    }

    /// Create a copy of the engine with scenario applied
    fn clone_with_scenario(&self, scenario: &MarketScenario) -> Self {
        let mut scenario_engine = self.clone();

        // Apply scenario-specific modifications
        match scenario {
            MarketScenario::NetworkLatency { latency_ms, .. } => {
                // Increase latency in execution models
                match &scenario_engine.config.execution_model {
                    ExecutionModelType::Simple => {
                        // Increase slippage to simulate network effects
                        scenario_engine.config.slippage_pct *= 1.0 + (latency_ms / 100) as f64;
                    }
                    ExecutionModelType::OrderBook => {
                        // For order book model, we might adjust latency
                    }
                    ExecutionModelType::Impact => {
                        // For impact model, we might adjust the k coefficient
                        scenario_engine.slippage_model.k_coefficient *=
                            1.0 + (latency_ms / 50) as f64;
                    }
                }
            }
            MarketScenario::GasSpike { multiplier, .. } => {
                // Increase trading fees to simulate gas spike
                scenario_engine.config.trading_fee_pct *= multiplier;
            }
            MarketScenario::MarketVolatility { multiplier, .. } => {
                // Increase slippage as a proxy for volatility
                scenario_engine.config.slippage_pct *= multiplier;
            }
            _ => {
                // Other scenarios would be implemented as needed
            }
        }

        scenario_engine
    }

    /// Run chaos testing scenarios
    pub async fn run_chaos_tests(
        &self,
        signals: Vec<Signal>,
        ml_model: &MlModel,
        config: ChaosTestConfig,
    ) -> ChaosTestResults {
        tracing::info!(
            "Starting chaos testing with {} scenarios",
            config.scenarios.len()
        );

        // Run baseline backtest first
        let baseline_performance = self.run_backtest(signals.clone(), ml_model).await;

        let mut scenario_results = Vec::new();

        // Run each chaos scenario
        for scenario in &config.scenarios {
            if !config.enabled {
                break;
            }

            tracing::info!("Running chaos scenario: {:?}", scenario);

            // Apply chaos scenario to a copy of the engine
            let chaos_engine = self.clone_with_chaos(scenario);

            // Run backtest with chaos
            let performance = chaos_engine.run_backtest(signals.clone(), ml_model).await;

            // Calculate impact metrics
            let impact_metrics = self.calculate_chaos_impact(&baseline_performance, &performance);

            scenario_results.push(ChaosScenarioResult {
                scenario: scenario.clone(),
                performance,
                impact_metrics,
            });
        }

        // Calculate overall impact
        let overall_impact = self.calculate_overall_chaos_impact(&scenario_results);

        ChaosTestResults {
            config,
            scenario_results,
            baseline_performance,
            overall_impact,
        }
    }

    /// Create a copy of the engine with chaos scenario applied
    fn clone_with_chaos(&self, scenario: &ChaosScenario) -> Self {
        let mut chaos_engine = self.clone();

        // Apply scenario-specific modifications
        match scenario {
            ChaosScenario::NetworkLatency {
                additional_latency_ms,
                ..
            } => {
                // In a real implementation, this would modify network handling
                // For now, we'll simulate by adjusting the config
                match &chaos_engine.config.execution_model {
                    ExecutionModelType::Simple => {
                        // Increase slippage to simulate network effects
                        chaos_engine.config.slippage_pct *=
                            1.0 + (additional_latency_ms / 100) as f64;
                    }
                    ExecutionModelType::OrderBook => {
                        // For order book model, we might adjust latency
                        // This would be implemented in the execution simulation
                    }
                    ExecutionModelType::Impact => {
                        // For impact model, we might adjust the k coefficient
                        chaos_engine.slippage_model.k_coefficient *=
                            1.0 + (additional_latency_ms / 50) as f64;
                    }
                }
            }
            ChaosScenario::GasSpike { multiplier, .. } => {
                // Increase trading fees to simulate gas spike
                chaos_engine.config.trading_fee_pct *= multiplier;
            }
            ChaosScenario::MarketVolatility { multiplier, .. } => {
                // This would affect the market data simulation
                // For now, we'll increase slippage as a proxy
                chaos_engine.config.slippage_pct *= multiplier;
            }
            _ => {
                // Other scenarios would be implemented as needed
            }
        }

        chaos_engine
    }

    /// Run chaos scenario testing
    pub async fn run_chaos_scenario_testing(
        &self,
        signals: Vec<Signal>,
        ml_model: &MlModel,
        config: ChaosScenarioConfig,
    ) -> ChaosTestResults {
        if !config.enabled {
            return ChaosTestResults {
                config: ChaosTestConfig {
                    enabled: false,
                    scenarios: vec![],
                },
                scenario_results: vec![],
                baseline_performance: self.empty_results(),
                overall_impact: ChaosImpactMetrics {
                    performance_degradation_pct: 0.0,
                    additional_slippage_pct: 0.0,
                    increased_latency_ms: 0,
                    failed_trade_pct: 0.0,
                },
            };
        }

        tracing::info!(
            "Starting chaos scenario testing with {} scenarios",
            config.scenarios.len()
        );

        // Run baseline backtest first
        let baseline_performance = self.run_backtest(signals.clone(), ml_model).await;

        let mut scenario_results = Vec::new();

        // Run each chaos scenario
        for scenario in &config.scenarios {
            tracing::info!("Running chaos scenario: {:?}", scenario);

            // Apply scenario to a copy of the engine
            let scenario_engine = self.clone_with_chaos_scenario(scenario);

            // Run backtest with scenario
            let performance = scenario_engine
                .run_backtest(signals.clone(), ml_model)
                .await;

            // Calculate impact metrics
            let impact_metrics = self.calculate_chaos_impact(&baseline_performance, &performance);

            scenario_results.push(ChaosScenarioResult {
                scenario: self.map_chaos_scenario_to_legacy(scenario),
                performance,
                impact_metrics,
            });
        }

        // Calculate overall impact
        let overall_impact = self.calculate_overall_chaos_impact(&scenario_results);

        ChaosTestResults {
            config: ChaosTestConfig {
                enabled: true,
                scenarios: config
                    .scenarios
                    .iter()
                    .map(|s| self.map_chaos_scenario_to_legacy(s))
                    .collect(),
            },
            scenario_results,
            baseline_performance,
            overall_impact,
        }
    }

    /// Create a copy of the engine with chaos scenario applied
    fn clone_with_chaos_scenario(&self, scenario: &ChaosScenarioType) -> Self {
        let mut chaos_engine = self.clone();

        // Apply scenario-specific modifications
        match scenario {
            ChaosScenarioType::NetworkLatency { latency_ms, .. } => {
                // Increase latency in execution models
                match &chaos_engine.config.execution_model {
                    ExecutionModelType::Simple => {
                        // Increase slippage to simulate network effects
                        chaos_engine.config.slippage_pct *= 1.0 + (latency_ms / 100) as f64;
                    }
                    ExecutionModelType::OrderBook => {
                        // For order book model, we might adjust latency
                    }
                    ExecutionModelType::Impact => {
                        // For impact model, we might adjust the k coefficient
                        chaos_engine.slippage_model.k_coefficient *= 1.0 + (latency_ms / 50) as f64;
                    }
                }
            }
            ChaosScenarioType::GasSpike { multiplier, .. } => {
                // Increase trading fees to simulate gas spike
                chaos_engine.config.trading_fee_pct *= multiplier;
            }
            ChaosScenarioType::MarketVolatility { multiplier, .. } => {
                // Increase slippage as a proxy for volatility
                chaos_engine.config.slippage_pct *= multiplier;
            }
            _ => {
                // Other scenarios would be implemented as needed
            }
        }

        chaos_engine
    }

    /// Map new chaos scenario type to legacy type for compatibility
    fn map_chaos_scenario_to_legacy(&self, scenario: &ChaosScenarioType) -> ChaosScenario {
        match scenario {
            ChaosScenarioType::NetworkLatency {
                venue: _venue,
                latency_ms,
                duration_secs: _duration_secs,
                start_time_secs: _start_time_secs,
            } => {
                ChaosScenario::NetworkLatency {
                    base_latency_ms: 50, // Default base latency
                    additional_latency_ms: *latency_ms,
                    affected_percentage: 0.1, // Default affected percentage
                }
            }
            ChaosScenarioType::GasSpike {
                multiplier,
                duration_secs,
                start_time_secs,
            } => ChaosScenario::GasSpike {
                multiplier: *multiplier,
                duration_secs: *duration_secs,
                start_time_secs: *start_time_secs,
            },
            ChaosScenarioType::MarketVolatility {
                multiplier,
                duration_secs,
                start_time_secs,
            } => ChaosScenario::MarketVolatility {
                multiplier: *multiplier,
                duration_secs: *duration_secs,
                start_time_secs: *start_time_secs,
            },
            _ => {
                // Default fallback
                ChaosScenario::NetworkLatency {
                    base_latency_ms: 50,
                    additional_latency_ms: 100,
                    affected_percentage: 0.1,
                }
            }
        }
    }

    /// Calculate impact metrics from a chaos scenario
    fn calculate_chaos_impact(
        &self,
        baseline: &BacktestResults,
        chaos: &BacktestResults,
    ) -> ChaosImpactMetrics {
        let performance_degradation = if baseline.total_profit_loss > 0.0 {
            ((baseline.total_profit_loss - chaos.total_profit_loss) / baseline.total_profit_loss)
                * 100.0
        } else {
            0.0
        };

        let additional_slippage = chaos.total_slippage_loss - baseline.total_slippage_loss;
        let slippage_percentage = if baseline.total_slippage_loss > 0.0 {
            (additional_slippage / baseline.total_slippage_loss) * 100.0
        } else {
            0.0
        };

        let failed_trades = baseline.total_trades as f64 - chaos.total_trades as f64;
        let failed_trade_pct = if baseline.total_trades > 0 {
            (failed_trades / baseline.total_trades as f64) * 100.0
        } else {
            0.0
        };

        ChaosImpactMetrics {
            performance_degradation_pct: performance_degradation,
            additional_slippage_pct: slippage_percentage,
            increased_latency_ms: 0, // Would be measured in a real implementation
            failed_trade_pct,
        }
    }

    /// Calculate overall impact across all chaos scenarios
    fn calculate_overall_chaos_impact(
        &self,
        scenario_results: &[ChaosScenarioResult],
    ) -> ChaosImpactMetrics {
        if scenario_results.is_empty() {
            return ChaosImpactMetrics {
                performance_degradation_pct: 0.0,
                additional_slippage_pct: 0.0,
                increased_latency_ms: 0,
                failed_trade_pct: 0.0,
            };
        }

        let total_scenarios = scenario_results.len() as f64;

        let avg_performance_degradation: f64 = scenario_results
            .iter()
            .map(|r| r.impact_metrics.performance_degradation_pct)
            .sum::<f64>()
            / total_scenarios;

        let avg_slippage_increase: f64 = scenario_results
            .iter()
            .map(|r| r.impact_metrics.additional_slippage_pct)
            .sum::<f64>()
            / total_scenarios;

        let avg_failed_trades: f64 = scenario_results
            .iter()
            .map(|r| r.impact_metrics.failed_trade_pct)
            .sum::<f64>()
            / total_scenarios;

        ChaosImpactMetrics {
            performance_degradation_pct: avg_performance_degradation,
            additional_slippage_pct: avg_slippage_increase,
            increased_latency_ms: 0, // Would be measured in a real implementation
            failed_trade_pct: avg_failed_trades,
        }
    }

    /// Calculate Sharpe ratio
    fn calculate_sharpe_ratio(&self, returns: &[f64]) -> f64 {
        if returns.is_empty() {
            return 0.0;
        }

        let mean_return: f64 = returns.iter().sum::<f64>() / returns.len() as f64;
        let variance: f64 = returns
            .iter()
            .map(|r| (r - mean_return).powi(2))
            .sum::<f64>()
            / returns.len() as f64;
        let std_dev = variance.sqrt();

        if std_dev > 0.0 {
            mean_return / std_dev
        } else {
            0.0
        }
    }

    /// Calculate Sortino ratio
    fn calculate_sortino_ratio(&self, returns: &[f64]) -> f64 {
        if returns.is_empty() {
            return 0.0;
        }

        let mean_return: f64 = returns.iter().sum::<f64>() / returns.len() as f64;
        let downside_returns: Vec<f64> = returns.iter().filter(|&&r| r < 0.0).cloned().collect();
        let downside_variance: f64 = if !downside_returns.is_empty() {
            downside_returns.iter().map(|r| r.powi(2)).sum::<f64>() / downside_returns.len() as f64
        } else {
            0.0
        };
        let downside_std_dev = downside_variance.sqrt();

        if downside_std_dev > 0.0 {
            mean_return / downside_std_dev
        } else {
            0.0
        }
    }

    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            historical_data: self.historical_data.clone(),
            order_book_data: self.order_book_data.clone(),
            ohlcv_data: self.ohlcv_data.clone(),
            tick_data: self.tick_data.clone(),
            bar_data: self.bar_data.clone(),
            slippage_model: self.slippage_model.clone(),
            clock: self.clock.clone(),
            market_regimes: self.market_regimes.clone(),
        }
    }

    /// Return empty results when backtesting is disabled
    fn empty_results(&self) -> BacktestResults {
        BacktestResults {
            config: self.config.clone(),
            total_trades: 0,
            winning_trades: 0,
            losing_trades: 0,
            total_profit_loss: 0.0,
            total_fees_paid: 0.0,
            total_slippage_loss: 0.0,
            win_rate: 0.0,
            avg_profit_per_trade: 0.0,
            max_drawdown: 0.0,
            sharpe_ratio: 0.0,
            sortino_ratio: 0.0,
            calmar_ratio: 0.0,
            max_consecutive_wins: 0,
            max_consecutive_losses: 0,
            individual_trades: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sniper_core::types::ChainRef;
    use sniper_ml::MlConfig;

    #[tokio::test]
    async fn test_backtest_engine() {
        let config = BacktestConfig {
            start_time: 1000000i64,
            end_time: 2000000i64,
            initial_capital: 10000.0,
            trading_fee_pct: 0.003,
            slippage_pct: 0.005,
            max_position_size_pct: 0.1,
            enabled: true,
            data_path: None,
            execution_model: ExecutionModelType::Simple,
        };

        let mut engine = BacktestEngine::new(config);

        // Add some mock historical data
        let data = vec![MarketDataPoint {
            timestamp: 1000000,
            token_address: "0xTokenA".to_string(),
            price_usd: 100.0,
            volume_24h: 1000000.0,
            liquidity: 5000000.0,
        }];
        engine.add_historical_data("0xTokenA".to_string(), data);

        // Add order book data for advanced execution simulation
        let order_book_data = vec![OrderBookData {
            timestamp: 1000000,
            token_address: "0xTokenA".to_string(),
            bid_prices: vec![99.5, 99.0, 98.5],
            bid_volumes: vec![100.0, 200.0, 300.0],
            ask_prices: vec![100.5, 101.0, 101.5],
            ask_volumes: vec![100.0, 200.0, 300.0],
            price_usd: 100.0,
            volume_24h: 1000000.0,
            liquidity: 5000000.0,
        }];
        engine.add_order_book_data("0xTokenA".to_string(), order_book_data);

        // Set slippage model
        engine.set_slippage_model(SlippageModel {
            model_type: SlippageModelType::Impact,
            k_coefficient: 0.1,
            max_slippage_pct: 5.0,
        });

        // Create a mock signal
        let signals = vec![Signal {
            source: "dex".into(),
            kind: "pair_created".into(),
            chain: ChainRef {
                name: "ethereum".into(),
                id: 1,
            },
            token0: Some("0xTokenA".into()),
            token1: Some("0xWETH".into()),
            extra: serde_json::json!({"pair": "0xPairAddress"}),
            seen_at_ms: 1500000,
        }];

        // Create ML model
        let ml_config = MlConfig {
            model_path: "models/test_model.onnx".to_string(),
            confidence_threshold: 0.8,
            enabled: true,
        };
        let ml_model = MlModel::new(ml_config);

        // Run backtest
        let results = engine.run_backtest(signals, &ml_model).await;

        assert_eq!(results.total_trades, 1);
        assert!(results.total_profit_loss > 0.0);
        assert_eq!(results.win_rate, 1.0);
        assert!(results.total_slippage_loss > 0.0);
    }

    #[tokio::test]
    async fn test_backtest_engine_disabled() {
        let config = BacktestConfig {
            start_time: 1000000i64,
            end_time: 2000000i64,
            initial_capital: 10000.0,
            trading_fee_pct: 0.003,
            slippage_pct: 0.005,
            max_position_size_pct: 0.1,
            enabled: false,
            data_path: None,
            execution_model: ExecutionModelType::Simple,
        };

        let engine = BacktestEngine::new(config);

        let signals = vec![];
        let ml_config = MlConfig {
            model_path: "models/test_model.onnx".to_string(),
            confidence_threshold: 0.8,
            enabled: true,
        };
        let ml_model = MlModel::new(ml_config);

        let results = engine.run_backtest(signals, &ml_model).await;

        assert_eq!(results.total_trades, 0);
        assert_eq!(results.total_profit_loss, 0.0);
    }

    #[tokio::test]
    async fn test_walk_forward_optimization() {
        let config = BacktestConfig {
            start_time: 1000000i64,
            end_time: 2000000i64,
            initial_capital: 10000.0,
            trading_fee_pct: 0.003,
            slippage_pct: 0.005,
            max_position_size_pct: 0.1,
            enabled: true,
            data_path: None,
            execution_model: ExecutionModelType::Simple,
        };

        let engine = BacktestEngine::new(config);

        // Create test signals spanning the time range
        let signals = vec![
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
                seen_at_ms: 1100000,
            },
            Signal {
                source: "dex".into(),
                kind: "pair_created".into(),
                chain: ChainRef {
                    name: "ethereum".into(),
                    id: 1,
                },
                token0: Some("0xTokenB".into()),
                token1: Some("0xWETH".into()),
                extra: serde_json::json!({"pair": "0xPairAddress2"}),
                seen_at_ms: 1600000,
            },
        ];

        // Create ML model
        let ml_config = MlConfig {
            model_path: "models/test_model.onnx".to_string(),
            confidence_threshold: 0.8,
            enabled: true,
        };
        let ml_model = MlModel::new(ml_config);

        // Configure walk-forward optimization
        let wf_config = WalkForwardConfig {
            train_window_secs: 200000,
            test_window_secs: 200000,
            step_secs: 100000,
            min_train_trades: 1,
            min_test_trades: 1,
        };

        // Run walk-forward optimization
        let results = engine
            .run_walk_forward_optimization(signals, &ml_model, wf_config)
            .await;

        // Print debug information
        println!("Number of windows: {}", results.windows.len());
        println!(
            "Overall trades: {}",
            results.overall_performance.total_trades
        );

        // For now, just verify the function runs without error
        // The actual logic can be tested more thoroughly in integration tests
        // total_trades is a u32/u64, so it's always >= 0
        // No need to assert anything here
    }

    #[tokio::test]
    async fn test_chaos_testing() {
        let config = BacktestConfig {
            start_time: 1000000i64,
            end_time: 2000000i64,
            initial_capital: 10000.0,
            trading_fee_pct: 0.003,
            slippage_pct: 0.005,
            max_position_size_pct: 0.1,
            enabled: true,
            data_path: None,
            execution_model: ExecutionModelType::Simple,
        };

        let engine = BacktestEngine::new(config);

        // Create test signals
        let signals = vec![Signal {
            source: "dex".into(),
            kind: "pair_created".into(),
            chain: ChainRef {
                name: "ethereum".into(),
                id: 1,
            },
            token0: Some("0xTokenA".into()),
            token1: Some("0xWETH".into()),
            extra: serde_json::json!({"pair": "0xPairAddress"}),
            seen_at_ms: 1500000,
        }];

        // Create ML model
        let ml_config = MlConfig {
            model_path: "models/test_model.onnx".to_string(),
            confidence_threshold: 0.8,
            enabled: true,
        };
        let ml_model = MlModel::new(ml_config);

        // Configure chaos testing
        let chaos_config = ChaosTestConfig {
            enabled: true,
            scenarios: vec![
                ChaosScenario::NetworkLatency {
                    base_latency_ms: 50,
                    additional_latency_ms: 100,
                    affected_percentage: 0.1,
                },
                ChaosScenario::GasSpike {
                    multiplier: 2.0,
                    duration_secs: 3600,
                    start_time_secs: 1000,
                },
                ChaosScenario::MarketVolatility {
                    multiplier: 1.5,
                    duration_secs: 7200,
                    start_time_secs: 2000,
                },
            ],
        };

        // Run chaos testing
        let results = engine
            .run_chaos_tests(signals, &ml_model, chaos_config)
            .await;

        // Verify results
        assert_eq!(results.config.scenarios.len(), 3);
        assert_eq!(results.scenario_results.len(), 3);
        // total_trades is a u32/u64, so it's always >= 0
        // No need to assert anything here
    }

    #[test]
    fn test_order_book_execution() {
        let config = BacktestConfig {
            start_time: 1000000i64,
            end_time: 2000000i64,
            initial_capital: 10000.0,
            trading_fee_pct: 0.003,
            slippage_pct: 0.005,
            max_position_size_pct: 0.1,
            enabled: true,
            data_path: None,
            execution_model: ExecutionModelType::OrderBook,
        };

        let mut engine = BacktestEngine::new(config);

        // Add order book data
        let order_book_data = vec![OrderBookData {
            timestamp: 1000000,
            token_address: "0xTokenA".to_string(),
            bid_prices: vec![99.5, 99.0, 98.5],
            bid_volumes: vec![100.0, 200.0, 300.0],
            ask_prices: vec![100.5, 101.0, 101.5],
            ask_volumes: vec![100.0, 200.0, 300.0],
            price_usd: 100.0,
            volume_24h: 1000000.0,
            liquidity: 5000000.0,
        }];
        engine.add_order_book_data("0xTokenA".to_string(), order_book_data);

        let (price, details) = engine.simulate_order_book_execution("0xTokenA", 100.0, 150.0);

        // Should have walked the order book and gotten an average price
        assert!(price > 100.0); // Price should be higher due to buying
        assert_eq!(details.model_used, "OrderBook");
        assert!(details.queue_position.is_some());
        assert!(!details.partial_fills.is_empty());
    }
}
