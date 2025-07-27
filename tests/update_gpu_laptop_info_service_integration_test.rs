use sqlx::SqlitePool;

use sd_its_benchmark::{
    models::{gpu::Gpu, runs::Run},
    repositories::{
        gpu_repository::GpuRepository,
        runs_repository::RunsRepository,
        traits::Repository,
    },
    services::data_processing::update_gpu_laptop_info_service::UpdateGpuLaptopInfoService,
};

/// Integration test for Update GPU Laptop Info Service
/// 
/// This test:
/// 1. Sets up a test database with sample GPU data
/// 2. Calls the Update GPU Laptop Info Service
/// 3. Verifies the results match expected output
/// 4. Tests error handling and edge cases
#[tokio::test]
async fn test_update_gpu_laptop_info_service_integration() -> Result<(), Box<dyn std::error::Error>> {
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
    let service = UpdateGpuLaptopInfoService::new(gpu_repository);
    
    // Call the service
    let result = service.update_gpu_laptop_info().await?;
    
    // Verify the result
    assert!(result.success, "Service should return success");
    assert_eq!(result.total_updates, 4, "Should update 4 GPUs");
    assert_eq!(result.laptop_only_updates, 2, "Should have 2 laptop GPUs");
    
    // Verify GPU records were updated
    let gpu_repo_for_verify = GpuRepository::new(pool.clone());
    let updated_gpus = gpu_repo_for_verify.find_all().await?;
    

    
    verify_gpu_laptop_updates(&updated_gpus).await?;
    
    println!("✅ Update GPU Laptop Info Service integration test passed!");
    Ok(())
}

/// Test with empty database
#[tokio::test]
async fn test_update_gpu_laptop_info_service_empty_database() -> Result<(), Box<dyn std::error::Error>> {
    // Setup test database
    let pool = setup_test_database().await?;
    
    // Create service
    let gpu_repository = GpuRepository::new(pool.clone());
    let service = UpdateGpuLaptopInfoService::new(gpu_repository);
    
    // Call the service with empty database
    let result = service.update_gpu_laptop_info().await?;
    
    // Verify the result
    assert!(result.success, "Service should return success");
    assert_eq!(result.total_updates, 0, "Should update 0 GPUs");
    assert_eq!(result.laptop_only_updates, 0, "Should have 0 laptop GPUs");
    
    println!("✅ Update GPU Laptop Info Service empty database test passed!");
    Ok(())
}

