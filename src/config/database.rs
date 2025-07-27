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
    // Create runs table
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

    // Create performanceResult table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS performanceResult (
            id INTEGER PRIMARY KEY,
            run_id INTEGER,
            its TEXT,
            avg_its REAL,
            FOREIGN KEY (run_id) REFERENCES runs(id)
        )
        "#
    ).execute(pool).await?;

    // Create AppDetails table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS AppDetails (
            id INTEGER PRIMARY KEY,
            run_id INTEGER,
            app_name TEXT,
            updated TEXT,
            hash TEXT,
            url TEXT,
            FOREIGN KEY (run_id) REFERENCES runs(id)
        )
        "#
    ).execute(pool).await?;

    // Create SystemInfo table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS SystemInfo (
            id INTEGER PRIMARY KEY,
            run_id INTEGER,
            arch TEXT,
            cpu TEXT,
            system TEXT,
            release TEXT,
            python TEXT,
            FOREIGN KEY (run_id) REFERENCES runs(id)
        )
        "#
    ).execute(pool).await?;

    // Create Libraries table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS Libraries (
            id INTEGER PRIMARY KEY,
            run_id INTEGER,
            torch TEXT,
            xformers TEXT,
            xformers1 TEXT,
            diffusers TEXT,
            transformers TEXT,
            FOREIGN KEY (run_id) REFERENCES runs(id)
        )
        "#
    ).execute(pool).await?;

    // Create GPU table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS GPU (
            id INTEGER PRIMARY KEY,
            run_id INTEGER,
            device TEXT,
            driver TEXT,
            gpu_chip TEXT,
            brand TEXT,
            isLaptop BOOLEAN,
            FOREIGN KEY (run_id) REFERENCES runs(id)
        )
        "#
    ).execute(pool).await?;

    // Create RunMoreDetails table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS RunMoreDetails (
            id INTEGER PRIMARY KEY,
            run_id INTEGER,
            timestamp TEXT,
            model_name TEXT,
            user TEXT,
            notes TEXT,
            ModelMapId INTEGER,
            FOREIGN KEY (run_id) REFERENCES runs(id)
        )
        "#
    ).execute(pool).await?;

    // Create ModelMap table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS ModelMap (
            id INTEGER PRIMARY KEY,
            model_name TEXT,
            base_model TEXT
        )
        "#
    ).execute(pool).await?;

    // Create GPUMap table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS GPUMap (
            id INTEGER PRIMARY KEY,
            gpu_name TEXT,
            base_gpu_id INTEGER REFERENCES GPUBase(id)
        )
        "#
    ).execute(pool).await?;

    // Create GPUBase table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS GPUBase (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL UNIQUE,
            brand TEXT
        )
        "#
    ).execute(pool).await?;

    // Create indexes
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_performanceResult_run_id ON performanceResult (run_id)").execute(pool).await?;
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_AppDetails_run_id ON AppDetails (run_id)").execute(pool).await?;
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_SystemInfo_run_id ON SystemInfo (run_id)").execute(pool).await?;
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_Libraries_run_id ON Libraries (run_id)").execute(pool).await?;
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_GPU_run_id ON GPU (run_id)").execute(pool).await?;
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_GPU_device ON GPU (device)").execute(pool).await?;
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_RunMoreDetails_run_id ON RunMoreDetails (run_id)").execute(pool).await?;
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_RunMoreDetails_model_name ON RunMoreDetails (model_name)").execute(pool).await?;
    
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
