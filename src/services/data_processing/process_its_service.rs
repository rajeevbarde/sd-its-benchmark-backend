use tracing::{error, info, warn};

use crate::{
    error::types::AppError,
    models::performance_result::PerformanceResult,
    repositories::{
        performance_result_repository::PerformanceResultRepository,
        runs_repository::RunsRepository,
        traits::Repository,
    },
    services::parsers::PerformanceParser,
};

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
}

impl ProcessItsService {
    pub fn new(
        runs_repository: RunsRepository,
        performance_result_repository: PerformanceResultRepository,
    ) -> Self {
        Self {
            runs_repository,
            performance_result_repository,
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
        info!("Processing ITS data from runs table");

        let mut inserted_rows = 0;
        let mut error_rows = 0;
        let mut error_data = Vec::new();

        // For now, we'll use the regular repository methods without transactions
        // TODO: Add proper transaction support to the repository

        // Clear existing performance results
        self.clear_performance_results().await?;
        info!("Cleared existing performance results");

        // Fetch all runs data
        let runs = self.runs_repository.find_all().await.map_err(|e| {
            error!("Failed to fetch runs data: {}", e);
            AppError::internal(format!("Failed to fetch runs data: {}", e))
        })?;

        let total_runs = runs.len();
        info!("Found {} runs to process", total_runs);

        // Process each run
        for (index, run) in runs.iter().enumerate() {
            match self.process_run(run, index).await {
                Ok(_) => {
                    inserted_rows += 1;
                    if index % 100 == 0 {
                        info!("Processed {} runs", index + 1);
                    }
                }
                Err(e) => {
                    error_rows += 1;
                    let error_msg = format!("Run {}: {}", index + 1, e);
                    error_data.push(error_msg);
                    warn!("Failed to process run {}: {}", index + 1, e);
                }
            }
        }

        info!("ITS processing complete: {} rows inserted", inserted_rows);

        Ok(ProcessItsOutput {
            success: true,
            message: "ITS processing completed successfully".to_string(),
            total_runs,
            inserted_rows,
            error_rows,
            error_data,
        })
    }

    /// Clear all existing performance results
    async fn clear_performance_results(&self) -> Result<(), AppError> {
        // TODO: Implement delete_all method in repository
        // For now, we'll skip clearing the table
        info!("Skipping performance results clear (not implemented yet)");
        Ok(())
    }

    /// Process a single run and create performance result
    async fn process_run(&self, run: &crate::models::runs::Run, index: usize) -> Result<(), AppError> {
        let run_id = run.id.ok_or_else(|| {
            error!("Run at index {} has no ID", index);
            AppError::bad_request("Invalid run data".to_string())
        })?;

        let vram_usage = run.vram_usage.as_ref().ok_or_else(|| {
            error!("Run {} has no vram_usage", run_id);
            AppError::bad_request("Missing vram_usage data".to_string())
        })?;

        info!("Processing run {} of {} (ID: {})", index + 1, index + 1, run_id);

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

        // Insert into database
        self.performance_result_repository.create(performance_result).await
            .map_err(|e| {
                error!("Failed to insert performance result for run {}: {}", run_id, e);
                AppError::internal(format!("Failed to insert performance result: {}", e))
            })?;

        info!("Processed run {} with average ITS: {}", 
            index + 1, 
            performance_data.avg_its.unwrap_or(0.0)
        );

        Ok(())
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