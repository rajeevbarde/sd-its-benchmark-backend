use sqlx::SqlitePool;

use sd_its_benchmark::{
    models::{app_details::AppDetails, runs::Run},
    repositories::{
        app_details_repository::AppDetailsRepository,
        runs_repository::RunsRepository,
        traits::Repository,
    },
    services::data_processing::analyze_app_details_service::AnalyzeAppDetailsService,
};

/// Integration test for Analyze App Details Service
/// 
/// This test:
/// 1. Sets up a test database with sample app details data
/// 2. Calls the Analyze App Details Service
/// 3. Verifies the results match expected output
/// 4. Tests various data quality scenarios
#[tokio::test]
async fn test_analyze_app_details_service_integration() -> Result<(), Box<dyn std::error::Error>> {
    // Setup test database
    let pool = setup_test_database().await?;
    
    // Create required runs first (for foreign key constraints)
    create_required_runs(&pool).await?;
    
    // Insert test app details data with various scenarios
    let test_app_details = create_test_app_details();
    let app_details_repo_for_insert = AppDetailsRepository::new(pool.clone());
    for app_detail in test_app_details {
        app_details_repo_for_insert.create(app_detail).await?;
    }
    
    // Verify test data was inserted
    let app_details_repo_for_check = AppDetailsRepository::new(pool.clone());
    let all_app_details = app_details_repo_for_check.find_all().await?;
    assert_eq!(all_app_details.len(), 4, "Should have 4 test app details");
    
    // Create service
    let app_details_repository = AppDetailsRepository::new(pool.clone());
    let service = AnalyzeAppDetailsService::new(app_details_repository);
    
    // Call the service
    let result = service.analyze_app_details().await?;
    
    // Verify the result
    assert_eq!(result.total_rows, 4, "Should have 4 total rows");
    assert_eq!(result.null_app_name_null_url, 1, "Should have 1 null app_name null url");
    assert_eq!(result.null_app_name_non_null_url, 2, "Should have 2 null app_name non-null url");
    
    println!("✅ Analyze App Details Service integration test passed!");
    Ok(())
}

/// Test with empty database
#[tokio::test]
async fn test_analyze_app_details_service_empty_database() -> Result<(), Box<dyn std::error::Error>> {
    // Setup test database
    let pool = setup_test_database().await?;
    
    // Create service
    let app_details_repository = AppDetailsRepository::new(pool.clone());
    let service = AnalyzeAppDetailsService::new(app_details_repository);
    
    // Call the service with empty database
    let result = service.analyze_app_details().await?;
    
    // Verify the result
    assert_eq!(result.total_rows, 0, "Should have 0 total rows");
    assert_eq!(result.null_app_name_null_url, 0, "Should have 0 null app_name null url");
    assert_eq!(result.null_app_name_non_null_url, 0, "Should have 0 null app_name non-null url");
    
    println!("✅ Analyze App Details Service empty database test passed!");
    Ok(())
}

/// Test with only complete data (no nulls)
#[tokio::test]
async fn test_analyze_app_details_service_complete_data() -> Result<(), Box<dyn std::error::Error>> {
    // Setup test database
    let pool = setup_test_database().await?;
    
    // Create required runs first (for foreign key constraints)
    create_required_runs(&pool).await?;
    
    // Insert test app details data with complete data only
    let test_app_details = create_complete_app_details();
    let app_details_repo_for_insert = AppDetailsRepository::new(pool.clone());
    for app_detail in test_app_details {
        app_details_repo_for_insert.create(app_detail).await?;
    }
    
    // Create service
    let app_details_repository = AppDetailsRepository::new(pool.clone());
    let service = AnalyzeAppDetailsService::new(app_details_repository);
    
    // Call the service
    let result = service.analyze_app_details().await?;
    
    // Verify the result
    assert_eq!(result.total_rows, 2, "Should have 2 total rows");
    assert_eq!(result.null_app_name_null_url, 0, "Should have 0 null app_name null url");
    assert_eq!(result.null_app_name_non_null_url, 0, "Should have 0 null app_name non-null url");
    
    println!("✅ Analyze App Details Service complete data test passed!");
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

/// Create required runs for AppDetails foreign key constraints
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

/// Create test app details with various data quality scenarios
fn create_test_app_details() -> Vec<AppDetails> {
    vec![
        // Complete data
        AppDetails {
            id: None,
            run_id: Some(1),
            app_name: Some("test-app-1".to_string()),
            updated: Some("2024-01-01".to_string()),
            hash: Some("abc123".to_string()),
            url: Some("https://example.com/app1".to_string()),
        },
        // Null app_name but has URL
        AppDetails {
            id: None,
            run_id: Some(2),
            app_name: None,
            updated: Some("2024-01-02".to_string()),
            hash: Some("def456".to_string()),
            url: Some("https://example.com/app2".to_string()),
        },
        // Null app_name but has URL (another case)
        AppDetails {
            id: None,
            run_id: Some(3),
            app_name: None,
            updated: Some("2024-01-03".to_string()),
            hash: Some("ghi789".to_string()),
            url: Some("https://example.com/app3".to_string()),
        },
        // Both app_name and URL are null
        AppDetails {
            id: None,
            run_id: Some(4),
            app_name: None,
            updated: Some("2024-01-04".to_string()),
            hash: Some("jkl012".to_string()),
            url: None,
        },
    ]
}

/// Create test app details with complete data only
fn create_complete_app_details() -> Vec<AppDetails> {
    vec![
        // Complete data
        AppDetails {
            id: None,
            run_id: Some(1),
            app_name: Some("complete-app-1".to_string()),
            updated: Some("2024-01-01".to_string()),
            hash: Some("abc123".to_string()),
            url: Some("https://example.com/complete1".to_string()),
        },
        // Complete data
        AppDetails {
            id: None,
            run_id: Some(2),
            app_name: Some("complete-app-2".to_string()),
            updated: Some("2024-01-02".to_string()),
            hash: Some("def456".to_string()),
            url: Some("https://example.com/complete2".to_string()),
        },
    ]
} 