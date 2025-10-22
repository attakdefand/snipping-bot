use axum::{
    routing::{get},
    Router,
};
use sniper_telemetry::{TelemetrySystem, TelemetryConfig};
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::{services::ServeDir, cors::CorsLayer};
use anyhow::Result;
use serde::Deserialize;
use std::fs;
use std::path::Path;

/// Server configuration
#[derive(Debug, Deserialize)]
struct ServerConfig {
    host: String,
    port: u16,
    workers: usize,
}

/// Assets configuration
#[derive(Debug, Deserialize)]
struct AssetsConfig {
    static_path: String,
    wasm_path: String,
}

/// Logging configuration
#[derive(Debug, Deserialize)]
struct LoggingConfig {
    level: String,
    format: String,
    file: String,
}

/// CORS configuration
#[derive(Debug, Deserialize)]
struct CorsConfig {
    enabled: bool,
    allowed_origins: Vec<String>,
    allowed_methods: Vec<String>,
    allowed_headers: Vec<String>,
}

/// Service configuration
#[derive(Debug, Deserialize)]
struct Config {
    server: ServerConfig,
    assets: AssetsConfig,
    logging: LoggingConfig,
    cors: CorsConfig,
}

/// Load configuration from TOML file
fn load_config() -> Result<Config> {
    let config_str = fs::read_to_string("crates/svc-wasm-dashboard/config/service.toml")?;
    let config: Config = toml::from_str(&config_str)?;
    Ok(config)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load configuration
    let config = load_config()?;
    
    tracing_subscriber::fmt()
        .with_env_filter(&config.logging.level)
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
    
    // Construct the static path relative to the workspace root
    let static_path = Path::new("crates/svc-wasm-dashboard").join(&config.assets.static_path);
    
    // Check if the static path exists
    if !static_path.exists() {
        tracing::error!("Static path does not exist: {:?}", static_path);
        return Err(anyhow::anyhow!("Static path does not exist: {:?}", static_path));
    }
    
    tracing::info!("Serving static files from: {:?}", static_path.canonicalize()?);
    
    // Serve static files (for the WASM frontend)
    let serve_dir = ServeDir::new(&static_path)
        .append_index_html_on_directories(true);
    
    // Set up HTTP routes with CORS for WASM
    let app = Router::new()
        .route("/", get(root))
        .route("/dashboard", get(dashboard))
        .nest_service("/dashboard/", serve_dir.clone())
        .layer(CorsLayer::permissive()); // Allow all CORS for development

    // Run HTTP server
    let addr = format!("{}:{}", config.server.host, config.server.port);
    let listener = TcpListener::bind(&addr).await?;
    tracing::info!("WASM Dashboard service listening on {}", addr);
    
    axum::serve(listener, app).await?;
    
    Ok(())
}

async fn root() -> &'static str {
    "Sniper Bot WASM Dashboard Service - Visit /dashboard for the WASM UI"
}

async fn dashboard() -> &'static str {
    "Sniper Bot WASM Dashboard - Redirecting to index.html"
}