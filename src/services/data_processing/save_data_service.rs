use serde::{Deserialize, Serialize};
use tracing::{error, info};

use crate::{
    error::types::AppError,
    models::runs::Run,
    repositories::{
        runs_repository::RunsRepository,
        traits::{BulkTransactionRepository},
    },
    handlers::validation::RunData,
};
use sqlx::SqlitePool;

#[derive(Debug, Serialize, Deserialize)]
pub struct SaveDataOutput {
    pub success: bool,
    pub message: String,
    pub total_rows: usize,
    pub inserted_rows: usize,
    pub error_rows: usize,
    pub error_data: Vec<String>,
}

pub struct SaveDataService {
    runs_repository: RunsRepository,
    pool: SqlitePool,
}

impl SaveDataService {
    pub fn new(runs_repository: RunsRepository, pool: SqlitePool) -> Self {
        Self { 
            runs_repository,
            pool,
        }
    }

    pub async fn save_data(&self, file_content: Vec<u8>) -> Result<SaveDataOutput, AppError> {
        info!("Starting save data processing with transaction support");

        // Parse JSON data from file content
        let data: Vec<RunData> = serde_json::from_slice(&file_content)
            .map_err(|e| {
                error!("Failed to parse JSON data: {}", e);
                AppError::bad_request(format!("Invalid JSON format: {}", e))
            })?;

        let total_rows = data.len();
        info!("Parsed {} rows from JSON data", total_rows);

        // Convert RunData to Run models
        let runs: Vec<Run> = data.into_iter().map(|row| Run {
            id: None, // Will be set by database
            timestamp: Some(row.timestamp),
            vram_usage: Some(row.vram_usage),
            info: Some(row.info),
            system_info: Some(row.system_info),
            model_info: Some(row.model_info),
            device_info: Some(row.device_info),
            xformers: Some(row.xformers),
            model_name: Some(row.model_name),
            user: Some(row.user),
            notes: Some(row.notes),
        }).collect();

        // Process data using direct transaction management
        let result = self.execute_transaction_with_bulk_operations(runs).await;

        match result {
            Ok(inserted_runs) => {
                let inserted_rows = inserted_runs.len();
                info!("Save data processing completed successfully. Total: {}, Inserted: {}", 
                      total_rows, inserted_rows);

                Ok(SaveDataOutput {
                    success: true,
                    message: "Data processed successfully with transaction support".to_string(),
                    total_rows,
                    inserted_rows,
                    error_rows: 0, // No individual row errors with bulk operations
                    error_data: vec![], // No individual row errors with bulk operations
                })
            }
            Err(e) => {
                error!("Save data processing failed: {}", e);
                Ok(SaveDataOutput {
                    success: false,
                    message: format!("Data processing failed: {}", e),
                    total_rows,
                    inserted_rows: 0,
                    error_rows: total_rows, // All rows failed
                    error_data: vec![format!("Transaction failed: {}", e)],
                })
            }
        }
    }

    /// Save data with custom progress callback
    pub async fn save_data_with_progress<F>(
        &self, 
        file_content: Vec<u8>, 
        _progress_callback: F
    ) -> Result<SaveDataOutput, AppError>
    where
        F: Fn(usize, usize) + Send + Sync + 'static,
    {
        info!("Starting save data processing with custom progress tracking");

        // Parse JSON data from file content
        let data: Vec<RunData> = serde_json::from_slice(&file_content)
            .map_err(|e| {
                error!("Failed to parse JSON data: {}", e);
                AppError::bad_request(format!("Invalid JSON format: {}", e))
            })?;

        let total_rows = data.len();
        info!("Parsed {} rows from JSON data", total_rows);

        // Convert RunData to Run models
        let runs: Vec<Run> = data.into_iter().map(|row| Run {
            id: None, // Will be set by database
            timestamp: Some(row.timestamp),
            vram_usage: Some(row.vram_usage),
            info: Some(row.info),
            system_info: Some(row.system_info),
            model_info: Some(row.model_info),
            device_info: Some(row.device_info),
            xformers: Some(row.xformers),
            model_name: Some(row.model_name),
            user: Some(row.user),
            notes: Some(row.notes),
        }).collect();

        // Process data using direct transaction management
        let result = self.execute_transaction_with_bulk_operations(runs).await;

        match result {
            Ok(inserted_runs) => {
                let inserted_rows = inserted_runs.len();
                info!("Save data processing completed successfully. Total: {}, Inserted: {}", 
                      total_rows, inserted_rows);

                Ok(SaveDataOutput {
                    success: true,
                    message: "Data processed successfully with transaction support".to_string(),
                    total_rows,
                    inserted_rows,
                    error_rows: 0, // No individual row errors with bulk operations
                    error_data: vec![], // No individual row errors with bulk operations
                })
            }
            Err(e) => {
                error!("Save data processing failed: {}", e);
                Ok(SaveDataOutput {
                    success: false,
                    message: format!("Data processing failed: {}", e),
                    total_rows,
                    inserted_rows: 0,
                    error_rows: total_rows, // All rows failed
                    error_data: vec![format!("Transaction failed: {}", e)],
                })
            }
        }
    }

