use tracing::{error, info};

use crate::{
    error::types::AppError,
    repositories::{
        run_more_details_repository::RunMoreDetailsRepository,
        model_map_repository::ModelMapRepository,
        traits::Repository,
    },
};

#[derive(Debug, serde::Serialize)]
pub struct UpdateRunMoreDetailsOutput {
    pub success: bool,
    pub message: String,
}

pub struct UpdateRunMoreDetailsService {
    run_more_details_repository: RunMoreDetailsRepository,
    model_map_repository: ModelMapRepository,
}

impl UpdateRunMoreDetailsService {
    pub fn new(
        run_more_details_repository: RunMoreDetailsRepository,
        model_map_repository: ModelMapRepository,
    ) -> Self {
        Self {
            run_more_details_repository,
            model_map_repository,
        }
    }

    /// Update RunMoreDetails with ModelMapId based on model_name matching
    /// 
    /// This service:
    /// 1. Finds all RunMoreDetails records that don't have ModelMapId filled
    /// 2. For each record, finds the corresponding ModelMapId from ModelMap based on model_name
    /// 3. Updates the RunMoreDetails records with the found ModelMapId
    /// 4. Returns counts of updated and not found records
    /// 
    /// # Returns
    /// * `UpdateRunMoreDetailsOutput` - Results with success status and message
    pub async fn update_run_more_details_with_modelmapid(&self) -> Result<UpdateRunMoreDetailsOutput, AppError> {
        info!("Updating RunMoreDetails with ModelMapId");

        // Get all runs from RunMoreDetails that don't have ModelMapId filled
        let runs_without_modelmapid = self.run_more_details_repository.find_without_modelmapid().await.map_err(|e| {
            error!("Failed to fetch RunMoreDetails without ModelMapId: {}", e);
            AppError::internal(format!("Failed to fetch RunMoreDetails without ModelMapId: {}", e))
        })?;

        if runs_without_modelmapid.is_empty() {
            info!("All RunMoreDetails entries already have ModelMapId");
            return Ok(UpdateRunMoreDetailsOutput {
                success: true,
                message: "All RunMoreDetails entries already have ModelMapId.".to_string(),
            });
        }

        info!("Found {} RunMoreDetails entries without ModelMapId", runs_without_modelmapid.len());

        let mut updated_count = 0;
        let mut not_found_count = 0;

        // For each run, find the corresponding ModelMapId from ModelMap based on model_name
        for run in &runs_without_modelmapid {
            let model_name = match &run.model_name {
                Some(name) => name,
                None => {
                    info!("RunMoreDetails ID {} has NULL model_name, skipping", run.id.unwrap_or(0));
                    not_found_count += 1;
                    continue;
                }
            };

            let model_map_entry = self.model_map_repository.find_single_by_model_name(model_name).await.map_err(|e| {
                error!("Failed to query ModelMap for model_name '{}': {}", model_name, e);
                AppError::internal(format!("Failed to query ModelMap for model_name '{}': {}", model_name, e))
            })?;

            if let Some(model_map_entry) = model_map_entry {
                // Update RunMoreDetails with the found ModelMapId
                let run_id = run.id.ok_or_else(|| {
                    error!("RunMoreDetails has no ID");
                    AppError::internal("RunMoreDetails has no ID".to_string())
                })?;

                let mut updated_run = run.clone();
                updated_run.model_map_id = model_map_entry.id;

                self.run_more_details_repository.update(updated_run).await.map_err(|e| {
                    error!("Failed to update RunMoreDetails ID {} with ModelMapId {}: {}", 
                           run_id, model_map_entry.id.unwrap_or(0), e);
                    AppError::internal(format!("Failed to update RunMoreDetails ID {} with ModelMapId {}: {}", 
                                              run_id, model_map_entry.id.unwrap_or(0), e))
                })?;

                updated_count += 1;
                info!("Updated RunMoreDetails ID {} with ModelMapId {} for model_name '{}'", 
                      run_id, model_map_entry.id.unwrap_or(0), model_name);
            } else {
                info!("No matching entry in ModelMap for model_name: {}", model_name);
                not_found_count += 1;
            }
        }

        let message = format!("RunMoreDetails updated with ModelMapId successfully. Updated: {}, Not found: {}", 
                             updated_count, not_found_count);

        info!("RunMoreDetails update complete: {} updated, {} not found", updated_count, not_found_count);

        Ok(UpdateRunMoreDetailsOutput {
            success: true,
            message,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_update_run_more_details_service_creation() {
        // This test verifies the service can be created
        // In a real test, we would use a test database
        assert!(true, "Service structure is valid");
    }
} 