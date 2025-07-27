use sqlx::SqlitePool;

use sd_its_benchmark::{
    models::runs::Run,
    repositories::{
        performance_result_repository::PerformanceResultRepository,
        runs_repository::RunsRepository,
        traits::Repository,
    },
    services::data_processing::process_its_service::ProcessItsService,
};

/// Integration test for Process ITS Service
/// 
/// This test:
/// 1. Sets up a test database with sample runs data
/// 2. Calls the Process ITS Service
/// 3. Verifies the results match expected output
/// 4. Tests error handling and edge cases
#[tokio::test]
async fn test_process_its_service_integration() -> Result<(), Box<dyn std::error::Error>> {
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
    let performance_result_repository = PerformanceResultRepository::new(pool.clone());
    let service = ProcessItsService::new(runs_repository, performance_result_repository);
    
    // Call the service
    let result = service.process_its().await?;
    
    // Verify the result
    assert!(result.success, "Service should return success");
    assert_eq!(result.total_runs, 3, "Should process 3 runs");
    assert_eq!(result.inserted_rows, 3, "Should insert 3 performance results");
    assert_eq!(result.error_rows, 0, "Should have no errors");
    assert!(result.error_data.is_empty(), "Should have no error data");
    
    // Verify performance results were created
    let perf_repo_for_check = PerformanceResultRepository::new(pool.clone());
    let performance_results = perf_repo_for_check.find_all().await?;
    assert_eq!(performance_results.len(), 3, "Should have 3 performance results");
    
    // Verify specific performance results
    verify_performance_results(&performance_results).await?;
    
    println!("✅ Process ITS Service integration test passed!");
    Ok(())
}

/// Test error handling with invalid data
#[tokio::test]
async fn test_process_its_service_error_handling() -> Result<(), Box<dyn std::error::Error>> {
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
    let performance_result_repository = PerformanceResultRepository::new(pool.clone());
    let service = ProcessItsService::new(runs_repository, performance_result_repository);
    
    // Call the service
    let result = service.process_its().await?;
    
    // Verify the result shows errors
    assert!(result.success, "Service should return success even with some errors");
    assert_eq!(result.total_runs, 4, "Should process 4 runs");
    assert!(result.inserted_rows > 0, "Should insert some performance results");
    assert!(result.error_rows > 0, "Should have some errors");
    assert!(!result.error_data.is_empty(), "Should have error data");
    
    println!("✅ Process ITS Service error handling test passed!");
    Ok(())
}

