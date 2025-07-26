// Configuration management module
pub mod settings;
pub mod database;
pub mod utils;

pub use settings::Settings;

// Type alias for application configuration used in handlers
pub type AppConfig = Settings;

pub use utils::{
    initialize_config_directories,
    validate_config,
    load_config_with_fallback,
};
