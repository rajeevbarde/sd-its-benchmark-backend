use sqlx::{SqlitePool, Row};
use tracing::{info, error};

use sd_its_benchmark::{
    config::database::create_pool,
    models::{
        runs::Run,
        performance_result::PerformanceResult,
        app_details::AppDetails,
        system_info::SystemInfo,
        libraries::Libraries,
        gpu::Gpu,
        run_more_details::RunMoreDetails,
    },
    repositories::{
        runs_repository::RunsRepository,
        performance_result_repository::PerformanceResultRepository,
        app_details_repository::AppDetailsRepository,
        system_info_repository::SystemInfoRepository,
        libraries_repository::LibrariesRepository,
        gpu_repository::GpuRepository,
        run_more_details_repository::RunMoreDetailsRepository,
    },
    services::data_processing::{
        SaveDataService,
        ProcessItsService,
        ProcessAppDetailsService,
        ProcessSystemInfoService,
        ProcessLibrariesService,
        ProcessGpuService,
        ProcessRunDetailsService,
    },
    handlers::validation::RunData,
};

/// Setup test database with all required tables
async fn setup_test_database() -> Result<SqlitePool, Box<dyn std::error::Error>> {
    let pool = create_pool().await?;
    
    // Create all required tables
    sqlx::query(include_str!("../migrations/001_create_runs_table.sql"))
        .execute(&pool)
        .await?;
    
    sqlx::query(include_str!("../migrations/002_create_performance_result_table.sql"))
        .execute(&pool)
        .await?;
    
    sqlx::query(include_str!("../migrations/003_create_app_details_table.sql"))
        .execute(&pool)
        .await?;
    
    sqlx::query(include_str!("../migrations/004_create_system_info_table.sql"))
        .execute(&pool)
        .await?;
    
    sqlx::query(include_str!("../migrations/005_create_libraries_table.sql"))
        .execute(&pool)
        .await?;
    
    sqlx::query(include_str!("../migrations/006_create_gpu_table.sql"))
        .execute(&pool)
        .await?;
    
    sqlx::query(include_str!("../migrations/007_create_run_more_details_table.sql"))
        .execute(&pool)
        .await?;
    
    sqlx::query(include_str!("../migrations/008_create_model_map_table.sql"))
        .execute(&pool)
        .await?;
    
    sqlx::query(include_str!("../migrations/009_create_gpu_map_table.sql"))
        .execute(&pool)
        .await?;
    
    sqlx::query(include_str!("../migrations/010_create_gpu_base_table.sql"))
        .execute(&pool)
        .await?;
    
    sqlx::query(include_str!("../migrations/011_create_indexes.sql"))
        .execute(&pool)
        .await?;

    Ok(pool)
}

/// Create test data that mimics real application data
fn create_test_run_data() -> Vec<RunData> {
    vec![
        RunData {
            timestamp: "2024-01-01T10:00:00Z".to_string(),
            vram_usage: "1.5/2.0/1.8".to_string(),
            info: "app_name:test_app updated:2024-01-01 hash:abc123 url:https://example.com".to_string(),
            system_info: "arch:x86_64 cpu:Intel i7 system:Linux release:Ubuntu 22.04 python:3.9.0".to_string(),
            model_info: "torch:2.0.0 xformers:0.0.22 diffusers:0.21.0 transformers:4.30.0".to_string(),
            device_info: "device:NVIDIA GeForce RTX 3080 driver:470.82.01 gpu_chip:GA102".to_string(),
            xformers: "0.0.22".to_string(),
            model_name: "stable-diffusion-v1-5".to_string(),
            user: "test_user".to_string(),
            notes: "Test run 1".to_string(),
        },
        RunData {
            timestamp: "2024-01-01T11:00:00Z".to_string(),
            vram_usage: "2.1/2.3/2.0".to_string(),
            info: "app_name:another_app updated:2024-01-01 hash:def456 url:https://example2.com".to_string(),
            system_info: "arch:x86_64 cpu:AMD Ryzen 9 system:Windows release:Windows 11 python:3.10.0".to_string(),
            model_info: "torch:2.1.0 xformers:0.0.23 diffusers:0.22.0 transformers:4.31.0".to_string(),
            device_info: "device:NVIDIA GeForce RTX 4090 driver:520.56.06 gpu_chip:AD102".to_string(),
            xformers: "0.0.23".to_string(),
            model_name: "stable-diffusion-v2-1".to_string(),
            user: "test_user2".to_string(),
            notes: "Test run 2".to_string(),
        },
        RunData {
            timestamp: "2024-01-01T12:00:00Z".to_string(),
            vram_usage: "1.8/1.9/1.7".to_string(),
            info: "app_name:third_app updated:2024-01-01 hash:ghi789 url:https://example3.com".to_string(),
            system_info: "arch:arm64 cpu:Apple M1 system:macOS release:macOS 13.0 python:3.11.0".to_string(),
            model_info: "torch:2.2.0 xformers:0.0.24 diffusers:0.23.0 transformers:4.32.0".to_string(),
            device_info: "device:Apple M1 Pro driver:1.0.0 gpu_chip:M1_Pro".to_string(),
            xformers: "0.0.24".to_string(),
            model_name: "stable-diffusion-v2-1-768".to_string(),
            user: "test_user3".to_string(),
            notes: "Test run 3".to_string(),
        },
    ]
}

