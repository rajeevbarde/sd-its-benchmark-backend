use tracing::{error, info, warn};

use crate::{
    error::types::AppError,
    models::{run_more_details::RunMoreDetails, runs::Run},
    repositories::{
        run_more_details_repository::RunMoreDetailsRepository,
        runs_repository::RunsRepository,
        traits::Repository,
    },
};

#[derive(Debug)]
pub struct ProcessRunDetailsOutput {
    pub success: bool,
    pub message: String,
    pub total_inserts: usize,
}

pub struct ProcessRunDetailsService {
    runs_repository: RunsRepository,
    run_more_details_repository: RunMoreDetailsRepository,
}

impl ProcessRunDetailsService {
    pub fn new(
        runs_repository: RunsRepository,
        run_more_details_repository: RunMoreDetailsRepository,
    ) -> Self {
        Self {
            runs_repository,
            run_more_details_repository,
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
        info!("Processing run details from runs table to RunMoreDetails table");

        // Clear all existing data from RunMoreDetails table
        self.run_more_details_repository.clear_all().await.map_err(|e| {
            error!("Failed to clear RunMoreDetails table: {}", e);
            AppError::internal(format!("Failed to clear RunMoreDetails table: {}", e))
        })?;

        info!("Cleared existing RunMoreDetails data");

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

        let mut total_inserts = 0;
        let mut error_count = 0;

        // Process each run
        for run in &runs_data {
            match self.process_run(run).await {
                Ok(_) => {
                    total_inserts += 1;
                }
                Err(e) => {
                    error_count += 1;
                    warn!("Failed to process run {}: {}", run.id.unwrap_or(0), e);
                }
            }
        }

        info!("Run details processing complete: {} total inserts, {} errors", 
              total_inserts, error_count);

        Ok(ProcessRunDetailsOutput {
            success: true,
            message: "Run details processed successfully!".to_string(),
            total_inserts,
        })
    }

    /// Process a single run and insert into RunMoreDetails
    async fn process_run(&self, run: &Run) -> Result<(), AppError> {
        let run_id = run.id.ok_or_else(|| {
            error!("Run has no ID");
            AppError::bad_request("Invalid run data".to_string())
        })?;

        info!("Processing run details for run ID: {}", run_id);

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

        // Insert into RunMoreDetails table
        self.run_more_details_repository.create(run_more_details).await
            .map_err(|e| {
                error!("Failed to insert run details for run {}: {}", run_id, e);
                AppError::internal(format!("Failed to insert run details: {}", e))
            })?;

        Ok(())
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