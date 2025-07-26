use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
    routing::{get, post},
    Router,
};
use sqlx::SqlitePool;
use tower::ServiceExt;

use sd_its_benchmark::{
    models::{runs::Run, performance_result::PerformanceResult, app_details::AppDetails},
    repositories::{RunsRepository, PerformanceResultRepository, AppDetailsRepository, traits::{Repository, TransactionRepository}},
    AppState,
    config::settings::Settings,
    handlers,
};

async fn create_test_pool() -> SqlitePool {
    SqlitePool::connect("sqlite::memory:").await.unwrap()
}

fn create_test_app(app_state: AppState) -> Router {
    Router::new()
        .route("/health", get(|| async { "OK" }))
        .route("/api/process-its", post(handlers::admin::process_its))
        .route("/api/process-app-details", post(handlers::admin::process_app_details))
        .with_state(app_state)
}

async fn setup_test_data(pool: &SqlitePool) -> Vec<Run> {
    // Run migrations to create tables
    sqlx::migrate!("./migrations")
        .run(pool)
        .await
        .expect("Failed to run migrations");

    let runs_repo = RunsRepository::new(pool.clone());

    // Create test runs with different ITS patterns
    let test_runs = vec![
        Run {
            id: None,
            timestamp: Some("2024-01-01T00:00:00Z".to_string()),
            vram_usage: Some("10.5/11.2/9.8".to_string()), // Valid ITS values
            info: Some("Test run 1".to_string()),
            system_info: Some("Windows 11".to_string()),
            model_info: Some("Test model".to_string()),
            device_info: Some("RTX 4090".to_string()),
            xformers: Some("enabled".to_string()),
            model_name: Some("test-model-1".to_string()),
            user: Some("test-user".to_string()),
            notes: Some("Test notes".to_string()),
        },
        Run {
            id: None,
            timestamp: Some("2024-01-02T00:00:00Z".to_string()),
            vram_usage: Some("15.0".to_string()), // Single ITS value
            info: Some("Test run 2".to_string()),
            system_info: Some("Windows 11".to_string()),
            model_info: Some("Test model".to_string()),
            device_info: Some("RTX 4090".to_string()),
            xformers: Some("enabled".to_string()),
            model_name: Some("test-model-2".to_string()),
            user: Some("test-user".to_string()),
            notes: Some("Test notes".to_string()),
        },
        Run {
            id: None,
            timestamp: Some("2024-01-03T00:00:00Z".to_string()),
            vram_usage: Some("invalid/nan/12.5".to_string()), // Mixed valid/invalid values
            info: Some("Test run 3".to_string()),
            system_info: Some("Windows 11".to_string()),
            model_info: Some("Test model".to_string()),
            device_info: Some("RTX 4090".to_string()),
            xformers: Some("enabled".to_string()),
            model_name: Some("test-model-3".to_string()),
            user: Some("test-user".to_string()),
            notes: Some("Test notes".to_string()),
        },
        Run {
            id: None,
            timestamp: Some("2024-01-04T00:00:00Z".to_string()),
            vram_usage: Some("".to_string()), // Empty string
            info: Some("Test run 4".to_string()),
            system_info: Some("Windows 11".to_string()),
            model_info: Some("Test model".to_string()),
            device_info: Some("RTX 4090".to_string()),
            xformers: Some("enabled".to_string()),
            model_name: Some("test-model-4".to_string()),
            user: Some("test-user".to_string()),
            notes: Some("Test notes".to_string()),
        },
    ];

    let mut created_runs = Vec::new();
    for run in test_runs {
        let created_run = runs_repo.create(run).await.expect("Failed to create test run");
        created_runs.push(created_run);
    }

    created_runs
}