#[tokio::test]
async fn test_save_data_service_integration() -> Result<(), Box<dyn std::error::Error>> {
    info!("Testing SaveDataService integration with transaction support");
    
    let pool = setup_test_database().await?;
    let runs_repository = RunsRepository::new(pool.clone());
    let save_data_service = SaveDataService::new(runs_repository, pool.clone());
    
    // Create test data
    let test_data = create_test_run_data();
    let json_data = serde_json::to_vec(&test_data)?;
    
    // Test save data with transaction support
    let result = save_data_service.save_data(json_data).await?;
    
    assert!(result.success, "Save data should succeed");
    assert_eq!(result.total_rows, 3, "Should process 3 rows");
    assert_eq!(result.inserted_rows, 3, "Should insert 3 rows");
    assert_eq!(result.error_rows, 0, "Should have no errors");
    
    // Verify data was actually inserted
    let runs = runs_repository.find_all().await?;
    assert_eq!(runs.len(), 3, "Should have 3 runs in database");
    
    info!("SaveDataService integration test passed");
    Ok(())
}

#[tokio::test]
async fn test_process_its_service_integration() -> Result<(), Box<dyn std::error::Error>> {
    info!("Testing ProcessItsService integration with transaction support");
    
    let pool = setup_test_database().await?;
    let runs_repository = RunsRepository::new(pool.clone());
    let performance_result_repository = PerformanceResultRepository::new(pool.clone());
    let process_its_service = ProcessItsService::new(runs_repository, performance_result_repository, pool.clone());
    
    // First, insert some test runs
    let test_data = create_test_run_data();
    let json_data = serde_json::to_vec(&test_data)?;
    let save_data_service = SaveDataService::new(RunsRepository::new(pool.clone()), pool.clone());
    save_data_service.save_data(json_data).await?;
    
    // Test ITS processing with transaction support
    let result = process_its_service.process_its().await?;
    
    assert!(result.success, "ITS processing should succeed");
    assert_eq!(result.total_runs, 3, "Should process 3 runs");
    assert_eq!(result.inserted_rows, 3, "Should insert 3 performance results");
    assert_eq!(result.error_rows, 0, "Should have no errors");
    
    // Verify performance results were actually inserted
    let performance_results = performance_result_repository.find_all().await?;
    assert_eq!(performance_results.len(), 3, "Should have 3 performance results");
    
    // Verify ITS values were calculated correctly
    for result in &performance_results {
        assert!(result.its.is_some(), "ITS should be present");
        assert!(result.avg_its.is_some(), "Average ITS should be calculated");
        assert!(result.avg_its.unwrap() > 0.0, "Average ITS should be positive");
    }
    
    info!("ProcessItsService integration test passed");
    Ok(())
}

#[tokio::test]
async fn test_process_app_details_service_integration() -> Result<(), Box<dyn std::error::Error>> {
    info!("Testing ProcessAppDetailsService integration with transaction support");
    
    let pool = setup_test_database().await?;
    let runs_repository = RunsRepository::new(pool.clone());
    let app_details_repository = AppDetailsRepository::new(pool.clone());
    let process_app_details_service = ProcessAppDetailsService::new(runs_repository, app_details_repository, pool.clone());
    
    // First, insert some test runs
    let test_data = create_test_run_data();
    let json_data = serde_json::to_vec(&test_data)?;
    let save_data_service = SaveDataService::new(RunsRepository::new(pool.clone()), pool.clone());
    save_data_service.save_data(json_data).await?;
    
    // Test app details processing with transaction support
    let result = process_app_details_service.process_app_details().await?;
    
    assert!(result.success, "App details processing should succeed");
    assert_eq!(result.total_runs, 3, "Should process 3 runs");
    assert_eq!(result.inserted_rows, 3, "Should insert 3 app details");
    assert_eq!(result.error_rows, 0, "Should have no errors");
    
    // Verify app details were actually inserted
    let app_details = app_details_repository.find_all().await?;
    assert_eq!(app_details.len(), 3, "Should have 3 app details");
    
    // Verify app details were parsed correctly
    for detail in &app_details {
        assert!(detail.app_name.is_some(), "App name should be present");
        assert!(detail.url.is_some(), "URL should be present");
    }
    
    info!("ProcessAppDetailsService integration test passed");
    Ok(())
}

