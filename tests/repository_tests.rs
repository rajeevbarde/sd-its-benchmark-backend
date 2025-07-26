use sqlx::SqlitePool;
use sd_its_benchmark::models::{runs::Run, performance_result::PerformanceResult, app_details::AppDetails, system_info::SystemInfo, libraries::Libraries, gpu::Gpu, run_more_details::RunMoreDetails, model_map::ModelMap, gpu_map::GpuMap, gpu_base::GpuBase};
use sd_its_benchmark::repositories::{RunsRepository, PerformanceResultRepository, AppDetailsRepository, SystemInfoRepository, LibrariesRepository, GpuRepository, RunMoreDetailsRepository, ModelMapRepository, GpuMapRepository, GpuBaseRepository, traits::Repository};

async fn create_test_pool() -> SqlitePool {
    SqlitePool::connect("sqlite::memory:").await.unwrap()
}

#[tokio::test]
async fn test_runs_repository_basic_operations() {
    let pool = create_test_pool().await;
    
    // Run migrations to create tables
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    let repo = RunsRepository::new(pool);

    // Test create
    let new_run = Run {
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

    let created_run = repo.create(new_run).await.expect("Failed to create run");
    assert!(created_run.id.is_some());
    assert_eq!(created_run.timestamp, Some("2024-01-01T00:00:00Z".to_string()));

    // Test find_by_id
    let found_run = repo.find_by_id(created_run.id.unwrap()).await.expect("Failed to find run");
    assert!(found_run.is_some());
    let found_run = found_run.unwrap();
    assert_eq!(found_run.timestamp, Some("2024-01-01T00:00:00Z".to_string()));

    // Test update
    let mut updated_run = found_run.clone();
    updated_run.notes = Some("Updated notes".to_string());
    let updated_run = repo.update(updated_run).await.expect("Failed to update run");
    assert_eq!(updated_run.notes, Some("Updated notes".to_string()));

    // Test count
    let count = repo.count().await.expect("Failed to count runs");
    assert_eq!(count, 1);

    // Test delete
    repo.delete(created_run.id.unwrap()).await.expect("Failed to delete run");
    let count_after_delete = repo.count().await.expect("Failed to count runs after delete");
    assert_eq!(count_after_delete, 0);
}

#[tokio::test]
async fn test_performance_result_repository_basic_operations() {
    let pool = create_test_pool().await;
    
    // Run migrations to create tables
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    // First create a Run to reference
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

    let repo = PerformanceResultRepository::new(pool);

    // Test create
    let new_result = PerformanceResult {
        id: None,
        run_id: Some(run_id),
        its: Some("10.5".to_string()),
        avg_its: Some(10.5),
    };

    let created_result = repo.create(new_result).await.expect("Failed to create performance result");
    assert!(created_result.id.is_some());
    assert_eq!(created_result.run_id, Some(run_id));
    assert_eq!(created_result.its, Some("10.5".to_string()));

    // Test find_by_id
    let found_result = repo.find_by_id(created_result.id.unwrap()).await.expect("Failed to find performance result");
    assert!(found_result.is_some());
    let found_result = found_result.unwrap();
    assert_eq!(found_result.run_id, Some(run_id));

    // Test find_by_run_id
    let results_by_run = repo.find_by_run_id(run_id).await.expect("Failed to find results by run_id");
    assert_eq!(results_by_run.len(), 1);
    assert_eq!(results_by_run[0].run_id, Some(run_id));

    // Test update
    let mut updated_result = found_result.clone();
    updated_result.avg_its = Some(11.0);
    let updated_result = repo.update(updated_result).await.expect("Failed to update performance result");
    assert_eq!(updated_result.avg_its, Some(11.0));

    // Test count
    let count = repo.count().await.expect("Failed to count performance results");
    assert_eq!(count, 1);

    // Test delete
    repo.delete(created_result.id.unwrap()).await.expect("Failed to delete performance result");
    let count_after_delete = repo.count().await.expect("Failed to count performance results after delete");
    assert_eq!(count_after_delete, 0);
}

#[tokio::test]
async fn test_app_details_repository_basic_operations() {
    let pool = create_test_pool().await;
    
    // Run migrations to create tables
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    // First create a Run to reference
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

    let repo = AppDetailsRepository::new(pool);

    // Test create
    let new_app_details = AppDetails {
        id: None,
        run_id: Some(run_id),
        app_name: Some("test-app".to_string()),
        updated: Some("2024-01-01T00:00:00Z".to_string()),
        hash: Some("abc123".to_string()),
        url: Some("https://example.com".to_string()),
    };

    let created_app_details = repo.create(new_app_details).await.expect("Failed to create app details");
    assert!(created_app_details.id.is_some());
    assert_eq!(created_app_details.run_id, Some(run_id));
    assert_eq!(created_app_details.app_name, Some("test-app".to_string()));

    // Test find_by_id
    let found_app_details = repo.find_by_id(created_app_details.id.unwrap()).await.expect("Failed to find app details");
    assert!(found_app_details.is_some());
    let found_app_details = found_app_details.unwrap();
    assert_eq!(found_app_details.run_id, Some(run_id));

    // Test find_by_run_id
    let results_by_run = repo.find_by_run_id(run_id).await.expect("Failed to find app details by run_id");
    assert_eq!(results_by_run.len(), 1);
    assert_eq!(results_by_run[0].run_id, Some(run_id));

    // Test find_by_app_name
    let results_by_name = repo.find_by_app_name("test-app").await.expect("Failed to find app details by app_name");
    assert_eq!(results_by_name.len(), 1);
    assert_eq!(results_by_name[0].app_name, Some("test-app".to_string()));

    // Test update
    let mut updated_app_details = found_app_details.clone();
    updated_app_details.url = Some("https://updated-example.com".to_string());
    let updated_app_details = repo.update(updated_app_details).await.expect("Failed to update app details");
    assert_eq!(updated_app_details.url, Some("https://updated-example.com".to_string()));

    // Test count
    let count = repo.count().await.expect("Failed to count app details");
    assert_eq!(count, 1);

    // Test delete
    repo.delete(created_app_details.id.unwrap()).await.expect("Failed to delete app details");
    let count_after_delete = repo.count().await.expect("Failed to count app details after delete");
    assert_eq!(count_after_delete, 0);
}

#[tokio::test]
async fn test_system_info_repository_basic_operations() {
    let pool = create_test_pool().await;
    
    // Run migrations to create tables
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    // First create a Run to reference
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

    let repo = SystemInfoRepository::new(pool);

    // Test create
    let new_system_info = SystemInfo {
        id: None,
        run_id: Some(run_id),
        arch: Some("x86_64".to_string()),
        cpu: Some("Intel i7-12700K".to_string()),
        system: Some("Windows".to_string()),
        release: Some("11.0.0".to_string()),
        python: Some("3.9.0".to_string()),
    };

    let created_system_info = repo.create(new_system_info).await.expect("Failed to create system info");
    assert!(created_system_info.id.is_some());
    assert_eq!(created_system_info.run_id, Some(run_id));
    assert_eq!(created_system_info.arch, Some("x86_64".to_string()));

    // Test find_by_id
    let found_system_info = repo.find_by_id(created_system_info.id.unwrap()).await.expect("Failed to find system info");
    assert!(found_system_info.is_some());
    let found_system_info = found_system_info.unwrap();
    assert_eq!(found_system_info.run_id, Some(run_id));

    // Test find_by_run_id
    let results_by_run = repo.find_by_run_id(run_id).await.expect("Failed to find system info by run_id");
    assert_eq!(results_by_run.len(), 1);
    assert_eq!(results_by_run[0].run_id, Some(run_id));

    // Test find_by_arch
    let results_by_arch = repo.find_by_arch("x86_64").await.expect("Failed to find system info by arch");
    assert_eq!(results_by_arch.len(), 1);
    assert_eq!(results_by_arch[0].arch, Some("x86_64".to_string()));

    // Test find_by_system
    let results_by_system = repo.find_by_system("Windows").await.expect("Failed to find system info by system");
    assert_eq!(results_by_system.len(), 1);
    assert_eq!(results_by_system[0].system, Some("Windows".to_string()));

    // Test update
    let mut updated_system_info = found_system_info.clone();
    updated_system_info.python = Some("3.10.0".to_string());
    let updated_system_info = repo.update(updated_system_info).await.expect("Failed to update system info");
    assert_eq!(updated_system_info.python, Some("3.10.0".to_string()));

    // Test count
    let count = repo.count().await.expect("Failed to count system info");
    assert_eq!(count, 1);

    // Test delete
    repo.delete(created_system_info.id.unwrap()).await.expect("Failed to delete system info");
    let count_after_delete = repo.count().await.expect("Failed to count system info after delete");
    assert_eq!(count_after_delete, 0);
} 

#[tokio::test]
async fn test_libraries_repository_basic_operations() {
    let pool = create_test_pool().await;
    
    // Run migrations to create tables
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    // First create a Run to reference
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

    let repo = LibrariesRepository::new(pool);

    // Test create
    let new_libraries = Libraries {
        id: None,
        run_id: Some(run_id),
        torch: Some("1.12.0".to_string()),
        xformers: Some("0.0.16".to_string()),
        xformers1: Some("0.0.17".to_string()),
        diffusers: Some("0.7.2".to_string()),
        transformers: Some("4.19.2".to_string()),
    };

    let created_libraries = repo.create(new_libraries).await.expect("Failed to create libraries");
    assert!(created_libraries.id.is_some());
    assert_eq!(created_libraries.run_id, Some(run_id));
    assert_eq!(created_libraries.torch, Some("1.12.0".to_string()));

    // Test find_by_id
    let found_libraries = repo.find_by_id(created_libraries.id.unwrap()).await.expect("Failed to find libraries");
    assert!(found_libraries.is_some());
    let found_libraries = found_libraries.unwrap();
    assert_eq!(found_libraries.run_id, Some(run_id));

    // Test find_by_run_id
    let results_by_run = repo.find_by_run_id(run_id).await.expect("Failed to find libraries by run_id");
    assert_eq!(results_by_run.len(), 1);
    assert_eq!(results_by_run[0].run_id, Some(run_id));

    // Test update
    let mut updated_libraries = found_libraries.clone();
    updated_libraries.torch = Some("1.13.0".to_string());
    let updated_libraries = repo.update(updated_libraries).await.expect("Failed to update libraries");
    assert_eq!(updated_libraries.torch, Some("1.13.0".to_string()));

    // Test count
    let count = repo.count().await.expect("Failed to count libraries");
    assert_eq!(count, 1);

    // Test delete
    repo.delete(created_libraries.id.unwrap()).await.expect("Failed to delete libraries");
    let count_after_delete = repo.count().await.expect("Failed to count libraries after delete");
    assert_eq!(count_after_delete, 0);
} 

#[tokio::test]
async fn test_gpu_repository_basic_operations() {
    let pool = create_test_pool().await;
    
    // Run migrations to create tables
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    // First create a Run to reference
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

    let repo = GpuRepository::new(pool);

    // Test create
    let new_gpu = Gpu {
        id: None,
        run_id: Some(run_id),
        device: Some("NVIDIA GeForce RTX 4090".to_string()),
        driver: Some("525.89.01".to_string()),
        gpu_chip: Some("AD102".to_string()),
        brand: Some("NVIDIA".to_string()),
        is_laptop: Some(false),
    };

    let created_gpu = repo.create(new_gpu).await.expect("Failed to create GPU");
    assert!(created_gpu.id.is_some());
    assert_eq!(created_gpu.run_id, Some(run_id));
    assert_eq!(created_gpu.device, Some("NVIDIA GeForce RTX 4090".to_string()));

    // Test find_by_id
    let found_gpu = repo.find_by_id(created_gpu.id.unwrap()).await.expect("Failed to find GPU");
    assert!(found_gpu.is_some());
    let found_gpu = found_gpu.unwrap();
    assert_eq!(found_gpu.run_id, Some(run_id));

    // Test find_by_run_id
    let results_by_run = repo.find_by_run_id(run_id).await.expect("Failed to find GPUs by run_id");
    assert_eq!(results_by_run.len(), 1);
    assert_eq!(results_by_run[0].run_id, Some(run_id));

    // Test find_by_brand
    let results_by_brand = repo.find_by_brand("NVIDIA").await.expect("Failed to find GPUs by brand");
    assert_eq!(results_by_brand.len(), 1);
    assert_eq!(results_by_brand[0].brand, Some("NVIDIA".to_string()));

    // Test find_by_laptop_status
    let results_by_laptop = repo.find_by_laptop_status(false).await.expect("Failed to find GPUs by laptop status");
    assert_eq!(results_by_laptop.len(), 1);
    assert_eq!(results_by_laptop[0].is_laptop, Some(false));

    // Test update
    let mut updated_gpu = found_gpu.clone();
    updated_gpu.driver = Some("530.41.03".to_string());
    let updated_gpu = repo.update(updated_gpu).await.expect("Failed to update GPU");
    assert_eq!(updated_gpu.driver, Some("530.41.03".to_string()));

    // Test count
    let count = repo.count().await.expect("Failed to count GPUs");
    assert_eq!(count, 1);

    // Test delete
    repo.delete(created_gpu.id.unwrap()).await.expect("Failed to delete GPU");
    let count_after_delete = repo.count().await.expect("Failed to count GPUs after delete");
    assert_eq!(count_after_delete, 0);
} 

#[tokio::test]
async fn test_run_more_details_repository_basic_operations() {
    let pool = create_test_pool().await;
    
    // Run migrations to create tables
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    // First create a Run to reference
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

    let repo = RunMoreDetailsRepository::new(pool);

    // Test create
    let new_details = RunMoreDetails {
        id: None,
        run_id: Some(run_id),
        timestamp: Some("2024-01-01T12:00:00Z".to_string()),
        model_name: Some("stable-diffusion-v1-5".to_string()),
        user: Some("test-user".to_string()),
        notes: Some("Additional test details".to_string()),
        model_map_id: Some(1),
    };

    let created_details = repo.create(new_details).await.expect("Failed to create run more details");
    assert!(created_details.id.is_some());
    assert_eq!(created_details.run_id, Some(run_id));
    assert_eq!(created_details.model_name, Some("stable-diffusion-v1-5".to_string()));

    // Test find_by_id
    let found_details = repo.find_by_id(created_details.id.unwrap()).await.expect("Failed to find run more details");
    assert!(found_details.is_some());
    let found_details = found_details.unwrap();
    assert_eq!(found_details.run_id, Some(run_id));

    // Test find_by_run_id
    let results_by_run = repo.find_by_run_id(run_id).await.expect("Failed to find details by run_id");
    assert_eq!(results_by_run.len(), 1);
    assert_eq!(results_by_run[0].run_id, Some(run_id));

    // Test find_by_model_name
    let results_by_model = repo.find_by_model_name("stable-diffusion-v1-5").await.expect("Failed to find details by model_name");
    assert_eq!(results_by_model.len(), 1);
    assert_eq!(results_by_model[0].model_name, Some("stable-diffusion-v1-5".to_string()));

    // Test find_by_user
    let results_by_user = repo.find_by_user("test-user").await.expect("Failed to find details by user");
    assert_eq!(results_by_user.len(), 1);
    assert_eq!(results_by_user[0].user, Some("test-user".to_string()));

    // Test update
    let mut updated_details = found_details.clone();
    updated_details.notes = Some("Updated test details".to_string());
    let updated_details = repo.update(updated_details).await.expect("Failed to update run more details");
    assert_eq!(updated_details.notes, Some("Updated test details".to_string()));

    // Test count
    let count = repo.count().await.expect("Failed to count run more details");
    assert_eq!(count, 1);

    // Test delete
    repo.delete(created_details.id.unwrap()).await.expect("Failed to delete run more details");
    let count_after_delete = repo.count().await.expect("Failed to count run more details after delete");
    assert_eq!(count_after_delete, 0);
} 

#[tokio::test]
async fn test_model_map_repository_basic_operations() {
    let pool = create_test_pool().await;
    
    // Run migrations to create tables
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    let repo = ModelMapRepository::new(pool);

    // Test create
    let new_model_map = ModelMap {
        id: None,
        model_name: Some("stable-diffusion-v1-5".to_string()),
        base_model: Some("stable-diffusion-v1-5".to_string()),
    };

    let created_model_map = repo.create(new_model_map).await.expect("Failed to create model map");
    assert!(created_model_map.id.is_some());
    assert_eq!(created_model_map.model_name, Some("stable-diffusion-v1-5".to_string()));
    assert_eq!(created_model_map.base_model, Some("stable-diffusion-v1-5".to_string()));

    // Test find_by_id
    let found_model_map = repo.find_by_id(created_model_map.id.unwrap()).await.expect("Failed to find model map");
    assert!(found_model_map.is_some());
    let found_model_map = found_model_map.unwrap();
    assert_eq!(found_model_map.model_name, Some("stable-diffusion-v1-5".to_string()));

    // Test find_by_model_name
    let results_by_model_name = repo.find_by_model_name("stable-diffusion-v1-5").await.expect("Failed to find model map by model_name");
    assert_eq!(results_by_model_name.len(), 1);
    assert_eq!(results_by_model_name[0].model_name, Some("stable-diffusion-v1-5".to_string()));

    // Test find_by_base_model
    let results_by_base_model = repo.find_by_base_model("stable-diffusion-v1-5").await.expect("Failed to find model map by base_model");
    assert_eq!(results_by_base_model.len(), 1);
    assert_eq!(results_by_base_model[0].base_model, Some("stable-diffusion-v1-5".to_string()));

    // Test update
    let mut updated_model_map = found_model_map.clone();
    updated_model_map.base_model = Some("stable-diffusion-v2-1".to_string());
    let updated_model_map = repo.update(updated_model_map).await.expect("Failed to update model map");
    assert_eq!(updated_model_map.base_model, Some("stable-diffusion-v2-1".to_string()));

    // Test count
    let count = repo.count().await.expect("Failed to count model maps");
    assert_eq!(count, 1);

    // Test delete
    repo.delete(created_model_map.id.unwrap()).await.expect("Failed to delete model map");
    let count_after_delete = repo.count().await.expect("Failed to count model maps after delete");
    assert_eq!(count_after_delete, 0);
} 

#[tokio::test]
async fn test_gpu_map_repository_basic_operations() {
    let pool = create_test_pool().await;
    
    // Run migrations to create tables
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    // First create a GPUBase record to reference
    let gpu_base_id = sqlx::query!(
        r#"
        INSERT INTO GPUBase (name, brand)
        VALUES (?, ?)
        "#,
        "RTX 4090 Base",
        "NVIDIA"
    )
    .execute(&pool)
    .await
    .expect("Failed to create GPU base")
    .last_insert_rowid() as i64;

    let repo = GpuMapRepository::new(pool);

    // Test create
    let new_gpu_map = GpuMap {
        id: None,
        gpu_name: Some("RTX 4090".to_string()),
        base_gpu_id: Some(gpu_base_id),
    };

    let created_gpu_map = repo.create(new_gpu_map).await.expect("Failed to create GPU map");
    assert!(created_gpu_map.id.is_some());
    assert_eq!(created_gpu_map.gpu_name, Some("RTX 4090".to_string()));
    assert_eq!(created_gpu_map.base_gpu_id, Some(gpu_base_id));

    // Test find_by_id
    let found_gpu_map = repo.find_by_id(created_gpu_map.id.unwrap()).await.expect("Failed to find GPU map");
    assert!(found_gpu_map.is_some());
    let found_gpu_map = found_gpu_map.unwrap();
    assert_eq!(found_gpu_map.gpu_name, Some("RTX 4090".to_string()));

    // Test find_by_gpu_name
    let results_by_gpu_name = repo.find_by_gpu_name("RTX 4090").await.expect("Failed to find GPU map by gpu_name");
    assert_eq!(results_by_gpu_name.len(), 1);
    assert_eq!(results_by_gpu_name[0].gpu_name, Some("RTX 4090".to_string()));

    // Test find_by_base_gpu_id
    let results_by_base_gpu_id = repo.find_by_base_gpu_id(gpu_base_id).await.expect("Failed to find GPU map by base_gpu_id");
    assert_eq!(results_by_base_gpu_id.len(), 1);
    assert_eq!(results_by_base_gpu_id[0].base_gpu_id, Some(gpu_base_id));

    // Test update
    let mut updated_gpu_map = found_gpu_map.clone();
    updated_gpu_map.base_gpu_id = Some(gpu_base_id);
    let updated_gpu_map = repo.update(updated_gpu_map).await.expect("Failed to update GPU map");
    assert_eq!(updated_gpu_map.base_gpu_id, Some(gpu_base_id));

    // Test count
    let count = repo.count().await.expect("Failed to count GPU maps");
    assert_eq!(count, 1);

    // Test delete
    repo.delete(created_gpu_map.id.unwrap()).await.expect("Failed to delete GPU map");
    let count_after_delete = repo.count().await.expect("Failed to count GPU maps after delete");
    assert_eq!(count_after_delete, 0);
} 

#[tokio::test]
async fn test_gpu_base_repository_basic_operations() {
    let pool = create_test_pool().await;
    
    // Run migrations to create tables
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    let repo = GpuBaseRepository::new(pool);

    // Test create
    let new_gpu_base = GpuBase {
        id: None,
        name: "RTX 4090 Base".to_string(),
        brand: Some("NVIDIA".to_string()),
    };

    let created_gpu_base = repo.create(new_gpu_base).await.expect("Failed to create GPU base");
    assert!(created_gpu_base.id.is_some());
    assert_eq!(created_gpu_base.name, "RTX 4090 Base".to_string());
    assert_eq!(created_gpu_base.brand, Some("NVIDIA".to_string()));

    // Test find_by_id
    let found_gpu_base = repo.find_by_id(created_gpu_base.id.unwrap()).await.expect("Failed to find GPU base");
    assert!(found_gpu_base.is_some());
    let found_gpu_base = found_gpu_base.unwrap();
    assert_eq!(found_gpu_base.name, "RTX 4090 Base".to_string());

    // Test find_by_name
    let results_by_name = repo.find_by_name("RTX 4090 Base").await.expect("Failed to find GPU base by name");
    assert_eq!(results_by_name.len(), 1);
    assert_eq!(results_by_name[0].name, "RTX 4090 Base".to_string());

    // Test find_by_brand
    let results_by_brand = repo.find_by_brand("NVIDIA").await.expect("Failed to find GPU base by brand");
    assert_eq!(results_by_brand.len(), 1);
    assert_eq!(results_by_brand[0].brand, Some("NVIDIA".to_string()));

    // Test update
    let mut updated_gpu_base = found_gpu_base.clone();
    updated_gpu_base.brand = Some("NVIDIA Corporation".to_string());
    let updated_gpu_base = repo.update(updated_gpu_base).await.expect("Failed to update GPU base");
    assert_eq!(updated_gpu_base.brand, Some("NVIDIA Corporation".to_string()));

    // Test count
    let count = repo.count().await.expect("Failed to count GPU bases");
    assert_eq!(count, 1);

    // Test delete
    repo.delete(created_gpu_base.id.unwrap()).await.expect("Failed to delete GPU base");
    let count_after_delete = repo.count().await.expect("Failed to count GPU bases after delete");
    assert_eq!(count_after_delete, 0);
} 