use sqlx::SqlitePool;

use sd_its_benchmark::{
    models::runs::Run,
    repositories::{
        gpu_repository::GpuRepository,
        runs_repository::RunsRepository,
        traits::Repository,
    },
    services::data_processing::process_gpu_service::ProcessGpuService,
};

/// Integration test for Process GPU Service
/// 
/// This test:
/// 1. Sets up a test database with sample runs data
/// 2. Calls the Process GPU Service
/// 3. Verifies the results match expected output
/// 4. Tests error handling and edge cases
#[tokio::test]
async fn test_process_gpu_service_integration() -> Result<(), Box<dyn std::error::Error>> {
    // Setup test database
    let pool = setup_test_database().await?;
    
    // Insert test data
    let test_runs = create_test_runs();
    let runs_repo_for_insert = RunsRepository::new(pool.clone());
    for run in test_runs {
        runs_repo_for_insert.create(run).await?;
    }
    
    // Verify test data was inserted
    let runs_repo_for_check = RunsRepository::new(pool.clone());
    let all_runs = runs_repo_for_check.find_all().await?;
    assert_eq!(all_runs.len(), 3, "Should have 3 test runs");
    
    // Create service
    let runs_repository = RunsRepository::new(pool.clone());
    let gpu_repository = GpuRepository::new(pool.clone());
    let service = ProcessGpuService::new(runs_repository, gpu_repository);
    
    // Call the service
    let result = service.process_gpu().await?;
    
    // Verify the result
    assert!(result.success, "Service should return success");
    assert_eq!(result.total_runs, 3, "Should process 3 runs");
    assert_eq!(result.inserted_rows, 3, "Should insert 3 GPU records");
    assert_eq!(result.error_rows, 0, "Should have no errors");
    assert!(result.error_data.is_empty(), "Should have no error data");
    
    // Verify GPU records were created
    let gpu_repo_for_check = GpuRepository::new(pool.clone());
    let gpu_records = gpu_repo_for_check.find_all().await?;
    assert_eq!(gpu_records.len(), 3, "Should have 3 GPU records");
    
    // Verify specific GPU records
    verify_gpu_records(&gpu_records).await?;
    
    println!("✅ Process GPU Service integration test passed!");
    Ok(())
}

/// Test error handling with invalid data
#[tokio::test]
async fn test_process_gpu_service_error_handling() -> Result<(), Box<dyn std::error::Error>> {
    // Setup test database
    let pool = setup_test_database().await?;
    
    // Insert test data with some invalid entries
    let test_runs = create_test_runs_with_errors();
    let runs_repo_for_insert = RunsRepository::new(pool.clone());
    for run in test_runs {
        runs_repo_for_insert.create(run).await?;
    }
    
    // Create service
    let runs_repository = RunsRepository::new(pool.clone());
    let gpu_repository = GpuRepository::new(pool.clone());
    let service = ProcessGpuService::new(runs_repository, gpu_repository);
    
    // Call the service
    let result = service.process_gpu().await?;
    
    // Verify the result shows errors
    assert!(result.success, "Service should return success even with some errors");
    assert_eq!(result.total_runs, 4, "Should process 4 runs");
    assert!(result.inserted_rows > 0, "Should insert some GPU records");
    assert!(result.error_rows > 0, "Should have some errors");
    assert!(!result.error_data.is_empty(), "Should have error data");
    
    println!("✅ Process GPU Service error handling test passed!");
    Ok(())
}