#[tokio::test]
async fn test_process_system_info_service_integration() -> Result<(), Box<dyn std::error::Error>> {
    info!("Testing ProcessSystemInfoService integration with transaction support");
    
    let pool = setup_test_database().await?;
    let runs_repository = RunsRepository::new(pool.clone());
    let system_info_repository = SystemInfoRepository::new(pool.clone());
    let process_system_info_service = ProcessSystemInfoService::new(runs_repository, system_info_repository, pool.clone());
    
    // First, insert some test runs
    let test_data = create_test_run_data();
    let json_data = serde_json::to_vec(&test_data)?;
    let save_data_service = SaveDataService::new(RunsRepository::new(pool.clone()), pool.clone());
    save_data_service.save_data(json_data).await?;
    
    // Test system info processing with transaction support
    let result = process_system_info_service.process_system_info().await?;
    
    assert!(result.success, "System info processing should succeed");
    assert_eq!(result.total_runs, 3, "Should process 3 runs");
    assert_eq!(result.inserted_rows, 3, "Should insert 3 system info records");
    assert_eq!(result.error_rows, 0, "Should have no errors");
    
    // Verify system info was actually inserted
    let system_info = system_info_repository.find_all().await?;
    assert_eq!(system_info.len(), 3, "Should have 3 system info records");
    
    // Verify system info was parsed correctly
    for info in &system_info {
        assert!(info.arch.is_some(), "Architecture should be present");
        assert!(info.cpu.is_some(), "CPU should be present");
        assert!(info.system.is_some(), "System should be present");
        assert!(info.release.is_some(), "Release should be present");
        assert!(info.python.is_some(), "Python version should be present");
    }
    
    info!("ProcessSystemInfoService integration test passed");
    Ok(())
}

#[tokio::test]
async fn test_process_libraries_service_integration() -> Result<(), Box<dyn std::error::Error>> {
    info!("Testing ProcessLibrariesService integration with transaction support");
    
    let pool = setup_test_database().await?;
    let runs_repository = RunsRepository::new(pool.clone());
    let libraries_repository = LibrariesRepository::new(pool.clone());
    let process_libraries_service = ProcessLibrariesService::new(runs_repository, libraries_repository, pool.clone());
    
    // First, insert some test runs
    let test_data = create_test_run_data();
    let json_data = serde_json::to_vec(&test_data)?;
    let save_data_service = SaveDataService::new(RunsRepository::new(pool.clone()), pool.clone());
    save_data_service.save_data(json_data).await?;
    
    // Test libraries processing with transaction support
    let result = process_libraries_service.process_libraries().await?;
    
    assert!(result.success, "Libraries processing should succeed");
    assert_eq!(result.total_runs, 3, "Should process 3 runs");
    assert_eq!(result.inserted_rows, 3, "Should insert 3 library records");
    assert_eq!(result.error_rows, 0, "Should have no errors");
    
    // Verify libraries were actually inserted
    let libraries = libraries_repository.find_all().await?;
    assert_eq!(libraries.len(), 3, "Should have 3 library records");
    
    // Verify libraries were parsed correctly
    for lib in &libraries {
        assert!(lib.torch.is_some(), "Torch version should be present");
        assert!(lib.xformers.is_some(), "Xformers version should be present");
        assert!(lib.xformers1.is_some(), "Xformers1 should be present");
    }
    
    info!("ProcessLibrariesService integration test passed");
    Ok(())
}

