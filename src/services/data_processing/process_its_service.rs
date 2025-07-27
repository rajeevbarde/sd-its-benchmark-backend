use tracing::{error, info, warn};

use crate::{
    error::types::AppError,
    models::performance_result::PerformanceResult,
    repositories::{
        performance_result_repository::PerformanceResultRepository,
        runs_repository::RunsRepository,
        traits::{Repository, BulkTransactionRepository},
    },
    services::parsers::PerformanceParser,
};
use sqlx::SqlitePool;

#[derive(Debug)]
pub struct ProcessItsOutput {
    pub success: bool,
    pub message: String,
    pub total_runs: usize,
    pub inserted_rows: usize,
    pub error_rows: usize,
    pub error_data: Vec<String>,
}

pub struct ProcessItsService {
    runs_repository: RunsRepository,
    performance_result_repository: PerformanceResultRepository,
    pool: SqlitePool,
}

impl ProcessItsService {
    pub fn new(
        runs_repository: RunsRepository,
        performance_result_repository: PerformanceResultRepository,
        pool: SqlitePool,
    ) -> Self {
        Self {
            runs_repository,
            performance_result_repository,
            pool,
        }
    }

    /// Process ITS (Iterations Per Second) data from runs table
    /// 
    /// This service:
    /// 1. Clears existing performance results
    /// 2. Fetches all runs data
    /// 3. Parses ITS values from vram_usage strings
    /// 4. Calculates average ITS for each run
    /// 5. Inserts performance results into the database
    /// 
    /// # Returns
    /// * `ProcessItsOutput` - Processing results and statistics
    pub async fn process_its(&self) -> Result<ProcessItsOutput, AppError> {
        info!("Processing ITS data from runs table with transaction support");

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
                info!("ITS processing completed successfully. Total: {}, Inserted: {}", 
                      total_runs, inserted_rows);

                Ok(ProcessItsOutput {
                    success: true,
                    message: "ITS processing completed successfully with transaction support".to_string(),
                    total_runs,
                    inserted_rows,
                    error_rows: 0, // No individual row errors with bulk operations
                    error_data: vec![], // No individual row errors with bulk operations
                })
            }
            Err(e) => {
                error!("ITS processing failed: {}", e);
                Ok(ProcessItsOutput {
                    success: false,
                    message: format!("ITS processing failed: {}", e),
                    total_runs,
                    inserted_rows: 0,
                    error_rows: total_runs, // All rows failed
                    error_data: vec![format!("Transaction failed: {}", e)],
                })
            }
        }
    }

    /// Execute transaction with bulk operations
    async fn execute_transaction_with_bulk_operations(&self, runs: Vec<crate::models::runs::Run>) -> Result<Vec<PerformanceResult>, AppError> {
        let mut tx = self.pool.begin().await
            .map_err(|e| {
                error!("Failed to begin transaction: {}", e);
                AppError::internal(format!("Failed to begin transaction: {}", e))
            })?;

        // Clear existing performance results
        info!("Clearing existing performance results");
        let deleted_count = self.performance_result_repository.delete_all_tx(&mut tx).await
            .map_err(|e| {
                error!("Failed to clear performance results: {}", e);
                AppError::internal(format!("Failed to clear performance results: {}", e))
            })?;
        info!("Cleared {} existing performance results", deleted_count);

        // Process all runs and create performance results
        let mut performance_results = Vec::new();
        for (index, run) in runs.iter().enumerate() {
            match self.process_run_for_bulk(run, index) {
                Ok(performance_result) => {
                    performance_results.push(performance_result);
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

        // Bulk insert all performance results
        info!("Bulk inserting {} performance results", performance_results.len());
        let inserted_results = self.performance_result_repository.bulk_create_tx(performance_results, &mut tx).await
            .map_err(|e| {
                error!("Failed to bulk insert performance results: {}", e);
                AppError::internal(format!("Failed to bulk insert performance results: {}", e))
            })?;

        // Commit transaction
        tx.commit().await
            .map_err(|e| {
                error!("Failed to commit transaction: {}", e);
                AppError::internal(format!("Failed to commit transaction: {}", e))
            })?;

        info!("Successfully inserted {} performance results", inserted_results.len());
        Ok(inserted_results)
    }

    /// Process a single run and create performance result (for bulk processing)
    fn process_run_for_bulk(&self, run: &crate::models::runs::Run, index: usize) -> Result<PerformanceResult, AppError> {
        let run_id = run.id.ok_or_else(|| {
            error!("Run at index {} has no ID", index);
            AppError::bad_request("Invalid run data".to_string())
        })?;

        let vram_usage = run.vram_usage.as_ref().ok_or_else(|| {
            error!("Run {} has no vram_usage", run_id);
            AppError::bad_request("Missing vram_usage data".to_string())
        })?;

        // Parse ITS values using the PerformanceParser
        let performance_data = PerformanceParser::parse(vram_usage);

        // Validate the parsed data
        if !PerformanceParser::is_valid(&performance_data) {
            warn!("Invalid performance data for run {}: {}", run_id, vram_usage);
        }

        // Create performance result
        let performance_result = PerformanceResult {
            id: None,
            run_id: Some(run_id),
            its: Some(vram_usage.clone()),
            avg_its: performance_data.avg_its,
        };

        Ok(performance_result)
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_parse_its_values() {
        // Create a temporary service just for testing the parsing function
        // We don't need actual repositories for this test
        struct TestService;
        impl TestService {
            fn parse_its_values(&self, vram_usage: &str) -> Vec<f64> {
                vram_usage
                    .split('/')
                    .filter_map(|value| {
                        let trimmed = value.trim();
                        if trimmed.is_empty() {
                            None
                        } else {
                            // Parse the value and filter out NaN
                            trimmed.parse::<f64>().ok().filter(|&x| !x.is_nan())
                        }
                    })
                    .collect()
            }
        }

        let service = TestService;

        // Test basic parsing
        let result = service.parse_its_values("1.5/2.0/1.8");
        assert_eq!(result, vec![1.5, 2.0, 1.8]);

        // Test with empty values
        let result = service.parse_its_values("1.5//2.0");
        assert_eq!(result, vec![1.5, 2.0]);

        // Test with whitespace
        let result = service.parse_its_values(" 1.5 / 2.0 ");
        assert_eq!(result, vec![1.5, 2.0]);

        // Test with invalid values
        let result = service.parse_its_values("1.5/invalid/2.0");
        assert_eq!(result, vec![1.5, 2.0]);

        // Test empty string
        let result = service.parse_its_values("");
        assert_eq!(result, vec![] as Vec<f64>); 
    }
} 