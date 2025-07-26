// Configuration management module
pub mod settings;
pub mod database;
pub mod utils;

pub use settings::{
    Settings,
    ServerConfig,
    DatabaseSettings,
    LoggingConfig,
    ApplicationConfig,
    Environment,
    LogFormat,
    LogOutput,
};

pub use utils::{
    initialize_config_directories,
    validate_config,
    load_config_with_fallback,
    get_config_summary,
    get_database_url,
    get_log_file_path,
};