#[tokio::test]
async fn test_process_gpu_service_integration() -> Result<(), Box<dyn std::error::Error>> {
    info!("Testing ProcessGpuService integration with transaction support");
    
    let pool = setup_test_database().await?;
    let runs_repository = RunsRepository::new(pool.clone());
    let gpu_repository = GpuRepository::new(pool.clone());
    let process_gpu_service = ProcessGpuService::new(runs_repository, gpu_repository, pool.clone());
    
    // First, insert some test runs
    let test_data = create_test_run_data();
    let json_data = serde_json::to_vec(&test_data)?;
    let save_data_service = SaveDataService::new(RunsRepository::new(pool.clone()), pool.clone());
    save_data_service.save_data(json_data).await?;
    
    // Test GPU processing with transaction support
    let result = process_gpu_service.process_gpu().await?;
    
    assert!(result.success, "GPU processing should succeed");
    assert_eq!(result.total_runs, 3, "Should process 3 runs");
    assert_eq!(result.inserted_rows, 3, "Should insert 3 GPU records");
    assert_eq!(result.error_rows, 0, "Should have no errors");
    
    // Verify GPU records were actually inserted
    let gpu_records = gpu_repository.find_all().await?;
    assert_eq!(gpu_records.len(), 3, "Should have 3 GPU records");
    
    // Verify GPU info was parsed correctly
    for gpu in &gpu_records {
        assert!(gpu.device.is_some(), "Device should be present");
        assert!(gpu.driver.is_some(), "Driver should be present");
        assert!(gpu.gpu_chip.is_some(), "GPU chip should be present");
    }
    
    info!("ProcessGpuService integration test passed");
    Ok(())
}

#[tokio::test]
async fn test_process_run_details_service_integration() -> Result<(), Box<dyn std::error::Error>> {
    info!("Testing ProcessRunDetailsService integration with transaction support");
    
    let pool = setup_test_database().await?;
    let runs_repository = RunsRepository::new(pool.clone());
    let run_more_details_repository = RunMoreDetailsRepository::new(pool.clone());
    let process_run_details_service = ProcessRunDetailsService::new(runs_repository, run_more_details_repository, pool.clone());
    
    // First, insert some test runs
    let test_data = create_test_run_data();
    let json_data = serde_json::to_vec(&test_data)?;
    let save_data_service = SaveDataService::new(RunsRepository::new(pool.clone()), pool.clone());
    save_data_service.save_data(json_data).await?;
    
    // Test run details processing with transaction support
    let result = process_run_details_service.process_run_details().await?;
    
    assert!(result.success, "Run details processing should succeed");
    assert_eq!(result.total_inserts, 3, "Should insert 3 run details");
    
    // Verify run details were actually inserted
    let run_details = run_more_details_repository.find_all().await?;
    assert_eq!(run_details.len(), 3, "Should have 3 run details");
    
    // Verify run details were copied correctly
    for detail in &run_details {
        assert!(detail.run_id.is_some(), "Run ID should be present");
        assert!(detail.timestamp.is_some(), "Timestamp should be present");
        assert!(detail.model_name.is_some(), "Model name should be present");
        assert!(detail.user.is_some(), "User should be present");
    }
    
    info!("ProcessRunDetailsService integration test passed");
    Ok(())
}

