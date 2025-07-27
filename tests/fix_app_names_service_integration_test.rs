use sqlx::SqlitePool;

use sd_its_benchmark::{
    models::{app_details::AppDetails, runs::Run},
    repositories::{
        app_details_repository::AppDetailsRepository,
        runs_repository::RunsRepository,
        traits::Repository,
    },
    services::data_processing::fix_app_names_service::FixAppNamesService,
};

/// Integration test for Fix App Names Service
/// 
/// This test:
/// 1. Sets up a test database with sample app details data
/// 2. Calls the Fix App Names Service with various parameters
/// 3. Verifies the results match expected output
/// 4. Tests various data quality scenarios
#[tokio::test]
async fn test_fix_app_names_service_integration() -> Result<(), Box<dyn std::error::Error>> {
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
    assert_eq!(all_app_details.len(), 5, "Should have 5 test app details");
    
    // Create service
    let app_details_repository = AppDetailsRepository::new(pool.clone());
    let service = FixAppNamesService::new(app_details_repository);
    
    // Call the service
    let result = service.fix_app_names(
        "AUTOMATIC1111",
        "Vladmandic", 
        "StableDiffusion",
        "Unknown"
    ).await?;
    
    // Verify the result
    assert_eq!(result.updated_counts.automatic1111, 1, "Should update 1 AUTOMATIC1111 app name");
    assert_eq!(result.updated_counts.vladmandic, 1, "Should update 1 Vladmandic app name");
    assert_eq!(result.updated_counts.stable_diffusion, 1, "Should update 1 StableDiffusion app name");
    assert_eq!(result.updated_counts.null_app_name_null_url, 1, "Should update 1 null app_name null url");
    
    // Verify the actual database updates
    let updated_app_details = app_details_repo_for_check.find_all().await?;
    let automatic1111_count = updated_app_details.iter()
        .filter(|ad| ad.app_name.as_ref().map_or(false, |name| name == "AUTOMATIC1111"))
        .count();
    let vladmandic_count = updated_app_details.iter()
        .filter(|ad| ad.app_name.as_ref().map_or(false, |name| name == "Vladmandic"))
        .count();
    let stable_diffusion_count = updated_app_details.iter()
        .filter(|ad| ad.app_name.as_ref().map_or(false, |name| name == "StableDiffusion"))
        .count();
    let unknown_count = updated_app_details.iter()
        .filter(|ad| ad.app_name.as_ref().map_or(false, |name| name == "Unknown"))
        .count();
    
    assert_eq!(automatic1111_count, 1, "Should have 1 AUTOMATIC1111 app name in database");
    assert_eq!(vladmandic_count, 1, "Should have 1 Vladmandic app name in database");
    assert_eq!(stable_diffusion_count, 1, "Should have 1 StableDiffusion app name in database");
    assert_eq!(unknown_count, 1, "Should have 1 Unknown app name in database");
    
    println!("✅ Fix App Names Service integration test passed!");
    Ok(())
}

/// Test with no matching data
#[tokio::test]
async fn test_fix_app_names_service_no_matches() -> Result<(), Box<dyn std::error::Error>> {
    // Setup test database
    let pool = setup_test_database().await?;
    
    // Create required runs first (for foreign key constraints)
    create_required_runs(&pool).await?;
    
    // Insert test app details data that doesn't match any patterns
    let test_app_details = create_non_matching_app_details();
    let app_details_repo_for_insert = AppDetailsRepository::new(pool.clone());
    for app_detail in test_app_details {
        app_details_repo_for_insert.create(app_detail).await?;
    }
    
    // Create service
    let app_details_repository = AppDetailsRepository::new(pool.clone());
    let service = FixAppNamesService::new(app_details_repository);
    
    // Call the service with non-matching data
    let result = service.fix_app_names(
        "AUTOMATIC1111",
        "Vladmandic", 
        "StableDiffusion",
        "Unknown"
    ).await?;
    
    // Verify all counts are zero when no matches exist
    assert_eq!(result.updated_counts.automatic1111, 0, "Should have 0 AUTOMATIC1111 updates");
    assert_eq!(result.updated_counts.vladmandic, 0, "Should have 0 Vladmandic updates");
    assert_eq!(result.updated_counts.stable_diffusion, 0, "Should have 0 StableDiffusion updates");
    assert_eq!(result.updated_counts.null_app_name_null_url, 0, "Should have 0 null app_name null url updates");
    
    println!("✅ Fix App Names Service no matches test passed!");
    Ok(())
}

