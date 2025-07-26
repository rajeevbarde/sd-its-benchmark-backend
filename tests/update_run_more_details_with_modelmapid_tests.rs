use axum::{
    body::to_bytes,
    http::{Method, Request, StatusCode},
    Router,
};
use sqlx::SqlitePool;
use tower::ServiceExt;

use sd_its_benchmark::{
    AppState,
    handlers::admin::update_run_more_details_with_modelmapid,
    models::{run_more_details::RunMoreDetails, runs::Run, model_map::ModelMap},
    repositories::{
        run_more_details_repository::RunMoreDetailsRepository,
        runs_repository::RunsRepository,
        model_map_repository::ModelMapRepository,
        traits::Repository,
    },
};

async fn create_test_pool() -> SqlitePool {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    
    // Run migrations to create tables
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");
    
    pool
}

fn create_test_app(app_state: AppState) -> Router {
    Router::new()
        .route("/api/update-run-more-details-with-modelmapid", axum::routing::post(update_run_more_details_with_modelmapid))
        .with_state(app_state)
}

async fn setup_test_data(pool: &SqlitePool) -> (Vec<RunMoreDetails>, Vec<ModelMap>) {
    let runs_repo = RunsRepository::new(pool.clone());
    let run_more_details_repo = RunMoreDetailsRepository::new(pool.clone());
    let model_map_repo = ModelMapRepository::new(pool.clone());
    
    // First, create the required Run records
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

    // Create the Run records and collect their IDs
    let mut run_ids = Vec::new();
    for run in test_runs {
        let created_run = runs_repo.create(run).await.unwrap();
        run_ids.push(created_run.id.unwrap());
    }
    
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

    (created_run_more_details, created_model_maps)
}

// Test successful model mapping update
#[tokio::test]
async fn test_update_run_more_details_with_modelmapid_success() {
    let pool = create_test_pool().await;
    let (test_run_more_details, test_model_maps) = setup_test_data(&pool).await;
    assert!(!test_run_more_details.is_empty(), "Test data setup failed");

    let app_state = AppState { 
        db: pool.clone(),
        settings: sd_its_benchmark::config::settings::Settings::new().unwrap(),
    };
    let app = create_test_app(app_state);

    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/update-run-more-details-with-modelmapid")
        .body(axum::body::Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let response_json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    // Verify response structure
    assert_eq!(response_json["success"], true);
    assert!(response_json["message"].is_string());

    let message = response_json["message"].as_str().unwrap();
    assert!(message.contains("RunMoreDetails updated with ModelMapId successfully"));
    assert!(message.contains("Updated: 2")); // model-1 and model-2 should be updated
    assert!(message.contains("Not found: 1")); // model-3 has no matching ModelMap
}

// Test with no records to update
#[tokio::test]
async fn test_update_run_more_details_with_modelmapid_no_updates_needed() {
    let pool = create_test_pool().await;
    let runs_repo = RunsRepository::new(pool.clone());
    let run_more_details_repo = RunMoreDetailsRepository::new(pool.clone());
    let model_map_repo = ModelMapRepository::new(pool.clone());

    // Create a single run and run more details with existing ModelMapId
    let run = Run {
        id: None,
        timestamp: Some("2024-01-01T00:00:00Z".to_string()),
        vram_usage: Some("8GB".to_string()),
        info: Some("test-info".to_string()),
        system_info: Some("test-system".to_string()),
        model_info: Some("test-model".to_string()),
        device_info: Some("test-device".to_string()),
        xformers: Some("test-xformers".to_string()),
        model_name: Some("model-1".to_string()),
        user: Some("test-user".to_string()),
        notes: Some("test-notes".to_string()),
    };

    let created_run = runs_repo.create(run).await.unwrap();
    let run_id = created_run.id.unwrap();

    let model_map = ModelMap {
        id: None,
        model_name: Some("model-1".to_string()),
        base_model: Some("base-model-1".to_string()),
    };

    let created_model_map = model_map_repo.create(model_map).await.unwrap();
    let model_map_id = created_model_map.id.unwrap();

    let run_more_detail = RunMoreDetails {
        id: None,
        run_id: Some(run_id),
        timestamp: Some("2024-01-01T00:00:00Z".to_string()),
        model_name: Some("model-1".to_string()),
        user: Some("test-user".to_string()),
        notes: Some("test-notes".to_string()),
        model_map_id: Some(model_map_id), // Already has ModelMapId
    };

    run_more_details_repo.create(run_more_detail).await.unwrap();

    let app_state = AppState { 
        db: pool.clone(),
        settings: sd_its_benchmark::config::settings::Settings::new().unwrap(),
    };
    let app = create_test_app(app_state);

    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/update-run-more-details-with-modelmapid")
        .body(axum::body::Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let response_json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    // Verify response
    assert_eq!(response_json["success"], true);
    assert_eq!(response_json["message"], "All RunMoreDetails entries already have ModelMapId.");
}

// Test response format
#[tokio::test]
async fn test_update_run_more_details_with_modelmapid_response_format() {
    let pool = create_test_pool().await;
    let _test_data = setup_test_data(&pool).await;

    let app_state = AppState { 
        db: pool.clone(),
        settings: sd_its_benchmark::config::settings::Settings::new().unwrap(),
    };
    let app = create_test_app(app_state);

    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/update-run-more-details-with-modelmapid")
        .body(axum::body::Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let response_json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    // Verify response structure
    assert!(response_json["success"].is_boolean());
    assert!(response_json["message"].is_string());

    let success = response_json["success"].as_bool().unwrap();
    let message = response_json["message"].as_str().unwrap();

    assert!(success);
    assert!(!message.is_empty());
}

// Test edge cases with NULL model_name
#[tokio::test]
async fn test_update_run_more_details_with_modelmapid_null_model_name() {
    let pool = create_test_pool().await;
    let runs_repo = RunsRepository::new(pool.clone());
    let run_more_details_repo = RunMoreDetailsRepository::new(pool.clone());

    // Create a run
    let run = Run {
        id: None,
        timestamp: Some("2024-01-01T00:00:00Z".to_string()),
        vram_usage: Some("8GB".to_string()),
        info: Some("test-info".to_string()),
        system_info: Some("test-system".to_string()),
        model_info: Some("test-model".to_string()),
        device_info: Some("test-device".to_string()),
        xformers: Some("test-xformers".to_string()),
        model_name: Some("model-1".to_string()),
        user: Some("test-user".to_string()),
        notes: Some("test-notes".to_string()),
    };

    let created_run = runs_repo.create(run).await.unwrap();
    let run_id = created_run.id.unwrap();

    // Create RunMoreDetails with NULL model_name
    let run_more_detail = RunMoreDetails {
        id: None,
        run_id: Some(run_id),
        timestamp: Some("2024-01-01T00:00:00Z".to_string()),
        model_name: None, // NULL model_name
        user: Some("test-user".to_string()),
        notes: Some("test-notes".to_string()),
        model_map_id: None,
    };

    run_more_details_repo.create(run_more_detail).await.unwrap();

    let app_state = AppState { 
        db: pool.clone(),
        settings: sd_its_benchmark::config::settings::Settings::new().unwrap(),
    };
    let app = create_test_app(app_state);

    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/update-run-more-details-with-modelmapid")
        .body(axum::body::Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let response_json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    // Verify response
    assert_eq!(response_json["success"], true);
    let message = response_json["message"].as_str().unwrap();
    assert!(message.contains("Updated: 0")); // No updates due to NULL model_name
    assert!(message.contains("Not found: 1")); // NULL model_name counts as not found
} 