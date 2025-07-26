use config::{Config, ConfigError, Environment as ConfigEnvironment, File};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub server: ServerConfig,
    pub database: DatabaseSettings,
    pub logging: LoggingConfig,
    pub application: ApplicationConfig,
    #[serde(default)]
    pub file_upload: FileUploadConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub workers: usize,
    pub request_timeout: u64,  // Duration in seconds
    pub max_request_size: usize,
    pub cors_origins: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseSettings {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub idle_timeout: u64,  // Duration in seconds
    pub max_lifetime: u64,  // Duration in seconds
    pub connection_timeout: u64,  // Duration in seconds
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub format: LogFormat,
    pub output: LogOutput,
    pub file_path: Option<PathBuf>,
    pub max_file_size: usize,
    pub max_files: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationConfig {
    pub name: String,
    pub version: String,
    pub environment: Environment,
    pub upload_dir: PathBuf,
    pub max_upload_size: usize,
    pub allowed_file_types: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileUploadConfig {
    pub max_size_mb: usize,
    pub allowed_content_types: Vec<String>,
    pub temp_dir: PathBuf,
    pub cleanup_interval_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LogFormat {
    #[serde(rename = "json")]
    Json,
    #[serde(rename = "text")]
    Text,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LogOutput {
    #[serde(rename = "console")]
    Console,
    #[serde(rename = "file")]
    File,
    #[serde(rename = "both")]
    Both,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Environment {
    #[serde(rename = "development")]
    Development,
    #[serde(rename = "staging")]
    Staging,
    #[serde(rename = "production")]
    Production,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let environment = std::env::var("RUST_ENV")
            .unwrap_or_else(|_| "development".into());

        let config = Config::builder()
            // Start with default settings
            .add_source(File::with_name("config/default"))
            // Add environment-specific settings
            .add_source(File::with_name(&format!("config/{}", environment)).required(false))
            // Add local settings (for development)
            .add_source(File::with_name("config/local").required(false))
            // Add environment variables with prefix "APP_"
            .add_source(ConfigEnvironment::with_prefix("APP").separator("__"))
            .build()?;

        config.try_deserialize()
    }

    pub fn is_development(&self) -> bool {
        matches!(self.application.environment, Environment::Development)
    }

    pub fn is_production(&self) -> bool {
        matches!(self.application.environment, Environment::Production)
    }

    pub fn is_staging(&self) -> bool {
        matches!(self.application.environment, Environment::Staging)
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            server: ServerConfig::default(),
            database: DatabaseSettings::default(),
            logging: LoggingConfig::default(),
            application: ApplicationConfig::default(),
            file_upload: FileUploadConfig::default(),
        }
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 4022,
            workers: num_cpus::get(),
            request_timeout: 30,  // 30 seconds
            max_request_size: 10 * 1024 * 1024, // 10MB
            cors_origins: vec!["http://localhost:4022".to_string()],
        }
    }
}

impl Default for DatabaseSettings {
    fn default() -> Self {
        Self {
            url: "sqlite:./my-database.db".to_string(),
            max_connections: 10,
            min_connections: 1,
            idle_timeout: 600,  // 600 seconds (10 minutes)
            max_lifetime: 1800,  // 1800 seconds (30 minutes)
            connection_timeout: 30,  // 30 seconds
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            format: LogFormat::Text,
            output: LogOutput::Console,
            file_path: Some(PathBuf::from("logs/app.log")),
            max_file_size: 10 * 1024 * 1024, // 10MB
            max_files: 5,
        }
    }
}

impl Default for ApplicationConfig {
    fn default() -> Self {
        Self {
            name: "SD-ITS-Benchmark".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            environment: Environment::Development,
            upload_dir: PathBuf::from("uploads"),
            max_upload_size: 50 * 1024 * 1024, // 50MB
            allowed_file_types: vec![
                "json".to_string(),
                "txt".to_string(),
                "csv".to_string(),
            ],
        }
    }
}

impl Default for FileUploadConfig {
    fn default() -> Self {
        Self {
            max_size_mb: 50, // 50MB
            allowed_content_types: vec![
                "application/json".to_string(),
                "text/json".to_string(),
                "text/plain".to_string(),
                "application/octet-stream".to_string(),
            ],
            temp_dir: PathBuf::from("temp"),
            cleanup_interval_seconds: 3600, // 1 hour
        }
    }
}

impl std::fmt::Display for Environment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Environment::Development => write!(f, "development"),
            Environment::Staging => write!(f, "staging"),
            Environment::Production => write!(f, "production"),
        }
    }
}

