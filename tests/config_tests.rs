use sd_its_benchmark::config::{Settings, validate_config};
use sd_its_benchmark::config::settings::Environment;
use sd_its_benchmark::config::utils::{get_database_url, get_log_file_path, get_config_summary};
use std::path::PathBuf;

#[test]
fn test_validate_config_valid() {
    let settings = Settings::default();
    assert!(validate_config(&settings).is_ok());
}

#[test]
fn test_validate_config_invalid_port() {
    let mut settings = Settings::default();
    settings.server.port = 0;
    let result = validate_config(&settings);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| e.contains("port")));
}

#[test]
fn test_get_database_url() {
    assert_eq!(get_database_url("development"), "sqlite:./dev-database.db");
    assert_eq!(get_database_url("staging"), "sqlite:./staging-database.db");
    assert_eq!(get_database_url("production"), "sqlite:./production-database.db");
    assert_eq!(get_database_url("unknown"), "sqlite:./my-database.db");
}

#[test]
fn test_get_log_file_path() {
    assert_eq!(get_log_file_path("development"), PathBuf::from("logs/dev.log"));
    assert_eq!(get_log_file_path("staging"), PathBuf::from("logs/staging.log"));
    assert_eq!(get_log_file_path("production"), PathBuf::from("logs/production.log"));
    assert_eq!(get_log_file_path("unknown"), PathBuf::from("logs/app.log"));
}

#[test]
fn test_get_config_summary() {
    let settings = Settings::default();
    let summary = get_config_summary(&settings);
    assert!(summary.contains("Configuration Summary"));
    assert!(summary.contains("Environment: development"));
    assert!(summary.contains("Server: 127.0.0.1:4022")); // Updated to match current default port
}

#[test]
fn test_environment_display() {
    assert_eq!(Environment::Development.to_string(), "development");
    assert_eq!(Environment::Staging.to_string(), "staging");
    assert_eq!(Environment::Production.to_string(), "production");
}

#[test]
fn test_environment_from_str() {
    assert_eq!("development".parse::<Environment>().unwrap(), Environment::Development);
    assert_eq!("dev".parse::<Environment>().unwrap(), Environment::Development);
    assert_eq!("staging".parse::<Environment>().unwrap(), Environment::Staging);
    assert_eq!("stage".parse::<Environment>().unwrap(), Environment::Staging);
    assert_eq!("production".parse::<Environment>().unwrap(), Environment::Production);
    assert_eq!("prod".parse::<Environment>().unwrap(), Environment::Production);
    assert!("unknown".parse::<Environment>().is_err());
} 