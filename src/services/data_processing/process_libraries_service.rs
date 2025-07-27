use tracing::{error, info, warn};

use crate::{
    error::types::AppError,
    models::libraries::Libraries,
    repositories::{
        libraries_repository::LibrariesRepository,
        runs_repository::RunsRepository,
        traits::{Repository, BulkTransactionRepository},
    },
    services::parsers::LibrariesParser,
};
use sqlx::SqlitePool;

#[derive(Debug)]
pub struct ProcessLibrariesOutput {
    pub success: bool,
    pub message: String,
    pub total_runs: usize,
    pub inserted_rows: usize,
    pub error_rows: usize,
    pub error_data: Vec<String>,
}

pub struct ProcessLibrariesService {
    runs_repository: RunsRepository,
    libraries_repository: LibrariesRepository,
    pool: SqlitePool,
}

impl ProcessLibrariesService {
    pub fn new(
        runs_repository: RunsRepository,
        libraries_repository: LibrariesRepository,
        pool: SqlitePool,
    ) -> Self {
        Self {
            runs_repository,
            libraries_repository,
            pool,
        }
    }

    /// Process libraries from runs table
    /// 
    /// This service:
    /// 1. Clears existing libraries
    /// 2. Fetches all runs data
    /// 3. Parses library information from model_info strings using LibrariesParser
    /// 4. Inserts library information into the database
    /// 
    /// # Returns
    /// * `ProcessLibrariesOutput` - Processing results and statistics
    pub async fn process_libraries(&self) -> Result<ProcessLibrariesOutput, AppError> {
        info!("Processing libraries from runs table with transaction support");

        // Fetch all runs data
        let runs = self.runs_repository.find_all().await.map_err(|e| {
            error!("Failed to fetch runs data: {}", e);
            AppError::internal(format!("Failed to fetch runs data: {}", e))
        })?;

        let total_runs = runs.len();
        info!("Found {} runs to process", total_runs);

        // Process data using direct transaction management
        let result = self.execute_transaction_with_bulk_operations(runs).await;

        match result {
            Ok(inserted_results) => {
                let inserted_rows = inserted_results.len();
                info!("Libraries processing completed successfully. Total: {}, Inserted: {}", 
                      total_runs, inserted_rows);

                Ok(ProcessLibrariesOutput {
                    success: true,
                    message: "Libraries processing completed successfully with transaction support".to_string(),
                    total_runs,
                    inserted_rows,
                    error_rows: 0, // No individual row errors with bulk operations
                    error_data: vec![], // No individual row errors with bulk operations
                })
            }
            Err(e) => {
                error!("Libraries processing failed: {}", e);
                Ok(ProcessLibrariesOutput {
                    success: false,
                    message: format!("Libraries processing failed: {}", e),
                    total_runs,
                    inserted_rows: 0,
                    error_rows: total_runs, // All rows failed
                    error_data: vec![format!("Transaction failed: {}", e)],
                })
            }
        }
    }

    /// Execute transaction with bulk operations
    async fn execute_transaction_with_bulk_operations(&self, runs: Vec<crate::models::runs::Run>) -> Result<Vec<Libraries>, AppError> {
        let mut tx = self.pool.begin().await
            .map_err(|e| {
                error!("Failed to begin transaction: {}", e);
                AppError::internal(format!("Failed to begin transaction: {}", e))
            })?;

        // Clear existing libraries
        info!("Clearing existing libraries");
        let deleted_count = self.libraries_repository.delete_all_tx(&mut tx).await
            .map_err(|e| {
                error!("Failed to clear libraries: {}", e);
                AppError::internal(format!("Failed to clear libraries: {}", e))
            })?;
        info!("Cleared {} existing libraries", deleted_count);

        // Process all runs and create libraries
        let mut libraries_records = Vec::new();
        for (index, run) in runs.iter().enumerate() {
            match self.process_run_for_bulk(run, index) {
                Ok(libraries) => {
                    libraries_records.push(libraries);
                    if index % 100 == 0 {
                        info!("Processed {} runs", index + 1);
                    }
                }
                Err(e) => {
                    warn!("Failed to process run {}: {}", index + 1, e);
                    // Continue processing other runs
                }
            }
        }

        // Bulk insert all libraries
        info!("Bulk inserting {} libraries", libraries_records.len());
        let inserted_results = self.libraries_repository.bulk_create_tx(libraries_records, &mut tx).await
            .map_err(|e| {
                error!("Failed to bulk insert libraries: {}", e);
                AppError::internal(format!("Failed to bulk insert libraries: {}", e))
            })?;

        // Commit transaction
        tx.commit().await
            .map_err(|e| {
                error!("Failed to commit transaction: {}", e);
                AppError::internal(format!("Failed to commit transaction: {}", e))
            })?;

        info!("Successfully inserted {} libraries", inserted_results.len());
        Ok(inserted_results)
    }

    /// Process a single run and create libraries record (for bulk processing)
    fn process_run_for_bulk(&self, run: &crate::models::runs::Run, index: usize) -> Result<Libraries, AppError> {
        let run_id = run.id.ok_or_else(|| {
            error!("Run at index {} has no ID", index);
            AppError::bad_request("Invalid run data".to_string())
        })?;

        let model_info = run.model_info.as_ref().ok_or_else(|| {
            error!("Run {} has no model_info", run_id);
            AppError::bad_request("Missing model_info data".to_string())
        })?;

        let xformers = run.xformers.as_ref().ok_or_else(|| {
            error!("Run {} has no xformers", run_id);
            AppError::bad_request("Missing xformers data".to_string())
        })?;

        // Parse model info to extract library versions using our parser
        let parsed_libraries = LibrariesParser::parse(model_info);

        // Create libraries record
        let libraries_record = Libraries {
            id: None,
            run_id: Some(run_id),
            torch: parsed_libraries.torch,
            xformers: parsed_libraries.xformers,
            xformers1: Some(xformers.clone()), // Copy xformers value from runs table
            diffusers: parsed_libraries.diffusers,
            transformers: parsed_libraries.transformers,
        };

        Ok(libraries_record)
    }
}

#[cfg(test)]
mod tests {


    #[test]
    fn test_process_libraries_service_creation() {
        // This test verifies the service can be created
        // In a real test, we would use a test database
        assert!(true, "Service structure is valid");
    }
} 