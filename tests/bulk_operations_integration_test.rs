use sqlx::SqlitePool;
use std::sync::Once;

use sd_its_benchmark::{
    models::{
        runs::Run,
        performance_result::PerformanceResult,
        app_details::AppDetails,
        system_info::SystemInfo,
        libraries::Libraries,
        gpu::Gpu,
        run_more_details::RunMoreDetails,
        model_map::ModelMap,
    },
    repositories::{
        runs_repository::RunsRepository,
        performance_result_repository::PerformanceResultRepository,
        app_details_repository::AppDetailsRepository,
        system_info_repository::SystemInfoRepository,
        libraries_repository::LibrariesRepository,
        gpu_repository::GpuRepository,
        run_more_details_repository::RunMoreDetailsRepository,
        model_map_repository::ModelMapRepository,
        traits::{Repository, BulkRepository},
    },
};

static INIT: Once = Once::new();

async fn setup_test_db() -> SqlitePool {
    INIT.call_once(|| {
        // Initialize test environment if needed
    });

    // Create in-memory database for testing
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    
    // Create all tables
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS runs (
            id INTEGER PRIMARY KEY,
            timestamp TEXT,
            vram_usage TEXT,
            info TEXT,
            system_info TEXT,
            model_info TEXT,
            device_info TEXT,
            xformers TEXT,
            model_name TEXT,
            user TEXT,
            notes TEXT
        )
        "#
    )
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS performanceResult (
            id INTEGER PRIMARY KEY,
            run_id INTEGER,
            its TEXT,
            avg_its REAL,
            FOREIGN KEY (run_id) REFERENCES runs(id)
        )
        "#
    )
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS AppDetails (
            id INTEGER PRIMARY KEY,
            run_id INTEGER,
            app_name TEXT,
            updated TEXT,
            hash TEXT,
            url TEXT,
            FOREIGN KEY (run_id) REFERENCES runs(id)
        )
        "#
    )
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS SystemInfo (
            id INTEGER PRIMARY KEY,
            run_id INTEGER,
            arch TEXT,
            cpu TEXT,
            system TEXT,
            release TEXT,
            python TEXT,
            FOREIGN KEY (run_id) REFERENCES runs(id)
        )
        "#
    )
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS Libraries (
            id INTEGER PRIMARY KEY,
            run_id INTEGER,
            torch TEXT,
            xformers TEXT,
            xformers1 TEXT,
            diffusers TEXT,
            transformers TEXT,
            FOREIGN KEY (run_id) REFERENCES runs(id)
        )
        "#
    )
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS GPU (
            id INTEGER PRIMARY KEY,
            run_id INTEGER,
            device TEXT,
            driver TEXT,
            gpu_chip TEXT,
            brand TEXT,
            isLaptop BOOLEAN,
            FOREIGN KEY (run_id) REFERENCES runs(id)
        )
        "#
    )
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS RunMoreDetails (
            id INTEGER PRIMARY KEY,
            run_id INTEGER,
            timestamp TEXT,
            model_name TEXT,
            user TEXT,
            notes TEXT,
            ModelMapId INTEGER,
            FOREIGN KEY (run_id) REFERENCES runs(id)
        )
        "#
    )
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS ModelMap (
            id INTEGER PRIMARY KEY,
            model_name TEXT,
            base_model TEXT
        )
        "#
    )
    .execute(&pool)
    .await
    .unwrap();

    pool
}

fn create_test_run(id: Option<i64>) -> Run {
    Run {
        id,
        timestamp: Some("2024-01-01T00:00:00Z".to_string()),
        vram_usage: Some("8GB/16GB".to_string()),
        info: Some("Test info".to_string()),
        system_info: Some("Test system".to_string()),
        model_info: Some("Test model".to_string()),
        device_info: Some("Test device".to_string()),
        xformers: Some("true".to_string()),
        model_name: Some("test-model".to_string()),
        user: Some("test-user".to_string()),
        notes: Some("Test notes".to_string()),
    }
}

