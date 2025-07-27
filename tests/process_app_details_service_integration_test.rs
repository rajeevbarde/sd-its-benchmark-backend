use sqlx::SqlitePool;

use sd_its_benchmark::{
    models::runs::Run,
    repositories::{
        app_details_repository::AppDetailsRepository,
        runs_repository::RunsRepository,
        traits::Repository,
    },
    services::data_processing::process_app_details_service::ProcessAppDetailsService,
};

/// Integration test for Process App Details Service
/// 
/// This test:
/// 1. Sets up a test database with sample runs data
/// 2. Calls the Process App Details Service
/// 3. Verifies the results match expected output
/// 4. Tests error handling and edge cases
#[tokio::test]
async fn test_process_app_details_service_integration() -> Result<(), Box<dyn std::error::Error>> {
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
    let app_details_repository = AppDetailsRepository::new(pool.clone());
    let service = ProcessAppDetailsService::new(runs_repository, app_details_repository);
    
    // Call the service
    let result = service.process_app_details().await?;
    
    // Verify the result
    assert!(result.success, "Service should return success");
    assert_eq!(result.total_runs, 3, "Should process 3 runs");
    assert_eq!(result.inserted_rows, 3, "Should insert 3 app details");
    assert_eq!(result.error_rows, 0, "Should have no errors");
    assert!(result.error_data.is_empty(), "Should have no error data");
    
    // Verify app details were created
    let app_details_repo_for_check = AppDetailsRepository::new(pool.clone());
    let app_details = app_details_repo_for_check.find_all().await?;
    assert_eq!(app_details.len(), 3, "Should have 3 app details");
    
    // Verify specific app details
    verify_app_details(&app_details).await?;
    
    println!("✅ Process App Details Service integration test passed!");
    Ok(())
}

/// Test error handling with invalid data
#[tokio::test]
async fn test_process_app_details_service_error_handling() -> Result<(), Box<dyn std::error::Error>> {
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
    let app_details_repository = AppDetailsRepository::new(pool.clone());
    let service = ProcessAppDetailsService::new(runs_repository, app_details_repository);
    
    // Call the service
    let result = service.process_app_details().await?;
    
    // Verify the result shows errors
    assert!(result.success, "Service should return success even with some errors");
    assert_eq!(result.total_runs, 4, "Should process 4 runs");
    assert!(result.inserted_rows > 0, "Should insert some app details");
    assert!(result.error_rows > 0, "Should have some errors");
    assert!(!result.error_data.is_empty(), "Should have error data");
    
    println!("✅ Process App Details Service error handling test passed!");
    Ok(())
}

