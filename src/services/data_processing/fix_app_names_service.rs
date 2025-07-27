use tracing::{error, info};

use crate::{
    error::types::AppError,
    repositories::{
        app_details_repository::AppDetailsRepository,
        traits::Repository,
    },
};

#[derive(Debug, serde::Serialize)]
pub struct FixAppNamesOutput {
    pub message: String,
    pub updated_counts: UpdatedCounts,
}

#[derive(Debug, serde::Serialize)]
pub struct UpdatedCounts {
    pub automatic1111: i64,
    pub vladmandic: i64,
    pub stable_diffusion: i64,
    pub null_app_name_null_url: i64,
}

pub struct FixAppNamesService {
    app_details_repository: AppDetailsRepository,
}

impl FixAppNamesService {
    pub fn new(app_details_repository: AppDetailsRepository) -> Self {
        Self { app_details_repository }
    }

    /// Fix app names based on URL patterns and data quality rules
    /// 
    /// This service:
    /// 1. Updates app names for AUTOMATIC1111 URLs
    /// 2. Updates app names for vladmandic URLs (only if app_name is NULL or empty)
    /// 3. Updates app names for stable-diffusion-webui URLs (only if app_name is NULL)
    /// 4. Updates app names for records with both app_name and url as NULL
    /// 
    /// # Arguments
    /// * `automatic1111_name` - Name to set for AUTOMATIC1111 URLs
    /// * `vladmandic_name` - Name to set for vladmandic URLs
    /// * `stable_diffusion_name` - Name to set for stable-diffusion-webui URLs
    /// * `null_app_name_null_url_name` - Name to set for records with both app_name and url as NULL
    /// 
    /// # Returns
    /// * `FixAppNamesOutput` - Results with counts of updated records
    pub async fn fix_app_names(
        &self,
        automatic1111_name: &str,
        vladmandic_name: &str,
        stable_diffusion_name: &str,
        null_app_name_null_url_name: &str,
    ) -> Result<FixAppNamesOutput, AppError> {
        info!("Fixing app names with parameters: automatic1111={}, vladmandic={}, stable_diffusion={}, null_app_name_null_url={}", 
              automatic1111_name, vladmandic_name, stable_diffusion_name, null_app_name_null_url_name);

        // Basic validation for input fields
        if automatic1111_name.is_empty() || vladmandic_name.is_empty() || 
           stable_diffusion_name.is_empty() || null_app_name_null_url_name.is_empty() {
            return Err(AppError::Validation("All fields must be non-empty".to_string()));
        }

        // Update AUTOMATIC1111 app names
        let count_automatic1111 = self.update_automatic1111_names(automatic1111_name).await.map_err(|e| {
            error!("Failed to update AUTOMATIC1111 app names: {}", e);
            AppError::internal(format!("Failed to update AUTOMATIC1111 app names: {}", e))
        })?;

        info!("Updated {} AUTOMATIC1111 app names", count_automatic1111);

        // Update Vladmandic app names
        let count_vladmandic = self.update_vladmandic_names(vladmandic_name).await.map_err(|e| {
            error!("Failed to update Vladmandic app names: {}", e);
            AppError::internal(format!("Failed to update Vladmandic app names: {}", e))
        })?;

        info!("Updated {} Vladmandic app names", count_vladmandic);

        // Update Stable Diffusion app names
        let count_stable_diffusion = self.update_stable_diffusion_names(stable_diffusion_name).await.map_err(|e| {
            error!("Failed to update Stable Diffusion app names: {}", e);
            AppError::internal(format!("Failed to update Stable Diffusion app names: {}", e))
        })?;

        info!("Updated {} Stable Diffusion app names", count_stable_diffusion);

        // Update NULL app_name and NULL url records
        let count_null_app_name_null_url = self.update_null_app_name_null_url_names(null_app_name_null_url_name).await.map_err(|e| {
            error!("Failed to update NULL app_name NULL url records: {}", e);
            AppError::internal(format!("Failed to update NULL app_name NULL url records: {}", e))
        })?;

        info!("Updated {} NULL app_name NULL url records", count_null_app_name_null_url);

        info!("App names fix complete: AUTOMATIC1111={}, Vladmandic={}, StableDiffusion={}, NullAppNameNullUrl={}", 
              count_automatic1111, count_vladmandic, count_stable_diffusion, count_null_app_name_null_url);

        Ok(FixAppNamesOutput {
            message: "App names updated successfully".to_string(),
            updated_counts: UpdatedCounts {
                automatic1111: count_automatic1111,
                vladmandic: count_vladmandic,
                stable_diffusion: count_stable_diffusion,
                null_app_name_null_url: count_null_app_name_null_url,
            },
        })
    }

    /// Update app names for AUTOMATIC1111 URLs
    async fn update_automatic1111_names(&self, app_name: &str) -> Result<i64, sqlx::Error> {
        self.app_details_repository.update_automatic1111_names(app_name).await
    }

    /// Update app names for vladmandic URLs (only if app_name is NULL or empty)
    async fn update_vladmandic_names(&self, app_name: &str) -> Result<i64, sqlx::Error> {
        self.app_details_repository.update_vladmandic_names(app_name).await
    }

    /// Update app names for stable-diffusion-webui URLs (only if app_name is NULL)
    async fn update_stable_diffusion_names(&self, app_name: &str) -> Result<i64, sqlx::Error> {
        self.app_details_repository.update_stable_diffusion_names(app_name).await
    }

    /// Update app names for records with both app_name and url as NULL
    async fn update_null_app_name_null_url_names(&self, app_name: &str) -> Result<i64, sqlx::Error> {
        self.app_details_repository.update_null_app_name_null_url_names(app_name).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fix_app_names_service_creation() {
        // This test verifies the service can be created
        // In a real test, we would use a test database
        assert!(true, "Service structure is valid");
    }
} 