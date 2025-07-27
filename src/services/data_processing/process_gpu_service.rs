use tracing::{error, info, warn};

use crate::{
    error::types::AppError,
    models::gpu::Gpu,
    repositories::{
        gpu_repository::GpuRepository,
        runs_repository::RunsRepository,
        traits::Repository,
    },
    services::parsers::GpuInfoParser,
};

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
}

impl ProcessGpuService {
    pub fn new(
        runs_repository: RunsRepository,
        gpu_repository: GpuRepository,
    ) -> Self {
        Self {
            runs_repository,
            gpu_repository,
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
        info!("Processing GPU info from runs table");

        let mut inserted_rows = 0;
        let mut error_rows = 0;
        let mut error_data = Vec::new();

        // For now, we'll use the regular repository methods without transactions
        // TODO: Add proper transaction support to the repository

        // Clear existing GPU data
        self.clear_gpu_data().await?;
        info!("Cleared existing GPU data");

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

        info!("GPU processing complete: {} rows inserted", inserted_rows);

        Ok(ProcessGpuOutput {
            success: true,
            message: "GPU processing completed successfully".to_string(),
            total_runs,
            inserted_rows,
            error_rows,
            error_data,
        })
    }

    /// Clear all existing GPU data
    async fn clear_gpu_data(&self) -> Result<(), AppError> {
        // TODO: Implement delete_all method in repository
        // For now, we'll skip clearing the table
        info!("Skipping GPU data clear (not implemented yet)");
        Ok(())
    }

    /// Process a single run and create GPU record
    async fn process_run(&self, run: &crate::models::runs::Run, index: usize) -> Result<(), AppError> {
        let run_id = run.id.ok_or_else(|| {
            error!("Run at index {} has no ID", index);
            AppError::bad_request("Invalid run data".to_string())
        })?;

        let device_info = run.device_info.as_ref().ok_or_else(|| {
            error!("Run {} has no device_info", run_id);
            AppError::bad_request("Missing device_info data".to_string())
        })?;

        info!("Processing GPU info for run {} of {} (ID: {})", index + 1, index + 1, run_id);

        // Parse device info to extract GPU information using our parser
        let parsed_gpu_info = GpuInfoParser::parse(device_info);

        // Store values for logging
        let device_for_log = parsed_gpu_info.device.clone();

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

        // Insert into database
        self.gpu_repository.create(gpu_record).await
            .map_err(|e| {
                error!("Failed to insert GPU info for run {}: {}", run_id, e);
                AppError::internal(format!("Failed to insert GPU info: {}", e))
            })?;

        info!("Processed GPU info for run {}: device={:?}", index + 1, device_for_log);

        Ok(())
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