fn create_test_performance_result(run_id: i64) -> PerformanceResult {
    PerformanceResult {
        id: None,
        run_id: Some(run_id),
        its: Some("10.5/11.2/9.8".to_string()),
        avg_its: Some(10.5),
    }
}

fn create_test_app_details(run_id: i64) -> AppDetails {
    AppDetails {
        id: None,
        run_id: Some(run_id),
        app_name: Some("TestApp".to_string()),
        updated: Some("2024-01-01".to_string()),
        hash: Some("abc123".to_string()),
        url: Some("https://test.com".to_string()),
    }
}

fn create_test_system_info(run_id: i64) -> SystemInfo {
    SystemInfo {
        id: None,
        run_id: Some(run_id),
        arch: Some("x86_64".to_string()),
        cpu: Some("Intel i7".to_string()),
        system: Some("Linux".to_string()),
        release: Some("Ubuntu 22.04".to_string()),
        python: Some("3.9.0".to_string()),
    }
}

fn create_test_libraries(run_id: i64) -> Libraries {
    Libraries {
        id: None,
        run_id: Some(run_id),
        torch: Some("2.0.0".to_string()),
        xformers: Some("0.0.20".to_string()),
        xformers1: Some("0.0.20".to_string()),
        diffusers: Some("0.18.0".to_string()),
        transformers: Some("4.30.0".to_string()),
    }
}

fn create_test_gpu(run_id: i64) -> Gpu {
    Gpu {
        id: None,
        run_id: Some(run_id),
        device: Some("NVIDIA GeForce RTX 4090".to_string()),
        driver: Some("535.86.10".to_string()),
        gpu_chip: Some("AD102".to_string()),
        brand: Some("nvidia".to_string()),
        is_laptop: Some(false),
    }
}

fn create_test_run_more_details(run_id: i64) -> RunMoreDetails {
    RunMoreDetails {
        id: None,
        run_id: Some(run_id),
        timestamp: Some("2024-01-01T00:00:00Z".to_string()),
        model_name: Some("test-model".to_string()),
        user: Some("test-user".to_string()),
        notes: Some("Test notes".to_string()),
        model_map_id: None,
    }
}

fn create_test_model_map() -> ModelMap {
    ModelMap {
        id: None,
        model_name: Some("test-model".to_string()),
        base_model: Some("stable-diffusion-v1-5".to_string()),
    }
}

