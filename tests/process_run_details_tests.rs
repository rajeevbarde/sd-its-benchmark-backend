use axum::{
    body::to_bytes,
    http::{Method, Request, StatusCode},
    Router,
};
use sqlx::SqlitePool;
use tower::ServiceExt;

use sd_its_benchmark::{
    AppState,
    handlers::admin::process_run_details,
    models::{runs::Run, run_more_details::RunMoreDetails},
    repositories::{
        runs_repository::RunsRepository,
        run_more_details_repository::RunMoreDetailsRepository,
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
        .route("/api/process-run-details", axum::routing::post(process_run_details))
        .with_state(app_state)
}

async fn setup_test_runs_data(pool: &SqlitePool) -> Vec<Run> {
    let runs_repo = RunsRepository::new(pool.clone());
    
    let test_runs = vec![
        Run {
            id: None,
            timestamp: Some("2024-01-01T10:00:00Z".to_string()),
            vram_usage: Some("8.5/16.0".to_string()),
            info: Some("app:test-app updated:2024-01-01 hash:abc123 url:https://example.com".to_string()),
            system_info: Some("arch:x86_64 cpu:Intel i5 system:Linux release:5.15.0 python:3.10".to_string()),
            model_info: Some("torch:2.0.1 xformers:0.0.22 diffusers:0.21.4 transformers:4.30.2".to_string()),
            device_info: Some("device:cuda:0 driver:535.86.10 gpu:RTX 4090 24GB".to_string()),
            xformers: Some("enabled".to_string()),
            model_name: Some("test-model-1".to_string()),
            user: Some("testuser1".to_string()),
            notes: Some("test notes 1".to_string()),
        },
        Run {
            id: None,
            timestamp: Some("2024-01-02T11:00:00Z".to_string()),
            vram_usage: Some("12.0/24.0".to_string()),
            info: Some("app:another-app updated:2024-01-02 hash:def456 url:https://example2.com".to_string()),
            system_info: Some("arch:amd64 cpu:AMD Ryzen system:Windows release:10.0 python:3.11".to_string()),
            model_info: Some("torch:2.1.0 xformers:0.0.23 diffusers:0.22.0 transformers:4.31.0".to_string()),
            device_info: Some("device:cuda:1 driver:545.23.08 gpu:RTX 4080 16GB".to_string()),
            xformers: Some("disabled".to_string()),
            model_name: Some("test-model-2".to_string()),
            user: Some("testuser2".to_string()),
            notes: Some("test notes 2".to_string()),
        },
        Run {
            id: None,
            timestamp: Some("2024-01-03T12:00:00Z".to_string()),
            vram_usage: Some("6.0/8.0".to_string()),
            info: Some("app:simple-app updated:2024-01-03 hash:ghi789 url:https://example3.com".to_string()),
            system_info: Some("arch:x86_64 cpu:Intel i7 system:macOS release:13.0 python:3.9".to_string()),
            model_info: Some("torch:1.13.1 xformers:0.0.20 diffusers:0.20.0 transformers:4.28.0".to_string()),
            device_info: Some("device:cpu driver:N/A gpu:Integrated".to_string()),
            xformers: Some("not-applicable".to_string()),
            model_name: Some("test-model-3".to_string()),
            user: Some("testuser3".to_string()),
            notes: Some("test notes 3".to_string()),
        },
    ];

    let mut created_runs = Vec::new();
    for run in test_runs {
        let created_run = runs_repo.create(run).await.unwrap();
        created_runs.push(created_run);
    }

    created_runs
}

// Test successful run details processing
#[tokio::test]
async fn test_process_run_details_success() {
    let pool = create_test_pool().await;
    let test_runs = setup_test_runs_data(&pool).await;
    assert!(!test_runs.is_empty(), "Test data setup failed");

    let app_state = AppState { 
        db: pool.clone(),
        settings: sd_its_benchmark::config::settings::Settings::new().unwrap(),
    };
    let app = create_test_app(app_state);

    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/process-run-details")
        .body(axum::body::Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let response_json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(response_json["success"], true);
    assert_eq!(response_json["total_inserts"], test_runs.len());

    // Verify data was inserted correctly
    let run_more_details_repo = RunMoreDetailsRepository::new(pool);
    let all_run_details = run_more_details_repo.find_all().await.unwrap();
    assert_eq!(all_run_details.len(), test_runs.len());

    // Check that all run details have the correct data
    for (i, run_detail) in all_run_details.iter().enumerate() {
        let original_run = &test_runs[i];
        assert_eq!(run_detail.run_id, original_run.id);
        assert_eq!(run_detail.timestamp, original_run.timestamp);
        assert_eq!(run_detail.model_name, original_run.model_name);
        assert_eq!(run_detail.user, original_run.user);
        assert_eq!(run_detail.notes, original_run.notes);
    }
}

// Test that existing data is cleared before processing
#[tokio::test]
async fn test_process_run_details_clears_existing_data() {
    let pool = create_test_pool().await;
    let test_runs = setup_test_runs_data(&pool).await;
    assert!(!test_runs.is_empty(), "Test data setup failed");

    // First, manually insert some data into RunMoreDetails using a valid run ID
    let run_more_details_repo = RunMoreDetailsRepository::new(pool.clone());
    let existing_detail = RunMoreDetails {
        id: None,
        run_id: test_runs[0].id, // Use the first valid run ID
        timestamp: Some("2023-01-01T00:00:00Z".to_string()),
        model_name: Some("old-model".to_string()),
        user: Some("old-user".to_string()),
        notes: Some("old-notes".to_string()),
        model_map_id: None,
    };
    run_more_details_repo.create(existing_detail).await.unwrap();

    // Verify the old data exists
    let old_data = run_more_details_repo.find_all().await.unwrap();
    assert_eq!(old_data.len(), 1);

    let app_state = AppState { 
        db: pool.clone(),
        settings: sd_its_benchmark::config::settings::Settings::new().unwrap(),
    };
    let app = create_test_app(app_state);

    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/process-run-details")
        .body(axum::body::Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let response_json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(response_json["success"], true);
    assert_eq!(response_json["total_inserts"], test_runs.len());

    // Verify old data was cleared and new data was inserted
    let all_run_details = run_more_details_repo.find_all().await.unwrap();
    assert_eq!(all_run_details.len(), test_runs.len());

    // Verify old data was replaced with new data
    for run_detail in &all_run_details {
        assert_ne!(run_detail.model_name, Some("old-model".to_string()));
        assert_ne!(run_detail.user, Some("old-user".to_string()));
    }
}

// Test with no runs data
#[tokio::test]
async fn test_process_run_details_with_no_runs() {
    let pool = create_test_pool().await;

    let app_state = AppState { 
        db: pool.clone(),
        settings: sd_its_benchmark::config::settings::Settings::new().unwrap(),
    };
    let app = create_test_app(app_state);

    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/process-run-details")
        .body(axum::body::Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let response_json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(response_json["success"], true);
    assert_eq!(response_json["total_inserts"], 0);

    // Verify no data was inserted
    let run_more_details_repo = RunMoreDetailsRepository::new(pool);
    let all_run_details = run_more_details_repo.find_all().await.unwrap();
    assert_eq!(all_run_details.len(), 0);
}

// Test response format
#[tokio::test]
async fn test_process_run_details_response_format() {
    let pool = create_test_pool().await;
    let _test_runs = setup_test_runs_data(&pool).await;

    let app_state = AppState { 
        db: pool.clone(),
        settings: sd_its_benchmark::config::settings::Settings::new().unwrap(),
    };
    let app = create_test_app(app_state);

    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/process-run-details")
        .body(axum::body::Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let response_json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    // Verify response structure
    assert!(response_json["success"].is_boolean());
    assert!(response_json["total_inserts"].is_number());

    // Verify success is true
    assert_eq!(response_json["success"], true);
    
    // Verify total_inserts is reasonable
    let total_inserts = response_json["total_inserts"].as_u64().unwrap();
    assert!(total_inserts > 0);
}

// Test repository clear methods
#[tokio::test]
async fn test_run_more_details_repository_clear_methods() {
    let pool = create_test_pool().await;
    let run_more_details_repo = RunMoreDetailsRepository::new(pool.clone());

    // Create a valid run first
    let runs_repo = RunsRepository::new(pool.clone());
    let test_run = Run {
        id: None,
        timestamp: Some("2024-01-01T10:00:00Z".to_string()),
        vram_usage: Some("8.5/16.0".to_string()),
        info: Some("app:test-app updated:2024-01-01 hash:abc123 url:https://example.com".to_string()),
        system_info: Some("arch:x86_64 cpu:Intel i5 system:Linux release:5.15.0 python:3.10".to_string()),
        model_info: Some("torch:2.0.1 xformers:0.0.22 diffusers:0.21.4 transformers:4.30.2".to_string()),
        device_info: Some("device:cuda:0 driver:535.86.10 gpu:RTX 4090 24GB".to_string()),
        xformers: Some("enabled".to_string()),
        model_name: Some("test-model".to_string()),
        user: Some("testuser".to_string()),
        notes: Some("test notes".to_string()),
    };
    let created_run = runs_repo.create(test_run).await.unwrap();
    let run_id = created_run.id.unwrap();

    // Insert some test data with valid run_id
    let test_detail = RunMoreDetails {
        id: None,
        run_id: Some(run_id),
        timestamp: Some("2024-01-01T10:00:00Z".to_string()),
        model_name: Some("test-model".to_string()),
        user: Some("testuser".to_string()),
        notes: Some("test notes".to_string()),
        model_map_id: None,
    };
    run_more_details_repo.create(test_detail).await.unwrap();

    // Verify data exists
    let all_details = run_more_details_repo.find_all().await.unwrap();
    assert_eq!(all_details.len(), 1);

    // Test clear_all method
    run_more_details_repo.clear_all().await.unwrap();

    // Verify data was cleared
    let all_details_after_clear = run_more_details_repo.find_all().await.unwrap();
    assert_eq!(all_details_after_clear.len(), 0);
}
