use sqlx::SqlitePool;

use sd_its_benchmark::{
    models::{gpu::Gpu, runs::Run},
    repositories::{
        gpu_repository::GpuRepository,
        runs_repository::RunsRepository,
        traits::Repository,
    },
    services::data_processing::update_gpu_brands_service::UpdateGpuBrandsService,
};

/// Integration test for Update GPU Brands Service
/// 
/// This test:
/// 1. Sets up a test database with sample GPU data
/// 2. Calls the Update GPU Brands Service
/// 3. Verifies the results match expected output
/// 4. Tests error handling and edge cases
#[tokio::test]
async fn test_update_gpu_brands_service_integration() -> Result<(), Box<dyn std::error::Error>> {
    // Setup test database
    let pool = setup_test_database().await?;
    
    // Create required runs first (for foreign key constraints)
    create_required_runs(&pool).await?;
    
    // Insert test GPU data
    let test_gpus = create_test_gpus();
    let gpu_repo_for_insert = GpuRepository::new(pool.clone());
    for gpu in test_gpus {
        gpu_repo_for_insert.create(gpu).await?;
    }
    
    // Verify test data was inserted
    let gpu_repo_for_check = GpuRepository::new(pool.clone());
    let all_gpus = gpu_repo_for_check.find_all().await?;
    assert_eq!(all_gpus.len(), 4, "Should have 4 test GPUs");
    
    // Create service
    let gpu_repository = GpuRepository::new(pool.clone());
    let service = UpdateGpuBrandsService::new(gpu_repository);
    
    // Call the service
    let result = service.update_gpu_brands().await?;
    
    // Verify the result
    assert!(result.success, "Service should return success");
    assert_eq!(result.total_updates, 4, "Should update 4 GPUs");
    
    // Verify brand counts
    let brand_counts: std::collections::HashMap<_, _> = result.update_counts_by_brand
        .iter()
        .map(|bc| (bc.brand_name.as_str(), bc.count))
        .collect();
    
    assert_eq!(brand_counts.get("Nvidia").unwrap_or(&0), &3, "Should have 3 NVIDIA GPUs");
    assert_eq!(brand_counts.get("Amd").unwrap_or(&0), &1, "Should have 1 AMD GPU");
    assert_eq!(brand_counts.get("Intel").unwrap_or(&0), &0, "Should have 0 Intel GPUs");
    assert_eq!(brand_counts.get("Unknown").unwrap_or(&0), &0, "Should have 0 Unknown GPUs");
    
    // Verify GPU records were updated
    let gpu_repo_for_verify = GpuRepository::new(pool.clone());
    let updated_gpus = gpu_repo_for_verify.find_all().await?;
    verify_gpu_brand_updates(&updated_gpus).await?;
    
    println!("✅ Update GPU Brands Service integration test passed!");
    Ok(())
}

/// Test with empty database
#[tokio::test]
async fn test_update_gpu_brands_service_empty_database() -> Result<(), Box<dyn std::error::Error>> {
    // Setup test database
    let pool = setup_test_database().await?;
    
    // Create service
    let gpu_repository = GpuRepository::new(pool.clone());
    let service = UpdateGpuBrandsService::new(gpu_repository);
    
    // Call the service with empty database
    let result = service.update_gpu_brands().await?;
    
    // Verify the result
    assert!(result.success, "Service should return success");
    assert_eq!(result.total_updates, 0, "Should update 0 GPUs");
    
    // Verify all brand counts are 0
    let brand_counts: std::collections::HashMap<_, _> = result.update_counts_by_brand
        .iter()
        .map(|bc| (bc.brand_name.as_str(), bc.count))
        .collect();
    
    assert_eq!(brand_counts.get("Nvidia").unwrap_or(&0), &0, "Should have 0 NVIDIA GPUs");
    assert_eq!(brand_counts.get("Amd").unwrap_or(&0), &0, "Should have 0 AMD GPUs");
    assert_eq!(brand_counts.get("Intel").unwrap_or(&0), &0, "Should have 0 Intel GPUs");
    assert_eq!(brand_counts.get("Unknown").unwrap_or(&0), &0, "Should have 0 Unknown GPUs");
    
    println!("✅ Update GPU Brands Service empty database test passed!");
    Ok(())
}

