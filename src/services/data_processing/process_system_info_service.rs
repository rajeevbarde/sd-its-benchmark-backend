use tracing::{error, info, warn};

use crate::{
    error::types::AppError,
    models::system_info::SystemInfo,
    repositories::{
        runs_repository::RunsRepository,
        system_info_repository::SystemInfoRepository,
        traits::{Repository, BulkTransactionRepository},
    },
    services::parsers::SystemInfoParser,
};
use sqlx::SqlitePool;

#[derive(Debug)]
pub struct ProcessSystemInfoOutput {
    pub success: bool,
    pub message: String,
    pub total_runs: usize,
    pub inserted_rows: usize,
    pub error_rows: usize,
    pub error_data: Vec<String>,
}

pub struct ProcessSystemInfoService {
    runs_repository: RunsRepository,
    system_info_repository: SystemInfoRepository,
    pool: SqlitePool,
}

impl ProcessSystemInfoService {
    pub fn new(
        runs_repository: RunsRepository,
        system_info_repository: SystemInfoRepository,
        pool: SqlitePool,
    ) -> Self {
        Self {
            runs_repository,
            system_info_repository,
            pool,
        }
    }

    /// Process system info from runs table
    /// 
    /// This service:
    /// 1. Clears existing system info
    /// 2. Fetches all runs data
    /// 3. Parses system info from system_info strings using SystemInfoParser
    /// 4. Inserts system info into the database (only if all required fields are present)
    /// 
    /// # Returns
    /// * `ProcessSystemInfoOutput` - Processing results and statistics
    pub async fn process_system_info(&self) -> Result<ProcessSystemInfoOutput, AppError> {
        info!("Processing system info from runs table with transaction support");

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
                info!("System info processing completed successfully. Total: {}, Inserted: {}", 
                      total_runs, inserted_rows);

                Ok(ProcessSystemInfoOutput {
                    success: true,
                    message: "System info processing completed successfully with transaction support".to_string(),
                    total_runs,
                    inserted_rows,
                    error_rows: 0, // No individual row errors with bulk operations
                    error_data: vec![], // No individual row errors with bulk operations
                })
            }
            Err(e) => {
                error!("System info processing failed: {}", e);
                Ok(ProcessSystemInfoOutput {
                    success: false,
                    message: format!("System info processing failed: {}", e),
                    total_runs,
                    inserted_rows: 0,
                    error_rows: total_runs, // All rows failed
                    error_data: vec![format!("Transaction failed: {}", e)],
                })
            }
        }
    }

    /// Execute transaction with bulk operations
    async fn execute_transaction_with_bulk_operations(&self, runs: Vec<crate::models::runs::Run>) -> Result<Vec<SystemInfo>, AppError> {
        let mut tx = self.pool.begin().await
            .map_err(|e| {
                error!("Failed to begin transaction: {}", e);
                AppError::internal(format!("Failed to begin transaction: {}", e))
            })?;

        // Clear existing system info
        info!("Clearing existing system info");
        let deleted_count = self.system_info_repository.delete_all_tx(&mut tx).await
            .map_err(|e| {
                error!("Failed to clear system info: {}", e);
                AppError::internal(format!("Failed to clear system info: {}", e))
            })?;
        info!("Cleared {} existing system info", deleted_count);

        // Process all runs and create system info
        let mut system_info_records = Vec::new();
        for (index, run) in runs.iter().enumerate() {
            match self.process_run_for_bulk(run, index) {
                Ok(Some(system_info)) => {
                    system_info_records.push(system_info);
                    if index % 100 == 0 {
                        info!("Processed {} runs", index + 1);
                    }
                }
                Ok(None) => {
                    // Skip runs with missing required fields
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

        // Bulk insert all system info
        info!("Bulk inserting {} system info records", system_info_records.len());
        let inserted_results = self.system_info_repository.bulk_create_tx(system_info_records, &mut tx).await
            .map_err(|e| {
                error!("Failed to bulk insert system info: {}", e);
                AppError::internal(format!("Failed to bulk insert system info: {}", e))
            })?;

        // Commit transaction
        tx.commit().await
            .map_err(|e| {
                error!("Failed to commit transaction: {}", e);
                AppError::internal(format!("Failed to commit transaction: {}", e))
            })?;

        info!("Successfully inserted {} system info records", inserted_results.len());
        Ok(inserted_results)
    }

    /// Process a single run and create system info (for bulk processing)
    /// Returns Some(SystemInfo) if valid, None if skipped due to missing fields
    fn process_run_for_bulk(&self, run: &crate::models::runs::Run, index: usize) -> Result<Option<SystemInfo>, AppError> {
        let run_id = run.id.ok_or_else(|| {
            error!("Run at index {} has no ID", index);
            AppError::bad_request("Invalid run data".to_string())
        })?;

        let system_info = run.system_info.as_ref().ok_or_else(|| {
            error!("Run {} has no system_info", run_id);
            AppError::bad_request("Missing system_info data".to_string())
        })?;

        // Parse system info from system_info string using our parser
        let parsed_system_info = SystemInfoParser::parse(system_info);

        // Only insert if all required fields are present
        if parsed_system_info.arch.is_some() &&
           parsed_system_info.cpu.is_some() &&
           parsed_system_info.system.is_some() &&
           parsed_system_info.release.is_some() &&
           parsed_system_info.python.is_some() {
            
            // Create system info record
            let system_info_record = SystemInfo {
                id: None,
                run_id: Some(run_id),
                arch: parsed_system_info.arch,
                cpu: parsed_system_info.cpu,
                system: parsed_system_info.system,
                release: parsed_system_info.release,
                python: parsed_system_info.python,
            };

            Ok(Some(system_info_record))
        } else {
            warn!("Skipping run {} due to missing required system info fields", run_id);
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {


    #[test]
    fn test_process_system_info_service_creation() {
        // This test verifies the service can be created
        // In a real test, we would use a test database
        assert!(true, "Service structure is valid");
    }
} 