#[tokio::test]
async fn test_bulk_operations_across_repositories() {
    let pool = setup_test_db().await;
    
    // Create repositories
    let runs_repo = RunsRepository::new(pool.clone());
    let performance_repo = PerformanceResultRepository::new(pool.clone());
    let app_details_repo = AppDetailsRepository::new(pool.clone());
    let system_info_repo = SystemInfoRepository::new(pool.clone());
    let libraries_repo = LibrariesRepository::new(pool.clone());
    let gpu_repo = GpuRepository::new(pool.clone());
    let run_details_repo = RunMoreDetailsRepository::new(pool.clone());
    let model_map_repo = ModelMapRepository::new(pool.clone());

    // Test bulk create runs
    let test_runs = vec![
        create_test_run(None),
        create_test_run(None),
        create_test_run(None),
    ];

    let created_runs = runs_repo.bulk_create(test_runs).await.unwrap();
    assert_eq!(created_runs.len(), 3);

    // Verify runs were created
    let count = runs_repo.count().await.unwrap();
    assert_eq!(count, 3);

    // Test bulk create performance results
    let performance_results: Vec<PerformanceResult> = created_runs
        .iter()
        .map(|run| create_test_performance_result(run.id.unwrap()))
        .collect();

    let created_performance = performance_repo.bulk_create(performance_results).await.unwrap();
    assert_eq!(created_performance.len(), 3);

    // Test bulk create app details
    let app_details: Vec<AppDetails> = created_runs
        .iter()
        .map(|run| create_test_app_details(run.id.unwrap()))
        .collect();

    let created_app_details = app_details_repo.bulk_create(app_details).await.unwrap();
    assert_eq!(created_app_details.len(), 3);

    // Test bulk create system info
    let system_infos: Vec<SystemInfo> = created_runs
        .iter()
        .map(|run| create_test_system_info(run.id.unwrap()))
        .collect();

    let created_system_infos = system_info_repo.bulk_create(system_infos).await.unwrap();
    assert_eq!(created_system_infos.len(), 3);

    // Test bulk create libraries
    let libraries: Vec<Libraries> = created_runs
        .iter()
        .map(|run| create_test_libraries(run.id.unwrap()))
        .collect();

    let created_libraries = libraries_repo.bulk_create(libraries).await.unwrap();
    assert_eq!(created_libraries.len(), 3);

    // Test bulk create GPUs
    let gpus: Vec<Gpu> = created_runs
        .iter()
        .map(|run| create_test_gpu(run.id.unwrap()))
        .collect();

    let created_gpus = gpu_repo.bulk_create(gpus).await.unwrap();
    assert_eq!(created_gpus.len(), 3);

    // Test bulk create run more details
    let run_details: Vec<RunMoreDetails> = created_runs
        .iter()
        .map(|run| create_test_run_more_details(run.id.unwrap()))
        .collect();

    let created_run_details = run_details_repo.bulk_create(run_details).await.unwrap();
    assert_eq!(created_run_details.len(), 3);

    // Test bulk create model maps
    let model_maps = vec![
        create_test_model_map(),
        create_test_model_map(),
    ];

    let created_model_maps = model_map_repo.bulk_create(model_maps).await.unwrap();
    assert_eq!(created_model_maps.len(), 2);

    // Verify all data was created
    assert_eq!(runs_repo.count().await.unwrap(), 3);
    assert_eq!(performance_repo.count().await.unwrap(), 3);
    assert_eq!(app_details_repo.count().await.unwrap(), 3);
    assert_eq!(system_info_repo.count().await.unwrap(), 3);
    assert_eq!(libraries_repo.count().await.unwrap(), 3);
    assert_eq!(gpu_repo.count().await.unwrap(), 3);
    assert_eq!(run_details_repo.count().await.unwrap(), 3);
    assert_eq!(model_map_repo.count().await.unwrap(), 2);
}

#[tokio::test]
async fn test_bulk_update_operations() {
    let pool = setup_test_db().await;
    let runs_repo = RunsRepository::new(pool.clone());
    let performance_repo = PerformanceResultRepository::new(pool.clone());

    // Create initial data
    let test_runs = vec![create_test_run(None), create_test_run(None)];
    let created_runs = runs_repo.bulk_create(test_runs).await.unwrap();

    // Create performance results
    let performance_results: Vec<PerformanceResult> = created_runs
        .iter()
        .map(|run| create_test_performance_result(run.id.unwrap()))
        .collect();
    let created_performance = performance_repo.bulk_create(performance_results).await.unwrap();

    // Update the performance results
    let mut updated_performance = created_performance.clone();
    updated_performance[0].avg_its = Some(15.5);
    updated_performance[1].avg_its = Some(20.0);

    let updated_results = performance_repo.bulk_update(updated_performance).await.unwrap();
    assert_eq!(updated_results.len(), 2);
    assert_eq!(updated_results[0].avg_its, Some(15.5));
    assert_eq!(updated_results[1].avg_its, Some(20.0));
}