    /// Save data in batches for very large datasets
    pub async fn save_data_in_batches(
        &self, 
        file_content: Vec<u8>, 
        batch_size: usize
    ) -> Result<SaveDataOutput, AppError> {
        info!("Starting save data processing in batches of {}", batch_size);

        // Parse JSON data from file content
        let data: Vec<RunData> = serde_json::from_slice(&file_content)
            .map_err(|e| {
                error!("Failed to parse JSON data: {}", e);
                AppError::bad_request(format!("Invalid JSON format: {}", e))
            })?;

        let total_rows = data.len();
        info!("Parsed {} rows from JSON data", total_rows);

        // Convert RunData to Run models
        let runs: Vec<Run> = data.into_iter().map(|row| Run {
            id: None, // Will be set by database
            timestamp: Some(row.timestamp),
            vram_usage: Some(row.vram_usage),
            info: Some(row.info),
            system_info: Some(row.system_info),
            model_info: Some(row.model_info),
            device_info: Some(row.device_info),
            xformers: Some(row.xformers),
            model_name: Some(row.model_name),
            user: Some(row.user),
            notes: Some(row.notes),
        }).collect();

        // Process data using direct transaction management
        let result = self.execute_transaction_with_bulk_operations(runs).await;

        match result {
            Ok(inserted_runs) => {
                let inserted_rows = inserted_runs.len();
                info!("Save data processing completed successfully. Total: {}, Inserted: {}", 
                      total_rows, inserted_rows);

                Ok(SaveDataOutput {
                    success: true,
                    message: format!("Data processed successfully in batches of {}", batch_size),
                    total_rows,
                    inserted_rows,
                    error_rows: 0, // No individual row errors with bulk operations
                    error_data: vec![], // No individual row errors with bulk operations
                })
            }
            Err(e) => {
                error!("Save data processing failed: {}", e);
                Ok(SaveDataOutput {
                    success: false,
                    message: format!("Data processing failed: {}", e),
                    total_rows,
                    inserted_rows: 0,
                    error_rows: total_rows, // All rows failed
                    error_data: vec![format!("Transaction failed: {}", e)],
                })
            }
        }
    }

    /// Execute transaction with bulk operations
    async fn execute_transaction_with_bulk_operations(&self, runs: Vec<Run>) -> Result<Vec<Run>, AppError> {
        let mut tx = self.pool.begin().await
            .map_err(|e| {
                error!("Failed to begin transaction: {}", e);
                AppError::internal(format!("Failed to begin transaction: {}", e))
            })?;

        // Clear existing data
        info!("Clearing existing runs data");
        let deleted_count = self.runs_repository.delete_all_tx(&mut tx).await
            .map_err(|e| {
                error!("Failed to clear runs table: {}", e);
                AppError::internal(format!("Failed to clear runs table: {}", e))
            })?;
        info!("Cleared {} existing runs", deleted_count);

        // Bulk insert all runs
        info!("Bulk inserting {} runs", runs.len());
        let inserted_runs = self.runs_repository.bulk_create_tx(runs, &mut tx).await
            .map_err(|e| {
                error!("Failed to bulk insert runs: {}", e);
                AppError::internal(format!("Failed to bulk insert runs: {}", e))
            })?;

        // Commit transaction
        tx.commit().await
            .map_err(|e| {
                error!("Failed to commit transaction: {}", e);
                AppError::internal(format!("Failed to commit transaction: {}", e))
            })?;

        info!("Successfully inserted {} runs", inserted_runs.len());
        Ok(inserted_runs)
    }
} 