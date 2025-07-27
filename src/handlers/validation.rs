use serde::{Deserialize, Serialize};
use validator::ValidationError;

// ============================================================================
// File Upload Validation
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct FileUploadRequest {
    pub file_content: Vec<u8>,
    pub file_name: String,
}

// ============================================================================
// Fix App Names Validation
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct FixAppNamesRequest {
    pub automatic1111: String,
    pub vladmandic: String,
    pub stable_diffusion: String,
    pub null_app_name_null_url: String,
}

// ============================================================================
// Data Processing Validation
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct RunData {
    pub timestamp: String,
    pub vram_usage: String,
    pub info: String,
    pub system_info: String,
    pub model_info: String,
    pub device_info: String,
    pub xformers: String,
    pub model_name: String,
    pub user: String,
    pub notes: String,
}

// ============================================================================
// Query Parameter Validation
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct PaginationQuery {
    pub page: Option<i32>,
    pub limit: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FilterQuery {
    pub model_name: Option<String>,
    pub user: Option<String>,
    pub gpu: Option<String>,
}

// ============================================================================
// Custom Validation Functions
// ============================================================================

pub fn validate_json_content(content: &[u8]) -> Result<(), ValidationError> {
    if content.is_empty() {
        return Err(ValidationError::new("empty_content"));
    }
    
    // Try to parse as JSON to validate structure
    if let Err(_) = serde_json::from_slice::<serde_json::Value>(content) {
        return Err(ValidationError::new("invalid_json"));
    }
    
    Ok(())
}

pub fn validate_timestamp_format(timestamp: &str) -> Result<(), ValidationError> {
    // Basic timestamp validation - can be enhanced based on your specific format
    if timestamp.is_empty() {
        return Err(ValidationError::new("empty_timestamp"));
    }
    
    // Check if it contains at least one digit
    if !timestamp.chars().any(|c| c.is_ascii_digit()) {
        return Err(ValidationError::new("invalid_timestamp_format"));
    }
    
    Ok(())
}

pub fn validate_vram_usage_format(vram_usage: &str) -> Result<(), ValidationError> {
    if vram_usage.is_empty() {
        return Err(ValidationError::new("empty_vram_usage"));
    }
    
    // Check if it contains valid VRAM usage patterns (e.g., "8GB", "16GB", etc.)
    let valid_patterns = ["GB", "MB", "KB", "B"];
    let has_valid_unit = valid_patterns.iter().any(|unit| vram_usage.contains(unit));
    
    if !has_valid_unit && !vram_usage.chars().any(|c| c.is_ascii_digit()) {
        return Err(ValidationError::new("invalid_vram_format"));
    }
    
    Ok(())
}

// ============================================================================
// Validation Helpers
// ============================================================================

pub fn validate_file_size(size: usize, max_size: usize) -> Result<(), ValidationError> {
    if size > max_size {
        return Err(ValidationError::new("file_too_large"));
    }
    if size == 0 {
        return Err(ValidationError::new("file_empty"));
    }
    Ok(())
}

pub fn validate_file_extension(filename: &str, allowed_extensions: &[&str]) -> Result<(), ValidationError> {
    if let Some(extension) = filename.split('.').last() {
        if !allowed_extensions.contains(&extension.to_lowercase().as_str()) {
            return Err(ValidationError::new("invalid_file_extension"));
        }
    } else {
        return Err(ValidationError::new("no_file_extension"));
    }
    Ok(())
}

// ============================================================================
// Validation Constants
// ============================================================================

pub const MAX_FILE_SIZE: usize = 50 * 1024 * 1024; // 50MB
pub const ALLOWED_FILE_EXTENSIONS: &[&str] = &["json"];

// ============================================================================
// Validation Error Messages
// ============================================================================

pub fn get_validation_error_message(field: &str, error_type: &str) -> String {
    match (field, error_type) {
        ("file_content", "file_too_large") => "File size exceeds maximum allowed size of 50MB".to_string(),
        ("file_content", "file_empty") => "Uploaded file is empty".to_string(),
        ("file_content", "invalid_json") => "Uploaded file is not valid JSON".to_string(),
        ("file_name", "invalid_file_extension") => "Only JSON files are allowed".to_string(),
        ("file_name", "no_file_extension") => "File must have an extension".to_string(),
        ("timestamp", "empty_timestamp") => "Timestamp cannot be empty".to_string(),
        ("timestamp", "invalid_timestamp_format") => "Invalid timestamp format".to_string(),
        ("vram_usage", "empty_vram_usage") => "VRAM usage cannot be empty".to_string(),
        ("vram_usage", "invalid_vram_format") => "Invalid VRAM usage format".to_string(),
        _ => format!("Validation error for field '{}': {}", field, error_type),
    }
} 