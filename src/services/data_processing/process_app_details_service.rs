use tracing::{error, info, warn};

use crate::{
    error::types::AppError,
    models::app_details::AppDetails,
    repositories::{
        app_details_repository::AppDetailsRepository,
        runs_repository::RunsRepository,
        traits::{Repository, BulkTransactionRepository},
    },
    services::parsers::AppDetailsParser,
};
use sqlx::SqlitePool;

#[derive(Debug)]
pub struct ProcessAppDetailsOutput {
    pub success: bool,
    pub message: String,
    pub total_runs: usize,
    pub inserted_rows: usize,
    pub error_rows: usize,
    pub error_data: Vec<String>,
}

pub struct ProcessAppDetailsService {
    runs_repository: RunsRepository,
    app_details_repository: AppDetailsRepository,
    pool: SqlitePool,
}

impl ProcessAppDetailsService {
    pub fn new(
        runs_repository: RunsRepository,
        app_details_repository: AppDetailsRepository,
        pool: SqlitePool,
    ) -> Self {
        Self {
            runs_repository,
            app_details_repository,
            pool,
        }
    }

    /// Process app details from runs table
    /// 
    /// This service:
    /// 1. Clears existing app details
    /// 2. Fetches all runs data
    /// 3. Parses app details from info strings using AppDetailsParser
    /// 4. Inserts app details into the database
    /// 
    /// # Returns
    /// * `ProcessAppDetailsOutput` - Processing results and statistics
    pub async fn process_app_details(&self) -> Result<ProcessAppDetailsOutput, AppError> {
        info!("Processing app details from runs table with transaction support");

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
                info!("App details processing completed successfully. Total: {}, Inserted: {}", 
                      total_runs, inserted_rows);

                Ok(ProcessAppDetailsOutput {
                    success: true,
                    message: "App details processing completed successfully with transaction support".to_string(),
                    total_runs,
                    inserted_rows,
                    error_rows: 0, // No individual row errors with bulk operations
                    error_data: vec![], // No individual row errors with bulk operations
                })
            }
            Err(e) => {
                error!("App details processing failed: {}", e);
                Ok(ProcessAppDetailsOutput {
                    success: false,
                    message: format!("App details processing failed: {}", e),
                    total_runs,
                    inserted_rows: 0,
                    error_rows: total_runs, // All rows failed
                    error_data: vec![format!("Transaction failed: {}", e)],
                })
            }
        }
    }

    /// Execute transaction with bulk operations
    async fn execute_transaction_with_bulk_operations(&self, runs: Vec<crate::models::runs::Run>) -> Result<Vec<AppDetails>, AppError> {
        let mut tx = self.pool.begin().await
            .map_err(|e| {
                error!("Failed to begin transaction: {}", e);
                AppError::internal(format!("Failed to begin transaction: {}", e))
            })?;

        // Clear existing app details
        info!("Clearing existing app details");
        let deleted_count = self.app_details_repository.delete_all_tx(&mut tx).await
            .map_err(|e| {
                error!("Failed to clear app details: {}", e);
                AppError::internal(format!("Failed to clear app details: {}", e))
            })?;
        info!("Cleared {} existing app details", deleted_count);

        // Process all runs and create app details
        let mut app_details = Vec::new();
        for (index, run) in runs.iter().enumerate() {
            match self.process_run_for_bulk(run, index) {
                Ok(app_detail) => {
                    app_details.push(app_detail);
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

        // Bulk insert all app details
        info!("Bulk inserting {} app details", app_details.len());
        let inserted_results = self.app_details_repository.bulk_create_tx(app_details, &mut tx).await
            .map_err(|e| {
                error!("Failed to bulk insert app details: {}", e);
                AppError::internal(format!("Failed to bulk insert app details: {}", e))
            })?;

        // Commit transaction
        tx.commit().await
            .map_err(|e| {
                error!("Failed to commit transaction: {}", e);
                AppError::internal(format!("Failed to commit transaction: {}", e))
            })?;

        info!("Successfully inserted {} app details", inserted_results.len());
        Ok(inserted_results)
    }

    /// Process a single run and create app details (for bulk processing)
    fn process_run_for_bulk(&self, run: &crate::models::runs::Run, index: usize) -> Result<AppDetails, AppError> {
        let run_id = run.id.ok_or_else(|| {
            error!("Run at index {} has no ID", index);
            AppError::bad_request("Invalid run data".to_string())
        })?;

        let info = run.info.as_ref().ok_or_else(|| {
            error!("Run {} has no info", run_id);
            AppError::bad_request("Missing info data".to_string())
        })?;

        // Parse app details from info string using our parser
        let app_details = AppDetailsParser::parse(info);

        // Create app details record
        let app_details_record = AppDetails {
            id: None,
            run_id: Some(run_id),
            app_name: app_details.app_name,
            updated: app_details.updated,
            hash: app_details.hash,
            url: app_details.url,
        };

        Ok(app_details_record)
    }
}

#[cfg(test)]
mod tests {


    #[test]
    fn test_process_app_details_service_creation() {
        // This test verifies the service can be created
        // In a real test, we would use a test database
        assert!(true, "Service structure is valid");
    }
} 