/// Test error handling with invalid data
#[tokio::test]
async fn test_update_gpu_laptop_info_service_error_handling() -> Result<(), Box<dyn std::error::Error>> {
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
    let service = UpdateGpuLaptopInfoService::new(gpu_repository);
    
    // Call the service
    let result = service.update_gpu_laptop_info().await?;
    
    // Verify the result shows some updates but may have errors
    assert!(result.success, "Service should return success even with some errors");
    assert!(result.total_updates > 0, "Should update some GPUs");
    assert!(result.laptop_only_updates > 0, "Should have some laptop GPUs");
    
    println!("✅ Update GPU Laptop Info Service error handling test passed!");
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

/// Create test GPUs with mix of laptop and desktop devices
fn create_test_gpus() -> Vec<Gpu> {
    vec![
        Gpu {
            id: None,
            run_id: Some(1),
            device: Some("NVIDIA GeForce RTX 4090".to_string()),
            driver: Some("470.82.01".to_string()),
            gpu_chip: Some("RTX 4090".to_string()),
            brand: Some("nvidia".to_string()),
            is_laptop: None, // Will be populated by the service
        },
        Gpu {
            id: None,
            run_id: Some(2),
            device: Some("NVIDIA GeForce RTX 4090 Laptop".to_string()),
            driver: Some("535.98.01".to_string()),
            gpu_chip: Some("RTX 4090".to_string()),
            brand: Some("nvidia".to_string()),
            is_laptop: None, // Will be populated by the service
        },
        Gpu {
            id: None,
            run_id: Some(3),
            device: Some("AMD Radeon RX 6800".to_string()),
            driver: Some("23.12.1".to_string()),
            gpu_chip: Some("RX 6800".to_string()),
            brand: Some("amd".to_string()),
            is_laptop: None, // Will be populated by the service
        },
        Gpu {
            id: None,
            run_id: Some(4),
            device: Some("AMD Radeon RX 6800M".to_string()),
            driver: Some("23.12.1".to_string()),
            gpu_chip: Some("RX 6800M".to_string()),
            brand: Some("amd".to_string()),
            is_laptop: None, // Will be populated by the service
        },
    ]
}

/// Create test GPUs with some invalid data for error testing
fn create_test_gpus_with_errors() -> Vec<Gpu> {
    vec![
        // Valid desktop GPU
        Gpu {
            id: None,
            run_id: Some(1),
            device: Some("NVIDIA GeForce RTX 4090".to_string()),
            driver: Some("470.82.01".to_string()),
            gpu_chip: Some("RTX 4090".to_string()),
            brand: Some("nvidia".to_string()),
            is_laptop: None,
        },
        // GPU with missing device (should cause error)
        Gpu {
            id: None,
            run_id: Some(2),
            device: None, // This will cause an error
            driver: Some("535.98.01".to_string()),
            gpu_chip: Some("RTX 4080".to_string()),
            brand: Some("nvidia".to_string()),
            is_laptop: None,
        },
        // Valid laptop GPU
        Gpu {
            id: None,
            run_id: Some(3),
            device: Some("NVIDIA GeForce RTX 4090 Laptop".to_string()),
            driver: Some("535.98.01".to_string()),
            gpu_chip: Some("RTX 4090".to_string()),
            brand: Some("nvidia".to_string()),
            is_laptop: None,
        },
        // Valid mobile GPU
        Gpu {
            id: None,
            run_id: Some(4),
            device: Some("AMD Radeon RX 6800M".to_string()),
            driver: Some("23.12.1".to_string()),
            gpu_chip: Some("RX 6800M".to_string()),
            brand: Some("amd".to_string()),
            is_laptop: None,
        },
    ]
}

/// Verify that GPU laptop info updates were applied correctly
async fn verify_gpu_laptop_updates(
    gpu_records: &[Gpu],
) -> Result<(), Box<dyn std::error::Error>> {
    assert_eq!(gpu_records.len(), 4, "Should have 4 GPU records");
    
    // Verify each GPU has laptop info assigned
    for gpu in gpu_records {
        assert!(gpu.is_laptop.is_some(), "Each GPU should have laptop info assigned");
        
        let is_laptop = gpu.is_laptop.unwrap();
        let device = gpu.device.as_ref().unwrap();
        
        // Verify laptop detection matches device
        let expected_laptop = device.contains("Laptop") || 
                             device.contains("Mobile") ||
                             (device.contains("AMD") && device.ends_with("M"));
        
        if expected_laptop {
            assert!(is_laptop, "Laptop/Mobile device should be marked as laptop");
        } else {
            assert!(!is_laptop, "Desktop device should not be marked as laptop");
        }
    }
    
    // Count laptop vs desktop GPUs
    let laptop_count = gpu_records.iter()
        .filter(|g| g.is_laptop.unwrap())
        .count();
    let desktop_count = gpu_records.iter()
        .filter(|g| !g.is_laptop.unwrap())
        .count();
    
    assert_eq!(laptop_count, 2, "Should have 2 laptop GPUs");
    assert_eq!(desktop_count, 2, "Should have 2 desktop GPUs");
    
    // Verify specific devices are correctly classified
    let laptop_devices: Vec<&String> = gpu_records.iter()
        .filter(|g| g.is_laptop.unwrap())
        .filter_map(|g| g.device.as_ref())
        .collect();
    
    let desktop_devices: Vec<&String> = gpu_records.iter()
        .filter(|g| !g.is_laptop.unwrap())
        .filter_map(|g| g.device.as_ref())
        .collect();
    
    assert!(laptop_devices.contains(&&"NVIDIA GeForce RTX 4090 Laptop".to_string()), "Should contain laptop GPU");
    assert!(laptop_devices.contains(&&"AMD Radeon RX 6800M".to_string()), "Should contain mobile GPU");
    assert!(desktop_devices.contains(&&"NVIDIA GeForce RTX 4090".to_string()), "Should contain desktop GPU");
    assert!(desktop_devices.contains(&&"AMD Radeon RX 6800".to_string()), "Should contain desktop GPU");
    
    Ok(())
} 