/// Test with empty database
#[tokio::test]
async fn test_process_gpu_service_empty_database() -> Result<(), Box<dyn std::error::Error>> {
    // Setup test database
    let pool = setup_test_database().await?;
    
    // Create service
    let runs_repository = RunsRepository::new(pool.clone());
    let gpu_repository = GpuRepository::new(pool.clone());
    let service = ProcessGpuService::new(runs_repository, gpu_repository);
    
    // Call the service with empty database
    let result = service.process_gpu().await?;
    
    // Verify the result
    assert!(result.success, "Service should return success");
    assert_eq!(result.total_runs, 0, "Should process 0 runs");
    assert_eq!(result.inserted_rows, 0, "Should insert 0 GPU records");
    assert_eq!(result.error_rows, 0, "Should have no errors");
    assert!(result.error_data.is_empty(), "Should have no error data");
    
    println!("✅ Process GPU Service empty database test passed!");
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

/// Create test runs with valid GPU data
fn create_test_runs() -> Vec<Run> {
    vec![
        Run {
            id: None,
            timestamp: Some("2024-01-01T10:00:00Z".to_string()),
            vram_usage: Some("1.5/2.0/1.8".to_string()),
            info: Some("app:automatic1111 updated:2024-01-01".to_string()),
            system_info: Some("arch:x86_64 cpu:Intel".to_string()),
            model_info: Some("torch:2.0.0 xformers:0.0.22".to_string()),
            device_info: Some("device:NVIDIA driver:470.82.01 NVIDIA GeForce RTX 4090".to_string()),
            xformers: Some("true".to_string()),
            model_name: Some("test-model".to_string()),
            user: Some("test-user".to_string()),
            notes: Some("Test run 1".to_string()),
        },
        Run {
            id: None,
            timestamp: Some("2024-01-01T11:00:00Z".to_string()),
            vram_usage: Some("2.1/2.3/2.0".to_string()),
            info: Some("app:vladmandic updated:2024-01-02".to_string()),
            system_info: Some("arch:amd64 cpu:AMD".to_string()),
            model_info: Some("torch:2.1.0 xformers:0.0.23".to_string()),
            device_info: Some("device:NVIDIA driver:535.98.01 NVIDIA GeForce RTX 4080".to_string()),
            xformers: Some("false".to_string()),
            model_name: Some("test-model".to_string()),
            user: Some("test-user".to_string()),
            notes: Some("Test run 2".to_string()),
        },
        Run {
            id: None,
            timestamp: Some("2024-01-01T12:00:00Z".to_string()),
            vram_usage: Some("1.0/1.2/1.1".to_string()),
            info: Some("app:stable-diffusion updated:2024-01-03".to_string()),
            system_info: Some("arch:arm64 cpu:Apple".to_string()),
            model_info: Some("torch:1.13.0 xformers:0.0.21".to_string()),
            device_info: Some("device:NVIDIA driver:525.85.05 NVIDIA GeForce RTX 3090".to_string()),
            xformers: Some("true".to_string()),
            model_name: Some("test-model".to_string()),
            user: Some("test-user".to_string()),
            notes: Some("Test run 3".to_string()),
        },
    ]
}

/// Create test runs with some invalid data for error testing
fn create_test_runs_with_errors() -> Vec<Run> {
    vec![
        // Valid run
        Run {
            id: None,
            timestamp: Some("2024-01-01T10:00:00Z".to_string()),
            vram_usage: Some("1.5/2.0/1.8".to_string()),
            info: Some("app:automatic1111 updated:2024-01-01".to_string()),
            system_info: Some("arch:x86_64 cpu:Intel".to_string()),
            model_info: Some("torch:2.0.0 xformers:0.0.22".to_string()),
            device_info: Some("device:NVIDIA driver:470.82.01 NVIDIA GeForce RTX 4090".to_string()),
            xformers: Some("true".to_string()),
            model_name: Some("test-model".to_string()),
            user: Some("test-user".to_string()),
            notes: Some("Valid test run".to_string()),
        },
        // Run with missing device_info (should cause error)
        Run {
            id: None,
            timestamp: Some("2024-01-01T11:00:00Z".to_string()),
            vram_usage: Some("2.1/2.3/2.0".to_string()),
            info: Some("app:vladmandic updated:2024-01-02".to_string()),
            system_info: Some("arch:amd64 cpu:AMD".to_string()),
            model_info: Some("torch:2.1.0 xformers:0.0.23".to_string()),
            device_info: None, // This will cause an error
            xformers: Some("false".to_string()),
            model_name: Some("test-model".to_string()),
            user: Some("test-user".to_string()),
            notes: Some("Invalid test run - no device_info".to_string()),
        },
        // Run with empty device_info string
        Run {
            id: None,
            timestamp: Some("2024-01-01T12:00:00Z".to_string()),
            vram_usage: Some("1.0/1.2/1.1".to_string()),
            info: Some("app:stable-diffusion updated:2024-01-03".to_string()),
            system_info: Some("arch:arm64 cpu:Apple".to_string()),
            model_info: Some("torch:1.13.0 xformers:0.0.21".to_string()),
            device_info: Some("".to_string()), // Empty device_info string
            xformers: Some("true".to_string()),
            model_name: Some("test-model".to_string()),
            user: Some("test-user".to_string()),
            notes: Some("Invalid test run - empty device_info".to_string()),
        },
        // Valid run
        Run {
            id: None,
            timestamp: Some("2024-01-01T13:00:00Z".to_string()),
            vram_usage: Some("2.5/2.7/2.6".to_string()),
            info: Some("app:vladmandic updated:2024-01-02".to_string()),
            system_info: Some("arch:amd64 cpu:AMD".to_string()),
            model_info: Some("torch:2.1.0 xformers:0.0.23".to_string()),
            device_info: Some("device:NVIDIA driver:535.98.01 NVIDIA GeForce RTX 4080".to_string()),
            xformers: Some("true".to_string()),
            model_name: Some("test-model".to_string()),
            user: Some("test-user".to_string()),
            notes: Some("Another valid test run".to_string()),
        },
    ]
}



/// Verify that GPU records were created correctly
async fn verify_gpu_records(
    gpu_records: &[sd_its_benchmark::models::gpu::Gpu],
) -> Result<(), Box<dyn std::error::Error>> {
    assert_eq!(gpu_records.len(), 3, "Should have 3 GPU records");
    
    // Verify each result has the expected structure
    for gpu in gpu_records {
        assert!(gpu.run_id.is_some(), "Each GPU should have a run_id");
        
        // Verify that at least one field is populated
        let has_data = gpu.device.is_some() || 
                      gpu.driver.is_some() || 
                      gpu.gpu_chip.is_some();
        
        assert!(has_data, "Each GPU should have at least one field populated");
        
        // Verify brand and is_laptop are None (will be populated by separate processes)
        assert!(gpu.brand.is_none(), "Brand should be None initially");
        assert!(gpu.is_laptop.is_none(), "Is_laptop should be None initially");
    }
    
    // Verify specific devices are present
    let devices: Vec<Option<&String>> = gpu_records.iter().map(|g| g.device.as_ref()).collect();
    assert!(devices.contains(&Some(&"NVIDIA".to_string())), "Should contain NVIDIA");
    
    // Verify specific drivers are present
    let drivers: Vec<Option<&String>> = gpu_records.iter().map(|g| g.driver.as_ref()).collect();
    assert!(drivers.contains(&Some(&"470.82.01".to_string())), "Should contain driver 470.82.01");
    assert!(drivers.contains(&Some(&"535.98.01".to_string())), "Should contain driver 535.98.01");
    assert!(drivers.contains(&Some(&"525.85.05".to_string())), "Should contain driver 525.85.05");
    
    // Verify specific GPU chips are present
    let gpu_chips: Vec<Option<&String>> = gpu_records.iter().map(|g| g.gpu_chip.as_ref()).collect();
    assert!(gpu_chips.contains(&Some(&"NVIDIA GeForce RTX 4090".to_string())), "Should contain NVIDIA GeForce RTX 4090");
    assert!(gpu_chips.contains(&Some(&"NVIDIA GeForce RTX 4080".to_string())), "Should contain NVIDIA GeForce RTX 4080");
    assert!(gpu_chips.contains(&Some(&"NVIDIA GeForce RTX 3090".to_string())), "Should contain NVIDIA GeForce RTX 3090");
    
    Ok(())
} 