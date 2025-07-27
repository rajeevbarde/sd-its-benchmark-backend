use axum::{
    extract::State,
    response::IntoResponse,
};
use axum_extra::extract::Multipart;
use serde_json::Value;
use tempfile::NamedTempFile;
use tokio::fs;
use tracing::{error, info, warn};

use crate::{
    config::Settings,
    handlers::common::{
        create_error_response, create_file_upload_response, validate_content_type, validate_file_size,
        validate_json_content, FileUploadResponse,
    },
    AppState,
};
use crate::error::AppError;

/// File upload handler for processing multipart form data
pub async fn upload_file(
    State(config): State<Settings>,
    mut multipart: Multipart,
) -> Result<axum::response::Response, AppError> {
    let mut uploaded_files = Vec::new();
    let mut errors = Vec::new();

    // Process each field in the multipart form
    while let Some(field) = multipart.next_field().await.map_err(|e| {
        error!("Failed to read multipart field: {}", e);
        AppError::FileUpload("Failed to read multipart form data".to_string())
    })? {
        let field_name = field.name().unwrap_or("unknown").to_string();
        let filename = field.file_name().unwrap_or("unknown").to_string();
        let content_type = field.content_type().unwrap_or("application/octet-stream").to_string();

        info!("Processing file upload: {} ({}), type: {}", filename, field_name, content_type);

        // Validate content type
        if let Err(e) = validate_content_type(&content_type) {
            errors.push(format!("File '{}': {}", filename, e));
            continue;
        }

        // Read file content
        let file_data = field.bytes().await.map_err(|e| {
            error!("Failed to read file data for {}: {}", filename, e);
            AppError::FileUpload(format!("Failed to read file data: {}", e))
        })?;

        let file_size = file_data.len();

        // Validate file size
        if let Err(e) = validate_file_size(file_size, config.file_upload.max_size_mb * 1024 * 1024) {
            errors.push(format!("File '{}': {}", filename, e));
            continue;
        }

        // Validate JSON content
        if let Err(e) = validate_json_content(&file_data) {
            errors.push(format!("File '{}': {}", filename, e));
            continue;
        }

        // Convert bytes to string for processing
        let content = String::from_utf8(file_data.to_vec()).map_err(|e| {
            error!("Invalid UTF-8 in file {}: {}", filename, e);
            AppError::FileUpload(format!("Invalid UTF-8 encoding: {}", e))
        })?;

        // Parse JSON data
        let json_data: Value = serde_json::from_str(&content).map_err(|e| {
            error!("Invalid JSON in file {}: {}", filename, e);
            AppError::FileUpload(format!("Invalid JSON format: {}", e))
        })?;

        // Save to temporary file
        let temp_file = match save_to_temp_file(&content, &filename).await {
            Ok(temp_file) => temp_file,
            Err(e) => {
                errors.push(format!("File '{}': Failed to save temporary file: {}", filename, e));
                continue;
            }
        };

        // Create upload response using the new format
        let upload_response = create_file_upload_response(
            "File processed successfully",
            &filename,
            file_size,
            1, // rows_processed
            1, // rows_inserted
            0, // rows_failed
            axum::http::StatusCode::OK,
        );

        uploaded_files.push((upload_response, json_data, temp_file));

        info!("Successfully processed file: {} ({} bytes)", filename, file_size);
    }

    // If there were errors but also successful uploads, return partial success
    if !errors.is_empty() && !uploaded_files.is_empty() {
        warn!("File upload completed with {} errors", errors.len());
        return Ok(create_error_response(
            "FILE_UPLOAD_ERROR",
            "Some files failed to upload",
            axum::http::StatusCode::BAD_REQUEST,
            None,
        ).into_response());
    }

    // If there were only errors, return error response
    if !errors.is_empty() {
        error!("File upload failed with {} errors", errors.len());
        return Ok(create_error_response(
            "FILE_UPLOAD_ERROR",
            "All files failed to upload",
            axum::http::StatusCode::BAD_REQUEST,
            None,
        ).into_response());
    }

    // If no files were uploaded, return error
    if uploaded_files.is_empty() {
        return Ok(create_error_response(
            "NO_FILES_UPLOADED",
            "No files were uploaded",
            axum::http::StatusCode::BAD_REQUEST,
            None,
        ).into_response());
    }

    // Return success response with the first uploaded file
    let (response, _, _) = uploaded_files.remove(0);
    Ok(response.into_response())
}

/// Compatible file upload handler that works with AppState
pub async fn upload_file_compat(
    State(app_state): State<AppState>,
    multipart: Multipart,
) -> Result<impl IntoResponse, AppError> {
    upload_file(State(app_state.settings), multipart).await
}

/// Save content to a temporary file
async fn save_to_temp_file(content: &str, _filename: &str) -> Result<NamedTempFile, std::io::Error> {
    let temp_file = NamedTempFile::new()?;
    fs::write(&temp_file, content).await?;
    Ok(temp_file)
}

/// Clean up temporary files
pub async fn cleanup_temp_files(temp_files: Vec<NamedTempFile>) {
    for temp_file in temp_files {
        if let Err(e) = fs::remove_file(temp_file.path()).await {
            warn!("Failed to remove temporary file: {}", e);
        }
    }
}

/// Extract JSON data from uploaded files
pub fn extract_json_data(uploaded_files: &[(axum::response::Json<FileUploadResponse>, Value, NamedTempFile)]) -> Vec<Value> {
    uploaded_files
        .iter()
        .map(|(_, json_data, _)| json_data.clone())
        .collect()
}

/// Validate file structure
pub fn validate_file_structure(json_data: &Value) -> bool {
    match json_data {
        Value::Array(arr) => {
            if arr.is_empty() {
                warn!("File contains empty array");
                return false;
            }
            // Check if all elements are objects
            arr.iter().all(|item| item.is_object())
        }
        Value::Object(_) => {
            warn!("File contains single object, expected array");
            false
        }
        _ => {
            warn!("File contains invalid JSON structure");
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_validate_file_structure_object() {
        let data = json!({"key": "value"});
        assert!(!validate_file_structure(&data));
    }

    #[test]
    fn test_validate_file_structure_array() {
        let data = json!([{"key": "value"}, {"key2": "value2"}]);
        assert!(validate_file_structure(&data));
    }

    #[test]
    fn test_validate_file_structure_empty_object() {
        let data = json!({});
        assert!(!validate_file_structure(&data));
    }

    #[test]
    fn test_validate_file_structure_empty_array() {
        let data = json!([]);
        assert!(!validate_file_structure(&data));
    }

    #[test]
    fn test_extract_json_data() {
        let temp_file = NamedTempFile::new().unwrap();
        let response = create_file_upload_response(
            "test",
            "test.json",
            100,
            1,
            1,
            0,
            axum::http::StatusCode::OK,
        );
        let json_data = json!({"test": "data"});
        
        let uploaded_files = vec![(response, json_data.clone(), temp_file)];
        let extracted = extract_json_data(&uploaded_files);
        
        assert_eq!(extracted.len(), 1);
        assert_eq!(extracted[0], json_data);
    }

    #[tokio::test]
    async fn test_save_to_temp_file() {
        let content = r#"{"test": "data"}"#;
        let filename = "test.json";
        
        let temp_file = save_to_temp_file(content, filename).await.unwrap();
        let saved_content = fs::read_to_string(temp_file.path()).await.unwrap();
        
        assert_eq!(saved_content, content);
    }
} 