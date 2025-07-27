use sqlx::SqlitePool;

use sd_its_benchmark::{
    models::runs::Run,
    repositories::{
        libraries_repository::LibrariesRepository,
        runs_repository::RunsRepository,
        traits::Repository,
    },
    services::data_processing::process_libraries_service::ProcessLibrariesService,
};

/// Integration test for Process Libraries Service
/// 
/// This test:
/// 1. Sets up a test database with sample runs data
/// 2. Calls the Process Libraries Service
/// 3. Verifies the results match expected output
/// 4. Tests error handling and edge cases
#[tokio::test]
async fn test_process_libraries_service_integration() -> Result<(), Box<dyn std::error::Error>> {
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
    let libraries_repository = LibrariesRepository::new(pool.clone());
    let service = ProcessLibrariesService::new(runs_repository, libraries_repository);
    
    // Call the service
    let result = service.process_libraries().await?;
    
    // Verify the result
    assert!(result.success, "Service should return success");
    assert_eq!(result.total_runs, 3, "Should process 3 runs");
    assert_eq!(result.inserted_rows, 3, "Should insert 3 library records");
    assert_eq!(result.error_rows, 0, "Should have no errors");
    assert!(result.error_data.is_empty(), "Should have no error data");
    
    // Verify libraries were created
    let libraries_repo_for_check = LibrariesRepository::new(pool.clone());
    let libraries_records = libraries_repo_for_check.find_all().await?;
    assert_eq!(libraries_records.len(), 3, "Should have 3 library records");
    
    // Verify specific libraries
    verify_libraries(&libraries_records).await?;
    
    println!("✅ Process Libraries Service integration test passed!");
    Ok(())
}

/// Test error handling with invalid data
#[tokio::test]
async fn test_process_libraries_service_error_handling() -> Result<(), Box<dyn std::error::Error>> {
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
    let libraries_repository = LibrariesRepository::new(pool.clone());
    let service = ProcessLibrariesService::new(runs_repository, libraries_repository);
    
    // Call the service
    let result = service.process_libraries().await?;
    
    // Verify the result shows errors
    assert!(result.success, "Service should return success even with some errors");
    assert_eq!(result.total_runs, 4, "Should process 4 runs");
    assert!(result.inserted_rows > 0, "Should insert some library records");
    assert!(result.error_rows > 0, "Should have some errors");
    assert!(!result.error_data.is_empty(), "Should have error data");
    
    println!("✅ Process Libraries Service error handling test passed!");
    Ok(())
}

