use sqlx::SqlitePool;

use sd_its_benchmark::{
    models::{run_more_details::RunMoreDetails, runs::Run, model_map::ModelMap},
    repositories::{
        run_more_details_repository::RunMoreDetailsRepository,
        runs_repository::RunsRepository,
        model_map_repository::ModelMapRepository,
        traits::Repository,
    },
    services::data_processing::update_run_more_details_service::UpdateRunMoreDetailsService,
};

async fn setup_test_database() -> SqlitePool {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    
    // Run migrations to create tables
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");
    
    pool
}

async fn create_required_runs(pool: &SqlitePool) -> Vec<i64> {
    let runs_repo = RunsRepository::new(pool.clone());
    
    let test_runs = vec![
        Run {
            id: None,
            timestamp: Some("2024-01-01T00:00:00Z".to_string()),
            vram_usage: Some("8GB".to_string()),
            info: Some("test-info-1".to_string()),
            system_info: Some("test-system-1".to_string()),
            model_info: Some("test-model-1".to_string()),
            device_info: Some("test-device-1".to_string()),
            xformers: Some("test-xformers-1".to_string()),
            model_name: Some("model-1".to_string()),
            user: Some("test-user-1".to_string()),
            notes: Some("test-notes-1".to_string()),
        },
        Run {
            id: None,
            timestamp: Some("2024-01-02T00:00:00Z".to_string()),
            vram_usage: Some("8GB".to_string()),
            info: Some("test-info-2".to_string()),
            system_info: Some("test-system-2".to_string()),
            model_info: Some("test-model-2".to_string()),
            device_info: Some("test-device-2".to_string()),
            xformers: Some("test-xformers-2".to_string()),
            model_name: Some("model-2".to_string()),
            user: Some("test-user-2".to_string()),
            notes: Some("test-notes-2".to_string()),
        },
        Run {
            id: None,
            timestamp: Some("2024-01-03T00:00:00Z".to_string()),
            vram_usage: Some("8GB".to_string()),
            info: Some("test-info-3".to_string()),
            system_info: Some("test-system-3".to_string()),
            model_info: Some("test-model-3".to_string()),
            device_info: Some("test-device-3".to_string()),
            xformers: Some("test-xformers-3".to_string()),
            model_name: Some("model-3".to_string()),
            user: Some("test-user-3".to_string()),
            notes: Some("test-notes-3".to_string()),
        },
    ];

    let mut run_ids = Vec::new();
    for run in test_runs {
        let created_run = runs_repo.create(run).await.unwrap();
        run_ids.push(created_run.id.unwrap());
    }
    
    run_ids
}

// Test successful model mapping update
#[tokio::test]
async fn test_update_run_more_details_service_integration() {
    let pool = setup_test_database().await;
    let run_ids = create_required_runs(&pool).await;
    
    let run_more_details_repo = RunMoreDetailsRepository::new(pool.clone());
    let model_map_repo = ModelMapRepository::new(pool.clone());
    
    // Create ModelMap records
    let test_model_maps = vec![
        ModelMap {
            id: None,
            model_name: Some("model-1".to_string()),
            base_model: Some("base-model-1".to_string()),
        },
        ModelMap {
            id: None,
            model_name: Some("model-2".to_string()),
            base_model: Some("base-model-2".to_string()),
        },
    ];

    let mut created_model_maps = Vec::new();
    for model_map in test_model_maps {
        let created_model_map = model_map_repo.create(model_map).await.unwrap();
        created_model_maps.push(created_model_map);
    }
    
    // Create RunMoreDetails records (some with NULL ModelMapId, some with existing ModelMapId)
    let test_run_more_details = vec![
        RunMoreDetails {
            id: None,
            run_id: Some(run_ids[0]),
            timestamp: Some("2024-01-01T00:00:00Z".to_string()),
            model_name: Some("model-1".to_string()),
            user: Some("test-user-1".to_string()),
            notes: Some("test-notes-1".to_string()),
            model_map_id: None, // Will be updated
        },
        RunMoreDetails {
            id: None,
            run_id: Some(run_ids[1]),
            timestamp: Some("2024-01-02T00:00:00Z".to_string()),
            model_name: Some("model-2".to_string()),
            user: Some("test-user-2".to_string()),
            notes: Some("test-notes-2".to_string()),
            model_map_id: None, // Will be updated
        },
        RunMoreDetails {
            id: None,
            run_id: Some(run_ids[2]),
            timestamp: Some("2024-01-03T00:00:00Z".to_string()),
            model_name: Some("model-3".to_string()),
            user: Some("test-user-3".to_string()),
            notes: Some("test-notes-3".to_string()),
            model_map_id: None, // Will not be updated (no matching ModelMap)
        },
    ];

    let mut created_run_more_details = Vec::new();
    for run_more_detail in test_run_more_details {
        let created_run_more_detail = run_more_details_repo.create(run_more_detail).await.unwrap();
        created_run_more_details.push(created_run_more_detail);
    }

    // Create and run the service
    let service = UpdateRunMoreDetailsService::new(run_more_details_repo, model_map_repo);
    let result = service.update_run_more_details_with_modelmapid().await.unwrap();

    // Verify the result
    assert!(result.success);
    assert!(result.message.contains("RunMoreDetails updated with ModelMapId successfully"));
    assert!(result.message.contains("Updated: 2")); // model-1 and model-2 should be updated
    assert!(result.message.contains("Not found: 1")); // model-3 has no matching ModelMap

    // Verify the database state
    let updated_run_more_details_repo = RunMoreDetailsRepository::new(pool.clone());
    let all_details = updated_run_more_details_repo.find_all().await.unwrap();
    
    // Check that model-1 and model-2 have ModelMapId set
    let model_1_details = updated_run_more_details_repo.find_by_model_name("model-1").await.unwrap();
    let model_2_details = updated_run_more_details_repo.find_by_model_name("model-2").await.unwrap();
    let model_3_details = updated_run_more_details_repo.find_by_model_name("model-3").await.unwrap();
    
    assert!(!model_1_details.is_empty());
    assert!(model_1_details[0].model_map_id.is_some(), "model-1 should have ModelMapId set");
    
    assert!(!model_2_details.is_empty());
    assert!(model_2_details[0].model_map_id.is_some(), "model-2 should have ModelMapId set");
    
    assert!(!model_3_details.is_empty());
    assert!(model_3_details[0].model_map_id.is_none(), "model-3 should not have ModelMapId set");
}

