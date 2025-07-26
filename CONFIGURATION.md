# Configuration Guide

This document describes the configuration system for the SD-ITS-Benchmark application.

## Overview

The application uses a hierarchical configuration system that supports:
- Environment-based configuration files
- Environment variables
- Default values
- Local development overrides

## Configuration Hierarchy

Configuration is loaded in the following order (later values override earlier ones):

1. **Default Configuration** (`config/default.toml`)
2. **Environment Configuration** (`config/{environment}.toml`)
3. **Local Configuration** (`config/local.toml`) - Optional, ignored by git
4. **Environment Variables** (with `APP__` prefix)

## Environment Variables

Set the `RUST_ENV` environment variable to specify which environment configuration to load:
- `development` - Development environment
- `staging` - Staging environment  
- `production` - Production environment

## Configuration Files

### Default Configuration (`config/default.toml`)
Base configuration that applies to all environments.

### Environment-Specific Configurations
- `config/development.toml` - Development environment settings
- `config/staging.toml` - Staging environment settings
- `config/production.toml` - Production environment settings

### Local Configuration (`config/local.toml`)
Optional local development overrides. This file is ignored by git and allows developers to customize settings for their local environment.

## Configuration Sections

### Server Configuration
```toml
[server]
host = "127.0.0.1"           # Server host
port = 3000                  # Server port
workers = 4                  # Number of worker threads
request_timeout = 30         # Request timeout in seconds
max_request_size = 10485760  # Max request size in bytes (10MB)
cors_origins = ["http://localhost:3000"]  # Allowed CORS origins
```

### Database Configuration
```toml
[database]
url = "sqlite:./my-database.db"  # Database connection URL
max_connections = 10             # Maximum database connections
min_connections = 1              # Minimum database connections
idle_timeout = 600               # Connection idle timeout in seconds
max_lifetime = 1800              # Connection max lifetime in seconds
connection_timeout = 30          # Connection timeout in seconds
```

### Logging Configuration
```toml
[logging]
level = "info"                    # Log level (debug, info, warn, error)
format = "text"                   # Log format (text, json)
output = "console"                # Output destination (console, file, both)
file_path = "logs/app.log"        # Log file path (for file output)
max_file_size = 10485760          # Max log file size in bytes (10MB)
max_files = 5                     # Maximum number of log files to keep
```

### Application Configuration
```toml
[application]
name = "SD-ITS-Benchmark"         # Application name
version = "0.1.0"                 # Application version
environment = "development"        # Environment (development, staging, production)
upload_dir = "uploads"            # Upload directory path
max_upload_size = 52428800        # Max upload size in bytes (50MB)
allowed_file_types = ["json", "txt", "csv"]  # Allowed file types
```

## Environment Variables

You can override any configuration setting using environment variables with the `APP__` prefix. The double underscore (`__`) is used as a separator for nested configuration keys.

### Examples

```bash
# Override server port
export APP__SERVER__PORT=8080

# Override database URL
export APP__DATABASE__URL="sqlite:./production.db"

# Override logging level
export APP__LOGGING__LEVEL=debug

# Override application environment
export APP__APPLICATION__ENVIRONMENT=production
```

### Environment Variable Mapping

| Configuration Path | Environment Variable |
|-------------------|---------------------|
| `server.host` | `APP__SERVER__HOST` |
| `server.port` | `APP__SERVER__PORT` |
| `database.url` | `APP__DATABASE__URL` |
| `logging.level` | `APP__LOGGING__LEVEL` |
| `application.environment` | `APP__APPLICATION__ENVIRONMENT` |

## Usage in Code

### Loading Configuration
```rust
use crate::config::{Settings, load_config_with_fallback};

// Load configuration with fallback to defaults
let settings = load_config_with_fallback()?;

// Or load with error handling
let settings = Settings::new()?;
```

### Accessing Configuration
```rust
// Server configuration
let port = settings.server.port;
let host = &settings.server.host;

// Database configuration
let db_url = &settings.database.url;
let max_connections = settings.database.max_connections;

// Logging configuration
let log_level = &settings.logging.level;
let log_format = &settings.logging.format;

// Application configuration
let env = &settings.application.environment;
let upload_dir = &settings.application.upload_dir;
```

### Environment Checks
```rust
if settings.is_development() {
    // Development-specific code
}

if settings.is_production() {
    // Production-specific code
}

if settings.is_staging() {
    // Staging-specific code
}
```

## Configuration Validation

The application validates configuration settings on startup:

```rust
use crate::config::{validate_config, get_config_summary};

// Validate configuration
if let Err(errors) = validate_config(&settings) {
    eprintln!("Configuration errors: {:?}", errors);
    std::process::exit(1);
}

// Print configuration summary
println!("{}", get_config_summary(&settings));
```

## Directory Structure

```
config/
├── default.toml           # Default configuration
├── development.toml       # Development environment
├── staging.toml          # Staging environment
├── production.toml       # Production environment
└── local.toml.example    # Example local configuration
```

## Best Practices

1. **Never commit sensitive data** to configuration files
2. **Use environment variables** for secrets and environment-specific values
3. **Keep local.toml** for development-specific overrides
4. **Validate configuration** on application startup
5. **Use appropriate log levels** for each environment
6. **Set reasonable timeouts** and connection limits
7. **Configure CORS origins** properly for production

## Troubleshooting

### Common Issues

1. **Configuration not loading**: Check that configuration files exist and are valid TOML
2. **Environment variables not working**: Ensure they use the `APP__` prefix and double underscore separator
3. **Permission errors**: Check that the application can read configuration files and write to log/upload directories
4. **Database connection issues**: Verify database URL and connection settings

### Debug Configuration

```rust
use crate::config::check_config_files;

// Check if configuration files exist
if let Err(missing_files) = check_config_files() {
    eprintln!("Missing configuration files: {:?}", missing_files);
}
```

### Environment Variable Debugging

```bash
# Print all environment variables with APP__ prefix
env | grep APP__

# Test configuration loading
RUST_LOG=debug cargo run
``` 