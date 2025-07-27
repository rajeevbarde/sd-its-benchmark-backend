use sqlx::SqlitePool;

use sd_its_benchmark::{
    models::{run_more_details::RunMoreDetails, runs::Run},
    repositories::{
        run_more_details_repository::RunMoreDetailsRepository,
        runs_repository::RunsRepository,
        traits::Repository,
    },
    services::data_processing::process_run_details_service::ProcessRunDetailsService,
};

/// Integration test for Process Run Details Service
/// 
/// This test:
/// 1. Sets up a test database with sample run data
/// 2. Calls the Process Run Details Service
/// 3. Verifies the results match expected output
/// 4. Tests error handling and edge cases
#[tokio::test]
async fn test_process_run_details_service_integration() -> Result<(), Box<dyn std::error::Error>> {
    // Setup test database
    let pool = setup_test_database().await?;
    
    // Insert test run data
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
    let run_more_details_repository = RunMoreDetailsRepository::new(pool.clone());
    let service = ProcessRunDetailsService::new(runs_repository, run_more_details_repository);
    
    // Call the service
    let result = service.process_run_details().await?;
    
    // Verify the result
    assert!(result.success, "Service should return success");
    assert_eq!(result.total_inserts, 3, "Should insert 3 run details");
    
    // Verify RunMoreDetails records were created
    let run_more_details_repo_for_verify = RunMoreDetailsRepository::new(pool.clone());
    let run_more_details = run_more_details_repo_for_verify.find_all().await?;
    
    verify_run_details_updates(&run_more_details).await?;
    
    println!("✅ Process Run Details Service integration test passed!");
    Ok(())
}

/// Test with empty database
#[tokio::test]
async fn test_process_run_details_service_empty_database() -> Result<(), Box<dyn std::error::Error>> {
    // Setup test database
    let pool = setup_test_database().await?;
    
    // Create service
    let runs_repository = RunsRepository::new(pool.clone());
    let run_more_details_repository = RunMoreDetailsRepository::new(pool.clone());
    let service = ProcessRunDetailsService::new(runs_repository, run_more_details_repository);
    
    // Call the service with empty database
    let result = service.process_run_details().await?;
    
    // Verify the result
    assert!(result.success, "Service should return success");
    assert_eq!(result.total_inserts, 0, "Should insert 0 run details");
    
    println!("✅ Process Run Details Service empty database test passed!");
    Ok(())
}

