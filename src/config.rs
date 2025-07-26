// Configuration management module
pub mod settings;
pub mod database;

pub use database::{
    create_pool, 
    health_check, 
    DatabaseConfig, 
    initialize_database,
    get_all_runs,
    get_performance_results,
    get_gpu_info,
};
