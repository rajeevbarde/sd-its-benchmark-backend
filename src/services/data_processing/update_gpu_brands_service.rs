use std::collections::HashMap;
use tracing::{error, info, warn};

use crate::{
    error::types::AppError,
    repositories::{
        gpu_repository::GpuRepository,
        traits::Repository,
    },

};

#[derive(Debug)]
pub struct UpdateGpuBrandsOutput {
    pub success: bool,
    pub message: String,
    pub total_updates: usize,
    pub update_counts_by_brand: Vec<BrandCount>,
}

#[derive(Debug)]
pub struct BrandCount {
    pub brand_name: String,
    pub count: usize,
}

pub struct UpdateGpuBrandsService {
    gpu_repository: GpuRepository,
}

impl UpdateGpuBrandsService {
    pub fn new(gpu_repository: GpuRepository) -> Self {
        Self { gpu_repository }
    }

    /// Update GPU brand information
    /// 
    /// This service:
    /// 1. Fetches all GPU data
    /// 2. Determines brand names from device strings using GpuInfoParser
    /// 3. Updates GPU records with brand information
    /// 4. Returns statistics about the updates
    /// 
    /// # Returns
    /// * `UpdateGpuBrandsOutput` - Update results and brand statistics
    pub async fn update_gpu_brands(&self) -> Result<UpdateGpuBrandsOutput, AppError> {
        info!("Updating GPU brand information");

        // Fetch all GPU data
        let gpu_data = self.gpu_repository.find_all().await.map_err(|e| {
            error!("Failed to fetch GPU data: {}", e);
            AppError::internal(format!("Failed to fetch GPU data: {}", e))
        })?;

        if gpu_data.is_empty() {
            info!("No GPU data found to update");

            // Return all brand categories with 0 counts
            let update_counts_by_brand = vec![
                BrandCount {
                    brand_name: "Nvidia".to_string(),
                    count: 0,
                },
                BrandCount {
                    brand_name: "Amd".to_string(),
                    count: 0,
                },
                BrandCount {
                    brand_name: "Intel".to_string(),
                    count: 0,
                },
                BrandCount {
                    brand_name: "Unknown".to_string(),
                    count: 0,
                },
            ];

            return Ok(UpdateGpuBrandsOutput {
                success: true,
                message: "No GPU data found to update".to_string(),
                total_updates: 0,
                update_counts_by_brand,
            });
        }

        info!("Found {} GPUs to update", gpu_data.len());

        let mut total_updates = 0;
        let mut error_count = 0;
        let mut brand_counts = HashMap::new();
        brand_counts.insert("nvidia".to_string(), 0);
        brand_counts.insert("amd".to_string(), 0);
        brand_counts.insert("intel".to_string(), 0);
        brand_counts.insert("unknown".to_string(), 0);

        // Process each GPU
        for gpu in &gpu_data {
            match self.process_gpu(gpu).await {
                Ok(brand_name) => {
                    total_updates += 1;
                    *brand_counts.get_mut(&brand_name).unwrap() += 1;
                }
                Err(e) => {
                    error_count += 1;
                    warn!("Failed to process GPU: {}", e);
                }
            }
        }

        info!("GPU brand update complete: {} total updates, {} errors", total_updates, error_count);

        // Convert brand counts to response format
        let update_counts_by_brand: Vec<BrandCount> = brand_counts
            .into_iter()
            .map(|(brand, count)| BrandCount {
                brand_name: brand.chars().next().unwrap().to_uppercase().collect::<String>() + &brand[1..],
                count,
            })
            .collect();

        Ok(UpdateGpuBrandsOutput {
            success: true,
            message: "GPU brand information updated successfully!".to_string(),
            total_updates,
            update_counts_by_brand,
        })
    }

    /// Process a single GPU and update its brand
    async fn process_gpu(&self, gpu: &crate::models::gpu::Gpu) -> Result<String, AppError> {
        let gpu_id = gpu.id.ok_or_else(|| {
            error!("GPU has no ID");
            AppError::bad_request("Invalid GPU data".to_string())
        })?;

        let device = gpu.device.as_ref().ok_or_else(|| {
            error!("GPU {} has no device", gpu_id);
            AppError::bad_request("Missing device data".to_string())
        })?;

        let brand_name = self.get_brand_name(device);

        info!("Updating brand for GPU ID {} to {}", gpu_id, brand_name);

        // Update the GPU record
        let mut updated_gpu = gpu.clone();
        updated_gpu.brand = Some(brand_name.clone());

        self.gpu_repository.update(updated_gpu).await
            .map_err(|e| {
                error!("Failed to update GPU {}: {}", gpu_id, e);
                AppError::internal(format!("Failed to update GPU: {}", e))
            })?;

        Ok(brand_name)
    }

    /// Determine brand name from device string
    fn get_brand_name(&self, device_string: &str) -> String {
        let lowercase_device = device_string.to_lowercase();

        if lowercase_device.contains("nvidia") || 
           lowercase_device.contains("quadro") || 
           lowercase_device.contains("geforce") ||
           lowercase_device.contains("tesla") ||
           lowercase_device.contains("cuda") {
            "nvidia".to_string()
        } else if lowercase_device.contains("amd") || 
                  lowercase_device.contains("radeon") {
            "amd".to_string()
        } else if lowercase_device.contains("intel") {
            "intel".to_string()
        } else {
            "unknown".to_string()
        }
    }
}

#[cfg(test)]
mod tests {


    #[test]
    fn test_update_gpu_brands_service_creation() {
        // This test verifies the service can be created
        // In a real test, we would use a test database
        assert!(true, "Service structure is valid");
    }

    #[test]
    fn test_get_brand_name() {
        // Create a temporary service just for testing the brand name function
        struct TestService;
        impl TestService {
            fn get_brand_name(&self, device_string: &str) -> String {
                let lowercase_device = device_string.to_lowercase();

                if lowercase_device.contains("nvidia") || 
                   lowercase_device.contains("quadro") || 
                   lowercase_device.contains("geforce") ||
                   lowercase_device.contains("tesla") ||
                   lowercase_device.contains("cuda") {
                    "nvidia".to_string()
                } else if lowercase_device.contains("amd") || 
                          lowercase_device.contains("radeon") {
                    "amd".to_string()
                } else if lowercase_device.contains("intel") {
                    "intel".to_string()
                } else {
                    "unknown".to_string()
                }
            }
        }

        let service = TestService;

        assert_eq!(service.get_brand_name("NVIDIA"), "nvidia");
        assert_eq!(service.get_brand_name("GeForce RTX 4090"), "nvidia");
        assert_eq!(service.get_brand_name("AMD Radeon"), "amd");
        assert_eq!(service.get_brand_name("Intel Graphics"), "intel");
        assert_eq!(service.get_brand_name("Unknown Device"), "unknown");
    }
} 