/// Test error handling with invalid data
#[tokio::test]
async fn test_update_gpu_brands_service_error_handling() -> Result<(), Box<dyn std::error::Error>> {
    // Setup test database
    let pool = setup_test_database().await?;
    
    // Create required runs first (for foreign key constraints)
    create_required_runs(&pool).await?;
    
    // Insert test GPU data with some invalid entries
    let test_gpus = create_test_gpus_with_errors();
    let gpu_repo_for_insert = GpuRepository::new(pool.clone());
    for gpu in test_gpus {
        gpu_repo_for_insert.create(gpu).await?;
    }
    
    // Create service
    let gpu_repository = GpuRepository::new(pool.clone());
    let service = UpdateGpuBrandsService::new(gpu_repository);
    
    // Call the service
    let result = service.update_gpu_brands().await?;
    
    // Verify the result shows some updates but may have errors
    assert!(result.success, "Service should return success even with some errors");
    assert!(result.total_updates > 0, "Should update some GPUs");
    
    // Verify brand counts
    let brand_counts: std::collections::HashMap<_, _> = result.update_counts_by_brand
        .iter()
        .map(|bc| (bc.brand_name.as_str(), bc.count))
        .collect();
    
    assert!(brand_counts.get("Nvidia").unwrap_or(&0) > &0, "Should have some NVIDIA GPUs");
    assert!(brand_counts.get("Unknown").unwrap_or(&0) > &0, "Should have some Unknown GPUs");
    
    println!("✅ Update GPU Brands Service error handling test passed!");
    Ok(())
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Setup test database with required tables
async fn setup_test_database() -> Result<SqlitePool, Box<dyn std::error::Error>> {
    // Create in-memory database for testing
    let pool = SqlitePool::connect("sqlite::memory:").await?;
    
    // Run migrations to create tables
    sqlx::migrate!("./migrations").run(&pool).await?;
    
    Ok(pool)
}

/// Create required runs for GPU foreign key constraints
async fn create_required_runs(pool: &SqlitePool) -> Result<(), Box<dyn std::error::Error>> {
    let runs_repository = RunsRepository::new(pool.clone());
    
    // Create 4 runs with IDs 1, 2, 3, 4
    for i in 1..=4 {
        let run = Run {
            id: None,
            timestamp: Some(format!("2024-01-01T{:02}:00:00Z", i + 9)),
            vram_usage: Some("1.5/2.0/1.8".to_string()),
            info: Some("app:test updated:2024-01-01".to_string()),
            system_info: Some("arch:x86_64 cpu:Intel".to_string()),
            model_info: Some("torch:2.0.0 xformers:0.0.22".to_string()),
            device_info: Some("device:NVIDIA driver:470.82.01".to_string()),
            xformers: Some("true".to_string()),
            model_name: Some("test-model".to_string()),
            user: Some("test-user".to_string()),
            notes: Some(format!("Test run {}", i)),
        };
        runs_repository.create(run).await?;
    }
    
    Ok(())
}

/// Create test GPUs with valid device data
fn create_test_gpus() -> Vec<Gpu> {
    vec![
        Gpu {
            id: None,
            run_id: Some(1),
            device: Some("NVIDIA GeForce RTX 4090".to_string()),
            driver: Some("470.82.01".to_string()),
            gpu_chip: Some("RTX 4090".to_string()),
            brand: None, // Will be populated by the service
            is_laptop: None,
        },
        Gpu {
            id: None,
            run_id: Some(2),
            device: Some("NVIDIA GeForce RTX 4080".to_string()),
            driver: Some("535.98.01".to_string()),
            gpu_chip: Some("RTX 4080".to_string()),
            brand: None, // Will be populated by the service
            is_laptop: None,
        },
        Gpu {
            id: None,
            run_id: Some(3),
            device: Some("NVIDIA Quadro RTX 5000".to_string()),
            driver: Some("525.85.05".to_string()),
            gpu_chip: Some("RTX 5000".to_string()),
            brand: None, // Will be populated by the service
            is_laptop: None,
        },
        Gpu {
            id: None,
            run_id: Some(4),
            device: Some("AMD Radeon RX 7900 XTX".to_string()),
            driver: Some("23.12.1".to_string()),
            gpu_chip: Some("RX 7900 XTX".to_string()),
            brand: None, // Will be populated by the service
            is_laptop: None,
        },
    ]
}

/// Create test GPUs with some invalid data for error testing
fn create_test_gpus_with_errors() -> Vec<Gpu> {
    vec![
        // Valid NVIDIA GPU
        Gpu {
            id: None,
            run_id: Some(1),
            device: Some("NVIDIA GeForce RTX 4090".to_string()),
            driver: Some("470.82.01".to_string()),
            gpu_chip: Some("RTX 4090".to_string()),
            brand: None,
            is_laptop: None,
        },
        // GPU with missing device (should cause error)
        Gpu {
            id: None,
            run_id: Some(2),
            device: None, // This will cause an error
            driver: Some("535.98.01".to_string()),
            gpu_chip: Some("RTX 4080".to_string()),
            brand: None,
            is_laptop: None,
        },
        // Unknown GPU
        Gpu {
            id: None,
            run_id: Some(3),
            device: Some("Unknown Graphics Device".to_string()),
            driver: Some("1.0.0".to_string()),
            gpu_chip: Some("Unknown".to_string()),
            brand: None,
            is_laptop: None,
        },
        // Valid NVIDIA GPU
        Gpu {
            id: None,
            run_id: Some(4),
            device: Some("NVIDIA Tesla V100".to_string()),
            driver: Some("450.80.02".to_string()),
            gpu_chip: Some("Tesla V100".to_string()),
            brand: None,
            is_laptop: None,
        },
    ]
}

/// Verify that GPU brand updates were applied correctly
async fn verify_gpu_brand_updates(
    gpu_records: &[Gpu],
) -> Result<(), Box<dyn std::error::Error>> {
    assert_eq!(gpu_records.len(), 4, "Should have 4 GPU records");
    
    // Verify each GPU has a brand assigned
    for gpu in gpu_records {
        assert!(gpu.brand.is_some(), "Each GPU should have a brand assigned");
        
        let brand = gpu.brand.as_ref().unwrap();
        let device = gpu.device.as_ref().unwrap();
        
        // Verify brand matches device
        if device.to_lowercase().contains("nvidia") || 
           device.to_lowercase().contains("geforce") || 
           device.to_lowercase().contains("quadro") ||
           device.to_lowercase().contains("tesla") {
            assert_eq!(brand, "nvidia", "NVIDIA device should have nvidia brand");
        } else if device.to_lowercase().contains("amd") || 
                  device.to_lowercase().contains("radeon") {
            assert_eq!(brand, "amd", "AMD device should have amd brand");
        } else if device.to_lowercase().contains("intel") {
            assert_eq!(brand, "intel", "Intel device should have intel brand");
        } else {
            assert_eq!(brand, "unknown", "Unknown device should have unknown brand");
        }
    }
    
    // Verify specific brands are present
    let brands: Vec<&String> = gpu_records.iter()
        .filter_map(|g| g.brand.as_ref())
        .collect();
    
    assert!(brands.contains(&&"nvidia".to_string()), "Should contain nvidia brand");
    assert!(brands.contains(&&"amd".to_string()), "Should contain amd brand");
    
    // Count brands
    let nvidia_count = brands.iter().filter(|&&b| b == "nvidia").count();
    let amd_count = brands.iter().filter(|&&b| b == "amd").count();
    
    assert_eq!(nvidia_count, 3, "Should have 3 NVIDIA GPUs");
    assert_eq!(amd_count, 1, "Should have 1 AMD GPU");
    
    Ok(())
} 