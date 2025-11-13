use axum::{
    routing::{get, post},
    Json, Router, extract::State,
};
use sniper_core::bus::InMemoryBus;
use sniper_analytics::{AnalyticsSystem, AnalyticsConfig, TradeAnalytics, PortfolioAnalytics};
use sniper_telemetry::{TelemetrySystem, TelemetryConfig, alerts::{AlertManagerConfig, SlackConfig, WebhookConfig}};
use std::sync::Arc;
use tokio::net::TcpListener;
use anyhow::Result;

#[derive(Clone)]
struct AppState {
    analytics: Arc<AnalyticsSystem>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .json()
        .init();
    dotenvy::dotenv().ok();

    // Initialize telemetry system with alerting configuration
    let alert_manager_config = AlertManagerConfig {
        slack_config: Some(SlackConfig {
            webhook_url: std::env::var("SLACK_WEBHOOK_URL").unwrap_or_else(|_| "https://hooks.slack.com/services/YOUR/SLACK/WEBHOOK".to_string()),
            channel: Some("#alerts".to_string()),
            username: Some("SnippingBot-Analytics".to_string()),
            enabled: true,
        }),
        webhook_config: Some(WebhookConfig {
            url: std::env::var("ALERT_WEBHOOK_URL").unwrap_or_else(|_| "http://localhost:8080/alerts".to_string()),
            method: "POST".to_string(),
            headers: None,
            auth_token: std::env::var("WEBHOOK_AUTH_TOKEN").ok(),
            auth_header: Some("Authorization".to_string()),
            enabled: true,
            timeout_seconds: 30,
        }),
        enabled: true,
    };

    let telemetry_config = TelemetryConfig {
        metrics_enabled: true,
        tracing_enabled: true,
        alerting_enabled: true,
        alert_manager_config: Some(alert_manager_config),
    };
    let telemetry = TelemetrySystem::new(telemetry_config)?;
    
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
        analytics,
    };
    
    // Set up HTTP routes
    let app = Router::new()
        .route("/", get(root))
        .route("/analytics/trades", get(get_recent_trades))
        .route("/analytics/portfolio", get(get_portfolio))
        .route("/analytics/performance", get(get_performance_metrics))
        .route("/analytics/record/trade", post(record_trade))
        .route("/analytics/record/portfolio", post(record_portfolio))
        .with_state(state);

    // Run HTTP server
    let listener = TcpListener::bind("0.0.0.0:3003").await?;
    tracing::info!("Analytics service listening on 0.0.0.0:3003");
    
    axum::serve(listener, app).await?;
    
    Ok(())
}

async fn root() -> &'static str {
    "Analytics Service"
}

async fn get_recent_trades(State(state): State<AppState>) -> Json<Vec<TradeAnalytics>> {
    let trades = state.analytics.get_recent_trades(50).await;
    Json(trades)
}

async fn get_portfolio(State(state): State<AppState>) -> Json<Option<PortfolioAnalytics>> {
    let portfolio = state.analytics.get_portfolio().await;
    Json(portfolio)
}

async fn get_performance_metrics(State(state): State<AppState>) -> Json<serde_json::Value> {
    let metrics = state.analytics.calculate_performance_metrics().await;
    Json(serde_json::to_value(metrics).unwrap())
}

async fn record_trade() -> &'static str {
    // In a real implementation, this would record trade data
    "Trade recorded"
}

async fn record_portfolio() -> &'static str {
    // In a real implementation, this would record portfolio data
    "Portfolio recorded"
}