/// Test clearing existing data
#[tokio::test]
async fn test_process_run_details_service_clears_existing_data() -> Result<(), Box<dyn std::error::Error>> {
    // Setup test database
    let pool = setup_test_database().await?;
    
    // Insert test run data
    let test_runs = create_test_runs();
    let runs_repo_for_insert = RunsRepository::new(pool.clone());
    for run in test_runs {
        runs_repo_for_insert.create(run).await?;
    }
    
    // Create service and process run details first time
    let runs_repository = RunsRepository::new(pool.clone());
    let run_more_details_repository = RunMoreDetailsRepository::new(pool.clone());
    let service = ProcessRunDetailsService::new(runs_repository, run_more_details_repository);
    
    let result1 = service.process_run_details().await?;
    assert_eq!(result1.total_inserts, 3, "Should insert 3 run details");
    
    // Add some existing RunMoreDetails data manually
    let run_more_details_repo = RunMoreDetailsRepository::new(pool.clone());
    
    // First, create a dummy run to satisfy foreign key constraint
    let dummy_run = Run {
        id: None,
        timestamp: Some("2024-01-01T00:00:00Z".to_string()),
        vram_usage: Some("1.0/1.0/1.0".to_string()),
        info: Some("app:dummy updated:2024-01-01".to_string()),
        system_info: Some("arch:x86_64 cpu:Intel".to_string()),
        model_info: Some("torch:1.0.0 xformers:0.0.1".to_string()),
        device_info: Some("device:Intel driver:1.0.0".to_string()),
        xformers: Some("false".to_string()),
        model_name: Some("dummy-model".to_string()),
        user: Some("dummy-user".to_string()),
        notes: Some("Dummy run for testing".to_string()),
    };
    let dummy_run = runs_repo_for_insert.create(dummy_run).await?;
    let dummy_run_id = dummy_run.id.unwrap();
    
    let existing_detail = RunMoreDetails {
        id: None,
        run_id: Some(dummy_run_id),
        timestamp: Some("2024-01-01T00:00:00Z".to_string()),
        model_name: Some("old-model".to_string()),
        user: Some("old-user".to_string()),
        notes: Some("old notes".to_string()),
        model_map_id: None,
    };
    run_more_details_repo.create(existing_detail).await?;
    
    // Verify old data exists
    let all_details_before = run_more_details_repo.find_all().await?;
    assert_eq!(all_details_before.len(), 4, "Should have 4 details before clearing");
    
    // Process run details again (should clear existing data)
    let runs_repository2 = RunsRepository::new(pool.clone());
    let run_more_details_repository2 = RunMoreDetailsRepository::new(pool.clone());
    let service2 = ProcessRunDetailsService::new(runs_repository2, run_more_details_repository2);
    
    let result2 = service2.process_run_details().await?;
    assert_eq!(result2.total_inserts, 4, "Should insert 4 run details (3 original + 1 dummy)");
    
    // Verify old data was cleared and only new data exists
    let all_details_after = run_more_details_repo.find_all().await?;
    assert_eq!(all_details_after.len(), 4, "Should have 4 details after clearing (3 original + 1 dummy)");
    
    // Verify old data was replaced (not completely gone, since the dummy run still exists)
    let dummy_details = all_details_after.iter()
        .filter(|d| d.run_id == Some(dummy_run_id))
        .collect::<Vec<_>>();
    assert_eq!(dummy_details.len(), 1, "Should have 1 detail for dummy run");
    
    // Verify the dummy run detail has the correct data (from the dummy run, not the old manual entry)
    let dummy_detail = dummy_details[0];
    assert_eq!(dummy_detail.model_name.as_ref().unwrap(), "dummy-model", "Dummy detail should have dummy-model");
    assert_eq!(dummy_detail.user.as_ref().unwrap(), "dummy-user", "Dummy detail should have dummy-user");
    assert_eq!(dummy_detail.notes.as_ref().unwrap(), "Dummy run for testing", "Dummy detail should have dummy notes");
    
    println!("✅ Process Run Details Service clears existing data test passed!");
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

/// Create test runs with various data
fn create_test_runs() -> Vec<Run> {
    vec![
        Run {
            id: None,
            timestamp: Some("2024-01-01T10:00:00Z".to_string()),
            vram_usage: Some("1.5/2.0/1.8".to_string()),
            info: Some("app:test updated:2024-01-01".to_string()),
            system_info: Some("arch:x86_64 cpu:Intel".to_string()),
            model_info: Some("torch:2.0.0 xformers:0.0.22".to_string()),
            device_info: Some("device:NVIDIA driver:470.82.01".to_string()),
            xformers: Some("true".to_string()),
            model_name: Some("test-model-1".to_string()),
            user: Some("test-user-1".to_string()),
            notes: Some("Test run 1 notes".to_string()),
        },
        Run {
            id: None,
            timestamp: Some("2024-01-02T11:00:00Z".to_string()),
            vram_usage: Some("2.1/2.5/2.3".to_string()),
            info: Some("app:another updated:2024-01-02".to_string()),
            system_info: Some("arch:amd64 cpu:AMD".to_string()),
            model_info: Some("torch:2.1.0 xformers:0.0.23".to_string()),
            device_info: Some("device:AMD driver:23.12.1".to_string()),
            xformers: Some("false".to_string()),
            model_name: Some("test-model-2".to_string()),
            user: Some("test-user-2".to_string()),
            notes: Some("Test run 2 notes".to_string()),
        },
        Run {
            id: None,
            timestamp: Some("2024-01-03T12:00:00Z".to_string()),
            vram_usage: Some("0.8/1.2/1.0".to_string()),
            info: Some("app:third updated:2024-01-03".to_string()),
            system_info: Some("arch:arm64 cpu:Apple".to_string()),
            model_info: Some("torch:2.2.0 xformers:0.0.24".to_string()),
            device_info: Some("device:Apple driver:1.0.0".to_string()),
            xformers: Some("true".to_string()),
            model_name: Some("test-model-3".to_string()),
            user: Some("test-user-3".to_string()),
            notes: Some("Test run 3 notes".to_string()),
        },
    ]
}

/// Verify that run details were processed correctly
async fn verify_run_details_updates(
    run_details: &[RunMoreDetails],
) -> Result<(), Box<dyn std::error::Error>> {
    assert_eq!(run_details.len(), 3, "Should have 3 run details");
    
    // Verify each run detail has the correct structure
    for run_detail in run_details {
        assert!(run_detail.id.is_some(), "Each run detail should have an ID");
        assert!(run_detail.run_id.is_some(), "Each run detail should have a run_id");
        assert!(run_detail.timestamp.is_some(), "Each run detail should have a timestamp");
        assert!(run_detail.model_name.is_some(), "Each run detail should have a model_name");
        assert!(run_detail.user.is_some(), "Each run detail should have a user");
        assert!(run_detail.notes.is_some(), "Each run detail should have notes");
        assert!(run_detail.model_map_id.is_none(), "model_map_id should be None initially");
    }
    
    // Verify specific data matches expected values
    let model_names: Vec<&String> = run_details.iter()
        .filter_map(|d| d.model_name.as_ref())
        .collect();
    
    let users: Vec<&String> = run_details.iter()
        .filter_map(|d| d.user.as_ref())
        .collect();
    
    assert!(model_names.contains(&&"test-model-1".to_string()), "Should contain test-model-1");
    assert!(model_names.contains(&&"test-model-2".to_string()), "Should contain test-model-2");
    assert!(model_names.contains(&&"test-model-3".to_string()), "Should contain test-model-3");
    
    assert!(users.contains(&&"test-user-1".to_string()), "Should contain test-user-1");
    assert!(users.contains(&&"test-user-2".to_string()), "Should contain test-user-2");
    assert!(users.contains(&&"test-user-3".to_string()), "Should contain test-user-3");
    
    // Verify timestamps are in correct format
    for run_detail in run_details {
        let timestamp = run_detail.timestamp.as_ref().unwrap();
        assert!(timestamp.contains("2024-01-"), "Timestamp should contain 2024-01-");
        assert!(timestamp.contains("T"), "Timestamp should contain T separator");
        assert!(timestamp.ends_with("Z"), "Timestamp should end with Z");
    }
    
    Ok(())
} 