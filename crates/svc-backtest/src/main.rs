use axum::{
    routing::{get, post},
    Json, Router, extract::State,
};
use serde::{Deserialize, Serialize};
use sniper_core::{bus::InMemoryBus, prelude::*, types::Signal};
use sniper_backtest::{BacktestEngine, BacktestConfig, BacktestResults, WalkForwardConfig, WalkForwardResults};
use sniper_ml::{MlConfig, MlModel};
use sniper_telemetry::{TelemetrySystem, TelemetryConfig};
use std::sync::Arc;
use tokio::net::TcpListener;

#[derive(Clone)]
struct AppState {
    backtest_engine: Arc<BacktestEngine>,
    ml_model: Arc<MlModel>,
    bus: InMemoryBus,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .json()
        .init();
    dotenvy::dotenv().ok();

    // Initialize telemetry system
    let telemetry_config = TelemetryConfig {
        metrics_enabled: true,
        tracing_enabled: true,
        alerting_enabled: true,
    };
    let telemetry = TelemetrySystem::new(telemetry_config)?;
    
    // Initialize ML model
    let ml_config = MlConfig {
        model_path: "models/trading_model.onnx".to_string(),
        confidence_threshold: 0.8,
        enabled: true,
    };
    let ml_model = Arc::new(MlModel::new(ml_config));
    
    // Initialize backtest engine
    let backtest_config = BacktestConfig {
        start_time: 1000000,
        end_time: 2000000,
        initial_capital: 10000.0,
        trading_fee_pct: 0.003,
        slippage_pct: 0.005,
        max_position_size_pct: 0.1,
        enabled: true,
        data_path: Some("data/historical".to_string()),
        execution_model: sniper_backtest::ExecutionModelType::Simple,
    };
    let backtest_engine = Arc::new(BacktestEngine::new(backtest_config));
    
    // Initialize message bus
    let bus = InMemoryBus::new(1024);
    
    // Set up application state
    let state = AppState {
        backtest_engine,
        ml_model,
        bus,
    };
    
    // Set up HTTP routes
    let app = Router::new()
        .route("/", get(root))
        .route("/backtest/run", post(run_backtest))
        .route("/backtest/walk-forward", post(run_walk_forward))
        .route("/backtest/config", get(get_config).put(update_config))
        .with_state(state);

    // Run HTTP server
    let listener = TcpListener::bind("0.0.0.0:3004").await?;
    tracing::info!("Backtest service listening on 0.0.0.0:3004");
    
    axum::serve(listener, app).await?;
    
    Ok(())
}

async fn root() -> &'static str {
    "Backtest Service"
}

#[derive(Serialize, Deserialize)]
struct BacktestRequest {
    signals: Vec<Signal>,
}

async fn run_backtest(State(state): State<AppState>, Json(payload): Json<BacktestRequest>) -> Json<BacktestResults> {
    let results = state.backtest_engine.run_backtest(payload.signals, &state.ml_model).await;
    Json(results)
}

#[derive(Serialize, Deserialize)]
struct WalkForwardRequest {
    signals: Vec<Signal>,
    config: WalkForwardConfig,
}

async fn run_walk_forward(State(state): State<AppState>, Json(payload): Json<WalkForwardRequest>) -> Json<WalkForwardResults> {
    let results = state.backtest_engine.run_walk_forward_optimization(payload.signals, &state.ml_model, payload.config).await;
    Json(results)
}

async fn get_config(State(state): State<AppState>) -> Json<BacktestConfig> {
    Json(state.backtest_engine.config().clone())
}

async fn update_config(State(state): State<AppState>, Json(payload): Json<BacktestConfig>) -> &'static str {
    // In a real implementation, this would update the backtest engine configuration
    // For now, we'll just log the update
    tracing::info!("Backtest config updated: {:?}", payload);
    "Configuration updated"
}