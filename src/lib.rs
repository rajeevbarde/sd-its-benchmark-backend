pub mod config;
pub mod models;
pub mod error;
pub mod handlers;
pub mod repositories;
pub mod services;
pub mod middleware;

use sqlx::SqlitePool;

pub use config::{
    Settings,
    load_config_with_fallback,
    validate_config,
    initialize_config_directories,
};

pub use error::{AppError};

#[derive(Clone)]
pub struct AppState {
    pub db: SqlitePool,
    pub settings: Settings,
}