#[tokio::test]
async fn test_its_parsing_logic() {
    // Test the ITS parsing logic directly
    let test_cases = vec![
        ("10.5/11.2/9.8", vec![10.5, 11.2, 9.8]),
        ("15.0", vec![15.0]),
        ("invalid/nan/12.5", vec![12.5]), // "nan" should be filtered out as NaN
        ("", vec![]),
        ("  10.5  /  11.2  /  9.8  ", vec![10.5, 11.2, 9.8]),
    ];

    for (input, expected) in test_cases {
        let its_values: Vec<f64> = input
            .split('/')
            .filter_map(|value| {
                let trimmed = value.trim();
                if trimmed.is_empty() {
                    None
                } else {
                    // Parse the value and filter out NaN
                    trimmed.parse::<f64>().ok().filter(|&x| !x.is_nan())
                }
            })
            .collect();

        println!("Input: '{}', Parsed: {:?}, Expected: {:?}", input, its_values, expected);
        assert_eq!(its_values, expected);
    }
}

#[tokio::test]
async fn test_process_its_success() {
    let pool = create_test_pool().await;
    let test_runs = setup_test_data(&pool).await;

    // Create app state
    let settings = Settings::default();
    let app_state = AppState {
        db: pool.clone(),
        settings,
    };

    // Create the application
    let app = create_test_app(app_state);

    // Make request to process-its endpoint
    let request = Request::builder()
        .method("POST")
        .uri("/api/process-its")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // Check response status
    assert_eq!(response.status(), StatusCode::OK);

    // Parse response body
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let response_json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    // Check response structure
    assert!(response_json["success"].as_bool().unwrap());
    assert_eq!(response_json["rows_inserted"].as_u64().unwrap(), 4);

    // Verify database state
    let perf_repo = PerformanceResultRepository::new(pool);
    let results = perf_repo.find_all().await.expect("Failed to fetch performance results");
    
    assert_eq!(results.len(), 4);

    // Check specific results
    let run_1_result = results.iter().find(|r| r.run_id == test_runs[0].id).unwrap();
    assert_eq!(run_1_result.its, Some("10.5/11.2/9.8".to_string()));
    // Calculate expected average: (10.5 + 11.2 + 9.8) / 3 = 10.5
    let expected_avg_1 = (10.5 + 11.2 + 9.8) / 3.0;
    assert!((run_1_result.avg_its.unwrap() - expected_avg_1).abs() < 0.01);

    let run_2_result = results.iter().find(|r| r.run_id == test_runs[1].id).unwrap();
    assert_eq!(run_2_result.its, Some("15.0".to_string()));
    assert_eq!(run_2_result.avg_its, Some(15.0));

    let run_3_result = results.iter().find(|r| r.run_id == test_runs[2].id).unwrap();
    assert_eq!(run_3_result.its, Some("invalid/nan/12.5".to_string()));
    // Debug: Let's see what the actual avg_its value is
    println!("Run 3 avg_its: {:?}", run_3_result.avg_its);
    // The parsing should filter out "invalid" and "nan", leaving only 12.5
    // Now that we've fixed the NaN filtering, this should work correctly
    assert_eq!(run_3_result.avg_its, Some(12.5));

    let run_4_result = results.iter().find(|r| r.run_id == test_runs[3].id).unwrap();
    assert_eq!(run_4_result.its, Some("".to_string()));
    assert_eq!(run_4_result.avg_its, None); // No valid values
}

#[tokio::test]
async fn test_process_its_clears_existing_data() {
    let pool = create_test_pool().await;
    let test_runs = setup_test_data(&pool).await;

    // Create some existing performance results with valid run_id
    let perf_repo = PerformanceResultRepository::new(pool.clone());
    let existing_result = PerformanceResult {
        id: None,
        run_id: test_runs[0].id, // Use a valid run_id from the test data
        its: Some("old_data".to_string()),
        avg_its: Some(5.0),
    };
    perf_repo.create(existing_result).await.expect("Failed to create existing result");

    // Verify existing data exists
    let initial_results = perf_repo.find_all().await.expect("Failed to fetch initial results");
    assert_eq!(initial_results.len(), 1);

    // Create app state
    let settings = Settings::default();
    let app_state = AppState {
        db: pool.clone(),
        settings,
    };

    // Create the application
    let app = create_test_app(app_state);

    // Make request to process-its endpoint
    let request = Request::builder()
        .method("POST")
        .uri("/api/process-its")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // Check response status
    assert_eq!(response.status(), StatusCode::OK);

    // Verify old data was cleared and new data was inserted
    let final_results = perf_repo.find_all().await.expect("Failed to fetch final results");
    assert_eq!(final_results.len(), 4); // Should have 4 new results, old one cleared

    // Verify the old result is gone (check by its content, not run_id since run_id will be reused)
    let old_result_exists = final_results.iter().any(|r| r.its == Some("old_data".to_string()));
    assert!(!old_result_exists);
}

