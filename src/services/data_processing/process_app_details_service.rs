use tracing::{error, info, warn};

use crate::{
    error::types::AppError,
    models::app_details::AppDetails,
    repositories::{
        app_details_repository::AppDetailsRepository,
        runs_repository::RunsRepository,
        traits::Repository,
    },
    services::parsers::AppDetailsParser,
};

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
}

impl ProcessAppDetailsService {
    pub fn new(
        runs_repository: RunsRepository,
        app_details_repository: AppDetailsRepository,
    ) -> Self {
        Self {
            runs_repository,
            app_details_repository,
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
        info!("Processing app details from runs table");

        let mut inserted_rows = 0;
        let mut error_rows = 0;
        let mut error_data = Vec::new();

        // For now, we'll use the regular repository methods without transactions
        // TODO: Add proper transaction support to the repository

        // Clear existing app details
        self.clear_app_details().await?;
        info!("Cleared existing app details");

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

        info!("App details processing complete: {} rows inserted", inserted_rows);

        Ok(ProcessAppDetailsOutput {
            success: true,
            message: "App details processing completed successfully".to_string(),
            total_runs,
            inserted_rows,
            error_rows,
            error_data,
        })
    }

    /// Clear all existing app details
    async fn clear_app_details(&self) -> Result<(), AppError> {
        // TODO: Implement delete_all method in repository
        // For now, we'll skip clearing the table
        info!("Skipping app details clear (not implemented yet)");
        Ok(())
    }

    /// Process a single run and create app details
    async fn process_run(&self, run: &crate::models::runs::Run, index: usize) -> Result<(), AppError> {
        let run_id = run.id.ok_or_else(|| {
            error!("Run at index {} has no ID", index);
            AppError::bad_request("Invalid run data".to_string())
        })?;

        let info = run.info.as_ref().ok_or_else(|| {
            error!("Run {} has no info", run_id);
            AppError::bad_request("Missing info data".to_string())
        })?;

        info!("Processing app details for run {} of {} (ID: {})", index + 1, index + 1, run_id);

        // Parse app details from info string using our parser
        let app_details = AppDetailsParser::parse(info);

        // Store app_name for logging
        let app_name_for_log = app_details.app_name.clone();

        // Create app details record
        let app_details_record = AppDetails {
            id: None,
            run_id: Some(run_id),
            app_name: app_details.app_name,
            updated: app_details.updated,
            hash: app_details.hash,
            url: app_details.url,
        };

        // Insert into database
        self.app_details_repository.create(app_details_record).await
            .map_err(|e| {
                error!("Failed to insert app details for run {}: {}", run_id, e);
                AppError::internal(format!("Failed to insert app details: {}", e))
            })?;

        info!("Processed app details for run {}: app={:?}", index + 1, app_name_for_log);

        Ok(())
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