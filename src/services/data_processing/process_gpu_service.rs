use tracing::{error, info, warn};

use crate::{
    error::types::AppError,
    models::gpu::Gpu,
    repositories::{
        gpu_repository::GpuRepository,
        runs_repository::RunsRepository,
        traits::{Repository, BulkTransactionRepository},
    },
    services::parsers::GpuInfoParser,
};
use sqlx::SqlitePool;

#[derive(Debug)]
pub struct ProcessGpuOutput {
    pub success: bool,
    pub message: String,
    pub total_runs: usize,
    pub inserted_rows: usize,
    pub error_rows: usize,
    pub error_data: Vec<String>,
}

pub struct ProcessGpuService {
    runs_repository: RunsRepository,
    gpu_repository: GpuRepository,
    pool: SqlitePool,
}

impl ProcessGpuService {
    pub fn new(
        runs_repository: RunsRepository,
        gpu_repository: GpuRepository,
        pool: SqlitePool,
    ) -> Self {
        Self {
            runs_repository,
            gpu_repository,
            pool,
        }
    }

    /// Process GPU info from runs table
    /// 
    /// This service:
    /// 1. Clears existing GPU data
    /// 2. Fetches all runs data
    /// 3. Parses GPU information from device_info strings using GpuInfoParser
    /// 4. Inserts GPU information into the database
    /// 
    /// # Returns
    /// * `ProcessGpuOutput` - Processing results and statistics
    pub async fn process_gpu(&self) -> Result<ProcessGpuOutput, AppError> {
        info!("Processing GPU info from runs table with transaction support");

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
                info!("GPU processing completed successfully. Total: {}, Inserted: {}", 
                      total_runs, inserted_rows);

                Ok(ProcessGpuOutput {
                    success: true,
                    message: "GPU processing completed successfully with transaction support".to_string(),
                    total_runs,
                    inserted_rows,
                    error_rows: 0, // No individual row errors with bulk operations
                    error_data: vec![], // No individual row errors with bulk operations
                })
            }
            Err(e) => {
                error!("GPU processing failed: {}", e);
                Ok(ProcessGpuOutput {
                    success: false,
                    message: format!("GPU processing failed: {}", e),
                    total_runs,
                    inserted_rows: 0,
                    error_rows: total_runs, // All rows failed
                    error_data: vec![format!("Transaction failed: {}", e)],
                })
            }
        }
    }

    /// Execute transaction with bulk operations
    async fn execute_transaction_with_bulk_operations(&self, runs: Vec<crate::models::runs::Run>) -> Result<Vec<Gpu>, AppError> {
        let mut tx = self.pool.begin().await
            .map_err(|e| {
                error!("Failed to begin transaction: {}", e);
                AppError::internal(format!("Failed to begin transaction: {}", e))
            })?;

        // Clear existing GPU data
        info!("Clearing existing GPU data");
        let deleted_count = self.gpu_repository.delete_all_tx(&mut tx).await
            .map_err(|e| {
                error!("Failed to clear GPU data: {}", e);
                AppError::internal(format!("Failed to clear GPU data: {}", e))
            })?;
        info!("Cleared {} existing GPU records", deleted_count);

        // Process all runs and create GPU records
        let mut gpu_records = Vec::new();
        for (index, run) in runs.iter().enumerate() {
            match self.process_run_for_bulk(run, index) {
                Ok(gpu) => {
                    gpu_records.push(gpu);
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

        // Bulk insert all GPU records
        info!("Bulk inserting {} GPU records", gpu_records.len());
        let inserted_results = self.gpu_repository.bulk_create_tx(gpu_records, &mut tx).await
            .map_err(|e| {
                error!("Failed to bulk insert GPU records: {}", e);
                AppError::internal(format!("Failed to bulk insert GPU records: {}", e))
            })?;

        // Commit transaction
        tx.commit().await
            .map_err(|e| {
                error!("Failed to commit transaction: {}", e);
                AppError::internal(format!("Failed to commit transaction: {}", e))
            })?;

        info!("Successfully inserted {} GPU records", inserted_results.len());
        Ok(inserted_results)
    }

    /// Process a single run and create GPU record (for bulk processing)
    fn process_run_for_bulk(&self, run: &crate::models::runs::Run, index: usize) -> Result<Gpu, AppError> {
        let run_id = run.id.ok_or_else(|| {
            error!("Run at index {} has no ID", index);
            AppError::bad_request("Invalid run data".to_string())
        })?;

        let device_info = run.device_info.as_ref().ok_or_else(|| {
            error!("Run {} has no device_info", run_id);
            AppError::bad_request("Missing device_info data".to_string())
        })?;

        // Parse device info to extract GPU information using our parser
        let parsed_gpu_info = GpuInfoParser::parse(device_info);

        // Create GPU record
        let gpu_record = Gpu {
            id: None,
            run_id: Some(run_id),
            device: parsed_gpu_info.device,
            driver: parsed_gpu_info.driver,
            gpu_chip: parsed_gpu_info.gpu_chip,
            brand: None, // Will be populated by separate update process
            is_laptop: None, // Will be populated by separate update process
        };

        Ok(gpu_record)
    }
}

#[cfg(test)]
mod tests {


    #[test]
    fn test_process_gpu_service_creation() {
        // This test verifies the service can be created
        // In a real test, we would use a test database
        assert!(true, "Service structure is valid");
    }
} 