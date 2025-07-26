use sqlx::{SqlitePool, Error};

/// Get a connection from the pool (for explicit connection management, if needed)
pub async fn get_connection(pool: &SqlitePool) -> Result<sqlx::pool::PoolConnection<sqlx::Sqlite>, Error> {
    pool.acquire().await
}

/// Perform a health check on the database connection pool
pub async fn health_check(pool: &SqlitePool) -> Result<(), Error> {
    sqlx::query("SELECT 1;").execute(pool).await?;
    Ok(())
} 