/// Test edge cases with specific data patterns
#[tokio::test]
async fn test_fix_app_names_service_edge_cases() -> Result<(), Box<dyn std::error::Error>> {
    // Setup test database
    let pool = setup_test_database().await?;
    
    // Create required runs first (for foreign key constraints)
    create_required_runs(&pool).await?;
    
    // Insert test app details with edge cases
    let test_app_details = create_edge_case_app_details();
    let app_details_repo_for_insert = AppDetailsRepository::new(pool.clone());
    for app_detail in test_app_details {
        app_details_repo_for_insert.create(app_detail).await?;
    }
    
    // Create service
    let app_details_repository = AppDetailsRepository::new(pool.clone());
    let service = FixAppNamesService::new(app_details_repository);
    
    // Call the service
    let result = service.fix_app_names(
        "AUTOMATIC1111",
        "Vladmandic", 
        "StableDiffusion",
        "Unknown"
    ).await?;
    
    // Only the empty string app_name should be updated by vladmandic rule
    assert_eq!(result.updated_counts.automatic1111, 0, "Should have 0 AUTOMATIC1111 updates");
    assert_eq!(result.updated_counts.vladmandic, 1, "Should have 1 Vladmandic update (empty string app_name)");
    assert_eq!(result.updated_counts.stable_diffusion, 0, "Should have 0 StableDiffusion updates (existing app_name)");
    assert_eq!(result.updated_counts.null_app_name_null_url, 0, "Should have 0 null app_name null url updates");
    
    println!("✅ Fix App Names Service edge cases test passed!");
    Ok(())
}

/// Test validation error with empty parameters
#[tokio::test]
async fn test_fix_app_names_service_validation_error() -> Result<(), Box<dyn std::error::Error>> {
    // Setup test database
    let pool = setup_test_database().await?;
    
    // Create service
    let app_details_repository = AppDetailsRepository::new(pool.clone());
    let service = FixAppNamesService::new(app_details_repository);
    
    // Call the service with empty parameters (should fail validation)
    let result = service.fix_app_names("", "Vladmandic", "StableDiffusion", "Unknown").await;
    
    // Verify validation error
    assert!(result.is_err(), "Should return validation error for empty parameters");
    if let Err(error) = result {
        assert!(error.to_string().contains("All fields must be non-empty"), 
                "Error should mention non-empty fields");
    }
    
    println!("✅ Fix App Names Service validation error test passed!");
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
    
    // Create 5 runs with IDs 1, 2, 3, 4, 5
    for i in 1..=5 {
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

/// Create test app details with various scenarios that should be updated
fn create_test_app_details() -> Vec<AppDetails> {
    vec![
        // AUTOMATIC1111 URL - should be updated
        AppDetails {
            id: None,
            run_id: Some(1),
            app_name: None, // Will be updated by AUTOMATIC1111 rule
            url: Some("https://github.com/AUTOMATIC1111/stable-diffusion-webui".to_string()),
            hash: Some("abc123".to_string()),
            updated: Some("2024-01-01".to_string()),
        },
        // Vladmandic URL - should be updated (NULL app_name)
        AppDetails {
            id: None,
            run_id: Some(2),
            app_name: None, // Will be updated by vladmandic rule
            url: Some("https://github.com/vladmandic/automatic".to_string()),
            hash: Some("def456".to_string()),
            updated: Some("2024-01-02".to_string()),
        },
        // Stable Diffusion URL - should be updated (NULL app_name)
        AppDetails {
            id: None,
            run_id: Some(3),
            app_name: None, // Will be updated by stable-diffusion-webui rule
            url: Some("https://github.com/CompVis/stable-diffusion-webui".to_string()),
            hash: Some("ghi789".to_string()),
            updated: Some("2024-01-03".to_string()),
        },
        // Both app_name and URL are null - should be updated
        AppDetails {
            id: None,
            run_id: Some(4),
            app_name: None, // Will be updated by null app_name null url rule
            url: None,
            hash: Some("jkl012".to_string()),
            updated: Some("2024-01-04".to_string()),
        },
        // Existing app name - should not be updated
        AppDetails {
            id: None,
            run_id: Some(5),
            app_name: Some("existing-app".to_string()), // Should not be updated
            url: Some("https://github.com/some-other/app".to_string()),
            hash: Some("mno345".to_string()),
            updated: Some("2024-01-05".to_string()),
        },
    ]
}

/// Create test app details that don't match any patterns
fn create_non_matching_app_details() -> Vec<AppDetails> {
    vec![
        // Non-matching URL with existing app name
        AppDetails {
            id: None,
            run_id: Some(1),
            app_name: Some("existing-app".to_string()),
            url: Some("https://github.com/some-other/app".to_string()),
            hash: Some("abc123".to_string()),
            updated: Some("2024-01-01".to_string()),
        },
        // Non-matching URL with NULL app name
        AppDetails {
            id: None,
            run_id: Some(2),
            app_name: None,
            url: Some("https://github.com/another-app".to_string()),
            hash: Some("def456".to_string()),
            updated: Some("2024-01-02".to_string()),
        },
    ]
}

/// Create test app details with edge cases
fn create_edge_case_app_details() -> Vec<AppDetails> {
    vec![
        // Empty string app_name with vladmandic URL - should be updated
        AppDetails {
            id: None,
            run_id: Some(1),
            app_name: Some("".to_string()), // Empty string - should be updated by vladmandic rule
            url: Some("https://github.com/vladmandic/automatic".to_string()),
            hash: Some("hash1".to_string()),
            updated: Some("2024-01-01".to_string()),
        },
        // Existing app name with stable-diffusion-webui URL - should not be updated
        AppDetails {
            id: None,
            run_id: Some(2),
            app_name: Some("existing-app".to_string()), // Existing app name - should not be updated
            url: Some("https://github.com/CompVis/stable-diffusion-webui".to_string()),
            hash: Some("hash2".to_string()),
            updated: Some("2024-01-02".to_string()),
        },
    ]
} 