/// Test with empty database
#[tokio::test]
async fn test_process_libraries_service_empty_database() -> Result<(), Box<dyn std::error::Error>> {
    // Setup test database
    let pool = setup_test_database().await?;
    
    // Create service
    let runs_repository = RunsRepository::new(pool.clone());
    let libraries_repository = LibrariesRepository::new(pool.clone());
    let service = ProcessLibrariesService::new(runs_repository, libraries_repository);
    
    // Call the service with empty database
    let result = service.process_libraries().await?;
    
    // Verify the result
    assert!(result.success, "Service should return success");
    assert_eq!(result.total_runs, 0, "Should process 0 runs");
    assert_eq!(result.inserted_rows, 0, "Should insert 0 library records");
    assert_eq!(result.error_rows, 0, "Should have no errors");
    assert!(result.error_data.is_empty(), "Should have no error data");
    
    println!("✅ Process Libraries Service empty database test passed!");
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

/// Create test runs with valid library data
fn create_test_runs() -> Vec<Run> {
    vec![
        Run {
            id: None,
            timestamp: Some("2024-01-01T10:00:00Z".to_string()),
            vram_usage: Some("1.5/2.0/1.8".to_string()),
            info: Some("app:automatic1111 updated:2024-01-01".to_string()),
            system_info: Some("arch:x86_64 cpu:Intel".to_string()),
            model_info: Some("torch:2.0.0 xformers:0.0.22 diffusers:0.21.0 transformers:4.30.0".to_string()),
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
            system_info: Some("arch:amd64 cpu:AMD".to_string()),
            model_info: Some("torch:2.1.0 xformers:0.0.23 diffusers:0.22.0 transformers:4.31.0".to_string()),
            device_info: Some("device:NVIDIA driver:470.82.01".to_string()),
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
            model_info: Some("torch:1.13.0 xformers:0.0.21 diffusers:0.20.0 transformers:4.29.0".to_string()),
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
            system_info: Some("arch:x86_64 cpu:Intel".to_string()),
            model_info: Some("torch:2.0.0 xformers:0.0.22 diffusers:0.21.0 transformers:4.30.0".to_string()),
            device_info: Some("device:NVIDIA driver:470.82.01".to_string()),
            xformers: Some("true".to_string()),
            model_name: Some("test-model".to_string()),
            user: Some("test-user".to_string()),
            notes: Some("Valid test run".to_string()),
        },
        // Run with missing model_info (should cause error)
        Run {
            id: None,
            timestamp: Some("2024-01-01T11:00:00Z".to_string()),
            vram_usage: Some("2.1/2.3/2.0".to_string()),
            info: Some("app:vladmandic updated:2024-01-02".to_string()),
            system_info: Some("arch:amd64 cpu:AMD".to_string()),
            model_info: None, // This will cause an error
            device_info: Some("device:NVIDIA driver:470.82.01".to_string()),
            xformers: Some("false".to_string()),
            model_name: Some("test-model".to_string()),
            user: Some("test-user".to_string()),
            notes: Some("Invalid test run - no model_info".to_string()),
        },
        // Run with missing xformers (should cause error)
        Run {
            id: None,
            timestamp: Some("2024-01-01T12:00:00Z".to_string()),
            vram_usage: Some("1.0/1.2/1.1".to_string()),
            info: Some("app:stable-diffusion updated:2024-01-03".to_string()),
            system_info: Some("arch:arm64 cpu:Apple".to_string()),
            model_info: Some("torch:1.13.0 xformers:0.0.21 diffusers:0.20.0 transformers:4.29.0".to_string()),
            device_info: Some("device:NVIDIA driver:470.82.01".to_string()),
            xformers: None, // This will cause an error
            model_name: Some("test-model".to_string()),
            user: Some("test-user".to_string()),
            notes: Some("Invalid test run - no xformers".to_string()),
        },
        // Valid run
        Run {
            id: None,
            timestamp: Some("2024-01-01T13:00:00Z".to_string()),
            vram_usage: Some("2.5/2.7/2.6".to_string()),
            info: Some("app:vladmandic updated:2024-01-02".to_string()),
            system_info: Some("arch:amd64 cpu:AMD".to_string()),
            model_info: Some("torch:2.1.0 xformers:0.0.23 diffusers:0.22.0 transformers:4.31.0".to_string()),
            device_info: Some("device:NVIDIA driver:470.82.01".to_string()),
            xformers: Some("true".to_string()),
            model_name: Some("test-model".to_string()),
            user: Some("test-user".to_string()),
            notes: Some("Another valid test run".to_string()),
        },
    ]
}

/// Verify that libraries were created correctly
async fn verify_libraries(
    libraries_records: &[sd_its_benchmark::models::libraries::Libraries],
) -> Result<(), Box<dyn std::error::Error>> {
    assert_eq!(libraries_records.len(), 3, "Should have 3 library records");
    
    // Verify each result has the expected structure
    for library in libraries_records {
        assert!(library.run_id.is_some(), "Each library should have a run_id");
        
        // Verify that at least one field is populated
        let has_data = library.torch.is_some() || 
                      library.xformers.is_some() || 
                      library.xformers1.is_some() || 
                      library.diffusers.is_some() || 
                      library.transformers.is_some();
        
        assert!(has_data, "Each library should have at least one field populated");
    }
    
    // Verify specific torch versions are present
    let torch_versions: Vec<Option<&String>> = libraries_records.iter().map(|l| l.torch.as_ref()).collect();
    assert!(torch_versions.contains(&Some(&"2.0.0".to_string())), "Should contain torch 2.0.0");
    assert!(torch_versions.contains(&Some(&"2.1.0".to_string())), "Should contain torch 2.1.0");
    assert!(torch_versions.contains(&Some(&"1.13.0".to_string())), "Should contain torch 1.13.0");
    
    // Verify specific xformers versions are present
    let xformers_versions: Vec<Option<&String>> = libraries_records.iter().map(|l| l.xformers.as_ref()).collect();
    assert!(xformers_versions.contains(&Some(&"0.0.22".to_string())), "Should contain xformers 0.0.22");
    assert!(xformers_versions.contains(&Some(&"0.0.23".to_string())), "Should contain xformers 0.0.23");
    assert!(xformers_versions.contains(&Some(&"0.0.21".to_string())), "Should contain xformers 0.0.21");
    
    // Verify xformers1 field is populated (copied from runs.xformers)
    let xformers1_values: Vec<Option<&String>> = libraries_records.iter().map(|l| l.xformers1.as_ref()).collect();
    assert!(xformers1_values.contains(&Some(&"true".to_string())), "Should contain xformers1 true");
    assert!(xformers1_values.contains(&Some(&"false".to_string())), "Should contain xformers1 false");
    
    Ok(())
} 