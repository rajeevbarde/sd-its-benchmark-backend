use axum::{
    routing::get,
    Router,
};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::signal;
use tracing::{info, warn};
use dotenvy::dotenv;

use sd_its_benchmark::{create_pool, DatabaseConfig, initialize_database};

#[derive(Clone)]
struct AppState {
    db_pool: sqlx::SqlitePool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables from .env file
    dotenv().ok();
    
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    info!("SD-ITS Benchmark Backend Starting...");
    
    // Load configuration
    let config = DatabaseConfig::default();
    info!("Database URL: {}", config.url);
    
    // Create database pool
    let db_pool = create_pool(&config).await?;
    info!("Database pool created successfully");
    
    // Initialize database tables
    initialize_database(&db_pool).await?;
    info!("Database tables initialized");
    
    // Health check
    sd_its_benchmark::health_check(&db_pool).await?;
    info!("Database health check passed");
    
    // Create application state
    let state = Arc::new(AppState { db_pool });
    
    // Build application router
    let app = Router::new()
        .route("/health", get(health_handler))
        .with_state(state);
    
    // Get server configuration
    let host = std::env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = std::env::var("SERVER_PORT")
        .unwrap_or_else(|_| "3002".to_string())
        .parse::<u16>()?;
    
    let addr = SocketAddr::from((host.parse::<std::net::IpAddr>()?, port));
    info!("Server starting on {}", addr);
    
    // Start server with graceful shutdown
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    
    info!("Server shutdown complete");
    Ok(())
}

async fn health_handler() -> &'static str {
    "OK"
}


async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            warn!("Ctrl+C received, starting graceful shutdown");
        },
        _ = terminate => {
            warn!("SIGTERM received, starting graceful shutdown");
        },
    }
}
