use sqlx::SqlitePool;

use sd_its_benchmark::{
    models::runs::Run,
    repositories::{
        runs_repository::RunsRepository,
        system_info_repository::SystemInfoRepository,
        traits::Repository,
    },
    services::data_processing::process_system_info_service::ProcessSystemInfoService,
};

/// Integration test for Process System Info Service
/// 
/// This test:
/// 1. Sets up a test database with sample runs data
/// 2. Calls the Process System Info Service
/// 3. Verifies the results match expected output
/// 4. Tests error handling and edge cases
#[tokio::test]
async fn test_process_system_info_service_integration() -> Result<(), Box<dyn std::error::Error>> {
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
    let system_info_repository = SystemInfoRepository::new(pool.clone());
    let service = ProcessSystemInfoService::new(runs_repository, system_info_repository);
    
    // Call the service
    let result = service.process_system_info().await?;
    
    // Verify the result
    assert!(result.success, "Service should return success");
    assert_eq!(result.total_runs, 3, "Should process 3 runs");
    assert_eq!(result.inserted_rows, 3, "Should insert 3 system info records");
    assert_eq!(result.error_rows, 0, "Should have no errors");
    assert!(result.error_data.is_empty(), "Should have no error data");
    
    // Verify system info was created
    let system_info_repo_for_check = SystemInfoRepository::new(pool.clone());
    let system_info_records = system_info_repo_for_check.find_all().await?;
    assert_eq!(system_info_records.len(), 3, "Should have 3 system info records");
    
    // Verify specific system info
    verify_system_info(&system_info_records).await?;
    
    println!("✅ Process System Info Service integration test passed!");
    Ok(())
}

/// Test error handling with invalid data
#[tokio::test]
async fn test_process_system_info_service_error_handling() -> Result<(), Box<dyn std::error::Error>> {
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
    let system_info_repository = SystemInfoRepository::new(pool.clone());
    let service = ProcessSystemInfoService::new(runs_repository, system_info_repository);
    
    // Call the service
    let result = service.process_system_info().await?;
    
    // Verify the result shows errors
    assert!(result.success, "Service should return success even with some errors");
    assert_eq!(result.total_runs, 4, "Should process 4 runs");
    assert!(result.inserted_rows > 0, "Should insert some system info records");
    assert!(result.error_rows > 0, "Should have some errors");
    assert!(!result.error_data.is_empty(), "Should have error data");
    
    println!("✅ Process System Info Service error handling test passed!");
    Ok(())
}

