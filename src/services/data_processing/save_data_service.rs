use serde::{Deserialize, Serialize};
use tracing::{error, info, warn};

use crate::{
    error::types::AppError,
    models::runs::Run,
    repositories::{
        runs_repository::RunsRepository,
        traits::Repository,
    },
    handlers::validation::RunData,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct SaveDataOutput {
    pub success: bool,
    pub message: String,
    pub total_rows: usize,
    pub inserted_rows: usize,
    pub error_rows: usize,
    pub error_data: Vec<String>,
}

pub struct SaveDataService {
    runs_repository: RunsRepository,
}

impl SaveDataService {
    pub fn new(runs_repository: RunsRepository) -> Self {
        Self { runs_repository }
    }

    pub async fn save_data(&self, file_content: Vec<u8>) -> Result<SaveDataOutput, AppError> {
        info!("Starting save data processing");

        // Parse JSON data from file content
        let data: Vec<RunData> = serde_json::from_slice(&file_content)
            .map_err(|e| {
                error!("Failed to parse JSON data: {}", e);
                AppError::bad_request(format!("Invalid JSON format: {}", e))
            })?;

        let total_rows = data.len();
        let mut inserted_rows = 0;
        let mut error_rows = 0;
        let mut error_data = Vec::new();

        // For now, we'll use the regular repository methods without transactions
        // TODO: Add proper transaction support to the repository

        // Clear existing data
        self.clear_runs_table().await?;

        // Process each row
        for (index, row) in data.iter().enumerate() {
            match self.process_row(row).await {
                Ok(_) => {
                    inserted_rows += 1;
                    if index % 100 == 0 {
                        info!("Processed {} rows", index + 1);
                    }
                }
                Err(e) => {
                    error_rows += 1;
                    let error_msg = format!("Row {}: {}", index + 1, e);
                    error_data.push(error_msg);
                    warn!("Failed to process row {}: {}", index + 1, e);
                }
            }
        }

        info!("Save data processing completed. Total: {}, Inserted: {}, Errors: {}", 
              total_rows, inserted_rows, error_rows);

        Ok(SaveDataOutput {
            success: true,
            message: "Data processed successfully".to_string(),
            total_rows,
            inserted_rows,
            error_rows,
            error_data,
        })
    }

    async fn clear_runs_table(&self) -> Result<(), AppError> {
        // TODO: Implement delete_all method in repository
        // For now, we'll skip clearing the table
        info!("Skipping table clear (not implemented yet)");
        Ok(())
    }

    async fn process_row(&self, row: &RunData) -> Result<(), AppError> {
        // Convert RunData to Run model
        let run = Run {
            id: None, // Will be set by database
            timestamp: Some(row.timestamp.clone()),
            vram_usage: Some(row.vram_usage.clone()),
            info: Some(row.info.clone()),
            system_info: Some(row.system_info.clone()),
            model_info: Some(row.model_info.clone()),
            device_info: Some(row.device_info.clone()),
            xformers: Some(row.xformers.clone()),
            model_name: Some(row.model_name.clone()),
            user: Some(row.user.clone()),
            notes: Some(row.notes.clone()),
        };

        // Insert the run
        self.runs_repository.create(run).await
            .map_err(|e| {
                AppError::internal(format!("Failed to insert run: {}", e))
            })?;

        Ok(())
    }
} 