#[tokio::test]
async fn test_full_pipeline_integration() -> Result<(), Box<dyn std::error::Error>> {
    info!("Testing full pipeline integration with all services");
    
    let pool = setup_test_database().await?;
    
    // Create all services
    let runs_repository = RunsRepository::new(pool.clone());
    let performance_result_repository = PerformanceResultRepository::new(pool.clone());
    let app_details_repository = AppDetailsRepository::new(pool.clone());
    let system_info_repository = SystemInfoRepository::new(pool.clone());
    let libraries_repository = LibrariesRepository::new(pool.clone());
    let gpu_repository = GpuRepository::new(pool.clone());
    let run_more_details_repository = RunMoreDetailsRepository::new(pool.clone());
    
    let save_data_service = SaveDataService::new(runs_repository.clone(), pool.clone());
    let process_its_service = ProcessItsService::new(runs_repository.clone(), performance_result_repository, pool.clone());
    let process_app_details_service = ProcessAppDetailsService::new(runs_repository.clone(), app_details_repository, pool.clone());
    let process_system_info_service = ProcessSystemInfoService::new(runs_repository.clone(), system_info_repository, pool.clone());
    let process_libraries_service = ProcessLibrariesService::new(runs_repository.clone(), libraries_repository, pool.clone());
    let process_gpu_service = ProcessGpuService::new(runs_repository.clone(), gpu_repository, pool.clone());
    let process_run_details_service = ProcessRunDetailsService::new(runs_repository, run_more_details_repository, pool.clone());
    
    // Create test data
    let test_data = create_test_run_data();
    let json_data = serde_json::to_vec(&test_data)?;
    
    // Step 1: Save data
    info!("Step 1: Saving data");
    let save_result = save_data_service.save_data(json_data).await?;
    assert!(save_result.success, "Save data should succeed");
    assert_eq!(save_result.inserted_rows, 3, "Should insert 3 runs");
    
    // Step 2: Process ITS
    info!("Step 2: Processing ITS");
    let its_result = process_its_service.process_its().await?;
    assert!(its_result.success, "ITS processing should succeed");
    assert_eq!(its_result.inserted_rows, 3, "Should insert 3 performance results");
    
    // Step 3: Process app details
    info!("Step 3: Processing app details");
    let app_details_result = process_app_details_service.process_app_details().await?;
    assert!(app_details_result.success, "App details processing should succeed");
    assert_eq!(app_details_result.inserted_rows, 3, "Should insert 3 app details");
    
    // Step 4: Process system info
    info!("Step 4: Processing system info");
    let system_info_result = process_system_info_service.process_system_info().await?;
    assert!(system_info_result.success, "System info processing should succeed");
    assert_eq!(system_info_result.inserted_rows, 3, "Should insert 3 system info records");
    
    // Step 5: Process libraries
    info!("Step 5: Processing libraries");
    let libraries_result = process_libraries_service.process_libraries().await?;
    assert!(libraries_result.success, "Libraries processing should succeed");
    assert_eq!(libraries_result.inserted_rows, 3, "Should insert 3 library records");
    
    // Step 6: Process GPU
    info!("Step 6: Processing GPU");
    let gpu_result = process_gpu_service.process_gpu().await?;
    assert!(gpu_result.success, "GPU processing should succeed");
    assert_eq!(gpu_result.inserted_rows, 3, "Should insert 3 GPU records");
    
    // Step 7: Process run details
    info!("Step 7: Processing run details");
    let run_details_result = process_run_details_service.process_run_details().await?;
    assert!(run_details_result.success, "Run details processing should succeed");
    assert_eq!(run_details_result.total_inserts, 3, "Should insert 3 run details");
    
    // Verify all data was inserted correctly
    let runs_count = sqlx::query("SELECT COUNT(*) FROM runs").fetch_one(&pool).await?.get::<i64, _>(0);
    let performance_count = sqlx::query("SELECT COUNT(*) FROM performanceResult").fetch_one(&pool).await?.get::<i64, _>(0);
    let app_details_count = sqlx::query("SELECT COUNT(*) FROM AppDetails").fetch_one(&pool).await?.get::<i64, _>(0);
    let system_info_count = sqlx::query("SELECT COUNT(*) FROM SystemInfo").fetch_one(&pool).await?.get::<i64, _>(0);
    let libraries_count = sqlx::query("SELECT COUNT(*) FROM Libraries").fetch_one(&pool).await?.get::<i64, _>(0);
    let gpu_count = sqlx::query("SELECT COUNT(*) FROM GPU").fetch_one(&pool).await?.get::<i64, _>(0);
    let run_details_count = sqlx::query("SELECT COUNT(*) FROM RunMoreDetails").fetch_one(&pool).await?.get::<i64, _>(0);
    
    assert_eq!(runs_count, 3, "Should have 3 runs");
    assert_eq!(performance_count, 3, "Should have 3 performance results");
    assert_eq!(app_details_count, 3, "Should have 3 app details");
    assert_eq!(system_info_count, 3, "Should have 3 system info records");
    assert_eq!(libraries_count, 3, "Should have 3 library records");
    assert_eq!(gpu_count, 3, "Should have 3 GPU records");
    assert_eq!(run_details_count, 3, "Should have 3 run details");
    
    info!("Full pipeline integration test passed successfully!");
    Ok(())
}

#[tokio::test]
async fn test_transaction_rollback_scenario() -> Result<(), Box<dyn std::error::Error>> {
    info!("Testing transaction rollback scenario");
    
    let pool = setup_test_database().await?;
    let runs_repository = RunsRepository::new(pool.clone());
    let save_data_service = SaveDataService::new(runs_repository, pool.clone());
    
    // Create test data with invalid JSON to trigger rollback
    let invalid_json = b"invalid json data";
    
    // Test that invalid data triggers rollback
    let result = save_data_service.save_data(invalid_json.to_vec()).await?;
    
    assert!(!result.success, "Should fail with invalid data");
    assert_eq!(result.inserted_rows, 0, "Should insert 0 rows");
    assert_eq!(result.error_rows, 0, "Should have 0 error rows (transaction level error)");
    
    // Verify no data was actually inserted (rollback worked)
    let runs = sqlx::query("SELECT COUNT(*) FROM runs").fetch_one(&pool).await?.get::<i64, _>(0);
    assert_eq!(runs, 0, "Should have 0 runs after rollback");
    
    info!("Transaction rollback test passed");
    Ok(())
} 