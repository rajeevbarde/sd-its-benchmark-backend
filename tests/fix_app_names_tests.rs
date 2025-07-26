use axum::{
    body::to_bytes,
    http::{Method, Request, StatusCode},
    Router,
};
use sqlx::SqlitePool;
use tower::ServiceExt;

use sd_its_benchmark::{
    AppState,
    handlers::admin::{fix_app_names, FixAppNamesRequest},
    models::{app_details::AppDetails, runs::Run},
    repositories::{
        app_details_repository::AppDetailsRepository,
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
        .route("/api/fix-app-names", axum::routing::post(fix_app_names))
        .with_state(app_state)
}

async fn setup_test_app_details_data(pool: &SqlitePool) -> Vec<AppDetails> {
    let runs_repo = RunsRepository::new(pool.clone());
    let app_details_repo = AppDetailsRepository::new(pool.clone());
    
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
            model_name: Some("test-model-1".to_string()),
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
            model_name: Some("test-model-2".to_string()),
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
            model_name: Some("test-model-3".to_string()),
            user: Some("test-user-3".to_string()),
            notes: Some("test-notes-3".to_string()),
        },
        Run {
            id: None,
            timestamp: Some("2024-01-04T00:00:00Z".to_string()),
            vram_usage: Some("8GB".to_string()),
            info: Some("test-info-4".to_string()),
            system_info: Some("test-system-4".to_string()),
            model_info: Some("test-model-4".to_string()),
            device_info: Some("test-device-4".to_string()),
            xformers: Some("test-xformers-4".to_string()),
            model_name: Some("test-model-4".to_string()),
            user: Some("test-user-4".to_string()),
            notes: Some("test-notes-4".to_string()),
        },
        Run {
            id: None,
            timestamp: Some("2024-01-05T00:00:00Z".to_string()),
            vram_usage: Some("8GB".to_string()),
            info: Some("test-info-5".to_string()),
            system_info: Some("test-system-5".to_string()),
            model_info: Some("test-model-5".to_string()),
            device_info: Some("test-device-5".to_string()),
            xformers: Some("test-xformers-5".to_string()),
            model_name: Some("test-model-5".to_string()),
            user: Some("test-user-5".to_string()),
            notes: Some("test-notes-5".to_string()),
        },
    ];

    // Create the Run records and collect their IDs
    let mut run_ids = Vec::new();
    for run in test_runs {
        let created_run = runs_repo.create(run).await.unwrap();
        run_ids.push(created_run.id.unwrap());
    }
    
    let test_app_details = vec![
        AppDetails {
            id: None,
            run_id: Some(run_ids[0]),
            app_name: None, // Will be updated by AUTOMATIC1111 rule
            url: Some("https://github.com/AUTOMATIC1111/stable-diffusion-webui".to_string()),
            hash: Some("abc123".to_string()),
            updated: Some("2024-01-01".to_string()),
        },
        AppDetails {
            id: None,
            run_id: Some(run_ids[1]),
            app_name: None, // Will be updated by vladmandic rule
            url: Some("https://github.com/vladmandic/automatic".to_string()),
            hash: Some("def456".to_string()),
            updated: Some("2024-01-02".to_string()),
        },
        AppDetails {
            id: None,
            run_id: Some(run_ids[2]),
            app_name: None, // Will be updated by stable-diffusion-webui rule
            url: Some("https://github.com/CompVis/stable-diffusion-webui".to_string()),
            hash: Some("ghi789".to_string()),
            updated: Some("2024-01-03".to_string()),
        },
        AppDetails {
            id: None,
            run_id: Some(run_ids[3]),
            app_name: None, // Will be updated by null app_name null url rule
            url: None,
            hash: Some("jkl012".to_string()),
            updated: Some("2024-01-04".to_string()),
        },
        AppDetails {
            id: None,
            run_id: Some(run_ids[4]),
            app_name: Some("existing-app".to_string()), // Should not be updated
            url: Some("https://github.com/some-other/app".to_string()),
            hash: Some("mno345".to_string()),
            updated: Some("2024-01-05".to_string()),
        },
    ];

    let mut created_app_details = Vec::new();
    for app_detail in test_app_details {
        let created_app_detail = app_details_repo.create(app_detail).await.unwrap();
        created_app_details.push(created_app_detail);
    }

    created_app_details
}

