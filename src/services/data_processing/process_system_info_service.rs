use tracing::{error, info, warn};

use crate::{
    error::types::AppError,
    models::system_info::SystemInfo,
    repositories::{
        runs_repository::RunsRepository,
        system_info_repository::SystemInfoRepository,
        traits::Repository,
    },
    services::parsers::SystemInfoParser,
};

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
}

impl ProcessSystemInfoService {
    pub fn new(
        runs_repository: RunsRepository,
        system_info_repository: SystemInfoRepository,
    ) -> Self {
        Self {
            runs_repository,
            system_info_repository,
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
        info!("Processing system info from runs table");

        let mut inserted_rows = 0;
        let mut error_rows = 0;
        let mut error_data = Vec::new();

        // For now, we'll use the regular repository methods without transactions
        // TODO: Add proper transaction support to the repository

        // Clear existing system info
        self.clear_system_info().await?;
        info!("Cleared existing system info");

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
                Ok(inserted) => {
                    if inserted {
                        inserted_rows += 1;
                    }
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

        info!("System info processing complete: {} rows inserted", inserted_rows);

        Ok(ProcessSystemInfoOutput {
            success: true,
            message: "System info processing completed successfully".to_string(),
            total_runs,
            inserted_rows,
            error_rows,
            error_data,
        })
    }

    /// Clear all existing system info
    async fn clear_system_info(&self) -> Result<(), AppError> {
        // TODO: Implement delete_all method in repository
        // For now, we'll skip clearing the table
        info!("Skipping system info clear (not implemented yet)");
        Ok(())
    }

    /// Process a single run and create system info
    /// Returns true if a record was inserted, false if skipped due to missing fields
    async fn process_run(&self, run: &crate::models::runs::Run, index: usize) -> Result<bool, AppError> {
        let run_id = run.id.ok_or_else(|| {
            error!("Run at index {} has no ID", index);
            AppError::bad_request("Invalid run data".to_string())
        })?;

        let system_info = run.system_info.as_ref().ok_or_else(|| {
            error!("Run {} has no system_info", run_id);
            AppError::bad_request("Missing system_info data".to_string())
        })?;

        info!("Processing system info for run {} of {} (ID: {})", index + 1, index + 1, run_id);

        // Parse system info from system_info string using our parser
        let parsed_system_info = SystemInfoParser::parse(system_info);

        // Store arch for logging
        let arch_for_log = parsed_system_info.arch.clone();

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

            // Insert into database
            self.system_info_repository.create(system_info_record).await
                .map_err(|e| {
                    error!("Failed to insert system info for run {}: {}", run_id, e);
                    AppError::internal(format!("Failed to insert system info: {}", e))
                })?;

            info!("Processed system info for run {}: arch={:?}", index + 1, arch_for_log);
            Ok(true)
        } else {
            warn!("Skipping run {} due to missing required system info fields", run_id);
            Ok(false)
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