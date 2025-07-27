use axum::{
    extract::State,
    response::Json,
};
use axum_extra::extract::Multipart;
use serde::{Deserialize, Serialize};
use sqlx::{Sqlite, Transaction};
use tracing::{error, info, warn};
// validator::Validate removed as it's no longer used

use crate::{
    error::types::AppError,
    models::{runs::Run, performance_result::PerformanceResult, app_details::AppDetails, system_info::SystemInfo, libraries::Libraries, gpu::Gpu, run_more_details::RunMoreDetails},
    repositories::{
        runs_repository::RunsRepository,
        performance_result_repository::PerformanceResultRepository,
        app_details_repository::AppDetailsRepository,
        system_info_repository::SystemInfoRepository,
        libraries_repository::LibrariesRepository,
        gpu_repository::GpuRepository,
        run_more_details_repository::RunMoreDetailsRepository,
        traits::{Repository, TransactionRepository},
    },
    handlers::{common::create_file_upload_response, validation::{RunData, FixAppNamesRequest, validate_json_content, validate_timestamp_format, validate_vram_usage_format, MAX_FILE_SIZE, ALLOWED_FILE_EXTENSIONS}},
    middleware::validation::validate_file_upload,
    AppState,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct SaveDataResponse {
    pub success: bool,
    pub message: String,
    pub total_rows: usize,
    pub inserted_rows: usize,
    pub error_rows: usize,
    pub error_data: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProcessItsResponse {
    pub success: bool,
    pub rows_inserted: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProcessAppDetailsResponse {
    pub success: bool,
    pub rows_inserted: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProcessSystemInfoResponse {
    pub success: bool,
    pub rows_inserted: usize,
}

#[derive(Debug, Serialize)]
pub struct ProcessLibrariesResponse {
    pub success: bool,
    pub rows_inserted: usize,
}

#[derive(Debug, Serialize)]
pub struct ProcessGpuResponse {
    pub success: bool,
    pub rows_inserted: usize,
}

// RunData is now imported from validation module

pub async fn save_data(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Json<crate::handlers::common::FileUploadResponse>, AppError> {
    info!("Processing save-data request");

    // Extract file from multipart
    let mut file_content = None;
    let mut file_name = None;
    
    while let Some(field) = multipart.next_field().await.map_err(|e| {
        error!("Failed to read multipart field: {}", e);
        AppError::BadRequest("Invalid multipart data".to_string())
    })? {
        if field.name() == Some("file") {
            file_name = field.file_name().map(|s| s.to_string());
            let data = field.bytes().await.map_err(|e| {
                error!("Failed to read file bytes: {}", e);
                AppError::BadRequest("Failed to read uploaded file".to_string())
            })?;
            file_content = Some(data);
            break;
        }
    }

    let file_bytes = file_content.ok_or_else(|| {
        error!("No file provided in multipart data");
        AppError::BadRequest("No file provided".to_string())
    })?;

    let final_file_name = file_name.as_ref().unwrap_or(&"unknown.json".to_string()).to_string();

    // Validate file upload
    validate_file_upload(
        &file_bytes,
        &final_file_name,
        MAX_FILE_SIZE,
        ALLOWED_FILE_EXTENSIONS,
    )?;

    // Validate JSON content
    validate_json_content(&file_bytes).map_err(|e| {
        AppError::Validation(format!("Invalid JSON content: {}", e))
    })?;

    // Parse JSON from file
    let file_string = String::from_utf8(file_bytes.to_vec()).map_err(|e| {
        error!("Failed to convert file to UTF-8: {}", e);
        AppError::BadRequest("File is not valid UTF-8".to_string())
    })?;

    let run_data: Vec<RunData> = serde_json::from_str(&file_string).map_err(|e| {
        error!("Failed to parse JSON: {}", e);
        AppError::BadRequest("Invalid JSON format".to_string())
    })?;

    // Validate each run data entry
    for (index, data) in run_data.iter().enumerate() {
        // Additional custom validations
        validate_timestamp_format(&data.timestamp).map_err(|e| {
            AppError::Validation(format!("Invalid timestamp format at index {}: {}", index, e))
        })?;
        validate_vram_usage_format(&data.vram_usage).map_err(|e| {
            AppError::Validation(format!("Invalid VRAM usage format at index {}: {}", index, e))
        })?;
    }

    info!("Parsed {} rows from uploaded file", run_data.len());

    // Start transaction
    let mut tx = state.db.begin().await.map_err(|e| {
        error!("Failed to begin transaction: {}", e);
        AppError::Database(e)
    })?;

    // Clear existing data
    if let Err(e) = clear_runs_table(&mut tx).await {
        error!("Failed to clear runs table: {}", e);
        tx.rollback().await.map_err(|rollback_err| {
            error!("Failed to rollback transaction: {}", rollback_err);
            AppError::Database(rollback_err)
        })?;
        return Err(AppError::Database(e));
    }

    info!("Cleared existing runs data");

    // Process data insertion
    let runs_repo = RunsRepository::new(state.db.clone());
    let mut inserted_rows = 0;
    let mut error_rows = 0;
    let mut error_data = Vec::new();

    for (index, data) in run_data.iter().enumerate() {
        let run = Run {
            id: None,
            timestamp: Some(data.timestamp.clone()),
            vram_usage: Some(data.vram_usage.clone()),
            info: Some(data.info.clone()),
            system_info: Some(data.system_info.clone()),
            model_info: Some(data.model_info.clone()),
            device_info: Some(data.device_info.clone()),
            xformers: Some(data.xformers.clone()),
            model_name: Some(data.model_name.clone()),
            user: Some(data.user.clone()),
            notes: Some(data.notes.clone()),
        };

        match runs_repo.create_tx(run, &mut tx).await {
            Ok(_) => {
                inserted_rows += 1;
            }
            Err(e) => {
                error_rows += 1;
                let error_msg = format!("Row {}: {}", index + 1, e);
                error_data.push(error_msg.clone());
                warn!("Failed to insert row {}: {}", index + 1, e);
            }
        }
    }

    // Commit transaction
    if let Err(e) = tx.commit().await {
        error!("Failed to commit transaction: {}", e);
        return Err(AppError::Database(e));
    }

    info!(
        "Data processing complete: {} inserted, {} errors out of {} total",
        inserted_rows, error_rows, run_data.len()
    );

    let final_file_name = file_name.as_ref().unwrap_or(&"unknown.json".to_string()).to_string();
    
    Ok(create_file_upload_response(
        "Data processed successfully",
        &final_file_name,
        file_bytes.len(),
        run_data.len(),
        inserted_rows,
        error_rows,
        axum::http::StatusCode::OK,
    ))
}

async fn clear_runs_table(tx: &mut Transaction<'_, Sqlite>) -> Result<(), sqlx::Error> {
    // First disable foreign key constraints temporarily
    sqlx::query!("PRAGMA foreign_keys = OFF")
        .execute(&mut **tx)
        .await?;
    
    // Clear all dependent tables first
    sqlx::query!("DELETE FROM performanceResult")
        .execute(&mut **tx)
        .await?;
    sqlx::query!("DELETE FROM AppDetails")
        .execute(&mut **tx)
        .await?;
    sqlx::query!("DELETE FROM SystemInfo")
        .execute(&mut **tx)
        .await?;
    sqlx::query!("DELETE FROM Libraries")
        .execute(&mut **tx)
        .await?;
    sqlx::query!("DELETE FROM GPU")
        .execute(&mut **tx)
        .await?;
    sqlx::query!("DELETE FROM RunMoreDetails")
        .execute(&mut **tx)
        .await?;
    
    // Clear the runs table
    sqlx::query!("DELETE FROM runs")
        .execute(&mut **tx)
        .await?;
    
    // Re-enable foreign key constraints
    sqlx::query!("PRAGMA foreign_keys = ON")
        .execute(&mut **tx)
        .await?;
    
    Ok(())
}

pub async fn process_its(
    State(state): State<AppState>,
) -> Result<Json<crate::handlers::common::ProcessingResponse>, AppError> {
    info!("Processing ITS data from runs table");

    // Start transaction
    let mut tx = state.db.begin().await.map_err(|e| {
        error!("Failed to begin transaction: {}", e);
        AppError::Database(e)
    })?;

    // Clear existing performance results
    let perf_repo = PerformanceResultRepository::new(state.db.clone());
    if let Err(e) = perf_repo.clear_all_tx(&mut tx).await {
        error!("Failed to clear performance results: {}", e);
        tx.rollback().await.map_err(|rollback_err| {
            error!("Failed to rollback transaction: {}", rollback_err);
            AppError::Database(rollback_err)
        })?;
        return Err(AppError::Database(e));
    }

    info!("Cleared existing performance results");

    // Fetch all runs data
    let runs_repo = RunsRepository::new(state.db.clone());
    let runs = runs_repo.find_all().await.map_err(|e| {
        error!("Failed to fetch runs data: {}", e);
        AppError::Database(e)
    })?;

    info!("Found {} runs to process", runs.len());

    let mut inserted_rows = 0;

    // Process each run
    for (index, run) in runs.iter().enumerate() {
        let run_id = run.id.ok_or_else(|| {
            error!("Run at index {} has no ID", index);
            AppError::BadRequest("Invalid run data".to_string())
        })?;

        let vram_usage = run.vram_usage.as_ref().ok_or_else(|| {
            error!("Run {} has no vram_usage", run_id);
            AppError::BadRequest("Missing vram_usage data".to_string())
        })?;

        info!("Processing run {} of {} (ID: {})", index + 1, runs.len(), run_id);

        // Parse ITS values from vram_usage string
        let its_values: Vec<f64> = vram_usage
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

        // Calculate average ITS
        let avg_its = if its_values.is_empty() {
            None
        } else {
            let sum: f64 = its_values.iter().sum();
            Some(sum / its_values.len() as f64)
        };

        // Create performance result
        let performance_result = PerformanceResult {
            id: None,
            run_id: Some(run_id),
            its: Some(vram_usage.clone()),
            avg_its,
        };

        // Insert into database
        match perf_repo.create_tx(performance_result, &mut tx).await {
            Ok(_) => {
                inserted_rows += 1;
                info!("Processed run {} with average ITS: {}", index + 1, avg_its.unwrap_or(0.0));
            }
            Err(e) => {
                error!("Failed to insert performance result for run {}: {}", run_id, e);
                // Continue processing other runs
            }
        }
    }

    // Commit transaction
    if let Err(e) = tx.commit().await {
        error!("Failed to commit transaction: {}", e);
        return Err(AppError::Database(e));
    }

    info!("ITS processing complete: {} rows inserted", inserted_rows);

    Ok(crate::handlers::common::create_processing_response(
        "ITS processing completed successfully",
        runs.len(),
        inserted_rows,
        0, // rows_updated
        0, // rows_deleted
        vec![], // errors
        axum::http::StatusCode::OK,
    ))
}

pub async fn process_app_details(
    State(state): State<AppState>,
) -> Result<Json<ProcessAppDetailsResponse>, AppError> {
    info!("Processing app details from runs table");

    // Start transaction
    let mut tx = state.db.begin().await.map_err(|e| {
        error!("Failed to begin transaction: {}", e);
        AppError::Database(e)
    })?;

    // Clear existing app details
    let app_details_repo = AppDetailsRepository::new(state.db.clone());
    if let Err(e) = app_details_repo.clear_all_tx(&mut tx).await {
        error!("Failed to clear app details: {}", e);
        tx.rollback().await.map_err(|rollback_err| {
            error!("Failed to rollback transaction: {}", rollback_err);
            AppError::Database(rollback_err)
        })?;
        return Err(AppError::Database(e));
    }

    info!("Cleared existing app details");

    // Fetch all runs data
    let runs_repo = RunsRepository::new(state.db.clone());
    let runs = runs_repo.find_all().await.map_err(|e| {
        error!("Failed to fetch runs data: {}", e);
        AppError::Database(e)
    })?;

    info!("Found {} runs to process", runs.len());

    let mut inserted_rows = 0;

    // Process each run
    for (index, run) in runs.iter().enumerate() {
        let run_id = run.id.ok_or_else(|| {
            error!("Run at index {} has no ID", index);
            AppError::BadRequest("Invalid run data".to_string())
        })?;

        let info = run.info.as_ref().ok_or_else(|| {
            error!("Run {} has no info", run_id);
            AppError::BadRequest("Missing info data".to_string())
        })?;

        info!("Processing app details for run {} of {} (ID: {})", index + 1, runs.len(), run_id);

        // Parse app details from info string
        let app_details = parse_app_details(info);

        // Store app_name for logging
        let app_name_for_log = app_details.app_name.clone();

        // Create app details record
        let app_details_record = AppDetails {
            id: None,
            run_id: Some(run_id),
            app_name: app_details.app_name,
            updated: app_details.updated,
            hash: app_details.hash,
            url: app_details.url,
        };

        // Insert into database
        match app_details_repo.create_tx(app_details_record, &mut tx).await {
            Ok(_) => {
                inserted_rows += 1;
                info!("Processed app details for run {}: app={:?}", index + 1, app_name_for_log);
            }
            Err(e) => {
                error!("Failed to insert app details for run {}: {}", run_id, e);
                // Continue processing other runs
            }
        }
    }

    // Commit transaction
    if let Err(e) = tx.commit().await {
        error!("Failed to commit transaction: {}", e);
        return Err(AppError::Database(e));
    }

    info!("App details processing complete: {} rows inserted", inserted_rows);

    let response = ProcessAppDetailsResponse {
        success: true,
        rows_inserted: inserted_rows,
    };

    Ok(Json(response))
}

#[derive(Debug)]
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

pub async fn process_system_info(
    State(state): State<AppState>,
) -> Result<Json<ProcessSystemInfoResponse>, AppError> {
    info!("Processing system info from runs table");

    // Start transaction
    let mut tx = state.db.begin().await.map_err(|e| {
        error!("Failed to begin transaction: {}", e);
        AppError::Database(e)
    })?;

    // Clear existing system info
    let system_info_repo = SystemInfoRepository::new(state.db.clone());
    if let Err(e) = system_info_repo.clear_all_tx(&mut tx).await {
        error!("Failed to clear system info: {}", e);
        tx.rollback().await.map_err(|rollback_err| {
            error!("Failed to rollback transaction: {}", rollback_err);
            AppError::Database(rollback_err)
        })?;
        return Err(AppError::Database(e));
    }

    info!("Cleared existing system info");

    // Fetch all runs data
    let runs_repo = RunsRepository::new(state.db.clone());
    let runs = runs_repo.find_all().await.map_err(|e| {
        error!("Failed to fetch runs data: {}", e);
        AppError::Database(e)
    })?;

    info!("Found {} runs to process", runs.len());

    let mut inserted_rows = 0;

    // Process each run
    for (index, run) in runs.iter().enumerate() {
        let run_id = run.id.ok_or_else(|| {
            error!("Run at index {} has no ID", index);
            AppError::BadRequest("Invalid run data".to_string())
        })?;

        let system_info = run.system_info.as_ref().ok_or_else(|| {
            error!("Run {} has no system_info", run_id);
            AppError::BadRequest("Missing system_info data".to_string())
        })?;

        info!("Processing system info for run {} of {} (ID: {})", index + 1, runs.len(), run_id);

        // Parse system info from system_info string
        let parsed_system_info = parse_system_info(system_info);

        // Store arch for logging
        let arch_for_log = parsed_system_info.arch.clone();

        // Only insert if all required fields are present
        if parsed_system_info.arch.is_some() &&
           parsed_system_info.cpu.is_some() &&
           parsed_system_info.system.is_some() &&
           parsed_system_info.release.is_some() &&
           parsed_system_info.python.is_some() {
            
            // Create system info record
            let system_info_record = SystemInfo {
                id: None,
                run_id: Some(run_id),
                arch: parsed_system_info.arch,
                cpu: parsed_system_info.cpu,
                system: parsed_system_info.system,
                release: parsed_system_info.release,
                python: parsed_system_info.python,
            };

            // Insert into database
            match system_info_repo.create_tx(system_info_record, &mut tx).await {
                Ok(_) => {
                    inserted_rows += 1;
                    info!("Processed system info for run {}: arch={:?}", index + 1, arch_for_log);
                }
                Err(e) => {
                    error!("Failed to insert system info for run {}: {}", run_id, e);
                    // Continue processing other runs
                }
            }
        } else {
            warn!("Skipping run {} due to missing required system info fields", run_id);
        }
    }

    // Commit transaction
    if let Err(e) = tx.commit().await {
        error!("Failed to commit transaction: {}", e);
        return Err(AppError::Database(e));
    }

    info!("System info processing complete: {} rows inserted", inserted_rows);

    let response = ProcessSystemInfoResponse {
        success: true,
        rows_inserted: inserted_rows,
    };

    Ok(Json(response))
}

#[derive(Debug)]
struct ParsedSystemInfo {
    arch: Option<String>,
    cpu: Option<String>,
    system: Option<String>,
    release: Option<String>,
    python: Option<String>,
}

fn parse_system_info(system_info_string: &str) -> ParsedSystemInfo {
    let mut system_info = ParsedSystemInfo {
        arch: None,
        cpu: None,
        system: None,
        release: None,
        python: None,
    };

    // Split by spaces and process each part
    let parts: Vec<&str> = system_info_string.split(' ').collect();
    let mut i = 0;
    
    while i < parts.len() {
        let part = parts[i];
        
        if let Some(colon_index) = part.find(':') {
            let key = &part[..colon_index];
            let value = &part[colon_index + 1..];
            
            // Handle multi-word values
            let mut full_value = value.to_string();
            let mut next_i = i + 1;
            
            // Look ahead for multi-word values (especially for CPU)
            while next_i < parts.len() && !parts[next_i].contains(':') {
                full_value.push(' ');
                full_value.push_str(parts[next_i]);
                next_i += 1;
            }
            
            match key {
                "arch" => {
                    system_info.arch = Some(full_value);
                }
                "cpu" => {
                    system_info.cpu = Some(full_value);
                }
                "system" => {
                    system_info.system = Some(full_value);
                }
                "release" => {
                    system_info.release = Some(full_value);
                }
                "python" => {
                    system_info.python = Some(full_value);
                }
                _ => {}
            }
            
            i = next_i;
        } else {
            i += 1;
        }
    }

    system_info
}

#[derive(Debug)]
struct ParsedLibraries {
    torch: Option<String>,
    xformers: Option<String>,
    diffusers: Option<String>,
    transformers: Option<String>,
}

fn parse_model_info(model_info_string: &str) -> ParsedLibraries {
    let parts: Vec<&str> = model_info_string.split(' ').collect();
    let mut parsed_libraries = ParsedLibraries {
        torch: None,
        xformers: None,
        diffusers: None,
        transformers: None,
    };

    let mut torch_flag = false;
    let mut torch_value = String::new();

    for part in parts {
        let colon_index = match part.find(':') {
            Some(index) => index,
            None => {
                // If we're collecting torch info and this part has no colon, continue collecting
                if torch_flag {
                    torch_value.push(' ');
                    torch_value.push_str(part);
                    parsed_libraries.torch = Some(torch_value.trim().to_string());
                }
                continue;
            }
        };

        let key = &part[..colon_index];
        let value = &part[colon_index + 1..];

        match key {
            "torch" => {
                // Start collecting the torch value
                torch_flag = true;
                torch_value = value.to_string();
                parsed_libraries.torch = Some(torch_value.clone());
            }
            "xformers" => {
                torch_flag = false;
                parsed_libraries.xformers = Some(value.to_string());
            }
            "diffusers" => {
                torch_flag = false;
                parsed_libraries.diffusers = Some(value.to_string());
            }
            "transformers" => {
                torch_flag = false;
                parsed_libraries.transformers = Some(value.to_string());
            }
            _ => {
                // If we're collecting torch info, continue
                if torch_flag {
                    torch_value.push(' ');
                    torch_value.push_str(part);
                    parsed_libraries.torch = Some(torch_value.trim().to_string());
                }
            }
        }
    }

    parsed_libraries
}

pub async fn process_libraries(
    State(state): State<AppState>,
) -> Result<Json<ProcessLibrariesResponse>, AppError> {
    info!("Processing libraries from runs table");

    // Start transaction
    let mut tx = state.db.begin().await.map_err(|e| {
        error!("Failed to begin transaction: {}", e);
        AppError::Database(e)
    })?;

    // Clear existing libraries
    let libraries_repo = LibrariesRepository::new(state.db.clone());
    if let Err(e) = libraries_repo.clear_all_tx(&mut tx).await {
        error!("Failed to clear libraries: {}", e);
        tx.rollback().await.map_err(|rollback_err| {
            error!("Failed to rollback transaction: {}", rollback_err);
            AppError::Database(rollback_err)
        })?;
        return Err(AppError::Database(e));
    }

    info!("Cleared existing libraries");

    // Fetch all runs data
    let runs_repo = RunsRepository::new(state.db.clone());
    let runs = runs_repo.find_all().await.map_err(|e| {
        error!("Failed to fetch runs data: {}", e);
        AppError::Database(e)
    })?;

    info!("Found {} runs to process", runs.len());

    let mut inserted_rows = 0;

    // Process each run
    for (index, run) in runs.iter().enumerate() {
        let run_id = run.id.ok_or_else(|| {
            error!("Run at index {} has no ID", index);
            AppError::BadRequest("Invalid run data".to_string())
        })?;

        let model_info = run.model_info.as_ref().ok_or_else(|| {
            error!("Run {} has no model_info", run_id);
            AppError::BadRequest("Missing model_info data".to_string())
        })?;

        let xformers = run.xformers.as_ref().ok_or_else(|| {
            error!("Run {} has no xformers", run_id);
            AppError::BadRequest("Missing xformers data".to_string())
        })?;

        info!("Processing libraries for run {} of {} (ID: {})", index + 1, runs.len(), run_id);

        // Parse model info to extract library versions
        let parsed_libraries = parse_model_info(model_info);

        // Store values for logging
        let torch_for_log = parsed_libraries.torch.clone();
        let xformers_for_log = parsed_libraries.xformers.clone();

        // Create libraries record
        let libraries_record = Libraries {
            id: None,
            run_id: Some(run_id),
            torch: parsed_libraries.torch,
            xformers: parsed_libraries.xformers,
            xformers1: Some(xformers.clone()), // Copy xformers value from runs table
            diffusers: parsed_libraries.diffusers,
            transformers: parsed_libraries.transformers,
        };

        // Insert into database
        match libraries_repo.create_tx(libraries_record, &mut tx).await {
            Ok(_) => {
                inserted_rows += 1;
                info!("Processed libraries for run {}: torch={:?}, xformers={:?}", 
                      index + 1, torch_for_log, xformers_for_log);
            }
            Err(e) => {
                error!("Failed to insert libraries for run {}: {}", run_id, e);
                // Continue processing other runs
            }
        }
    }

    // Commit transaction
    if let Err(e) = tx.commit().await {
        error!("Failed to commit transaction: {}", e);
        return Err(AppError::Database(e));
    }

    info!("Libraries processing complete: {} rows inserted", inserted_rows);

    let response = ProcessLibrariesResponse {
        success: true,
        rows_inserted: inserted_rows,
    };

    Ok(Json(response))
}

#[derive(Debug)]
struct ParsedGpuInfo {
    device: Option<String>,
    driver: Option<String>,
    gpu_chip: Option<String>,
}

fn parse_device_info(device_info_string: &str) -> ParsedGpuInfo {
    let parts: Vec<&str> = device_info_string.split(' ').collect();
    let mut parsed_gpu_info = ParsedGpuInfo {
        device: None,
        driver: None,
        gpu_chip: None,
    };

    let mut in_gpu_chip = false;
    let mut gpu_chip_parts = Vec::new();

    for part in parts {
        let colon_index = match part.find(':') {
            Some(index) => index,
            None => {
                // Handle non-colon parts
                if part.contains("GB") {
                    // Append GB value to the device if it's a memory size
                    if let Some(ref mut device) = parsed_gpu_info.device {
                        device.push(' ');
                        device.push_str(part);
                    }
                } else if in_gpu_chip {
                    gpu_chip_parts.push(part);
                } else if let Some(ref mut device) = parsed_gpu_info.device {
                    device.push(' ');
                    device.push_str(part);
                }
                continue;
            }
        };

        let key = &part[..colon_index];
        let value = &part[colon_index + 1..];

        match key {
            "device" => {
                parsed_gpu_info.device = Some(value.to_string());
            }
            "driver" => {
                parsed_gpu_info.driver = Some(value.to_string());
            }
            _ => {
                // Any other colon-separated part goes to gpu_chip
                in_gpu_chip = true;
                gpu_chip_parts.push(part);
            }
        }
    }

    parsed_gpu_info.gpu_chip = if gpu_chip_parts.is_empty() {
        None
    } else {
        Some(gpu_chip_parts.join(" "))
    };

    parsed_gpu_info
}

pub async fn process_gpu(
    State(state): State<AppState>,
) -> Result<Json<ProcessGpuResponse>, AppError> {
    info!("Processing GPU info from runs table");

    // Start transaction
    let mut tx = state.db.begin().await.map_err(|e| {
        error!("Failed to begin transaction: {}", e);
        AppError::Database(e)
    })?;

    // Clear existing GPU data
    let gpu_repo = GpuRepository::new(state.db.clone());
    if let Err(e) = gpu_repo.clear_all_tx(&mut tx).await {
        error!("Failed to clear GPU data: {}", e);
        tx.rollback().await.map_err(|rollback_err| {
            error!("Failed to rollback transaction: {}", rollback_err);
            AppError::Database(rollback_err)
        })?;
        return Err(AppError::Database(e));
    }

    info!("Cleared existing GPU data");

    // Fetch all runs data
    let runs_repo = RunsRepository::new(state.db.clone());
    let runs = runs_repo.find_all().await.map_err(|e| {
        error!("Failed to fetch runs data: {}", e);
        AppError::Database(e)
    })?;

    info!("Found {} runs to process", runs.len());

    let mut inserted_rows = 0;

    // Process each run
    for (index, run) in runs.iter().enumerate() {
        let run_id = run.id.ok_or_else(|| {
            error!("Run at index {} has no ID", index);
            AppError::BadRequest("Invalid run data".to_string())
        })?;

        let device_info = run.device_info.as_ref().ok_or_else(|| {
            error!("Run {} has no device_info", run_id);
            AppError::BadRequest("Missing device_info data".to_string())
        })?;

        info!("Processing GPU info for run {} of {} (ID: {})", index + 1, runs.len(), run_id);

        // Parse device info to extract GPU information
        let parsed_gpu_info = parse_device_info(device_info);

        // Store values for logging
        let device_for_log = parsed_gpu_info.device.clone();

        // Create GPU record
        let gpu_record = Gpu {
            id: None,
            run_id: Some(run_id),
            device: parsed_gpu_info.device,
            driver: parsed_gpu_info.driver,
            gpu_chip: parsed_gpu_info.gpu_chip,
            brand: None, // Will be populated by separate update process
            is_laptop: None, // Will be populated by separate update process
        };

        // Insert into database
        match gpu_repo.create_tx(gpu_record, &mut tx).await {
            Ok(_) => {
                inserted_rows += 1;
                info!("Processed GPU info for run {}: device={:?}", index + 1, device_for_log);
            }
            Err(e) => {
                error!("Failed to insert GPU info for run {}: {}", run_id, e);
                // Continue processing other runs
            }
        }
    }

    // Commit transaction
    if let Err(e) = tx.commit().await {
        error!("Failed to commit transaction: {}", e);
        return Err(AppError::Database(e));
    }

    info!("GPU processing complete: {} rows inserted", inserted_rows);

    let response = ProcessGpuResponse {
        success: true,
        rows_inserted: inserted_rows,
    };

    Ok(Json(response))
}

#[derive(Debug, Serialize)]
pub struct UpdateGpuBrandsResponse {
    pub status: bool,
    pub message: String,
    pub total_updates: usize,
    pub update_counts_by_brand: Vec<BrandCount>,
}

#[derive(Debug, Serialize)]
pub struct BrandCount {
    pub brand_name: String,
    pub count: usize,
}

fn get_brand_name(device_string: &str) -> String {
    let lowercase_device = device_string.to_lowercase();

    if lowercase_device.contains("nvidia") || 
       lowercase_device.contains("quadro") || 
       lowercase_device.contains("geforce") ||
       lowercase_device.contains("tesla") ||
       lowercase_device.contains("cuda") {
        "nvidia".to_string()
    } else if lowercase_device.contains("amd") || 
              lowercase_device.contains("radeon") {
        "amd".to_string()
    } else if lowercase_device.contains("intel") {
        "intel".to_string()
    } else {
        "unknown".to_string()
    }
}

pub async fn update_gpu_brands(
    State(state): State<AppState>,
) -> Result<Json<UpdateGpuBrandsResponse>, AppError> {
    info!("Updating GPU brand information");

    // Start transaction
    let mut tx = state.db.begin().await.map_err(|e| {
        error!("Failed to begin transaction: {}", e);
        AppError::Database(e)
    })?;

    // Fetch all GPU data
    let gpu_repo = GpuRepository::new(state.db.clone());
    let gpu_data = gpu_repo.find_all().await.map_err(|e| {
        error!("Failed to fetch GPU data: {}", e);
        AppError::Database(e)
    })?;

    if gpu_data.is_empty() {
        info!("No GPU data found to update");
        tx.commit().await.map_err(|e| {
            error!("Failed to commit transaction: {}", e);
            AppError::Database(e)
        })?;

        // Return all brand categories with 0 counts
        let update_counts_by_brand = vec![
            BrandCount {
                brand_name: "Nvidia".to_string(),
                count: 0,
            },
            BrandCount {
                brand_name: "Amd".to_string(),
                count: 0,
            },
            BrandCount {
                brand_name: "Intel".to_string(),
                count: 0,
            },
            BrandCount {
                brand_name: "Unknown".to_string(),
                count: 0,
            },
        ];

        let response = UpdateGpuBrandsResponse {
            status: true,
            message: "No GPU data found to update".to_string(),
            total_updates: 0,
            update_counts_by_brand,
        };

        return Ok(Json(response));
    }

    info!("Found {} GPUs to update", gpu_data.len());

    let mut total_updates = 0;
    let mut brand_counts = std::collections::HashMap::new();
    brand_counts.insert("nvidia".to_string(), 0);
    brand_counts.insert("amd".to_string(), 0);
    brand_counts.insert("intel".to_string(), 0);
    brand_counts.insert("unknown".to_string(), 0);

    // Process each GPU
    for gpu in &gpu_data {
        let gpu_id = gpu.id.ok_or_else(|| {
            error!("GPU has no ID");
            AppError::BadRequest("Invalid GPU data".to_string())
        })?;

        let device = gpu.device.as_ref().ok_or_else(|| {
            error!("GPU {} has no device", gpu_id);
            AppError::BadRequest("Missing device data".to_string())
        })?;

        let brand_name = get_brand_name(device);

        // Update the count
        total_updates += 1;
        *brand_counts.get_mut(&brand_name).unwrap() += 1;

        info!("Updating brand for GPU ID {} to {}", gpu_id, brand_name);

        // Update the GPU record
        let mut updated_gpu = gpu.clone();
        updated_gpu.brand = Some(brand_name);

        if let Err(e) = gpu_repo.update_tx(updated_gpu, &mut tx).await {
            error!("Failed to update GPU {}: {}", gpu_id, e);
            // Continue processing other GPUs
        }
    }

    // Commit transaction
    if let Err(e) = tx.commit().await {
        error!("Failed to commit transaction: {}", e);
        return Err(AppError::Database(e));
    }

    info!("GPU brand update complete: {} total updates", total_updates);

    // Convert brand counts to response format
    let update_counts_by_brand: Vec<BrandCount> = brand_counts
        .into_iter()
        .map(|(brand, count)| BrandCount {
            brand_name: brand.chars().next().unwrap().to_uppercase().collect::<String>() + &brand[1..],
            count,
        })
        .collect();

    let response = UpdateGpuBrandsResponse {
        status: true,
        message: "GPU brand information updated successfully!".to_string(),
        total_updates,
        update_counts_by_brand,
    };

    Ok(Json(response))
}

#[derive(Debug, Serialize)]
pub struct UpdateGpuLaptopInfoResponse {
    pub status: bool,
    pub message: String,
    pub total_updates: usize,
    pub laptop_only_updates: usize,
}

fn is_gpu_in_laptop(device_string: &str) -> bool {
    device_string.contains("Laptop") || device_string.contains("Mobile")
}

pub async fn update_gpu_laptop_info(
    State(state): State<AppState>,
) -> Result<Json<UpdateGpuLaptopInfoResponse>, AppError> {
    info!("Updating GPU laptop information");

    // Start transaction
    let mut tx = state.db.begin().await.map_err(|e| {
        error!("Failed to begin transaction: {}", e);
        AppError::Database(e)
    })?;

    // Fetch all GPU data
    let gpu_repo = GpuRepository::new(state.db.clone());
    let gpu_data = gpu_repo.find_all().await.map_err(|e| {
        error!("Failed to fetch GPU data: {}", e);
        AppError::Database(e)
    })?;

    if gpu_data.is_empty() {
        info!("No GPU data found to update");
        tx.commit().await.map_err(|e| {
            error!("Failed to commit transaction: {}", e);
            AppError::Database(e)
        })?;

        let response = UpdateGpuLaptopInfoResponse {
            status: true,
            message: "No GPU data found to update".to_string(),
            total_updates: 0,
            laptop_only_updates: 0,
        };

        return Ok(Json(response));
    }

    info!("Found {} GPUs to update", gpu_data.len());

    let mut total_updates = 0;
    let mut laptop_only_updates = 0;

    // Process each GPU
    for gpu in &gpu_data {
        let gpu_id = gpu.id.ok_or_else(|| {
            error!("GPU has no ID");
            AppError::BadRequest("Invalid GPU data".to_string())
        })?;

        let device = gpu.device.as_ref().ok_or_else(|| {
            error!("GPU {} has no device", gpu_id);
            AppError::BadRequest("Missing device data".to_string())
        })?;

        let is_laptop = is_gpu_in_laptop(device);

        // Update the count
        total_updates += 1;
        if is_laptop {
            laptop_only_updates += 1;
        }

        info!("Updating laptop info for GPU ID {} to {}", gpu_id, is_laptop);

        // Update the GPU record
        let mut updated_gpu = gpu.clone();
        updated_gpu.is_laptop = Some(is_laptop);

        if let Err(e) = gpu_repo.update_tx(updated_gpu, &mut tx).await {
            error!("Failed to update GPU {}: {}", gpu_id, e);
            // Continue processing other GPUs
        }
    }

    // Commit transaction
    if let Err(e) = tx.commit().await {
        error!("Failed to commit transaction: {}", e);
        return Err(AppError::Database(e));
    }

    info!("GPU laptop info update complete: {} total updates, {} laptop updates", 
          total_updates, laptop_only_updates);

    let response = UpdateGpuLaptopInfoResponse {
        status: true,
        message: "GPU laptop information updated successfully!".to_string(),
        total_updates,
        laptop_only_updates,
    };

    Ok(Json(response))
}

#[derive(Debug, Serialize)]
pub struct ProcessRunDetailsResponse {
    pub success: bool,
    pub total_inserts: usize,
}

pub async fn process_run_details(
    State(state): State<AppState>,
) -> Result<Json<ProcessRunDetailsResponse>, AppError> {
    info!("Processing run details");

    // Start transaction
    let mut tx = state.db.begin().await.map_err(|e| {
        error!("Failed to begin transaction: {}", e);
        AppError::Database(e)
    })?;

    // Clear all existing data from RunMoreDetails table
    let run_more_details_repo = RunMoreDetailsRepository::new(state.db.clone());
    run_more_details_repo.clear_all_tx(&mut tx).await.map_err(|e| {
        error!("Failed to clear RunMoreDetails table: {}", e);
        AppError::Database(e)
    })?;

    info!("Cleared existing RunMoreDetails data");

    // Fetch data from runs table
    let runs_repo = RunsRepository::new(state.db.clone());
    let runs_data = runs_repo.find_all().await.map_err(|e| {
        error!("Failed to fetch runs data: {}", e);
        AppError::Database(e)
    })?;

    info!("Found {} runs to process", runs_data.len());

    let mut insert_count = 0;

    // Process each run and insert into RunMoreDetails
    for run in &runs_data {
        let run_id = run.id.ok_or_else(|| {
            error!("Run has no ID");
            AppError::BadRequest("Invalid run data".to_string())
        })?;

        info!("Processing run details for run ID: {}", run_id);

        let run_more_details = RunMoreDetails {
            id: None,
            run_id: Some(run_id),
            timestamp: run.timestamp.clone(),
            model_name: run.model_name.clone(),
            user: run.user.clone(),
            notes: run.notes.clone(),
            model_map_id: None,
        };

        if let Err(e) = run_more_details_repo.create_tx(run_more_details, &mut tx).await {
            error!("Failed to insert run details for run {}: {}", run_id, e);
            // Continue processing other runs
        } else {
            insert_count += 1;
        }
    }

    // Commit transaction
    if let Err(e) = tx.commit().await {
        error!("Failed to commit transaction: {}", e);
        return Err(AppError::Database(e));
    }

    info!("Run details processing complete: {} total inserts", insert_count);

    let response = ProcessRunDetailsResponse {
        success: true,
        total_inserts: insert_count,
    };

    Ok(Json(response))
}

#[derive(Debug, Serialize)]
pub struct AppDetailsAnalysisResponse {
    pub total_rows: i64,
    pub null_app_name_null_url: i64,
    pub null_app_name_non_null_url: i64,
}

pub async fn app_details_analysis(
    State(state): State<AppState>,
) -> Result<Json<crate::handlers::common::ApiResponse<AppDetailsAnalysisResponse>>, AppError> {
    info!("Analyzing app details");

    let result = sqlx::query!(
        r#"
        SELECT
            COALESCE(COUNT(*), 0) AS total_rows,
            COALESCE(SUM(CASE WHEN app_name IS NULL AND url IS NULL THEN 1 ELSE 0 END), 0) AS null_app_name_null_url,
            COALESCE(SUM(CASE WHEN app_name IS NULL AND url IS NOT NULL THEN 1 ELSE 0 END), 0) AS null_app_name_non_null_url
        FROM AppDetails
        "#
    )
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        error!("Failed to analyze app details: {}", e);
        AppError::Database(e)
    })?;

    let response = AppDetailsAnalysisResponse {
        total_rows: result.total_rows,
        null_app_name_null_url: result.null_app_name_null_url,
        null_app_name_non_null_url: result.null_app_name_non_null_url,
    };

    info!("App details analysis complete: {} total rows, {} null app_name null url, {} null app_name non-null url", 
          response.total_rows, response.null_app_name_null_url, response.null_app_name_non_null_url);

    Ok(crate::handlers::common::create_success_response(
        response,
        "App details analysis completed successfully",
        axum::http::StatusCode::OK,
    ))
}

#[derive(Debug, Serialize)]
pub struct FixAppNamesResponse {
    pub message: String,
    pub updated_counts: UpdatedCounts,
}

#[derive(Debug, Serialize)]
pub struct UpdatedCounts {
    pub automatic1111: i64,
    pub vladmandic: i64,
    pub stable_diffusion: i64,
    pub null_app_name_null_url: i64,
}

// FixAppNamesRequest is now imported from validation module

pub async fn fix_app_names(
    State(state): State<AppState>,
    Json(request): Json<FixAppNamesRequest>,
) -> Result<Json<crate::handlers::common::ApiResponse<FixAppNamesResponse>>, AppError> {
    info!("Fixing app names with parameters: automatic1111={}, vladmandic={}, stable_diffusion={}, null_app_name_null_url={}", 
          request.automatic1111, request.vladmandic, request.stable_diffusion, request.null_app_name_null_url);

    // Basic validation for request fields
    if request.automatic1111.is_empty() || request.vladmandic.is_empty() || 
       request.stable_diffusion.is_empty() || request.null_app_name_null_url.is_empty() {
        return Err(AppError::Validation("All fields must be non-empty".to_string()));
    }

    // Start a transaction
    let mut tx = state.db.begin().await.map_err(|e| {
        error!("Failed to begin transaction: {}", e);
        AppError::Database(e)
    })?;

    // Update AUTOMATIC1111 app names
    let count_automatic1111 = sqlx::query!(
        r#"
        UPDATE AppDetails
        SET app_name = ?
        WHERE url LIKE '%AUTOMATIC1111%'
        "#,
        request.automatic1111
    )
    .execute(&mut *tx)
    .await
    .map_err(|e| {
        error!("Failed to update AUTOMATIC1111 app names: {}", e);
        AppError::Database(e)
    })?
    .rows_affected();

    info!("Updated {} AUTOMATIC1111 app names", count_automatic1111);

    // Update Vladmandic app names
    let count_vladmandic = sqlx::query!(
        r#"
        UPDATE AppDetails
        SET app_name = ?
        WHERE url LIKE '%vladmandic%' AND (app_name IS NULL OR app_name = '')
        "#,
        request.vladmandic
    )
    .execute(&mut *tx)
    .await
    .map_err(|e| {
        error!("Failed to update Vladmandic app names: {}", e);
        AppError::Database(e)
    })?
    .rows_affected();

    info!("Updated {} Vladmandic app names", count_vladmandic);

    // Update Stable Diffusion app names
    let count_stable_diffusion = sqlx::query!(
        r#"
        UPDATE AppDetails
        SET app_name = ?
        WHERE url LIKE '%stable-diffusion-webui%' AND app_name IS NULL
        "#,
        request.stable_diffusion
    )
    .execute(&mut *tx)
    .await
    .map_err(|e| {
        error!("Failed to update Stable Diffusion app names: {}", e);
        AppError::Database(e)
    })?
    .rows_affected();

    info!("Updated {} Stable Diffusion app names", count_stable_diffusion);

    // Update NULL app_name and NULL url records
    let count_null_app_name_null_url = sqlx::query!(
        r#"
        UPDATE AppDetails
        SET app_name = ?
        WHERE app_name IS NULL AND url IS NULL
        "#,
        request.null_app_name_null_url
    )
    .execute(&mut *tx)
    .await
    .map_err(|e| {
        error!("Failed to update NULL app_name NULL url records: {}", e);
        AppError::Database(e)
    })?
    .rows_affected();

    info!("Updated {} NULL app_name NULL url records", count_null_app_name_null_url);

    // Commit transaction
    tx.commit().await.map_err(|e| {
        error!("Failed to commit transaction: {}", e);
        AppError::Database(e)
    })?;

    let response = FixAppNamesResponse {
        message: "App names updated successfully".to_string(),
        updated_counts: UpdatedCounts {
            automatic1111: count_automatic1111 as i64,
            vladmandic: count_vladmandic as i64,
            stable_diffusion: count_stable_diffusion as i64,
            null_app_name_null_url: count_null_app_name_null_url as i64,
        },
    };

    info!("App names fix complete: AUTOMATIC1111={}, Vladmandic={}, StableDiffusion={}, NullAppNameNullUrl={}", 
          count_automatic1111, count_vladmandic, count_stable_diffusion, count_null_app_name_null_url);

    Ok(crate::handlers::common::create_success_response(
        response,
        "App names updated successfully",
        axum::http::StatusCode::OK,
    ))
}

#[derive(Debug, Serialize)]
pub struct UpdateRunMoreDetailsWithModelMapIdResponse {
    pub success: bool,
    pub message: String,
}

pub async fn update_run_more_details_with_modelmapid(
    State(state): State<AppState>,
) -> Result<Json<UpdateRunMoreDetailsWithModelMapIdResponse>, AppError> {
    info!("Updating RunMoreDetails with ModelMapId");

    // Start a transaction
    let mut tx = state.db.begin().await.map_err(|e| {
        error!("Failed to begin transaction: {}", e);
        AppError::Database(e)
    })?;

    // Get all runs from RunMoreDetails that don't have ModelMapId filled
    let runs_without_modelmapid = sqlx::query!(
        r#"
        SELECT id, model_name FROM RunMoreDetails WHERE ModelMapId IS NULL
        "#
    )
    .fetch_all(&mut *tx)
    .await
    .map_err(|e| {
        error!("Failed to fetch RunMoreDetails without ModelMapId: {}", e);
        AppError::Database(e)
    })?;

    if runs_without_modelmapid.is_empty() {
        info!("All RunMoreDetails entries already have ModelMapId");
        tx.commit().await.map_err(|e| {
            error!("Failed to commit transaction: {}", e);
            AppError::Database(e)
        })?;

        let response = UpdateRunMoreDetailsWithModelMapIdResponse {
            success: true,
            message: "All RunMoreDetails entries already have ModelMapId.".to_string(),
        };

        return Ok(Json(response));
    }

    info!("Found {} RunMoreDetails entries without ModelMapId", runs_without_modelmapid.len());

    let mut updated_count = 0;
    let mut not_found_count = 0;

    // For each run, find the corresponding ModelMapId from ModelMap based on model_name
    for run in &runs_without_modelmapid {
        let model_name = match &run.model_name {
            Some(name) => name,
            None => {
                info!("RunMoreDetails ID {} has NULL model_name, skipping", run.id);
                not_found_count += 1;
                continue;
            }
        };

        let model_map_entry = sqlx::query!(
            r#"
            SELECT id FROM ModelMap WHERE model_name = ?
            "#,
            model_name
        )
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| {
            error!("Failed to query ModelMap for model_name '{}': {}", model_name, e);
            AppError::Database(e)
        })?;

        if let Some(model_map_entry) = model_map_entry {
            // Update RunMoreDetails with the found ModelMapId
            sqlx::query!(
                r#"
                UPDATE RunMoreDetails SET ModelMapId = ? WHERE id = ?
                "#,
                model_map_entry.id,
                run.id
            )
            .execute(&mut *tx)
            .await
            .map_err(|e| {
                error!("Failed to update RunMoreDetails ID {} with ModelMapId {}: {}", 
                       run.id, model_map_entry.id, e);
                AppError::Database(e)
            })?;

            updated_count += 1;
            info!("Updated RunMoreDetails ID {} with ModelMapId {} for model_name '{}'", 
                  run.id, model_map_entry.id, model_name);
        } else {
            info!("No matching entry in ModelMap for model_name: {}", model_name);
            not_found_count += 1;
        }
    }

    // Commit transaction
    tx.commit().await.map_err(|e| {
        error!("Failed to commit transaction: {}", e);
        AppError::Database(e)
    })?;

    let response = UpdateRunMoreDetailsWithModelMapIdResponse {
        success: true,
        message: format!("RunMoreDetails updated with ModelMapId successfully. Updated: {}, Not found: {}", 
                        updated_count, not_found_count),
    };

    info!("RunMoreDetails update complete: {} updated, {} not found", updated_count, not_found_count);

    Ok(Json(response))
}