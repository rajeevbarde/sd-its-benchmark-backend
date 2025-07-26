use axum::{
    body::to_bytes,
    http::{Method, Request, StatusCode},
    Router,
};
use sqlx::SqlitePool;
use tower::ServiceExt;

use sd_its_benchmark::{
    AppState,
    handlers::admin::app_details_analysis,
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
        .route("/api/app-details-analysis", axum::routing::get(app_details_analysis))
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
            app_name: Some("test-app-1".to_string()),
            url: Some("https://example1.com".to_string()),
            hash: Some("abc123".to_string()),
            updated: Some("2024-01-01".to_string()),
        },
        AppDetails {
            id: None,
            run_id: Some(run_ids[1]),
            app_name: None, // NULL app_name
            url: Some("https://example2.com".to_string()),
            hash: Some("def456".to_string()),
            updated: Some("2024-01-02".to_string()),
        },
        AppDetails {
            id: None,
            run_id: Some(run_ids[2]),
            app_name: None, // NULL app_name
            url: None, // NULL url
            hash: Some("ghi789".to_string()),
            updated: Some("2024-01-03".to_string()),
        },
        AppDetails {
            id: None,
            run_id: Some(run_ids[3]),
            app_name: Some("test-app-4".to_string()),
            url: Some("https://example4.com".to_string()),
            hash: Some("jkl012".to_string()),
            updated: Some("2024-01-04".to_string()),
        },
        AppDetails {
            id: None,
            run_id: Some(run_ids[4]),
            app_name: None, // NULL app_name
            url: Some("https://example5.com".to_string()),
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

// Test successful app details analysis
#[tokio::test]
async fn test_app_details_analysis_success() {
    let pool = create_test_pool().await;
    let test_app_details = setup_test_app_details_data(&pool).await;
    assert!(!test_app_details.is_empty(), "Test data setup failed");

    let app_state = AppState { 
        db: pool.clone(),
        settings: sd_its_benchmark::config::settings::Settings::new().unwrap(),
    };
    let app = create_test_app(app_state);

    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/app-details-analysis")
        .body(axum::body::Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let response_json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    // Verify response structure
    assert!(response_json["total_rows"].is_number());
    assert!(response_json["null_app_name_null_url"].is_number());
    assert!(response_json["null_app_name_non_null_url"].is_number());

    // Verify the counts match our test data
    let total_rows = response_json["total_rows"].as_u64().unwrap();
    let null_app_name_null_url = response_json["null_app_name_null_url"].as_u64().unwrap();
    let null_app_name_non_null_url = response_json["null_app_name_non_null_url"].as_u64().unwrap();

    assert_eq!(total_rows, test_app_details.len() as u64);
    assert_eq!(null_app_name_null_url, 1); // Only one record has both app_name and url as NULL
    assert_eq!(null_app_name_non_null_url, 2); // Two records have app_name as NULL but url as NOT NULL
}

// Test with no app details data
#[tokio::test]
async fn test_app_details_analysis_with_no_data() {
    let pool = create_test_pool().await;

    let app_state = AppState { 
        db: pool.clone(),
        settings: sd_its_benchmark::config::settings::Settings::new().unwrap(),
    };
    let app = create_test_app(app_state);

    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/app-details-analysis")
        .body(axum::body::Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let response_json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    // Verify all counts are zero when no data exists
    assert_eq!(response_json["total_rows"], 0);
    assert_eq!(response_json["null_app_name_null_url"], 0);
    assert_eq!(response_json["null_app_name_non_null_url"], 0);
}

// Test response format
#[tokio::test]
async fn test_app_details_analysis_response_format() {
    let pool = create_test_pool().await;
    let _test_app_details = setup_test_app_details_data(&pool).await;

    let app_state = AppState { 
        db: pool.clone(),
        settings: sd_its_benchmark::config::settings::Settings::new().unwrap(),
    };
    let app = create_test_app(app_state);

    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/app-details-analysis")
        .body(axum::body::Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let response_json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    // Verify response structure
    assert!(response_json["total_rows"].is_number());
    assert!(response_json["null_app_name_null_url"].is_number());
    assert!(response_json["null_app_name_non_null_url"].is_number());

    // Verify counts are reasonable
    let total_rows = response_json["total_rows"].as_u64().unwrap();
    let null_app_name_null_url = response_json["null_app_name_null_url"].as_u64().unwrap();
    let null_app_name_non_null_url = response_json["null_app_name_non_null_url"].as_u64().unwrap();

    assert!(total_rows > 0);
    assert!(null_app_name_null_url <= total_rows);
    assert!(null_app_name_non_null_url <= total_rows);
    assert!(null_app_name_null_url + null_app_name_non_null_url <= total_rows);
}

// Test edge cases with specific data patterns
#[tokio::test]
async fn test_app_details_analysis_edge_cases() {
    let pool = create_test_pool().await;
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
    ];

    // Create the Run records and collect their IDs
    let mut run_ids = Vec::new();
    for run in test_runs {
        let created_run = runs_repo.create(run).await.unwrap();
        run_ids.push(created_run.id.unwrap());
    }

    // Create test data with specific patterns
    let test_app_details = vec![
        AppDetails {
            id: None,
            run_id: Some(run_ids[0]),
            app_name: Some("app1".to_string()),
            url: Some("https://app1.com".to_string()),
            hash: Some("hash1".to_string()),
            updated: Some("2024-01-01".to_string()),
        },
        AppDetails {
            id: None,
            run_id: Some(run_ids[1]),
            app_name: None,
            url: None,
            hash: Some("hash2".to_string()),
            updated: Some("2024-01-02".to_string()),
        },
        AppDetails {
            id: None,
            run_id: Some(run_ids[2]),
            app_name: None,
            url: Some("https://app3.com".to_string()),
            hash: Some("hash3".to_string()),
            updated: Some("2024-01-03".to_string()),
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

    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/app-details-analysis")
        .body(axum::body::Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let response_json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    // Verify specific counts for this test data
    assert_eq!(response_json["total_rows"], 3);
    assert_eq!(response_json["null_app_name_null_url"], 1); // Only the second record
    assert_eq!(response_json["null_app_name_non_null_url"], 1); // Only the third record
} 