#[tokio::test]
async fn test_process_its_with_no_runs() {
    let pool = create_test_pool().await;
    
    // Run migrations but don't create any runs
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    // Create app state
    let settings = Settings::default();
    let app_state = AppState {
        db: pool.clone(),
        settings,
    };

    // Create the application
    let app = create_test_app(app_state);

    // Make request to process-its endpoint
    let request = Request::builder()
        .method("POST")
        .uri("/api/process-its")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // Check response status
    assert_eq!(response.status(), StatusCode::OK);

    // Parse response body
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let response_json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    // Check response structure
    assert!(response_json["success"].as_bool().unwrap());
    assert_eq!(response_json["rows_inserted"].as_u64().unwrap(), 0);

    // Verify no performance results were created
    let perf_repo = PerformanceResultRepository::new(pool);
    let results = perf_repo.find_all().await.expect("Failed to fetch performance results");
    assert_eq!(results.len(), 0);
}

#[tokio::test]
async fn test_process_its_its_calculation_edge_cases() {
    let pool = create_test_pool().await;
    
    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    let runs_repo = RunsRepository::new(pool.clone());

    // Create test runs with edge case ITS patterns
    let edge_case_runs = vec![
        Run {
            id: None,
            timestamp: Some("2024-01-01T00:00:00Z".to_string()),
            vram_usage: Some("  10.5  /  11.2  /  9.8  ".to_string()), // Extra whitespace
            info: Some("Test run 1".to_string()),
            system_info: Some("Windows 11".to_string()),
            model_info: Some("Test model".to_string()),
            device_info: Some("RTX 4090".to_string()),
            xformers: Some("enabled".to_string()),
            model_name: Some("test-model-1".to_string()),
            user: Some("test-user".to_string()),
            notes: Some("Test notes".to_string()),
        },
        Run {
            id: None,
            timestamp: Some("2024-01-02T00:00:00Z".to_string()),
            vram_usage: Some("0.0/0.0/0.0".to_string()), // All zeros
            info: Some("Test run 2".to_string()),
            system_info: Some("Windows 11".to_string()),
            model_info: Some("Test model".to_string()),
            device_info: Some("RTX 4090".to_string()),
            xformers: Some("enabled".to_string()),
            model_name: Some("test-model-2".to_string()),
            user: Some("test-user".to_string()),
            notes: Some("Test notes".to_string()),
        },
        Run {
            id: None,
            timestamp: Some("2024-01-03T00:00:00Z".to_string()),
            vram_usage: Some("999999.999/0.001".to_string()), // Very large and small numbers
            info: Some("Test run 3".to_string()),
            system_info: Some("Windows 11".to_string()),
            model_info: Some("Test model".to_string()),
            device_info: Some("RTX 4090".to_string()),
            xformers: Some("enabled".to_string()),
            model_name: Some("test-model-3".to_string()),
            user: Some("test-user".to_string()),
            notes: Some("Test notes".to_string()),
        },
    ];

    for run in edge_case_runs {
        runs_repo.create(run).await.expect("Failed to create test run");
    }

    // Create app state
    let settings = Settings::default();
    let app_state = AppState {
        db: pool.clone(),
        settings,
    };

    // Create the application
    let app = create_test_app(app_state);

    // Make request to process-its endpoint
    let request = Request::builder()
        .method("POST")
        .uri("/api/process-its")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // Check response status
    assert_eq!(response.status(), StatusCode::OK);

    // Parse response body
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let response_json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    // Check response structure
    assert!(response_json["success"].as_bool().unwrap());
    assert_eq!(response_json["rows_inserted"].as_u64().unwrap(), 3);

    // Verify database state
    let perf_repo = PerformanceResultRepository::new(pool);
    let results = perf_repo.find_all().await.expect("Failed to fetch performance results");
    
    assert_eq!(results.len(), 3);

    // Check that all results have valid avg_its values
    for result in results {
        assert!(result.avg_its.is_some());
        assert!(result.avg_its.unwrap() >= 0.0); // Should be non-negative
    }
}

