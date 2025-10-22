use axum::{
    routing::{get},
    Json, Router, extract::State,
};
use sniper_telemetry::{TelemetrySystem, TelemetryConfig};
use sniper_analytics::{AnalyticsSystem, AnalyticsConfig};
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::services::ServeDir;
use anyhow::Result;

#[derive(Clone)]
struct AppState {
    telemetry: Arc<TelemetrySystem>,
    analytics: Arc<AnalyticsSystem>,
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
    let telemetry = Arc::new(TelemetrySystem::new(telemetry_config)?);
    
    // Initialize analytics system
    let analytics_config = AnalyticsConfig {
        enabled: true,
        collection_interval_secs: 60,
    };
    let analytics = Arc::new(AnalyticsSystem::new(
        analytics_config, 
        Arc::new(telemetry.metrics().unwrap().snapshot())
    ));
    
    // Set up application state
    let state = AppState {
        telemetry,
        analytics,
    };
    
    // Serve static files (for the frontend dashboard)
    let serve_dir = ServeDir::new("static");
    
    // Set up HTTP routes
    let app = Router::new()
        .route("/", get(root))
        .route("/api/metrics", get(get_metrics))
        .route("/api/trades", get(get_recent_trades))
        .route("/api/portfolio", get(get_portfolio))
        .route("/api/performance", get(get_performance_metrics))
        .nest_service("/static", serve_dir)
        .with_state(state);

    // Run HTTP server
    let listener = TcpListener::bind("0.0.0.0:3005").await?;
    tracing::info!("Dashboard service listening on 0.0.0.0:3005");
    
    axum::serve(listener, app).await?;
    
    Ok(())
}

async fn root() -> &'static str {
    "Dashboard Service - Visit /static for the dashboard UI"
}

async fn get_metrics(State(state): State<AppState>) -> Json<serde_json::Value> {
    if let Some(metrics) = state.telemetry.metrics() {
        let snapshot = metrics.snapshot();
        Json(serde_json::to_value(snapshot).unwrap())
    } else {
        Json(serde_json::json!({}))
    }
}

async fn get_recent_trades(State(state): State<AppState>) -> Json<serde_json::Value> {
    let trades = state.analytics.get_recent_trades(50).await;
    Json(serde_json::to_value(trades).unwrap())
}

async fn get_portfolio(State(state): State<AppState>) -> Json<serde_json::Value> {
    let portfolio = state.analytics.get_portfolio().await;
    Json(serde_json::to_value(portfolio).unwrap())
}

async fn get_performance_metrics(State(state): State<AppState>) -> Json<serde_json::Value> {
    let metrics = state.analytics.calculate_performance_metrics().await;
    Json(serde_json::to_value(metrics).unwrap())
}