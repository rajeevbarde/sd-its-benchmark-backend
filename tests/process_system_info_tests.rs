use axum::{
    body::to_bytes,
    http::{Method, Request, StatusCode},
    routing::{get, post},
    Router,
};
use sqlx::SqlitePool;
use tower::ServiceExt;

use sd_its_benchmark::{
    handlers::admin::process_system_info,
    models::{runs::Run, system_info::SystemInfo},
    repositories::{
        runs_repository::RunsRepository,
        system_info_repository::SystemInfoRepository,
        traits::{Repository, TransactionRepository},
    },
    AppState,
};

async fn create_test_pool() -> SqlitePool {
    SqlitePool::connect("sqlite::memory:").await.unwrap()
}

// Helper function to create test app
fn create_test_app(app_state: AppState) -> Router {
    Router::new()
        .route("/health", get(|| async { "OK" }))
        .route("/api/process-system-info", post(process_system_info))
        .with_state(app_state)
}

// Helper function to setup test data
async fn setup_test_data(pool: &SqlitePool) -> Vec<Run> {
    // Run migrations to create tables
    sqlx::migrate!("./migrations")
        .run(pool)
        .await
        .expect("Failed to run migrations");

    let runs_repo = RunsRepository::new(pool.clone());
    
    let test_runs = vec![
        Run {
            id: None,
            timestamp: Some("2024-01-01T10:00:00Z".to_string()),
            vram_usage: Some("8.5/16.0".to_string()),
            info: Some("app:stable-diffusion-webui arch:x86_64 cpu:Intel(R) Core(TM) i7-10700K CPU @ 3.80GHz system:Linux release:5.15.0-91-generic python:3.10.12".to_string()),
            system_info: Some("arch:x86_64 cpu:Intel(R) Core(TM) i7-10700K CPU @ 3.80GHz system:Linux release:5.15.0-91-generic python:3.10.12".to_string()),
            model_info: Some("model:stable-diffusion-v1-5".to_string()),
            device_info: Some("device:cuda:0".to_string()),
            xformers: Some("enabled".to_string()),
            model_name: Some("stable-diffusion-v1-5".to_string()),
            user: Some("testuser".to_string()),
            notes: Some("Test run 1".to_string()),
        },
        Run {
            id: None,
            timestamp: Some("2024-01-01T11:00:00Z".to_string()),
            vram_usage: Some("12.0/24.0".to_string()),
            info: Some("app:comfyui arch:amd64 cpu:AMD Ryzen 9 5950X 16-Core Processor system:Windows release:10.0.19045 python:3.11.5".to_string()),
            system_info: Some("arch:amd64 cpu:AMD Ryzen 9 5950X 16-Core Processor system:Windows release:10.0.19045 python:3.11.5".to_string()),
            model_info: Some("model:stable-diffusion-xl".to_string()),
            device_info: Some("device:cuda:1".to_string()),
            xformers: Some("disabled".to_string()),
            model_name: Some("stable-diffusion-xl".to_string()),
            user: Some("testuser2".to_string()),
            notes: Some("Test run 2".to_string()),
        },
        Run {
            id: None,
            timestamp: Some("2024-01-01T12:00:00Z".to_string()),
            vram_usage: Some("6.0/8.0".to_string()),
            info: Some("app:invokeai arch:arm64 cpu:Apple M1 Pro system:Darwin release:22.6.0 python:3.9.18".to_string()),
            system_info: Some("arch:arm64 cpu:Apple M1 Pro system:Darwin release:22.6.0 python:3.9.18".to_string()),
            model_info: Some("model:stable-diffusion-v2-1".to_string()),
            device_info: Some("device:mps".to_string()),
            xformers: Some("n/a".to_string()),
            model_name: Some("stable-diffusion-v2-1".to_string()),
            user: Some("testuser3".to_string()),
            notes: Some("Test run 3".to_string()),
        },
    ];

    let mut created_runs = Vec::new();
    for run in test_runs {
        match runs_repo.create(run).await {
            Ok(created_run) => created_runs.push(created_run),
            Err(e) => eprintln!("Failed to create test run: {}", e),
        }
    }

    created_runs
}

