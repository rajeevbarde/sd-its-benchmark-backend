pub mod config;
pub mod models;
pub mod error;
pub mod handlers;
pub mod repositories;
pub mod services;
pub mod middleware;

pub use config::{
    create_pool, 
    health_check, 
    DatabaseConfig, 
    initialize_database,
    get_all_runs,
    get_performance_results,
    get_gpu_info,
};

pub use error::{AppError, AppResult, log_error, handle_anyhow_error};
