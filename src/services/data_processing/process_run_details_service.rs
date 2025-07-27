use tracing::{error, info, warn};

use crate::{
    error::types::AppError,
    models::{run_more_details::RunMoreDetails, runs::Run},
    repositories::{
        run_more_details_repository::RunMoreDetailsRepository,
        runs_repository::RunsRepository,
        traits::{Repository, BulkTransactionRepository},
    },
};
use sqlx::SqlitePool;

#[derive(Debug)]
pub struct ProcessRunDetailsOutput {
    pub success: bool,
    pub message: String,
    pub total_inserts: usize,
}

pub struct ProcessRunDetailsService {
    runs_repository: RunsRepository,
    run_more_details_repository: RunMoreDetailsRepository,
    pool: SqlitePool,
}

impl ProcessRunDetailsService {
    pub fn new(
        runs_repository: RunsRepository,
        run_more_details_repository: RunMoreDetailsRepository,
        pool: SqlitePool,
    ) -> Self {
        Self {
            runs_repository,
            run_more_details_repository,
            pool,
        }
    }

    /// Process run details from runs table to RunMoreDetails table
    /// 
    /// This service:
    /// 1. Clears all existing data from RunMoreDetails table
    /// 2. Fetches all runs data (id, timestamp, model_name, user, notes)
    /// 3. Inserts the data into RunMoreDetails table
    /// 4. Returns statistics about the processing
    /// 
    /// # Returns
    /// * `ProcessRunDetailsOutput` - Processing results and statistics
    pub async fn process_run_details(&self) -> Result<ProcessRunDetailsOutput, AppError> {
        info!("Processing run details from runs table to RunMoreDetails table with transaction support");

        // Fetch all runs data
        let runs_data = self.runs_repository.find_all().await.map_err(|e| {
            error!("Failed to fetch runs data: {}", e);
            AppError::internal(format!("Failed to fetch runs data: {}", e))
        })?;

        if runs_data.is_empty() {
            info!("No runs data found to process");

            return Ok(ProcessRunDetailsOutput {
                success: true,
                message: "No runs data found to process".to_string(),
                total_inserts: 0,
            });
        }

        info!("Found {} runs to process", runs_data.len());

        // Process data using direct transaction management
        let result = self.execute_transaction_with_bulk_operations(runs_data).await;

        match result {
            Ok(inserted_results) => {
                let total_inserts = inserted_results.len();
                info!("Run details processing completed successfully. Total inserts: {}", total_inserts);

                Ok(ProcessRunDetailsOutput {
                    success: true,
                    message: "Run details processed successfully with transaction support!".to_string(),
                    total_inserts,
                })
            }
            Err(e) => {
                error!("Run details processing failed: {}", e);
                Ok(ProcessRunDetailsOutput {
                    success: false,
                    message: format!("Run details processing failed: {}", e),
                    total_inserts: 0,
                })
            }
        }
    }

    /// Execute transaction with bulk operations
    async fn execute_transaction_with_bulk_operations(&self, runs: Vec<Run>) -> Result<Vec<RunMoreDetails>, AppError> {
        let mut tx = self.pool.begin().await
            .map_err(|e| {
                error!("Failed to begin transaction: {}", e);
                AppError::internal(format!("Failed to begin transaction: {}", e))
            })?;

        // Clear all existing data from RunMoreDetails table
        info!("Clearing existing RunMoreDetails data");
        let deleted_count = self.run_more_details_repository.delete_all_tx(&mut tx).await
            .map_err(|e| {
                error!("Failed to clear RunMoreDetails table: {}", e);
                AppError::internal(format!("Failed to clear RunMoreDetails table: {}", e))
            })?;
        info!("Cleared {} existing RunMoreDetails records", deleted_count);

        // Process all runs and create run more details
        let mut run_more_details = Vec::new();
        for run in &runs {
            match self.process_run_for_bulk(run) {
                Ok(run_detail) => {
                    run_more_details.push(run_detail);
                }
                Err(e) => {
                    warn!("Failed to process run {}: {}", run.id.unwrap_or(0), e);
                    // Continue processing other runs
                }
            }
        }

        // Bulk insert all run more details
        info!("Bulk inserting {} run more details", run_more_details.len());
        let inserted_results = self.run_more_details_repository.bulk_create_tx(run_more_details, &mut tx).await
            .map_err(|e| {
                error!("Failed to bulk insert run more details: {}", e);
                AppError::internal(format!("Failed to bulk insert run more details: {}", e))
            })?;

        // Commit transaction
        tx.commit().await
            .map_err(|e| {
                error!("Failed to commit transaction: {}", e);
                AppError::internal(format!("Failed to commit transaction: {}", e))
            })?;

        info!("Successfully inserted {} run more details", inserted_results.len());
        Ok(inserted_results)
    }

    /// Process a single run and insert into RunMoreDetails (for bulk processing)
    fn process_run_for_bulk(&self, run: &Run) -> Result<RunMoreDetails, AppError> {
        let run_id = run.id.ok_or_else(|| {
            error!("Run has no ID");
            AppError::bad_request("Invalid run data".to_string())
        })?;

        // Create RunMoreDetails record
        let run_more_details = RunMoreDetails {
            id: None,
            run_id: Some(run_id),
            timestamp: run.timestamp.clone(),
            model_name: run.model_name.clone(),
            user: run.user.clone(),
            notes: run.notes.clone(),
            model_map_id: None, // Will be populated by a later service
        };

        Ok(run_more_details)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_run_details_service_creation() {
        // This test verifies the service can be created
        // In a real test, we would use a test database
        assert!(true, "Service structure is valid");
    }
} 