// Unit test for system info parsing logic
#[tokio::test]
async fn test_system_info_parsing_logic() {
    // Test case 1: Standard system info
    let system_info_1 = "arch:x86_64 cpu:Intel(R) Core(TM) i7-10700K CPU @ 3.80GHz system:Linux release:5.15.0-91-generic python:3.10.12";
    let parsed_1 = parse_system_info(system_info_1);
    
    assert_eq!(parsed_1.arch, Some("x86_64".to_string()));
    assert_eq!(parsed_1.cpu, Some("Intel(R) Core(TM) i7-10700K CPU @ 3.80GHz".to_string()));
    assert_eq!(parsed_1.system, Some("Linux".to_string()));
    assert_eq!(parsed_1.release, Some("5.15.0-91-generic".to_string()));
    assert_eq!(parsed_1.python, Some("3.10.12".to_string()));

    // Test case 2: AMD CPU with spaces
    let system_info_2 = "arch:amd64 cpu:AMD Ryzen 9 5950X 16-Core Processor system:Windows release:10.0.19045 python:3.11.5";
    let parsed_2 = parse_system_info(system_info_2);
    
    assert_eq!(parsed_2.arch, Some("amd64".to_string()));
    assert_eq!(parsed_2.cpu, Some("AMD Ryzen 9 5950X 16-Core Processor".to_string()));
    assert_eq!(parsed_2.system, Some("Windows".to_string()));
    assert_eq!(parsed_2.release, Some("10.0.19045".to_string()));
    assert_eq!(parsed_2.python, Some("3.11.5".to_string()));

    // Test case 3: Apple Silicon
    let system_info_3 = "arch:arm64 cpu:Apple M1 Pro system:Darwin release:22.6.0 python:3.9.18";
    let parsed_3 = parse_system_info(system_info_3);
    
    assert_eq!(parsed_3.arch, Some("arm64".to_string()));
    assert_eq!(parsed_3.cpu, Some("Apple M1 Pro".to_string()));
    assert_eq!(parsed_3.system, Some("Darwin".to_string()));
    assert_eq!(parsed_3.release, Some("22.6.0".to_string()));
    assert_eq!(parsed_3.python, Some("3.9.18".to_string()));

    // Test case 4: Missing fields
    let system_info_4 = "arch:x86_64 cpu:Intel i5 system:Linux";
    let parsed_4 = parse_system_info(system_info_4);
    
    assert_eq!(parsed_4.arch, Some("x86_64".to_string()));
    assert_eq!(parsed_4.cpu, Some("Intel i5".to_string()));
    assert_eq!(parsed_4.system, Some("Linux".to_string()));
    assert_eq!(parsed_4.release, None);
    assert_eq!(parsed_4.python, None);

    // Test case 5: Empty string
    let system_info_5 = "";
    let parsed_5 = parse_system_info(system_info_5);
    
    assert_eq!(parsed_5.arch, None);
    assert_eq!(parsed_5.cpu, None);
    assert_eq!(parsed_5.system, None);
    assert_eq!(parsed_5.release, None);
    assert_eq!(parsed_5.python, None);
}