/// Test with empty database
#[tokio::test]
async fn test_process_app_details_service_empty_database() -> Result<(), Box<dyn std::error::Error>> {
    // Setup test database
    let pool = setup_test_database().await?;
    
    // Create service
    let runs_repository = RunsRepository::new(pool.clone());
    let app_details_repository = AppDetailsRepository::new(pool.clone());
    let service = ProcessAppDetailsService::new(runs_repository, app_details_repository);
    
    // Call the service with empty database
    let result = service.process_app_details().await?;
    
    // Verify the result
    assert!(result.success, "Service should return success");
    assert_eq!(result.total_runs, 0, "Should process 0 runs");
    assert_eq!(result.inserted_rows, 0, "Should insert 0 app details");
    assert_eq!(result.error_rows, 0, "Should have no errors");
    assert!(result.error_data.is_empty(), "Should have no error data");
    
    println!("✅ Process App Details Service empty database test passed!");
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

/// Create test runs with valid app details data
fn create_test_runs() -> Vec<Run> {
    vec![
        Run {
            id: None,
            timestamp: Some("2024-01-01T10:00:00Z".to_string()),
            vram_usage: Some("1.5/2.0/1.8".to_string()),
            info: Some("app:automatic1111 updated:2024-01-01 hash:abc123 url:https://github.com/AUTOMATIC1111/stable-diffusion-webui".to_string()),
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
            vram_usage: Some("2.1/2.3/2.0".to_string()),
            info: Some("app:vladmandic updated:2024-01-02 hash:def456 url:https://github.com/vladmandic/automatic".to_string()),
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
            vram_usage: Some("1.0/1.2/1.1".to_string()),
            info: Some("app:stable-diffusion updated:2024-01-03 hash:ghi789 url:https://github.com/CompVis/stable-diffusion".to_string()),
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
            info: Some("app:automatic1111 updated:2024-01-01 hash:abc123 url:https://github.com/AUTOMATIC1111/stable-diffusion-webui".to_string()),
            system_info: Some("arch:x86_64 cpu:Intel".to_string()),
            model_info: Some("torch:2.0.0 xformers:0.0.22".to_string()),
            device_info: Some("device:NVIDIA driver:470.82.01".to_string()),
            xformers: Some("true".to_string()),
            model_name: Some("test-model".to_string()),
            user: Some("test-user".to_string()),
            notes: Some("Valid test run".to_string()),
        },
        // Run with missing info (should cause error)
        Run {
            id: None,
            timestamp: Some("2024-01-01T11:00:00Z".to_string()),
            vram_usage: Some("2.1/2.3/2.0".to_string()),
            info: None, // This will cause an error
            system_info: Some("arch:x86_64 cpu:Intel".to_string()),
            model_info: Some("torch:2.0.0 xformers:0.0.22".to_string()),
            device_info: Some("device:NVIDIA driver:470.82.01".to_string()),
            xformers: Some("true".to_string()),
            model_name: Some("test-model".to_string()),
            user: Some("test-user".to_string()),
            notes: Some("Invalid test run - no info".to_string()),
        },
        // Run with empty info string
        Run {
            id: None,
            timestamp: Some("2024-01-01T12:00:00Z".to_string()),
            vram_usage: Some("1.0/1.2/1.1".to_string()),
            info: Some("".to_string()), // Empty info string
            system_info: Some("arch:x86_64 cpu:Intel".to_string()),
            model_info: Some("torch:2.0.0 xformers:0.0.22".to_string()),
            device_info: Some("device:NVIDIA driver:470.82.01".to_string()),
            xformers: Some("true".to_string()),
            model_name: Some("test-model".to_string()),
            user: Some("test-user".to_string()),
            notes: Some("Invalid test run - empty info".to_string()),
        },
        // Valid run
        Run {
            id: None,
            timestamp: Some("2024-01-01T13:00:00Z".to_string()),
            vram_usage: Some("2.5/2.7/2.6".to_string()),
            info: Some("app:vladmandic updated:2024-01-02 hash:def456 url:https://github.com/vladmandic/automatic".to_string()),
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

/// Verify that app details were created correctly
async fn verify_app_details(
    app_details: &[sd_its_benchmark::models::app_details::AppDetails],
) -> Result<(), Box<dyn std::error::Error>> {
    assert_eq!(app_details.len(), 3, "Should have 3 app details");
    
    // Verify each result has the expected structure
    for app_detail in app_details {
        assert!(app_detail.run_id.is_some(), "Each app detail should have a run_id");
        
        // Verify that at least one field is populated
        let has_data = app_detail.app_name.is_some() || 
                      app_detail.updated.is_some() || 
                      app_detail.hash.is_some() || 
                      app_detail.url.is_some();
        
        assert!(has_data, "Each app detail should have at least one field populated");
    }
    
    // Verify specific app names are present
    let app_names: Vec<Option<&String>> = app_details.iter().map(|ad| ad.app_name.as_ref()).collect();
    assert!(app_names.contains(&Some(&"automatic1111".to_string())), "Should contain automatic1111");
    assert!(app_names.contains(&Some(&"vladmandic".to_string())), "Should contain vladmandic");
    assert!(app_names.contains(&Some(&"stable-diffusion".to_string())), "Should contain stable-diffusion");
    
    Ok(())
} 