// Test successful app names fixing
#[tokio::test]
async fn test_fix_app_names_success() {
    let pool = create_test_pool().await;
    let test_app_details = setup_test_app_details_data(&pool).await;
    assert!(!test_app_details.is_empty(), "Test data setup failed");

    let app_state = AppState { 
        db: pool.clone(),
        settings: sd_its_benchmark::config::settings::Settings::new().unwrap(),
    };
    let app = create_test_app(app_state);

    let request_body = FixAppNamesRequest {
        automatic1111: "AUTOMATIC1111".to_string(),
        vladmandic: "Vladmandic".to_string(),
        stable_diffusion: "StableDiffusion".to_string(),
        null_app_name_null_url: "Unknown".to_string(),
    };

    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/fix-app-names")
        .header("Content-Type", "application/json")
        .body(axum::body::Body::from(serde_json::to_string(&request_body).unwrap()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let response_json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    // Verify response structure
    assert_eq!(response_json["message"], "App names updated successfully");
    assert!(response_json["updated_counts"].is_object());

    let updated_counts = &response_json["updated_counts"];
    assert!(updated_counts["automatic1111"].is_number());
    assert!(updated_counts["vladmandic"].is_number());
    assert!(updated_counts["stable_diffusion"].is_number());
    assert!(updated_counts["null_app_name_null_url"].is_number());

    // Verify the counts match our test data
    let automatic1111_count = updated_counts["automatic1111"].as_u64().unwrap();
    let vladmandic_count = updated_counts["vladmandic"].as_u64().unwrap();
    let stable_diffusion_count = updated_counts["stable_diffusion"].as_u64().unwrap();
    let null_app_name_null_url_count = updated_counts["null_app_name_null_url"].as_u64().unwrap();

    assert_eq!(automatic1111_count, 1); // One record with AUTOMATIC1111 in URL
    assert_eq!(vladmandic_count, 1); // One record with vladmandic in URL
    assert_eq!(stable_diffusion_count, 1); // One record with stable-diffusion-webui in URL
    assert_eq!(null_app_name_null_url_count, 1); // One record with both app_name and url as NULL
}

// Test with no matching data
#[tokio::test]
async fn test_fix_app_names_no_matches() {
    let pool = create_test_pool().await;
    let app_details_repo = AppDetailsRepository::new(pool.clone());
    let runs_repo = RunsRepository::new(pool.clone());

    // Create a single run and app detail that doesn't match any patterns
    let run = Run {
        id: None,
        timestamp: Some("2024-01-01T00:00:00Z".to_string()),
        vram_usage: Some("8GB".to_string()),
        info: Some("test-info".to_string()),
        system_info: Some("test-system".to_string()),
        model_info: Some("test-model".to_string()),
        device_info: Some("test-device".to_string()),
        xformers: Some("test-xformers".to_string()),
        model_name: Some("test-model".to_string()),
        user: Some("test-user".to_string()),
        notes: Some("test-notes".to_string()),
    };

    let created_run = runs_repo.create(run).await.unwrap();
    let run_id = created_run.id.unwrap();

    let app_detail = AppDetails {
        id: None,
        run_id: Some(run_id),
        app_name: Some("existing-app".to_string()),
        url: Some("https://github.com/some-other/app".to_string()),
        hash: Some("abc123".to_string()),
        updated: Some("2024-01-01".to_string()),
    };

    app_details_repo.create(app_detail).await.unwrap();

    let app_state = AppState { 
        db: pool.clone(),
        settings: sd_its_benchmark::config::settings::Settings::new().unwrap(),
    };
    let app = create_test_app(app_state);

    let request_body = FixAppNamesRequest {
        automatic1111: "AUTOMATIC1111".to_string(),
        vladmandic: "Vladmandic".to_string(),
        stable_diffusion: "StableDiffusion".to_string(),
        null_app_name_null_url: "Unknown".to_string(),
    };

    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/fix-app-names")
        .header("Content-Type", "application/json")
        .body(axum::body::Body::from(serde_json::to_string(&request_body).unwrap()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let response_json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    // Verify all counts are zero when no matches exist
    let updated_counts = &response_json["updated_counts"];
    assert_eq!(updated_counts["automatic1111"], 0);
    assert_eq!(updated_counts["vladmandic"], 0);
    assert_eq!(updated_counts["stable_diffusion"], 0);
    assert_eq!(updated_counts["null_app_name_null_url"], 0);
}

// Test response format
#[tokio::test]
async fn test_fix_app_names_response_format() {
    let pool = create_test_pool().await;
    let _test_app_details = setup_test_app_details_data(&pool).await;

    let app_state = AppState { 
        db: pool.clone(),
        settings: sd_its_benchmark::config::settings::Settings::new().unwrap(),
    };
    let app = create_test_app(app_state);

    let request_body = FixAppNamesRequest {
        automatic1111: "AUTOMATIC1111".to_string(),
        vladmandic: "Vladmandic".to_string(),
        stable_diffusion: "StableDiffusion".to_string(),
        null_app_name_null_url: "Unknown".to_string(),
    };

    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/fix-app-names")
        .header("Content-Type", "application/json")
        .body(axum::body::Body::from(serde_json::to_string(&request_body).unwrap()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let response_json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    // Verify response structure
    assert_eq!(response_json["message"], "App names updated successfully");
    assert!(response_json["updated_counts"].is_object());

    let updated_counts = &response_json["updated_counts"];
    assert!(updated_counts["automatic1111"].is_number());
    assert!(updated_counts["vladmandic"].is_number());
    assert!(updated_counts["stable_diffusion"].is_number());
    assert!(updated_counts["null_app_name_null_url"].is_number());

    // Verify counts are reasonable (non-negative)
    let automatic1111_count = updated_counts["automatic1111"].as_u64().unwrap();
    let vladmandic_count = updated_counts["vladmandic"].as_u64().unwrap();
    let stable_diffusion_count = updated_counts["stable_diffusion"].as_u64().unwrap();
    let null_app_name_null_url_count = updated_counts["null_app_name_null_url"].as_u64().unwrap();

    assert!(automatic1111_count >= 0);
    assert!(vladmandic_count >= 0);
    assert!(stable_diffusion_count >= 0);
    assert!(null_app_name_null_url_count >= 0);
}

// Test edge cases with specific data patterns
#[tokio::test]
async fn test_fix_app_names_edge_cases() {
    let pool = create_test_pool().await;
    let runs_repo = RunsRepository::new(pool.clone());
    let app_details_repo = AppDetailsRepository::new(pool.clone());

    // Create test runs
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
            model_name: Some("test-model-1".to_string()),
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
            model_name: Some("test-model-2".to_string()),
            user: Some("test-user-2".to_string()),
            notes: Some("test-notes-2".to_string()),
        },
    ];

    let mut run_ids = Vec::new();
    for run in test_runs {
        let created_run = runs_repo.create(run).await.unwrap();
        run_ids.push(created_run.id.unwrap());
    }

    // Create test app details with edge cases
    let test_app_details = vec![
        AppDetails {
            id: None,
            run_id: Some(run_ids[0]),
            app_name: Some("".to_string()), // Empty string - should be updated by vladmandic rule
            url: Some("https://github.com/vladmandic/automatic".to_string()),
            hash: Some("hash1".to_string()),
            updated: Some("2024-01-01".to_string()),
        },
        AppDetails {
            id: None,
            run_id: Some(run_ids[1]),
            app_name: Some("existing-app".to_string()), // Existing app name - should not be updated
            url: Some("https://github.com/CompVis/stable-diffusion-webui".to_string()),
            hash: Some("hash2".to_string()),
            updated: Some("2024-01-02".to_string()),
        },
    ];

    for app_detail in test_app_details {
        app_details_repo.create(app_detail).await.unwrap();
    }

    let app_state = AppState { 
        db: pool.clone(),
        settings: sd_its_benchmark::config::settings::Settings::new().unwrap(),
    };
    let app = create_test_app(app_state);

    let request_body = FixAppNamesRequest {
        automatic1111: "AUTOMATIC1111".to_string(),
        vladmandic: "Vladmandic".to_string(),
        stable_diffusion: "StableDiffusion".to_string(),
        null_app_name_null_url: "Unknown".to_string(),
    };

    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/fix-app-names")
        .header("Content-Type", "application/json")
        .body(axum::body::Body::from(serde_json::to_string(&request_body).unwrap()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let response_json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    let updated_counts = &response_json["updated_counts"];
    
    // Only the empty string app_name should be updated by vladmandic rule
    assert_eq!(updated_counts["automatic1111"], 0);
    assert_eq!(updated_counts["vladmandic"], 1); // Empty string app_name with vladmandic URL
    assert_eq!(updated_counts["stable_diffusion"], 0); // Existing app_name should not be updated
    assert_eq!(updated_counts["null_app_name_null_url"], 0);
} 