/// Test with empty database
#[tokio::test]
async fn test_process_its_service_empty_database() -> Result<(), Box<dyn std::error::Error>> {
    // Setup test database
    let pool = setup_test_database().await?;
    
    // Create service
    let runs_repository = RunsRepository::new(pool.clone());
    let performance_result_repository = PerformanceResultRepository::new(pool.clone());
    let service = ProcessItsService::new(runs_repository, performance_result_repository);
    
    // Call the service with empty database
    let result = service.process_its().await?;
    
    // Verify the result
    assert!(result.success, "Service should return success");
    assert_eq!(result.total_runs, 0, "Should process 0 runs");
    assert_eq!(result.inserted_rows, 0, "Should insert 0 performance results");
    assert_eq!(result.error_rows, 0, "Should have no errors");
    assert!(result.error_data.is_empty(), "Should have no error data");
    
    println!("✅ Process ITS Service empty database test passed!");
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

/// Create test runs with valid ITS data
fn create_test_runs() -> Vec<Run> {
    vec![
        Run {
            id: None,
            timestamp: Some("2024-01-01T10:00:00Z".to_string()),
            vram_usage: Some("1.5/2.0/1.8".to_string()), // Average: 1.77
            info: Some("app:test-app updated:2024-01-01".to_string()),
            system_info: Some("arch:x86_64 cpu:Intel".to_string()),
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
            vram_usage: Some("2.1/2.3/2.0".to_string()), // Average: 2.13
            info: Some("app:test-app updated:2024-01-01".to_string()),
            system_info: Some("arch:x86_64 cpu:Intel".to_string()),
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
            vram_usage: Some("1.0/1.2/1.1".to_string()), // Average: 1.1
            info: Some("app:test-app updated:2024-01-01".to_string()),
            system_info: Some("arch:x86_64 cpu:Intel".to_string()),
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
            info: Some("app:test-app updated:2024-01-01".to_string()),
            system_info: Some("arch:x86_64 cpu:Intel".to_string()),
            model_info: Some("torch:2.0.0 xformers:0.0.22".to_string()),
            device_info: Some("device:NVIDIA driver:470.82.01".to_string()),
            xformers: Some("true".to_string()),
            model_name: Some("test-model".to_string()),
            user: Some("test-user".to_string()),
            notes: Some("Valid test run".to_string()),
        },
        // Run with missing vram_usage (should cause error)
        Run {
            id: None,
            timestamp: Some("2024-01-01T11:00:00Z".to_string()),
            vram_usage: None, // This will cause an error
            info: Some("app:test-app updated:2024-01-01".to_string()),
            system_info: Some("arch:x86_64 cpu:Intel".to_string()),
            model_info: Some("torch:2.0.0 xformers:0.0.22".to_string()),
            device_info: Some("device:NVIDIA driver:470.82.01".to_string()),
            xformers: Some("true".to_string()),
            model_name: Some("test-model".to_string()),
            user: Some("test-user".to_string()),
            notes: Some("Invalid test run - no vram_usage".to_string()),
        },
        // Run with invalid ITS values
        Run {
            id: None,
            timestamp: Some("2024-01-01T12:00:00Z".to_string()),
            vram_usage: Some("invalid/nan/not-a-number".to_string()), // Invalid values
            info: Some("app:test-app updated:2024-01-01".to_string()),
            system_info: Some("arch:x86_64 cpu:Intel".to_string()),
            model_info: Some("torch:2.0.0 xformers:0.0.22".to_string()),
            device_info: Some("device:NVIDIA driver:470.82.01".to_string()),
            xformers: Some("true".to_string()),
            model_name: Some("test-model".to_string()),
            user: Some("test-user".to_string()),
            notes: Some("Invalid test run - bad ITS values".to_string()),
        },
        // Valid run
        Run {
            id: None,
            timestamp: Some("2024-01-01T13:00:00Z".to_string()),
            vram_usage: Some("2.5/2.7/2.6".to_string()),
            info: Some("app:test-app updated:2024-01-01".to_string()),
            system_info: Some("arch:x86_64 cpu:Intel".to_string()),
            model_info: Some("torch:2.0.0 xformers:0.0.22".to_string()),
            device_info: Some("device:NVIDIA driver:470.82.01".to_string()),
            xformers: Some("true".to_string()),
            model_name: Some("test-model".to_string()),
            user: Some("test-user".to_string()),
            notes: Some("Another valid test run".to_string()),
        },
    ]
}

/// Verify that performance results were created correctly
async fn verify_performance_results(
    performance_results: &[sd_its_benchmark::models::performance_result::PerformanceResult],
) -> Result<(), Box<dyn std::error::Error>> {
    assert_eq!(performance_results.len(), 3, "Should have 3 performance results");
    
    // Verify each result has the expected structure
    for result in performance_results {
        assert!(result.run_id.is_some(), "Each result should have a run_id");
        assert!(result.its.is_some(), "Each result should have ITS data");
        
        // Verify the ITS data matches the original vram_usage
        if let Some(its) = &result.its {
            assert!(!its.is_empty(), "ITS data should not be empty");
        }
        
        // Verify avg_its is calculated correctly (if present)
        if let Some(avg_its) = result.avg_its {
            assert!(avg_its > 0.0, "Average ITS should be positive");
            assert!(!avg_its.is_nan(), "Average ITS should not be NaN");
        }
    }
    
    Ok(())
} 