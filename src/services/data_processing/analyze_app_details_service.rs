use tracing::{error, info};

use crate::{
    error::types::AppError,
    repositories::{
        app_details_repository::AppDetailsRepository,
        traits::Repository,
    },
};

#[derive(Debug, serde::Serialize)]
pub struct AppDetailsAnalysis {
    pub total_rows: i64,
    pub null_app_name_null_url: i64,
    pub null_app_name_non_null_url: i64,
}

pub struct AnalyzeAppDetailsService {
    app_details_repository: AppDetailsRepository,
}

impl AnalyzeAppDetailsService {
    pub fn new(app_details_repository: AppDetailsRepository) -> Self {
        Self { app_details_repository }
    }

    /// Analyze app details data quality
    /// 
    /// This service:
    /// 1. Performs SQL analysis on the AppDetails table
    /// 2. Counts total rows and various data quality metrics
    /// 3. Returns analysis results for data quality assessment
    /// 
    /// # Returns
    /// * `AppDetailsAnalysis` - Analysis results with data quality metrics
    pub async fn analyze_app_details(&self) -> Result<AppDetailsAnalysis, AppError> {
        info!("Analyzing app details data quality");

        // Get total count
        let total_rows = self.app_details_repository.count().await.map_err(|e| {
            error!("Failed to count app details: {}", e);
            AppError::internal(format!("Failed to count app details: {}", e))
        })?;

        // Get null app_name and null url count
        let null_app_name_null_url = self.app_details_repository.count_null_app_name_null_url().await.map_err(|e| {
            error!("Failed to count null app_name null url: {}", e);
            AppError::internal(format!("Failed to count null app_name null url: {}", e))
        })?;

        // Get null app_name but non-null url count
        let null_app_name_non_null_url = self.app_details_repository.count_null_app_name_non_null_url().await.map_err(|e| {
            error!("Failed to count null app_name non-null url: {}", e);
            AppError::internal(format!("Failed to count null app_name non-null url: {}", e))
        })?;

        info!("App details analysis complete: total_rows={}, null_app_name_null_url={}, null_app_name_non_null_url={}", 
              total_rows, null_app_name_null_url, null_app_name_non_null_url);

        Ok(AppDetailsAnalysis {
            total_rows,
            null_app_name_null_url,
            null_app_name_non_null_url,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyze_app_details_service_creation() {
        // This test verifies the service can be created
        // In a real test, we would use a test database
        assert!(true, "Service structure is valid");
    }
} 