#[tokio::test]
async fn test_performance_result_repository_clear_methods() {
    let pool = create_test_pool().await;
    
    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    // First create a valid run to reference
    let runs_repo = RunsRepository::new(pool.clone());
    let test_run = Run {
        id: None,
        timestamp: Some("2024-01-01T00:00:00Z".to_string()),
        vram_usage: Some("8GB".to_string()),
        info: Some("Test run".to_string()),
        system_info: Some("Windows 11".to_string()),
        model_info: Some("Test model".to_string()),
        device_info: Some("RTX 4090".to_string()),
        xformers: Some("enabled".to_string()),
        model_name: Some("test-model".to_string()),
        user: Some("test-user".to_string()),
        notes: Some("Test notes".to_string()),
    };
    let created_run = runs_repo.create(test_run).await.expect("Failed to create test run");
    let run_id = created_run.id.unwrap();

    let repo = PerformanceResultRepository::new(pool.clone());

    // Create some test data
    let test_result = PerformanceResult {
        id: None,
        run_id: Some(run_id),
        its: Some("10.5".to_string()),
        avg_its: Some(10.5),
    };

    repo.create(test_result).await.expect("Failed to create test result");

    // Verify data exists
    let count_before = repo.count().await.expect("Failed to count before clear");
    assert_eq!(count_before, 1);

    // Test clear_all method
    repo.clear_all().await.expect("Failed to clear all results");

    // Verify data was cleared
    let count_after = repo.count().await.expect("Failed to count after clear");
    assert_eq!(count_after, 0);

    // Test transaction-based clear
    let mut tx = pool.begin().await.expect("Failed to begin transaction");
    
    // Create data again
    let test_result2 = PerformanceResult {
        id: None,
        run_id: Some(run_id),
        its: Some("15.0".to_string()),
        avg_its: Some(15.0),
    };

    repo.create_tx(test_result2, &mut tx).await.expect("Failed to create test result in transaction");

    // Clear within transaction
    repo.clear_all_tx(&mut tx).await.expect("Failed to clear all results in transaction");

    // Commit transaction
    tx.commit().await.expect("Failed to commit transaction");

    // Verify data was cleared
    let count_final = repo.count().await.expect("Failed to count after transaction clear");
    assert_eq!(count_final, 0);
}

// ===== PROCESS APP DETAILS TESTS =====

#[tokio::test]
async fn test_app_details_parsing_logic() {
    // Test the app details parsing logic directly
    let test_cases = vec![
        (
            "app:test-app updated:2024-01-01 hash:abc123 url:https://example.com",
            ParsedAppDetails {
                app_name: Some("test-app".to_string()),
                updated: Some("2024-01-01".to_string()),
                hash: Some("abc123".to_string()),
                url: Some("https://example.com".to_string()),
            }
        ),
        (
            "app:another-app hash:def456",
            ParsedAppDetails {
                app_name: Some("another-app".to_string()),
                updated: None,
                hash: Some("def456".to_string()),
                url: None,
            }
        ),
        (
            "updated:2024-01-02 url:https://test.com",
            ParsedAppDetails {
                app_name: None,
                updated: Some("2024-01-02".to_string()),
                hash: None,
                url: Some("https://test.com".to_string()),
            }
        ),
        (
            "invalid:data no:colon",
            ParsedAppDetails {
                app_name: None,
                updated: None,
                hash: None,
                url: None,
            }
        ),
        (
            "",
            ParsedAppDetails {
                app_name: None,
                updated: None,
                hash: None,
                url: None,
            }
        ),
    ];

    for (input, expected) in test_cases {
        let result = parse_app_details(input);
        println!("Input: '{}', Parsed: {:?}", input, result);
        assert_eq!(result.app_name, expected.app_name);
        assert_eq!(result.updated, expected.updated);
        assert_eq!(result.hash, expected.hash);
        assert_eq!(result.url, expected.url);
    }
}

