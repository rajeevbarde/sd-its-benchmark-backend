use axum::{
    body::to_bytes,
    http::{Method, Request, StatusCode},
    Router,
};
use sqlx::SqlitePool;
use tower::ServiceExt;

use sd_its_benchmark::{
    AppState,
    handlers::admin::update_gpu_laptop_info,
    models::{gpu::Gpu, runs::Run},
    repositories::{
        gpu_repository::GpuRepository,
        runs_repository::RunsRepository,
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
        .route("/api/update-gpu-laptop-info", axum::routing::post(update_gpu_laptop_info))
        .with_state(app_state)
}

async fn setup_test_gpu_data(pool: &SqlitePool) -> Vec<Gpu> {
    // Create test runs first
    let runs_repo = RunsRepository::new(pool.clone());
    let gpu_repo = GpuRepository::new(pool.clone());
    
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

    let mut created_gpus = Vec::new();
    for run in test_runs {
        let created_run = runs_repo.create(run).await.unwrap();
        let run_id = created_run.id.unwrap();

        // Create GPU records with different device types
        let gpu = Gpu {
            id: None,
            run_id: Some(run_id),
            device: Some(format!("cuda:{} 24GB", created_gpus.len())),
            driver: Some("535.86.10".to_string()),
            gpu_chip: Some("gpu:RTX 4090".to_string()),
            brand: Some("nvidia".to_string()),
            is_laptop: None, // Will be populated by the update process
        };

        let created_gpu = gpu_repo.create(gpu).await.unwrap();
        created_gpus.push(created_gpu);
    }

    created_gpus
}

// Test successful GPU laptop info update
#[tokio::test]
async fn test_update_gpu_laptop_info_success() {
    let pool = create_test_pool().await;
    let test_gpus = setup_test_gpu_data(&pool).await;
    assert!(!test_gpus.is_empty(), "Test data setup failed");

    let app_state = AppState { 
        db: pool.clone(),
        settings: sd_its_benchmark::config::settings::Settings::new().unwrap(),
    };
    let app = create_test_app(app_state);

    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/update-gpu-laptop-info")
        .body(axum::body::Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let response_json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(response_json["status"], true);
    assert_eq!(response_json["total_updates"], test_gpus.len());

    // Verify data was updated correctly
    let gpu_repo = GpuRepository::new(pool);
    let all_gpus = gpu_repo.find_all().await.unwrap();
    assert_eq!(all_gpus.len(), test_gpus.len());

    // Check that all GPUs now have laptop information
    for gpu in &all_gpus {
        assert!(gpu.is_laptop.is_some(), "GPU should have laptop information");
        // All our test devices don't contain "Laptop" or "Mobile", so they should be false
        assert_eq!(gpu.is_laptop.unwrap(), false);
    }
}

// Test laptop detection logic
#[tokio::test]
async fn test_laptop_detection_logic() {
    let pool = create_test_pool().await;
    let runs_repo = RunsRepository::new(pool.clone());
    let gpu_repo = GpuRepository::new(pool.clone());

    // Create test runs with different GPU types
    let test_devices = vec![
        ("cuda:0 24GB", false), // Desktop
        ("cuda:0 Laptop 24GB", true), // Laptop
        ("cuda:0 Mobile 16GB", true), // Mobile
        ("cuda:0 24GB Laptop", true), // Laptop at end
        ("Laptop cuda:0 24GB", true), // Laptop at start
        ("Mobile cuda:0 24GB", true), // Mobile at start
        ("cuda:0 24GB Mobile", true), // Mobile at end
        ("cuda:0 24GB", false), // Desktop again
    ];

    let mut created_gpus = Vec::new();
    for (device, expected_laptop) in test_devices {
        let test_run = Run {
            id: None,
            timestamp: Some("2024-01-01T10:00:00Z".to_string()),
            vram_usage: Some("8.5/16.0".to_string()),
            info: Some("app:test-app updated:2024-01-01 hash:abc123 url:https://example.com".to_string()),
            system_info: Some("arch:x86_64 cpu:Intel i5 system:Linux release:5.15.0 python:3.10".to_string()),
            model_info: Some("torch:2.0.1 xformers:0.0.22 diffusers:0.21.4 transformers:4.30.2".to_string()),
            device_info: Some(format!("device:{} driver:535.86.10 gpu:Test", device)),
            xformers: Some("enabled".to_string()),
            model_name: Some("test-model".to_string()),
            user: Some("testuser".to_string()),
            notes: Some("test notes".to_string()),
        };

        let created_run = runs_repo.create(test_run).await.unwrap();
        let run_id = created_run.id.unwrap();

        let gpu = Gpu {
            id: None,
            run_id: Some(run_id),
            device: Some(device.to_string()),
            driver: Some("535.86.10".to_string()),
            gpu_chip: Some("gpu:Test".to_string()),
            brand: Some("nvidia".to_string()),
            is_laptop: None,
        };

        let created_gpu = gpu_repo.create(gpu).await.unwrap();
        created_gpus.push((created_gpu, expected_laptop));
    }

    let app_state = AppState { 
        db: pool.clone(),
        settings: sd_its_benchmark::config::settings::Settings::new().unwrap(),
    };
    let app = create_test_app(app_state);

    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/update-gpu-laptop-info")
        .body(axum::body::Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Verify each GPU was classified correctly
    let gpu_repo = GpuRepository::new(pool);
    let all_gpus = gpu_repo.find_all().await.unwrap();
    
    for (original_gpu, expected_laptop) in created_gpus {
        let updated_gpu = all_gpus.iter().find(|g| g.id == original_gpu.id).unwrap();
        assert_eq!(updated_gpu.is_laptop.unwrap(), expected_laptop);
    }
}

// Test with no GPU data
#[tokio::test]
async fn test_update_gpu_laptop_info_with_no_gpus() {
    let pool = create_test_pool().await;

    let app_state = AppState { 
        db: pool.clone(),
        settings: sd_its_benchmark::config::settings::Settings::new().unwrap(),
    };
    let app = create_test_app(app_state);

    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/update-gpu-laptop-info")
        .body(axum::body::Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let response_json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(response_json["status"], true);
    assert_eq!(response_json["total_updates"], 0);
    assert_eq!(response_json["laptop_only_updates"], 0);
}

// Test response format
#[tokio::test]
async fn test_update_gpu_laptop_info_response_format() {
    let pool = create_test_pool().await;
    let _test_gpus = setup_test_gpu_data(&pool).await;

    let app_state = AppState { 
        db: pool.clone(),
        settings: sd_its_benchmark::config::settings::Settings::new().unwrap(),
    };
    let app = create_test_app(app_state);

    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/update-gpu-laptop-info")
        .body(axum::body::Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let response_json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    // Verify response structure
    assert_eq!(response_json["status"], true);
    assert!(response_json["message"].is_string());
    assert!(response_json["total_updates"].is_number());
    assert!(response_json["laptop_only_updates"].is_number());

    // Verify message content
    assert_eq!(response_json["message"], "GPU laptop information updated successfully!");
    
    // Verify counts are reasonable
    let total_updates = response_json["total_updates"].as_u64().unwrap();
    let laptop_updates = response_json["laptop_only_updates"].as_u64().unwrap();
    
    assert!(total_updates > 0);
    assert!(laptop_updates <= total_updates);
} 