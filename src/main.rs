use axum::{
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use tracing::{info, error, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod error;
mod handlers;

use crate::config::{
    load_config_with_fallback, 
    validate_config, 
    initialize_config_directories
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Starting SD-ITS-Benchmark application...");

    // Load environment variables from .env file
    dotenvy::dotenv().ok();
    
    // Log the current RUST_ENV value and which config TOML files exist
    let rust_env = std::env::var("RUST_ENV").unwrap_or_else(|_| "development".to_string());
    let config_files = [
        "config/default.toml",
        &format!("config/{}.toml", rust_env),
        "config/local.toml"
    ];
    let used_files: Vec<&str> = config_files.iter()
        .filter(|file| std::path::Path::new(file).exists())
        .map(|s| *s)
        .collect();
    info!("RUST_ENV is set to: {} | Using config files: {}", rust_env, used_files.join(", "));

    // Check if config files exist
    for file in &config_files {
        if std::path::Path::new(file).exists() {
            info!("Found config file: {}", file);
        } else {
            warn!("Missing config file: {}", file);
        }
    }

    // Load and validate configuration
    info!("Loading configuration...");
    let settings = load_config_with_fallback()?;
    info!("Configuration loaded - Port: {}", settings.server.port);
    
    // Validate configuration
    if let Err(errors) = validate_config(&settings) {
        error!("Configuration validation failed:");
        for error in errors {
            error!("  - {}", error);
        }
        std::process::exit(1);
    }

    // Initialize directories
    initialize_config_directories(&settings)?;

    // Bind to address (capture values before moving settings)
    let host = settings.server.host.clone();
    let port = settings.server.port;
    let addr = SocketAddr::from((host.parse::<std::net::IpAddr>()?, port));

    // Create application router
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/env", get(show_environment))
        .route("/api/upload", post(handlers::upload::upload_file))
        .with_state(settings);
    info!("Server starting on {}", addr);

    // Start server
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> &'static str {
    "OK"
}

async fn show_environment() -> String {
    let rust_env = std::env::var("RUST_ENV").unwrap_or_else(|_| "development".to_string());
    format!("Current RUST_ENV: {}", rust_env)
}