async fn setup_test_runs_with_app_details(pool: &SqlitePool) -> Vec<Run> {
    // Run migrations to create tables
    sqlx::migrate!("./migrations")
        .run(pool)
        .await
        .expect("Failed to run migrations");

    let runs_repo = RunsRepository::new(pool.clone());

    // Create test runs with different app details patterns
    let test_runs = vec![
        Run {
            id: None,
            timestamp: Some("2024-01-01T00:00:00Z".to_string()),
            vram_usage: Some("8GB".to_string()),
            info: Some("app:test-app-1 updated:2024-01-01 hash:abc123 url:https://example1.com".to_string()),
            system_info: Some("Windows 11".to_string()),
            model_info: Some("Test model".to_string()),
            device_info: Some("RTX 4090".to_string()),
            xformers: Some("enabled".to_string()),
            model_name: Some("test-model-1".to_string()),
            user: Some("test-user".to_string()),
            notes: Some("Test notes".to_string()),
        },
        Run {
            id: None,
            timestamp: Some("2024-01-02T00:00:00Z".to_string()),
            vram_usage: Some("6GB".to_string()),
            info: Some("app:test-app-2 hash:def456".to_string()),
            system_info: Some("Ubuntu 22.04".to_string()),
            model_info: Some("Test model".to_string()),
            device_info: Some("RTX 3080".to_string()),
            xformers: Some("disabled".to_string()),
            model_name: Some("test-model-2".to_string()),
            user: Some("test-user".to_string()),
            notes: Some("Test notes".to_string()),
        },
        Run {
            id: None,
            timestamp: Some("2024-01-03T00:00:00Z".to_string()),
            vram_usage: Some("4GB".to_string()),
            info: Some("updated:2024-01-03 url:https://example3.com".to_string()),
            system_info: Some("macOS".to_string()),
            model_info: Some("Test model".to_string()),
            device_info: Some("M1 Pro".to_string()),
            xformers: Some("enabled".to_string()),
            model_name: Some("test-model-3".to_string()),
            user: Some("test-user".to_string()),
            notes: Some("Test notes".to_string()),
        },
        Run {
            id: None,
            timestamp: Some("2024-01-04T00:00:00Z".to_string()),
            vram_usage: Some("12GB".to_string()),
            info: Some("invalid:data no:colon".to_string()),
            system_info: Some("Windows 11".to_string()),
            model_info: Some("Test model".to_string()),
            device_info: Some("RTX 4090".to_string()),
            xformers: Some("enabled".to_string()),
            model_name: Some("test-model-4".to_string()),
            user: Some("test-user".to_string()),
            notes: Some("Test notes".to_string()),
        },
    ];

    let mut created_runs = Vec::new();
    for run in test_runs {
        let created_run = runs_repo.create(run).await.expect("Failed to create test run");
        created_runs.push(created_run);
    }

    created_runs
}