/// Test with empty database
#[tokio::test]
async fn test_process_system_info_service_empty_database() -> Result<(), Box<dyn std::error::Error>> {
    // Setup test database
    let pool = setup_test_database().await?;
    
    // Create service
    let runs_repository = RunsRepository::new(pool.clone());
    let system_info_repository = SystemInfoRepository::new(pool.clone());
    let service = ProcessSystemInfoService::new(runs_repository, system_info_repository);
    
    // Call the service with empty database
    let result = service.process_system_info().await?;
    
    // Verify the result
    assert!(result.success, "Service should return success");
    assert_eq!(result.total_runs, 0, "Should process 0 runs");
    assert_eq!(result.inserted_rows, 0, "Should insert 0 system info records");
    assert_eq!(result.error_rows, 0, "Should have no errors");
    assert!(result.error_data.is_empty(), "Should have no error data");
    
    println!("✅ Process System Info Service empty database test passed!");
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

/// Create test runs with valid system info data
fn create_test_runs() -> Vec<Run> {
    vec![
        Run {
            id: None,
            timestamp: Some("2024-01-01T10:00:00Z".to_string()),
            vram_usage: Some("1.5/2.0/1.8".to_string()),
            info: Some("app:automatic1111 updated:2024-01-01".to_string()),
            system_info: Some("arch:x86_64 cpu:Intel Core i7 system:Linux release:Ubuntu 22.04 python:3.9.0".to_string()),
            model_info: Some("torch:2.0.0 xformers:0.0.22".to_string()),
            device_info: Some("device:NVIDIA driver:470.82.01".to_string()),
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
            system_info: Some("arch:amd64 cpu:AMD Ryzen 9 system:Windows release:Windows 11 python:3.10.0".to_string()),
            model_info: Some("torch:2.0.0 xformers:0.0.22".to_string()),
            device_info: Some("device:NVIDIA driver:470.82.01".to_string()),
            xformers: Some("true".to_string()),
            model_name: Some("test-model".to_string()),
            user: Some("test-user".to_string()),
            notes: Some("Test run 2".to_string()),
        },
        Run {
            id: None,
            timestamp: Some("2024-01-01T12:00:00Z".to_string()),
            vram_usage: Some("1.0/1.2/1.1".to_string()),
            info: Some("app:stable-diffusion updated:2024-01-03".to_string()),
            system_info: Some("arch:arm64 cpu:Apple M1 system:macOS release:macOS 13.0 python:3.8.0".to_string()),
            model_info: Some("torch:2.0.0 xformers:0.0.22".to_string()),
            device_info: Some("device:NVIDIA driver:470.82.01".to_string()),
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
            system_info: Some("arch:x86_64 cpu:Intel Core i7 system:Linux release:Ubuntu 22.04 python:3.9.0".to_string()),
            model_info: Some("torch:2.0.0 xformers:0.0.22".to_string()),
            device_info: Some("device:NVIDIA driver:470.82.01".to_string()),
            xformers: Some("true".to_string()),
            model_name: Some("test-model".to_string()),
            user: Some("test-user".to_string()),
            notes: Some("Valid test run".to_string()),
        },
        // Run with missing system_info (should cause error)
        Run {
            id: None,
            timestamp: Some("2024-01-01T11:00:00Z".to_string()),
            vram_usage: Some("2.1/2.3/2.0".to_string()),
            info: Some("app:vladmandic updated:2024-01-02".to_string()),
            system_info: None, // This will cause an error
            model_info: Some("torch:2.0.0 xformers:0.0.22".to_string()),
            device_info: Some("device:NVIDIA driver:470.82.01".to_string()),
            xformers: Some("true".to_string()),
            model_name: Some("test-model".to_string()),
            user: Some("test-user".to_string()),
            notes: Some("Invalid test run - no system_info".to_string()),
        },
        // Run with incomplete system_info (should be skipped)
        Run {
            id: None,
            timestamp: Some("2024-01-01T12:00:00Z".to_string()),
            vram_usage: Some("1.0/1.2/1.1".to_string()),
            info: Some("app:stable-diffusion updated:2024-01-03".to_string()),
            system_info: Some("arch:x86_64 cpu:Intel".to_string()), // Missing required fields
            model_info: Some("torch:2.0.0 xformers:0.0.22".to_string()),
            device_info: Some("device:NVIDIA driver:470.82.01".to_string()),
            xformers: Some("true".to_string()),
            model_name: Some("test-model".to_string()),
            user: Some("test-user".to_string()),
            notes: Some("Invalid test run - incomplete system_info".to_string()),
        },
        // Valid run
        Run {
            id: None,
            timestamp: Some("2024-01-01T13:00:00Z".to_string()),
            vram_usage: Some("2.5/2.7/2.6".to_string()),
            info: Some("app:vladmandic updated:2024-01-02".to_string()),
            system_info: Some("arch:amd64 cpu:AMD Ryzen 9 system:Windows release:Windows 11 python:3.10.0".to_string()),
            model_info: Some("torch:2.0.0 xformers:0.0.22".to_string()),
            device_info: Some("device:NVIDIA driver:470.82.01".to_string()),
            xformers: Some("true".to_string()),
            model_name: Some("test-model".to_string()),
            user: Some("test-user".to_string()),
            notes: Some("Another valid test run".to_string()),
        },
    ]
}

/// Verify that system info was created correctly
async fn verify_system_info(
    system_info_records: &[sd_its_benchmark::models::system_info::SystemInfo],
) -> Result<(), Box<dyn std::error::Error>> {
    assert_eq!(system_info_records.len(), 3, "Should have 3 system info records");
    
    // Verify each result has the expected structure
    for system_info in system_info_records {
        assert!(system_info.run_id.is_some(), "Each system info should have a run_id");
        
        // Verify all required fields are populated
        assert!(system_info.arch.is_some(), "Each system info should have arch");
        assert!(system_info.cpu.is_some(), "Each system info should have cpu");
        assert!(system_info.system.is_some(), "Each system info should have system");
        assert!(system_info.release.is_some(), "Each system info should have release");
        assert!(system_info.python.is_some(), "Each system info should have python");
    }
    
    // Verify specific architectures are present
    let architectures: Vec<Option<&String>> = system_info_records.iter().map(|si| si.arch.as_ref()).collect();
    assert!(architectures.contains(&Some(&"x86_64".to_string())), "Should contain x86_64");
    assert!(architectures.contains(&Some(&"amd64".to_string())), "Should contain amd64");
    assert!(architectures.contains(&Some(&"arm64".to_string())), "Should contain arm64");
    
    // Verify specific systems are present
    let systems: Vec<Option<&String>> = system_info_records.iter().map(|si| si.system.as_ref()).collect();
    assert!(systems.contains(&Some(&"Linux".to_string())), "Should contain Linux");
    assert!(systems.contains(&Some(&"Windows".to_string())), "Should contain Windows");
    assert!(systems.contains(&Some(&"macOS".to_string())), "Should contain macOS");
    
    Ok(())
} 