// Test with no records to update
#[tokio::test]
async fn test_update_run_more_details_service_no_updates_needed() {
    let pool = setup_test_database().await;
    let run_ids = create_required_runs(&pool).await;
    
    let run_more_details_repo = RunMoreDetailsRepository::new(pool.clone());
    let model_map_repo = ModelMapRepository::new(pool.clone());

    // Create a single run and run more details with existing ModelMapId
    let model_map = ModelMap {
        id: None,
        model_name: Some("model-1".to_string()),
        base_model: Some("base-model-1".to_string()),
    };

    let created_model_map = model_map_repo.create(model_map).await.unwrap();
    let model_map_id = created_model_map.id.unwrap();

    let run_more_detail = RunMoreDetails {
        id: None,
        run_id: Some(run_ids[0]),
        timestamp: Some("2024-01-01T00:00:00Z".to_string()),
        model_name: Some("model-1".to_string()),
        user: Some("test-user".to_string()),
        notes: Some("test-notes".to_string()),
        model_map_id: Some(model_map_id), // Already has ModelMapId
    };

    run_more_details_repo.create(run_more_detail).await.unwrap();

    // Create and run the service
    let service = UpdateRunMoreDetailsService::new(run_more_details_repo, model_map_repo);
    let result = service.update_run_more_details_with_modelmapid().await.unwrap();

    // Verify the result
    assert!(result.success);
    assert_eq!(result.message, "All RunMoreDetails entries already have ModelMapId.");
}

// Test edge cases with NULL model_name
#[tokio::test]
async fn test_update_run_more_details_service_null_model_name() {
    let pool = setup_test_database().await;
    let run_ids = create_required_runs(&pool).await;
    
    let run_more_details_repo = RunMoreDetailsRepository::new(pool.clone());
    let model_map_repo = ModelMapRepository::new(pool.clone());

    // Create RunMoreDetails with NULL model_name
    let run_more_detail = RunMoreDetails {
        id: None,
        run_id: Some(run_ids[0]),
        timestamp: Some("2024-01-01T00:00:00Z".to_string()),
        model_name: None, // NULL model_name
        user: Some("test-user".to_string()),
        notes: Some("test-notes".to_string()),
        model_map_id: None,
    };

    run_more_details_repo.create(run_more_detail).await.unwrap();

    // Create and run the service
    let service = UpdateRunMoreDetailsService::new(run_more_details_repo, model_map_repo);
    let result = service.update_run_more_details_with_modelmapid().await.unwrap();

    // Verify the result
    assert!(result.success);
    let message = result.message;
    assert!(message.contains("Updated: 0")); // No updates due to NULL model_name
    assert!(message.contains("Not found: 1")); // NULL model_name counts as not found
}

// Test empty database
#[tokio::test]
async fn test_update_run_more_details_service_empty_database() {
    let pool = setup_test_database().await;
    
    let run_more_details_repo = RunMoreDetailsRepository::new(pool.clone());
    let model_map_repo = ModelMapRepository::new(pool.clone());

    // Create and run the service with empty database
    let service = UpdateRunMoreDetailsService::new(run_more_details_repo, model_map_repo);
    let result = service.update_run_more_details_with_modelmapid().await.unwrap();

    // Verify the result
    assert!(result.success);
    assert_eq!(result.message, "All RunMoreDetails entries already have ModelMapId.");
} 