#[tokio::test]
async fn test_process_app_details_success() {
    let pool = create_test_pool().await;
    let test_runs = setup_test_runs_with_app_details(&pool).await;

    // Create app state
    let settings = Settings::default();
    let app_state = AppState {
        db: pool.clone(),
        settings,
    };

    // Create the application
    let app = create_test_app(app_state);

    // Make request to process-app-details endpoint
    let request = Request::builder()
        .method("POST")
        .uri("/api/process-app-details")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // Check response status
    assert_eq!(response.status(), StatusCode::OK);

    // Parse response body
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let response_json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    // Check response structure
    assert!(response_json["success"].as_bool().unwrap());
    assert_eq!(response_json["rows_inserted"].as_u64().unwrap(), 4);

    // Verify database state
    let app_details_repo = AppDetailsRepository::new(pool);
    let results = app_details_repo.find_all().await.expect("Failed to fetch app details");
    
    assert_eq!(results.len(), 4);

    // Check specific results
    let run_1_details = results.iter().find(|r| r.run_id == test_runs[0].id).unwrap();
    assert_eq!(run_1_details.app_name, Some("test-app-1".to_string()));
    assert_eq!(run_1_details.updated, Some("2024-01-01".to_string()));
    assert_eq!(run_1_details.hash, Some("abc123".to_string()));
    assert_eq!(run_1_details.url, Some("https://example1.com".to_string()));

    let run_2_details = results.iter().find(|r| r.run_id == test_runs[1].id).unwrap();
    assert_eq!(run_2_details.app_name, Some("test-app-2".to_string()));
    assert_eq!(run_2_details.updated, None);
    assert_eq!(run_2_details.hash, Some("def456".to_string()));
    assert_eq!(run_2_details.url, None);

    let run_3_details = results.iter().find(|r| r.run_id == test_runs[2].id).unwrap();
    assert_eq!(run_3_details.app_name, None);
    assert_eq!(run_3_details.updated, Some("2024-01-03".to_string()));
    assert_eq!(run_3_details.hash, None);
    assert_eq!(run_3_details.url, Some("https://example3.com".to_string()));

    let run_4_details = results.iter().find(|r| r.run_id == test_runs[3].id).unwrap();
    assert_eq!(run_4_details.app_name, None);
    assert_eq!(run_4_details.updated, None);
    assert_eq!(run_4_details.hash, None);
    assert_eq!(run_4_details.url, None);
}

#[tokio::test]
async fn test_process_app_details_clears_existing_data() {
    let pool = create_test_pool().await;
    let test_runs = setup_test_runs_with_app_details(&pool).await;

    // Create some existing app details
    let app_details_repo = AppDetailsRepository::new(pool.clone());
    let existing_details = AppDetails {
        id: None,
        run_id: test_runs[0].id,
        app_name: Some("old-app".to_string()),
        updated: Some("2023-01-01".to_string()),
        hash: Some("old-hash".to_string()),
        url: Some("https://old-url.com".to_string()),
    };
    app_details_repo.create(existing_details).await.expect("Failed to create existing app details");

    // Verify existing data exists
    let initial_results = app_details_repo.find_all().await.expect("Failed to fetch initial results");
    assert_eq!(initial_results.len(), 1);

    // Create app state
    let settings = Settings::default();
    let app_state = AppState {
        db: pool.clone(),
        settings,
    };

    // Create the application
    let app = create_test_app(app_state);

    // Make request to process-app-details endpoint
    let request = Request::builder()
        .method("POST")
        .uri("/api/process-app-details")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // Check response status
    assert_eq!(response.status(), StatusCode::OK);

    // Verify old data was cleared and new data was inserted
    let final_results = app_details_repo.find_all().await.expect("Failed to fetch final results");
    assert_eq!(final_results.len(), 4); // Should have 4 new results, old one cleared

    // Verify the old result is gone (check by app_name content)
    let old_result_exists = final_results.iter().any(|r| r.app_name == Some("old-app".to_string()));
    assert!(!old_result_exists);
}