#[tokio::test]
async fn test_bulk_delete_operations() {
    let pool = setup_test_db().await;
    let runs_repo = RunsRepository::new(pool.clone());
    let performance_repo = PerformanceResultRepository::new(pool.clone());

    // Create data
    let test_runs = vec![create_test_run(None), create_test_run(None)];
    let created_runs = runs_repo.bulk_create(test_runs).await.unwrap();

    let performance_results: Vec<PerformanceResult> = created_runs
        .iter()
        .map(|run| create_test_performance_result(run.id.unwrap()))
        .collect();
    performance_repo.bulk_create(performance_results).await.unwrap();

    // Verify data exists
    assert_eq!(runs_repo.count().await.unwrap(), 2);
    assert_eq!(performance_repo.count().await.unwrap(), 2);

    // Delete all performance results
    let deleted_count = performance_repo.delete_all().await.unwrap();
    assert_eq!(deleted_count, 2);

    // Verify deletion
    assert_eq!(performance_repo.count().await.unwrap(), 0);
    assert_eq!(runs_repo.count().await.unwrap(), 2); // Runs should still exist
}

#[tokio::test]
async fn test_transaction_rollback_on_error() {
    let pool = setup_test_db().await;
    let runs_repo = RunsRepository::new(pool.clone());
    let performance_repo = PerformanceResultRepository::new(pool.clone());

    // Create a run
    let test_run = create_test_run(None);
    let created_run = runs_repo.create(test_run).await.unwrap();

    // Try to create performance results with invalid run_id (should cause rollback)
    let invalid_performance_results = vec![
        PerformanceResult {
            id: None,
            run_id: Some(99999), // Invalid run_id
            its: Some("10.5".to_string()),
            avg_its: Some(10.5),
        },
        PerformanceResult {
            id: None,
            run_id: Some(created_run.id.unwrap()),
            its: Some("11.2".to_string()),
            avg_its: Some(11.2),
        },
    ];

    let result = performance_repo.bulk_create(invalid_performance_results).await;
    assert!(result.is_err());

    // Verify no performance results were created (rollback worked)
    assert_eq!(performance_repo.count().await.unwrap(), 0);
}

#[tokio::test]
async fn test_large_bulk_operations() {
    let pool = setup_test_db().await;
    let runs_repo = RunsRepository::new(pool.clone());

    // Create a large number of test runs
    let test_runs: Vec<Run> = (0..100)
        .map(|i| Run {
            id: None,
            timestamp: Some(format!("2024-01-01T{:02}:00:00Z", i % 24)),
            vram_usage: Some(format!("{}GB/16GB", (i % 8) + 1)),
            info: Some(format!("Test info {}", i)),
            system_info: Some("Test system".to_string()),
            model_info: Some("Test model".to_string()),
            device_info: Some("Test device".to_string()),
            xformers: Some("true".to_string()),
            model_name: Some(format!("test-model-{}", i)),
            user: Some("test-user".to_string()),
            notes: Some(format!("Test notes {}", i)),
        })
        .collect();

    // Test bulk create with large dataset
    let start = std::time::Instant::now();
    let result = runs_repo.bulk_create(test_runs).await;
    let duration = start.elapsed();

    assert!(result.is_ok());
    let created_runs = result.unwrap();
    assert_eq!(created_runs.len(), 100);

    println!("Bulk create of 100 runs took: {:?}", duration);

    // Verify all runs were created
    let count = runs_repo.count().await.unwrap();
    assert_eq!(count, 100);

    // Test bulk update
    let mut updated_runs = created_runs.clone();
    for run in &mut updated_runs {
        run.notes = Some("Updated notes".to_string());
    }

    let start = std::time::Instant::now();
    let update_result = runs_repo.bulk_update(updated_runs).await;
    let update_duration = start.elapsed();

    assert!(update_result.is_ok());
    println!("Bulk update of 100 runs took: {:?}", update_duration);

    // Test bulk delete
    let start = std::time::Instant::now();
    let delete_result = runs_repo.delete_all().await;
    let delete_duration = start.elapsed();

    assert!(delete_result.is_ok());
    assert_eq!(delete_result.unwrap(), 100);
    println!("Bulk delete of 100 runs took: {:?}", delete_duration);
} 