// Integration test for successful system info processing
#[tokio::test]
async fn test_process_system_info_success() {
    let pool = create_test_pool().await;

    // Setup test data
    let test_runs = setup_test_data(&pool).await;
    assert!(!test_runs.is_empty(), "Test data setup failed");

    let app_state = AppState { 
        db: pool.clone(),
        settings: sd_its_benchmark::config::settings::Settings::new().unwrap(),
    };
    let app = create_test_app(app_state);

    // Make request to process system info
    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/process-system-info")
        .body(axum::body::Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let response_json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(response_json["success"], true);
    assert!(response_json["rows_inserted"].as_u64().unwrap() > 0);

    // Verify data was inserted correctly
    let system_info_repo = SystemInfoRepository::new(pool);
    let all_system_info = system_info_repo.find_all().await.unwrap();
    
    assert_eq!(all_system_info.len(), test_runs.len());

    // Check specific entries
    for system_info in all_system_info {
        assert!(system_info.run_id.is_some());
        assert!(system_info.arch.is_some());
        assert!(system_info.cpu.is_some());
        assert!(system_info.system.is_some());
        assert!(system_info.release.is_some());
        assert!(system_info.python.is_some());
    }
}

// Test that existing data is cleared before processing
#[tokio::test]
async fn test_process_system_info_clears_existing_data() {
    let pool = create_test_pool().await;

    // Setup test data
    let test_runs = setup_test_data(&pool).await;
    assert!(!test_runs.is_empty(), "Test data setup failed");

    let app_state = AppState { 
        db: pool.clone(),
        settings: sd_its_benchmark::config::settings::Settings::new().unwrap(),
    };
    let app = create_test_app(app_state);

    // First call to process system info
    let request1 = Request::builder()
        .method(Method::POST)
        .uri("/api/process-system-info")
        .body(axum::body::Body::empty())
        .unwrap();

    let response1 = app.clone().oneshot(request1).await.unwrap();
    assert_eq!(response1.status(), StatusCode::OK);

    // Verify first batch was inserted
    let system_info_repo = SystemInfoRepository::new(pool.clone());
    let first_batch = system_info_repo.find_all().await.unwrap();
    assert_eq!(first_batch.len(), test_runs.len());

    // Second call to process system info (should clear existing data)
    let request2 = Request::builder()
        .method(Method::POST)
        .uri("/api/process-system-info")
        .body(axum::body::Body::empty())
        .unwrap();

    let response2 = app.oneshot(request2).await.unwrap();
    assert_eq!(response2.status(), StatusCode::OK);

    // Verify data was cleared and re-inserted
    let second_batch = system_info_repo.find_all().await.unwrap();
    assert_eq!(second_batch.len(), test_runs.len());
}

// Test with no runs data
#[tokio::test]
async fn test_process_system_info_with_no_runs() {
    let pool = create_test_pool().await;

    // Run migrations to create tables
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    let app_state = AppState { 
        db: pool.clone(),
        settings: sd_its_benchmark::config::settings::Settings::new().unwrap(),
    };
    let app = create_test_app(app_state);

    // Make request to process system info
    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/process-system-info")
        .body(axum::body::Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let response_json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(response_json["success"], true);
    assert_eq!(response_json["rows_inserted"], 0);

    // Verify no data was inserted
    let system_info_repo = SystemInfoRepository::new(pool);
    let all_system_info = system_info_repo.find_all().await.unwrap();
    assert_eq!(all_system_info.len(), 0);
}

// Test system info repository clear methods
#[tokio::test]
async fn test_system_info_repository_clear_methods() {
    let pool = create_test_pool().await;

    // Run migrations to create tables
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    let system_info_repo = SystemInfoRepository::new(pool.clone());

    // Create a test run first
    let runs_repo = RunsRepository::new(pool.clone());
    let test_run = Run {
        id: None,
        timestamp: Some("2024-01-01T10:00:00Z".to_string()),
        vram_usage: Some("8.5/16.0".to_string()),
        info: Some("test info".to_string()),
        system_info: Some("arch:x86_64 cpu:Intel i5 system:Linux release:5.15.0 python:3.10".to_string()),
        model_info: Some("test model".to_string()),
        device_info: Some("test device".to_string()),
        xformers: Some("enabled".to_string()),
        model_name: Some("test-model".to_string()),
        user: Some("testuser".to_string()),
        notes: Some("test notes".to_string()),
    };

    let created_run = runs_repo.create(test_run).await.unwrap();
    let run_id = created_run.id.unwrap();

    // Create test system info
    let test_system_info = SystemInfo {
        id: None,
        run_id: Some(run_id),
        arch: Some("x86_64".to_string()),
        cpu: Some("Intel i5".to_string()),
        system: Some("Linux".to_string()),
        release: Some("5.15.0".to_string()),
        python: Some("3.10".to_string()),
    };

    let created_system_info = system_info_repo.create(test_system_info).await.unwrap();
    assert!(created_system_info.id.is_some());

    // Test clear_all method
    system_info_repo.clear_all().await.unwrap();
    let all_system_info = system_info_repo.find_all().await.unwrap();
    assert_eq!(all_system_info.len(), 0);

    // Test clear_all_tx method
    let mut tx = pool.begin().await.unwrap();
    let test_system_info_2 = SystemInfo {
        id: None,
        run_id: Some(run_id),
        arch: Some("amd64".to_string()),
        cpu: Some("AMD Ryzen".to_string()),
        system: Some("Windows".to_string()),
        release: Some("10.0".to_string()),
        python: Some("3.11".to_string()),
    };

    system_info_repo.create_tx(test_system_info_2, &mut tx).await.unwrap();
    system_info_repo.clear_all_tx(&mut tx).await.unwrap();
    tx.commit().await.unwrap();

    let all_system_info_2 = system_info_repo.find_all().await.unwrap();
    assert_eq!(all_system_info_2.len(), 0);
}

// Copy the parsing function from the handler for testing
#[derive(Debug)]
struct ParsedSystemInfo {
    arch: Option<String>,
    cpu: Option<String>,
    system: Option<String>,
    release: Option<String>,
    python: Option<String>,
}

fn parse_system_info(system_info_string: &str) -> ParsedSystemInfo {
    let parts: Vec<&str> = system_info_string.split(' ').collect();
    let mut system_info = ParsedSystemInfo {
        arch: None,
        cpu: None,
        system: None,
        release: None,
        python: None,
    };

    let mut cpu_value = String::new();
    let mut is_cpu_field = false;

    for part in parts {
        let colon_index = match part.find(':') {
            Some(index) => index,
            None => {
                if is_cpu_field {
                    cpu_value.push(' ');
                    cpu_value.push_str(part);
                }
                continue;
            }
        };

        let key = &part[..colon_index];
        let value = &part[colon_index + 1..];

        match key {
            "arch" => {
                system_info.arch = Some(value.to_string());
            }
            "cpu" => {
                is_cpu_field = true;
                cpu_value = value.to_string();
            }
            "system" => {
                system_info.system = Some(value.to_string());
                is_cpu_field = false;
                system_info.cpu = Some(cpu_value.trim().to_string());
                cpu_value.clear();
            }
            "release" => {
                system_info.release = Some(value.to_string());
            }
            "python" => {
                system_info.python = Some(value.to_string());
            }
            _ => continue,
        }
    }

    // Handle case where cpu field is at the end
    if is_cpu_field {
        system_info.cpu = Some(cpu_value.trim().to_string());
    }

    system_info
} 