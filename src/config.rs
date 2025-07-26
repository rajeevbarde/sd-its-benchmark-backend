// Configuration management module
pub mod settings;
pub mod database;

pub use database::{create_pool, health_check, DatabaseConfig};
