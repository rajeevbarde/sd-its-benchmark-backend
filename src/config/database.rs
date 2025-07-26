use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
use std::time::Duration;
use std::path::Path;
use std::env;

pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub idle_timeout: Duration,
    pub max_lifetime: Duration,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        let url = env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:./my-database.db".to_string());
        
        // Extract file path from SQLite URL and ensure it's absolute
        if let Some(file_path) = url.strip_prefix("sqlite:") {
            let path = if file_path.starts_with("./") {
                // Convert relative path to absolute
                let current_dir = env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
                current_dir.join(file_path.strip_prefix("./").unwrap_or(file_path))
            } else {
                Path::new(file_path).to_path_buf()
            };
            
            // Create directory if it doesn't exist
            if let Some(parent) = path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            
            // Create empty database file if it doesn't exist
            if !path.exists() {
                let _ = std::fs::File::create(&path);
            }
        }
        
        Self {
            url,
            max_connections: 10,
            min_connections: 1,
            idle_timeout: Duration::from_secs(600),
            max_lifetime: Duration::from_secs(1800),
        }
    }
}

pub async fn create_pool(config: &DatabaseConfig) -> Result<SqlitePool, sqlx::Error> {
    SqlitePoolOptions::new()
        .max_connections(config.max_connections)
        .min_connections(config.min_connections)
        .idle_timeout(config.idle_timeout)
        .max_lifetime(config.max_lifetime)
        .connect(&config.url)
        .await
}

pub async fn health_check(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query("SELECT 1").execute(pool).await?;
    Ok(())
}

pub async fn initialize_database(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    // Create tables if they don't exist
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS runs (
            id INTEGER PRIMARY KEY,
            timestamp TEXT,
            vram_usage TEXT,
            info TEXT,
            system_info TEXT,
            model_info TEXT,
            device_info TEXT,
            xformers TEXT,
            model_name TEXT,
            user TEXT,
            notes TEXT
        )
        "#
    ).execute(pool).await?;
    
    Ok(())
}

// Add some sample queries for sqlx to analyze
pub async fn get_all_runs(pool: &SqlitePool) -> Result<Vec<(i32, String)>, sqlx::Error> {
    sqlx::query_as::<_, (i32, String)>("SELECT id, model_name FROM runs LIMIT 10")
        .fetch_all(pool)
        .await
}

pub async fn get_performance_results(pool: &SqlitePool) -> Result<Vec<(i32, f64)>, sqlx::Error> {
    sqlx::query_as::<_, (i32, f64)>("SELECT run_id, avg_its FROM performanceResult LIMIT 10")
        .fetch_all(pool)
        .await
}

pub async fn get_gpu_info(pool: &SqlitePool) -> Result<Vec<(i32, String, String)>, sqlx::Error> {
    sqlx::query_as::<_, (i32, String, String)>("SELECT id, device, brand FROM GPU LIMIT 10")
        .fetch_all(pool)
        .await
}
