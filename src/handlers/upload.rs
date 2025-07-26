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
        unified_error_response, unified_success_response, validate_content_type, validate_file_size,
        validate_json_content, FileUploadResponse, FileValidationResult,
    },
};
use crate::error::AppError;

/// File upload handler for processing multipart form data
pub async fn upload_file(
    State(config): State<Settings>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, AppError> {
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
        if let Err(e) = validate_content_type(&content_type, &config.file_upload.allowed_content_types) {
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
        if let Err(e) = validate_file_size(file_size, config.file_upload.max_size_mb) {
            errors.push(format!("File '{}': {}", filename, e));
            continue;
        }

        // Convert bytes to string for JSON validation
        let content = String::from_utf8(file_data.to_vec()).map_err(|e| {
            error!("Invalid UTF-8 in file {}: {}", filename, e);
            AppError::FileUpload(format!("Invalid UTF-8 encoding: {}", e))
        })?;

        // Validate JSON content
        let json_data = match validate_json_content(&content) {
            Ok(data) => data,
            Err(e) => {
                errors.push(format!("File '{}': {}", filename, e));
                continue;
            }
        };

        // Save to temporary file
        let temp_file = match save_to_temp_file(&content, &filename).await {
            Ok(temp_file) => temp_file,
            Err(e) => {
                errors.push(format!("File '{}': Failed to save temporary file: {}", filename, e));
                continue;
            }
        };

        // Create upload response
        let upload_response = FileUploadResponse {
            filename: filename.clone(),
            size: file_size,
            content_type,
            processed: true,
        };

        uploaded_files.push((upload_response, json_data, temp_file));

        info!("Successfully processed file: {} ({} bytes)", filename, file_size);
    }

    // If there were errors but also successful uploads, return partial success
    if !errors.is_empty() && !uploaded_files.is_empty() {
        warn!("File upload completed with {} errors", errors.len());
        return Ok(unified_error_response("Some files failed to upload", errors));
    }

    // If all files failed, return error
    if !errors.is_empty() {
        return Ok(unified_error_response("All files failed to upload", errors));
    }

    // All files processed successfully
    let responses: Vec<FileUploadResponse> = uploaded_files
        .iter()
        .map(|(response, _, _)| response.clone())
        .collect();

    Ok(unified_success_response(responses, "Files uploaded successfully"))
}

/// Save content to a temporary file
async fn save_to_temp_file(content: &str, filename: &str) -> Result<NamedTempFile, std::io::Error> {
    let temp_file = NamedTempFile::new()?;
    fs::write(&temp_file, content).await?;
    
    info!("Saved temporary file: {} -> {:?}", filename, temp_file.path());
    Ok(temp_file)
}

/// Clean up temporary files
pub async fn cleanup_temp_files(temp_files: Vec<NamedTempFile>) {
    for temp_file in temp_files {
        if let Err(e) = temp_file.close() {
            error!("Failed to close temporary file: {}", e);
        }
    }
}

/// Extract JSON data from uploaded files
pub fn extract_json_data(uploaded_files: &[(FileUploadResponse, Value, NamedTempFile)]) -> Vec<Value> {
    uploaded_files
        .iter()
        .map(|(_, json_data, _)| json_data.clone())
        .collect()
}

/// Validate uploaded file structure
pub fn validate_file_structure(json_data: &Value) -> FileValidationResult {
    let mut errors = Vec::new();
    
    // Basic JSON structure validation
    if !json_data.is_object() && !json_data.is_array() {
        errors.push("JSON must be an object or array".to_string());
    }

    // Check for required fields if it's an object
    if let Some(obj) = json_data.as_object() {
        if obj.is_empty() {
            errors.push("JSON object cannot be empty".to_string());
        }
    }

    // Check for required fields if it's an array
    if let Some(arr) = json_data.as_array() {
        if arr.is_empty() {
            errors.push("JSON array cannot be empty".to_string());
        }
    }

    FileValidationResult {
        is_valid: errors.is_empty(),
        errors,
        file_size: 0, // Will be set by caller
        content_type: "application/json".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_validate_file_structure_object() {
        let valid_json = json!({"key": "value"});
        let result = validate_file_structure(&valid_json);
        assert!(result.is_valid);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_validate_file_structure_array() {
        let valid_json = json!([{"key": "value"}]);
        let result = validate_file_structure(&valid_json);
        assert!(result.is_valid);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_validate_file_structure_empty_object() {
        let empty_json = json!({});
        let result = validate_file_structure(&empty_json);
        assert!(!result.is_valid);
        assert!(result.errors.contains(&"JSON object cannot be empty".to_string()));
    }

    #[test]
    fn test_validate_file_structure_empty_array() {
        let empty_json = json!([]);
        let result = validate_file_structure(&empty_json);
        assert!(!result.is_valid);
        assert!(result.errors.contains(&"JSON array cannot be empty".to_string()));
    }

    #[test]
    fn test_extract_json_data() {
        let test_response = FileUploadResponse {
            filename: "test.json".to_string(),
            size: 100,
            content_type: "application/json".to_string(),
            processed: true,
        };
        
        let test_json = json!({"test": "data"});
        let temp_file = NamedTempFile::new().unwrap();
        
        let uploaded_files = vec![(test_response, test_json.clone(), temp_file)];
        let extracted_data = extract_json_data(&uploaded_files);
        
        assert_eq!(extracted_data.len(), 1);
        assert_eq!(extracted_data[0], test_json);
    }

    #[test]
    fn test_save_to_temp_file() {
        let content = r#"{"test": "data"}"#;
        let filename = "test.json";
        
        let result = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(save_to_temp_file(content, filename));
        
        assert!(result.is_ok());
        let temp_file = result.unwrap();
        assert!(temp_file.path().exists());
        
        // Clean up
        let _ = temp_file.close();
    }
} 