impl std::fmt::Display for LogFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogFormat::Json => write!(f, "json"),
            LogFormat::Text => write!(f, "text"),
        }
    }
}

impl std::fmt::Display for LogOutput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogOutput::Console => write!(f, "console"),
            LogOutput::File => write!(f, "file"),
            LogOutput::Both => write!(f, "both"),
        }
    }
}

impl std::str::FromStr for Environment {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "development" | "dev" => Ok(Environment::Development),
            "staging" | "stage" => Ok(Environment::Staging),
            "production" | "prod" => Ok(Environment::Production),
            _ => Err(format!("Unknown environment: {}", s)),
        }
    }
}

impl std::str::FromStr for LogFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "json" => Ok(LogFormat::Json),
            "text" => Ok(LogFormat::Text),
            _ => Err(format!("Unknown log format: {}", s)),
        }
    }
}

impl std::str::FromStr for LogOutput {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "console" => Ok(LogOutput::Console),
            "file" => Ok(LogOutput::File),
            "both" => Ok(LogOutput::Both),
            _ => Err(format!("Unknown log output: {}", s)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_environment_display() {
        assert_eq!(Environment::Development.to_string(), "development");
        assert_eq!(Environment::Production.to_string(), "production");
        assert_eq!(Environment::Staging.to_string(), "staging");
    }

    #[test]
    fn test_environment_from_str() {
        assert_eq!("development".parse::<Environment>().unwrap(), Environment::Development);
        assert_eq!("dev".parse::<Environment>().unwrap(), Environment::Development);
        assert_eq!("production".parse::<Environment>().unwrap(), Environment::Production);
        assert_eq!("prod".parse::<Environment>().unwrap(), Environment::Production);
        assert!("unknown".parse::<Environment>().is_err());
    }

    #[test]
    fn test_log_format_from_str() {
        assert_eq!("json".parse::<LogFormat>().unwrap(), LogFormat::Json);
        assert_eq!("text".parse::<LogFormat>().unwrap(), LogFormat::Text);
        assert!("unknown".parse::<LogFormat>().is_err());
    }

    #[test]
    fn test_log_output_from_str() {
        assert_eq!("console".parse::<LogOutput>().unwrap(), LogOutput::Console);
        assert_eq!("file".parse::<LogOutput>().unwrap(), LogOutput::File);
        assert_eq!("both".parse::<LogOutput>().unwrap(), LogOutput::Both);
        assert!("unknown".parse::<LogOutput>().is_err());
    }

    #[test]
    fn test_settings_default() {
        let settings = Settings::default();
        assert_eq!(settings.server.port, 4022);  // Updated to match current default
        assert_eq!(settings.database.max_connections, 10);
        assert_eq!(settings.logging.level, "info");
        assert_eq!(settings.application.name, "SD-ITS-Benchmark");
    }

    #[test]
    fn test_environment_checks() {
        let mut settings = Settings::default();
        
        settings.application.environment = Environment::Development;
        assert!(settings.is_development());
        assert!(!settings.is_production());
        assert!(!settings.is_staging());

        settings.application.environment = Environment::Production;
        assert!(!settings.is_development());
        assert!(settings.is_production());
        assert!(!settings.is_staging());

        settings.application.environment = Environment::Staging;
        assert!(!settings.is_development());
        assert!(!settings.is_production());
        assert!(settings.is_staging());
    }
}
