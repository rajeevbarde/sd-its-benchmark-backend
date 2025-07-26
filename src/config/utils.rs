use crate::config::Settings;
use std::path::PathBuf;
use std::fs;
use tracing::{info, warn};

/// Initialize configuration directories and files
pub fn initialize_config_directories(settings: &Settings) -> Result<(), std::io::Error> {
    // Create logs directory
    if let Some(log_path) = &settings.logging.file_path {
        if let Some(parent) = log_path.parent() {
            fs::create_dir_all(parent)?;
            info!("Created logs directory: {:?}", parent);
        }
    }

    // Create upload directory
    fs::create_dir_all(&settings.application.upload_dir)?;
    info!("Created upload directory: {:?}", settings.application.upload_dir);

    Ok(())
}

/// Validate configuration settings
pub fn validate_config(settings: &Settings) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();

    // Validate server configuration
    if settings.server.port == 0 {
        errors.push("Server port cannot be 0".to_string());
    }

    if settings.server.workers == 0 {
        errors.push("Server workers cannot be 0".to_string());
    }

    // Validate database configuration
    if settings.database.max_connections == 0 {
        errors.push("Database max_connections cannot be 0".to_string());
    }

    if settings.database.min_connections > settings.database.max_connections {
        errors.push("Database min_connections cannot be greater than max_connections".to_string());
    }

    // Validate logging configuration
    if settings.logging.max_file_size == 0 {
        errors.push("Logging max_file_size cannot be 0".to_string());
    }

    if settings.logging.max_files == 0 {
        errors.push("Logging max_files cannot be 0".to_string());
    }

    // Validate application configuration
    if settings.application.max_upload_size == 0 {
        errors.push("Application max_upload_size cannot be 0".to_string());
    }

    if settings.application.allowed_file_types.is_empty() {
        errors.push("Application allowed_file_types cannot be empty".to_string());
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Get configuration summary for logging
pub fn get_config_summary(settings: &Settings) -> String {
    format!(
        "Configuration Summary:\n\
         Environment: {}\n\
         Server: {}:{}\n\
         Database: {}\n\
         Logging: {} ({})\n\
         Upload Dir: {:?}\n\
         Max Upload Size: {} MB",
        settings.application.environment,
        settings.server.host,
        settings.server.port,
        settings.database.url,
        settings.logging.level,
        settings.logging.format,
        settings.application.upload_dir.display(),
        settings.application.max_upload_size / 1024 / 1024
    )
}

/// Check if configuration files exist
pub fn check_config_files() -> Result<(), Vec<String>> {
    let mut missing_files = Vec::new();
    let config_files = [
        "config/default.toml",
        "config/development.toml",
        "config/staging.toml",
        "config/production.toml",
    ];

    for file in &config_files {
        if !PathBuf::from(file).exists() {
            missing_files.push(file.to_string());
        }
    }

    if missing_files.is_empty() {
        info!("All configuration files found");
        Ok(())
    } else {
        warn!("Missing configuration files: {:?}", missing_files);
        Err(missing_files)
    }
}

/// Load configuration with fallback to defaults
pub fn load_config_with_fallback() -> Result<Settings, Box<dyn std::error::Error>> {
    match Settings::new() {
        Ok(settings) => {
            info!("Configuration loaded successfully");
            Ok(settings)
        }
        Err(e) => {
            warn!("Failed to load configuration from files: {}", e);
            warn!("Using default configuration");
            Ok(Settings::default())
        }
    }
}

/// Get environment-specific database URL
pub fn get_database_url(environment: &str) -> String {
    match environment {
        "development" => "sqlite:./dev-database.db".to_string(),
        "staging" => "sqlite:./staging-database.db".to_string(),
        "production" => "sqlite:./production-database.db".to_string(),
        _ => "sqlite:./my-database.db".to_string(),
    }
}

/// Get environment-specific log file path
pub fn get_log_file_path(environment: &str) -> PathBuf {
    match environment {
        "development" => PathBuf::from("logs/dev.log"),
        "staging" => PathBuf::from("logs/staging.log"),
        "production" => PathBuf::from("logs/production.log"),
        _ => PathBuf::from("logs/app.log"),
    }
}

 