use sqlx::{SqlitePool, Transaction, Sqlite, Error};
use tracing::{info, error, warn};

use crate::error::types::AppError;

/// Transaction-aware service wrapper for data processing operations
pub struct TransactionService {
    pool: SqlitePool,
}

impl TransactionService {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Execute a function within a transaction with automatic rollback on error
    pub async fn execute_in_transaction<F, T, E>(&self, operation: F) -> Result<T, AppError>
    where
        F: FnOnce(&mut Transaction<'_, Sqlite>) -> E,
        E: std::future::Future<Output = Result<T, Error>>,
    {
        let mut tx = self.pool.begin().await
            .map_err(|e| {
                error!("Failed to begin transaction: {}", e);
                AppError::internal(format!("Failed to begin transaction: {}", e))
            })?;

        let result = operation(&mut tx).await;

        match result {
            Ok(value) => {
                tx.commit().await
                    .map_err(|e| {
                        error!("Failed to commit transaction: {}", e);
                        AppError::internal(format!("Failed to commit transaction: {}", e))
                    })?;
                Ok(value)
            }
            Err(e) => {
                tx.rollback().await
                    .map_err(|rollback_error| {
                        error!("Failed to rollback transaction: {}", rollback_error);
                        warn!("Original error: {}", e);
                    })
                    .ok(); // Ignore rollback errors, focus on original error
                
                Err(AppError::internal(format!("Transaction failed: {}", e)))
            }
        }
    }

    /// Execute a function within a transaction with progress tracking
    pub async fn execute_with_progress<F, T, E>(
        &self, 
        operation: F,
        _total_items: usize,
        progress_callback: Option<Box<dyn Fn(usize, usize) + Send + Sync>>
    ) -> Result<T, AppError>
    where
        F: FnOnce(&mut Transaction<'_, Sqlite>, &dyn Fn(usize, usize)) -> E,
        E: std::future::Future<Output = Result<T, Error>>,
    {
        let mut tx = self.pool.begin().await
            .map_err(|e| {
                error!("Failed to begin transaction: {}", e);
                AppError::internal(format!("Failed to begin transaction: {}", e))
            })?;

        let progress_fn = |current: usize, total: usize| {
            if let Some(ref callback) = progress_callback {
                callback(current, total);
            }
            if current % 100 == 0 || current == total {
                info!("Progress: {}/{} ({}%)", current, total, (current * 100) / total);
            }
        };

        let result = operation(&mut tx, &progress_fn).await;

        match result {
            Ok(value) => {
                tx.commit().await
                    .map_err(|e| {
                        error!("Failed to commit transaction: {}", e);
                        AppError::internal(format!("Failed to commit transaction: {}", e))
                    })?;
                Ok(value)
            }
            Err(e) => {
                tx.rollback().await
                    .map_err(|rollback_error| {
                        error!("Failed to rollback transaction: {}", rollback_error);
                        warn!("Original error: {}", e);
                    })
                    .ok();
                
                Err(AppError::internal(format!("Transaction failed: {}", e)))
            }
        }
    }

    /// Execute bulk operations with batch processing
    pub async fn execute_bulk_operation<F, T, E>(
        &self,
        items: Vec<T>,
        batch_size: usize,
        operation: F,
        progress_callback: Option<Box<dyn Fn(usize, usize) + Send + Sync>>
    ) -> Result<Vec<T>, AppError>
    where
        F: Fn(&mut Transaction<'_, Sqlite>, &[T]) -> E,
        E: std::future::Future<Output = Result<Vec<T>, Error>>,
        T: Clone,
    {
        if items.is_empty() {
            return Ok(vec![]);
        }

        let total_items = items.len();
        let mut results = Vec::with_capacity(total_items);
        let mut processed = 0;

        // Process items in batches
        for (batch_index, chunk) in items.chunks(batch_size).enumerate() {
            let batch_result = self.execute_in_transaction(|tx| {
                operation(tx, chunk)
            }).await?;

            results.extend(batch_result);
            processed += chunk.len();

            // Report progress
            if let Some(ref callback) = progress_callback {
                callback(processed, total_items);
            }

            info!("Processed batch {}/{}: {} items", 
                  batch_index + 1, 
                  (total_items + batch_size - 1) / batch_size, 
                  chunk.len());
        }

        Ok(results)
    }

    /// Get the underlying pool for direct access when needed
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }
} 