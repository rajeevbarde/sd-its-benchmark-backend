use axum::{
    body::to_bytes,
    http::{Method, Request, StatusCode},
    Router,
};
use sqlx::SqlitePool;
use tower::ServiceExt;

use sd_its_benchmark::{
    AppState,
    handlers::admin::process_libraries,
    models::{libraries::Libraries, runs::Run},
    repositories::{
        libraries_repository::LibrariesRepository,
        runs_repository::RunsRepository,
        traits::{Repository, TransactionRepository},
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
        .route("/api/process-libraries", axum::routing::post(process_libraries))
        .with_state(app_state)
}

async fn setup_test_data(pool: &SqlitePool) -> Vec<Run> {
    let runs_repo = RunsRepository::new(pool.clone());
    
    let test_runs = vec![
        Run {
            id: None,
            timestamp: Some("2024-01-01T10:00:00Z".to_string()),
            vram_usage: Some("8.5/16.0".to_string()),
            info: Some("app:test-app updated:2024-01-01 hash:abc123 url:https://example.com".to_string()),
            system_info: Some("arch:x86_64 cpu:Intel i5 system:Linux release:5.15.0 python:3.10".to_string()),
            model_info: Some("torch:2.0.1 autocast half xformers:0.0.22 diffusers:0.21.4 transformers:4.30.2".to_string()),
            device_info: Some("device:cuda:0 driver:535.86.10 gpu:RTX 4090 24GB".to_string()),
            xformers: Some("enabled".to_string()),
            model_name: Some("test-model".to_string()),
            user: Some("testuser".to_string()),
            notes: Some("test notes".to_string()),
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
            model_name: Some("another-model".to_string()),
            user: Some("anotheruser".to_string()),
            notes: Some("another test".to_string()),
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
            model_name: Some("simple-model".to_string()),
            user: Some("simpleuser".to_string()),
            notes: Some("simple test".to_string()),
        },
    ];

    let mut created_runs = Vec::new();
    for run in test_runs {
        let created_run = runs_repo.create(run).await.unwrap();
        created_runs.push(created_run);
    }

    created_runs
}

// Test successful libraries processing
#[tokio::test]
async fn test_process_libraries_success() {
    let pool = create_test_pool().await;
    let test_runs = setup_test_data(&pool).await;
    assert!(!test_runs.is_empty(), "Test data setup failed");

    let app_state = AppState { 
        db: pool.clone(),
        settings: sd_its_benchmark::config::settings::Settings::new().unwrap(),
    };
    let app = create_test_app(app_state);

    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/process-libraries")
        .body(axum::body::Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let response_json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(response_json["success"], true);
    assert_eq!(response_json["rows_inserted"], test_runs.len());

    // Verify data was inserted correctly
    let libraries_repo = LibrariesRepository::new(pool);
    let all_libraries = libraries_repo.find_all().await.unwrap();
    assert_eq!(all_libraries.len(), test_runs.len());

    // Check specific values from first run
    let first_library = &all_libraries[0];
    assert_eq!(first_library.run_id, test_runs[0].id);
    assert_eq!(first_library.torch, Some("2.0.1 autocast half".to_string()));
    assert_eq!(first_library.xformers, Some("0.0.22".to_string()));
    assert_eq!(first_library.xformers1, Some("enabled".to_string())); // From runs.xformers
    assert_eq!(first_library.diffusers, Some("0.21.4".to_string()));
    assert_eq!(first_library.transformers, Some("4.30.2".to_string()));
}

// Test that existing libraries are cleared
#[tokio::test]
async fn test_process_libraries_clears_existing_data() {
    let pool = create_test_pool().await;
    
    // Create a test run first
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

    // Create existing libraries data
    let libraries_repo = LibrariesRepository::new(pool.clone());
    let existing_library = Libraries {
        id: None,
        run_id: Some(run_id),
        torch: Some("old-torch".to_string()),
        xformers: Some("old-xformers".to_string()),
        xformers1: Some("old-xformers1".to_string()),
        diffusers: Some("old-diffusers".to_string()),
        transformers: Some("old-transformers".to_string()),
    };

    libraries_repo.create(existing_library).await.unwrap();
    
    // Verify existing data exists
    let existing_libraries = libraries_repo.find_all().await.unwrap();
    assert_eq!(existing_libraries.len(), 1);

    let app_state = AppState { 
        db: pool.clone(),
        settings: sd_its_benchmark::config::settings::Settings::new().unwrap(),
    };
    let app = create_test_app(app_state);

    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/process-libraries")
        .body(axum::body::Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let response_json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(response_json["success"], true);
    assert_eq!(response_json["rows_inserted"], 1);

    // Verify old data was replaced
    let updated_libraries = libraries_repo.find_all().await.unwrap();
    assert_eq!(updated_libraries.len(), 1);
    
    let updated_library = &updated_libraries[0];
    assert_eq!(updated_library.torch, Some("2.0.1".to_string()));
    assert_eq!(updated_library.xformers, Some("0.0.22".to_string()));
    assert_eq!(updated_library.xformers1, Some("enabled".to_string()));
    assert_eq!(updated_library.diffusers, Some("0.21.4".to_string()));
    assert_eq!(updated_library.transformers, Some("4.30.2".to_string()));
}

// Test with no runs data
#[tokio::test]
async fn test_process_libraries_with_no_runs() {
    let pool = create_test_pool().await;

    let app_state = AppState { 
        db: pool.clone(),
        settings: sd_its_benchmark::config::settings::Settings::new().unwrap(),
    };
    let app = create_test_app(app_state);

    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/process-libraries")
        .body(axum::body::Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let response_json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(response_json["success"], true);
    assert_eq!(response_json["rows_inserted"], 0);

    // Verify no data was inserted
    let libraries_repo = LibrariesRepository::new(pool);
    let all_libraries = libraries_repo.find_all().await.unwrap();
    assert_eq!(all_libraries.len(), 0);
}

// Test libraries repository clear methods
#[tokio::test]
async fn test_libraries_repository_clear_methods() {
    let pool = create_test_pool().await;

    let libraries_repo = LibrariesRepository::new(pool.clone());

    // Create a test run first
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

    // Create test libraries
    let test_library = Libraries {
        id: None,
        run_id: Some(run_id),
        torch: Some("2.0.1".to_string()),
        xformers: Some("0.0.22".to_string()),
        xformers1: Some("enabled".to_string()),
        diffusers: Some("0.21.4".to_string()),
        transformers: Some("4.30.2".to_string()),
    };

    let created_library = libraries_repo.create(test_library).await.unwrap();
    assert!(created_library.id.is_some());

    // Test clear_all method
    libraries_repo.clear_all().await.unwrap();
    let all_libraries = libraries_repo.find_all().await.unwrap();
    assert_eq!(all_libraries.len(), 0);

    // Test clear_all_tx method
    let mut tx = pool.begin().await.unwrap();
    let test_library_2 = Libraries {
        id: None,
        run_id: Some(run_id),
        torch: Some("2.1.0".to_string()),
        xformers: Some("0.0.23".to_string()),
        xformers1: Some("disabled".to_string()),
        diffusers: Some("0.22.0".to_string()),
        transformers: Some("4.31.0".to_string()),
    };

    libraries_repo.create_tx(test_library_2, &mut tx).await.unwrap();
    libraries_repo.clear_all_tx(&mut tx).await.unwrap();
    tx.commit().await.unwrap();

    let all_libraries_after_tx = libraries_repo.find_all().await.unwrap();
    assert_eq!(all_libraries_after_tx.len(), 0);
} 