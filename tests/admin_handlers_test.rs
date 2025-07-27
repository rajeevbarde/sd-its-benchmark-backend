use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
    routing::post,
};
use serde_json::{json, Value};
use tower::ServiceExt;

use sd_its_benchmark::{
    AppState,
    config::{Settings, database::{DatabaseConfig, create_pool, initialize_database}},
    handlers::admin::save_data,
    models::runs::Run,
    repositories::{runs_repository::RunsRepository, traits::Repository},
};

async fn create_test_app_state() -> AppState {
    let settings = Settings::default();
    let db_config = DatabaseConfig {
        url: "sqlite::memory:".to_string(),
        ..DatabaseConfig::default()
    };
    let db_pool = create_pool(&db_config).await.expect("Failed to create test pool");
    initialize_database(&db_pool).await.expect("Failed to initialize test database");
    
    AppState {
        db: db_pool,
        settings,
    }
}

fn create_multipart_body(json_data: &str) -> String {
    let boundary = "----WebKitFormBoundary7MA4YWxkTrZu0gW";
    format!(
        "--{boundary}\r\n\
        Content-Disposition: form-data; name=\"file\"; filename=\"test.json\"\r\n\
        Content-Type: application/json\r\n\
        \r\n\
        {json_data}\r\n\
        --{boundary}--\r\n",
        boundary = boundary,
        json_data = json_data
    )
}

#[tokio::test]
async fn test_save_data_success() {
    let app_state = create_test_app_state().await;
    
    let test_data = json!([
        {
            "timestamp": "2024-01-01T10:00:00Z",
            "vram_usage": "8GB",
            "info": "Test run 1",
            "system_info": "Windows 11",
            "model_info": "SDXL",
            "device_info": "RTX 4090",
            "xformers": "true",
            "model_name": "stable-diffusion-xl",
            "user": "testuser1",
            "notes": "Test notes"
        },
        {
            "timestamp": "2024-01-01T11:00:00Z",
            "vram_usage": "6GB",
            "info": "Test run 2",
            "system_info": "Ubuntu 22.04",
            "model_info": "SD1.5",
            "device_info": "RTX 3080",
            "xformers": "false",
            "model_name": "stable-diffusion-1-5",
            "user": "testuser2",
            "notes": "Test notes 2"
        }
    ]);

    let app = Router::new()
        .route("/api/save-data", post(save_data))
        .with_state(app_state.clone());

    let boundary = "----WebKitFormBoundary7MA4YWxkTrZu0gW";
    let body = create_multipart_body(&test_data.to_string());

    let request = Request::builder()
        .method("POST")
        .uri("/api/save-data")
        .header("content-type", format!("multipart/form-data; boundary={}", boundary))
        .body(Body::from(body))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let response_json: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(response_json["success"], true);
    assert_eq!(response_json["rows_processed"], 2);
    assert_eq!(response_json["rows_inserted"], 2);
    assert_eq!(response_json["rows_failed"], 0);

    // Verify data was actually saved
    let runs_repo = RunsRepository::new(app_state.db.clone());
    let all_runs = runs_repo.find_all().await.unwrap();
    assert_eq!(all_runs.len(), 2);
    
    // Verify specific data
    assert_eq!(all_runs[0].model_name, Some("stable-diffusion-1-5".to_string()));
    assert_eq!(all_runs[1].model_name, Some("stable-diffusion-xl".to_string()));
}

#[tokio::test]
async fn test_save_data_replaces_existing_data() {
    let app_state = create_test_app_state().await;
    
    // Insert some initial data
    let runs_repo = RunsRepository::new(app_state.db.clone());
    let initial_run = Run {
        id: None,
        timestamp: Some("2023-01-01T00:00:00Z".to_string()),
        vram_usage: Some("4GB".to_string()),
        info: Some("Initial run".to_string()),
        system_info: Some("Initial system".to_string()),
        model_info: Some("Initial model".to_string()),
        device_info: Some("Initial device".to_string()),
        xformers: Some("false".to_string()),
        model_name: Some("initial-model".to_string()),
        user: Some("initial-user".to_string()),
        notes: Some("Initial notes".to_string()),
    };
    runs_repo.create(initial_run).await.unwrap();

    // Verify initial data exists
    let initial_runs = runs_repo.find_all().await.unwrap();
    assert_eq!(initial_runs.len(), 1);

    // Now test save_data endpoint
    let test_data = json!([
        {
            "timestamp": "2024-01-01T10:00:00Z",
            "vram_usage": "8GB",
            "info": "New run",
            "system_info": "New system",
            "model_info": "New model",
            "device_info": "New device",
            "xformers": "true",
            "model_name": "new-model",
            "user": "new-user",
            "notes": "New notes"
        }
    ]);

    let app = Router::new()
        .route("/api/save-data", post(save_data))
        .with_state(app_state.clone());

    let boundary = "----WebKitFormBoundary7MA4YWxkTrZu0gW";
    let body = create_multipart_body(&test_data.to_string());

    let request = Request::builder()
        .method("POST")
        .uri("/api/save-data")
        .header("content-type", format!("multipart/form-data; boundary={}", boundary))
        .body(Body::from(body))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Verify old data was replaced
    let final_runs = runs_repo.find_all().await.unwrap();
    assert_eq!(final_runs.len(), 1);
    assert_eq!(final_runs[0].model_name, Some("new-model".to_string()));
    assert_eq!(final_runs[0].user, Some("new-user".to_string()));
}

#[tokio::test]
async fn test_save_data_invalid_json() {
    let app_state = create_test_app_state().await;

    let app = Router::new()
        .route("/api/save-data", post(save_data))
        .with_state(app_state);

    let boundary = "----WebKitFormBoundary7MA4YWxkTrZu0gW";
    let body = create_multipart_body("invalid json content");

    let request = Request::builder()
        .method("POST")
        .uri("/api/save-data")
        .header("content-type", format!("multipart/form-data; boundary={}", boundary))
        .body(Body::from(body))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_save_data_no_file() {
    let app_state = create_test_app_state().await;

    let app = Router::new()
        .route("/api/save-data", post(save_data))
        .with_state(app_state);

    let boundary = "----WebKitFormBoundary7MA4YWxkTrZu0gW";
    let body = format!(
        "--{boundary}\r\n\
        Content-Disposition: form-data; name=\"other_field\"\r\n\
        \r\n\
        some value\r\n\
        --{boundary}--\r\n",
        boundary = boundary
    );

    let request = Request::builder()
        .method("POST")
        .uri("/api/save-data")
        .header("content-type", format!("multipart/form-data; boundary={}", boundary))
        .body(Body::from(body))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_save_data_empty_array() {
    let app_state = create_test_app_state().await;

    let app = Router::new()
        .route("/api/save-data", post(save_data))
        .with_state(app_state);

    let boundary = "----WebKitFormBoundary7MA4YWxkTrZu0gW";
    let body = create_multipart_body("[]");

    let request = Request::builder()
        .method("POST")
        .uri("/api/save-data")
        .header("content-type", format!("multipart/form-data; boundary={}", boundary))
        .body(Body::from(body))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let response_json: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(response_json["success"], true);
    assert_eq!(response_json["rows_processed"], 0);
    assert_eq!(response_json["rows_inserted"], 0);
    assert_eq!(response_json["rows_failed"], 0);
}