#[tokio::test]
async fn test_process_app_details_with_no_runs() {
    let pool = create_test_pool().await;
    
    // Run migrations but don't create any runs
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    // Create app state
    let settings = Settings::default();
    let app_state = AppState {
        db: pool.clone(),
        settings,
    };

    // Create the application
    let app = create_test_app(app_state);

    // Make request to process-app-details endpoint
    let request = Request::builder()
        .method("POST")
        .uri("/api/process-app-details")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // Check response status
    assert_eq!(response.status(), StatusCode::OK);

    // Parse response body
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let response_json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    // Check response structure
    assert!(response_json["success"].as_bool().unwrap());
    assert_eq!(response_json["rows_inserted"].as_u64().unwrap(), 0);

    // Verify no app details were created
    let app_details_repo = AppDetailsRepository::new(pool);
    let results = app_details_repo.find_all().await.expect("Failed to fetch app details");
    assert_eq!(results.len(), 0);
}

#[tokio::test]
async fn test_app_details_repository_clear_methods() {
    let pool = create_test_pool().await;
    
    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    // First create a valid run to reference
    let runs_repo = RunsRepository::new(pool.clone());
    let test_run = Run {
        id: None,
        timestamp: Some("2024-01-01T00:00:00Z".to_string()),
        vram_usage: Some("8GB".to_string()),
        info: Some("app:test-app".to_string()),
        system_info: Some("Windows 11".to_string()),
        model_info: Some("Test model".to_string()),
        device_info: Some("RTX 4090".to_string()),
        xformers: Some("enabled".to_string()),
        model_name: Some("test-model".to_string()),
        user: Some("test-user".to_string()),
        notes: Some("Test notes".to_string()),
    };
    let created_run = runs_repo.create(test_run).await.expect("Failed to create test run");
    let run_id = created_run.id.unwrap();

    let repo = AppDetailsRepository::new(pool.clone());

    // Create some test data
    let test_details = AppDetails {
        id: None,
        run_id: Some(run_id),
        app_name: Some("test-app".to_string()),
        updated: Some("2024-01-01".to_string()),
        hash: Some("abc123".to_string()),
        url: Some("https://example.com".to_string()),
    };

    repo.create(test_details).await.expect("Failed to create test app details");

    // Verify data exists
    let count_before = repo.count().await.expect("Failed to count before clear");
    assert_eq!(count_before, 1);

    // Test clear_all method
    repo.clear_all().await.expect("Failed to clear all app details");

    // Verify data was cleared
    let count_after = repo.count().await.expect("Failed to count after clear");
    assert_eq!(count_after, 0);

    // Test transaction-based clear
    let mut tx = pool.begin().await.expect("Failed to begin transaction");
    
    // Create data again
    let test_details2 = AppDetails {
        id: None,
        run_id: Some(run_id),
        app_name: Some("test-app-2".to_string()),
        updated: Some("2024-01-02".to_string()),
        hash: Some("def456".to_string()),
        url: Some("https://example2.com".to_string()),
    };

    repo.create_tx(test_details2, &mut tx).await.expect("Failed to create test app details in transaction");

    // Clear within transaction
    repo.clear_all_tx(&mut tx).await.expect("Failed to clear all app details in transaction");

    // Commit transaction
    tx.commit().await.expect("Failed to commit transaction");

    // Verify data was cleared
    let count_final = repo.count().await.expect("Failed to count after transaction clear");
    assert_eq!(count_final, 0);
}

// Helper struct and function for testing (copied from handler)
#[derive(Debug, PartialEq)]
struct ParsedAppDetails {
    app_name: Option<String>,
    updated: Option<String>,
    hash: Option<String>,
    url: Option<String>,
}

fn parse_app_details(info_string: &str) -> ParsedAppDetails {
    let parts: Vec<&str> = info_string.split(' ').collect();
    let mut app_details = ParsedAppDetails {
        app_name: None,
        updated: None,
        hash: None,
        url: None,
    };

    for part in parts {
        let colon_index = match part.find(':') {
            Some(index) => index,
            None => continue,
        };

        let key = &part[..colon_index];
        let value = &part[colon_index + 1..];

        match key {
            "app" => app_details.app_name = Some(value.to_string()),
            "updated" => app_details.updated = Some(value.to_string()),
            "hash" => app_details.hash = Some(value.to_string()),
            "url" => app_details.url = Some(value.to_string()),
            _ => continue,
        }
    }

    app_details
} 