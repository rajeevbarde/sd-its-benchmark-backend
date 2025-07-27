use axum::{
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use time::OffsetDateTime;

// ============================================================================
// Standardized Response Structures
// ============================================================================

/// Standard success response wrapper
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub message: String,
    pub data: Option<T>,
    pub timestamp: String,
    pub status_code: u16,
}

/// Standard error response wrapper
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiErrorResponse {
    pub success: bool,
    pub error: String,
    pub message: String,
    pub timestamp: String,
    pub status_code: u16,
    pub details: Option<HashMap<String, String>>,
}

/// Pagination metadata for list responses
#[derive(Debug, Serialize, Deserialize)]
pub struct PaginationMeta {
    pub page: i32,
    pub limit: i32,
    pub total: i64,
    pub total_pages: i64,
    pub has_next: bool,
    pub has_prev: bool,
}

/// Standard list response with pagination
#[derive(Debug, Serialize, Deserialize)]
pub struct ListResponse<T> {
    pub success: bool,
    pub message: String,
    pub data: Vec<T>,
    pub pagination: Option<PaginationMeta>,
    pub timestamp: String,
    pub status_code: u16,
}

/// Processing operation response
#[derive(Debug, Serialize, Deserialize)]
pub struct ProcessingResponse {
    pub success: bool,
    pub message: String,
    pub rows_processed: usize,
    pub rows_inserted: usize,
    pub rows_updated: usize,
    pub rows_deleted: usize,
    pub errors: Vec<String>,
    pub timestamp: String,
    pub status_code: u16,
}

/// File upload response
#[derive(Debug, Serialize, Deserialize)]
pub struct FileUploadResponse {
    pub success: bool,
    pub message: String,
    pub file_name: String,
    pub file_size: usize,
    pub rows_processed: usize,
    pub rows_inserted: usize,
    pub rows_failed: usize,
    pub timestamp: String,
    pub status_code: u16,
}

// ============================================================================
// Response Builder Functions
// ============================================================================

/// Create a standardized success response
pub fn create_success_response<T: Serialize>(
    data: T,
    message: &str,
    status_code: StatusCode,
) -> Json<ApiResponse<T>> {
    Json(ApiResponse {
        success: true,
        message: message.to_string(),
        data: Some(data),
        timestamp: OffsetDateTime::now_utc().to_string(),
        status_code: status_code.as_u16(),
    })
}

/// Create a standardized success response without data
pub fn create_success_message(
    message: &str,
    status_code: StatusCode,
) -> Json<ApiResponse<()>> {
    Json(ApiResponse {
        success: true,
        message: message.to_string(),
        data: None,
        timestamp: OffsetDateTime::now_utc().to_string(),
        status_code: status_code.as_u16(),
    })
}

/// Create a standardized error response
pub fn create_error_response(
    error: &str,
    message: &str,
    status_code: StatusCode,
    details: Option<HashMap<String, String>>,
) -> Json<ApiErrorResponse> {
    Json(ApiErrorResponse {
        success: false,
        error: error.to_string(),
        message: message.to_string(),
        timestamp: OffsetDateTime::now_utc().to_string(),
        status_code: status_code.as_u16(),
        details,
    })
}

/// Create a standardized list response
pub fn create_list_response<T: Serialize>(
    data: Vec<T>,
    message: &str,
    status_code: StatusCode,
    pagination: Option<PaginationMeta>,
) -> Json<ListResponse<T>> {
    Json(ListResponse {
        success: true,
        message: message.to_string(),
        data,
        pagination,
        timestamp: OffsetDateTime::now_utc().to_string(),
        status_code: status_code.as_u16(),
    })
}

/// Create a processing operation response
pub fn create_processing_response(
    message: &str,
    rows_processed: usize,
    rows_inserted: usize,
    rows_updated: usize,
    rows_deleted: usize,
    errors: Vec<String>,
    status_code: StatusCode,
) -> Json<ProcessingResponse> {
    Json(ProcessingResponse {
        success: errors.is_empty(),
        message: message.to_string(),
        rows_processed,
        rows_inserted,
        rows_updated,
        rows_deleted,
        errors,
        timestamp: OffsetDateTime::now_utc().to_string(),
        status_code: status_code.as_u16(),
    })
}

/// Create a file upload response
pub fn create_file_upload_response(
    message: &str,
    file_name: &str,
    file_size: usize,
    rows_processed: usize,
    rows_inserted: usize,
    rows_failed: usize,
    status_code: StatusCode,
) -> Json<FileUploadResponse> {
    Json(FileUploadResponse {
        success: rows_failed == 0,
        message: message.to_string(),
        file_name: file_name.to_string(),
        file_size,
        rows_processed,
        rows_inserted,
        rows_failed,
        timestamp: OffsetDateTime::now_utc().to_string(),
        status_code: status_code.as_u16(),
    })
}

// ============================================================================
// Pagination Helper Functions
// ============================================================================

/// Calculate pagination metadata
pub fn calculate_pagination_meta(
    page: i32,
    limit: i32,
    total: i64,
) -> PaginationMeta {
    let total_pages = if total == 0 {
        0
    } else {
        ((total as f64) / (limit as f64)).ceil() as i64
    };

    PaginationMeta {
        page,
        limit,
        total,
        total_pages,
        has_next: i64::from(page) < total_pages,
        has_prev: page > 1,
    }
}

// ============================================================================
// Legacy Response Compatibility
// ============================================================================

/// Create a legacy-compatible success response (for backward compatibility)
pub fn create_legacy_success_response<T: Serialize>(data: T) -> Json<T> {
    Json(data)
}

/// Create a legacy-compatible error response (for backward compatibility)
pub fn create_legacy_error_response(error: &str) -> Json<serde_json::Value> {
    Json(json!({
        "error": error
    }))
}

// ============================================================================
// Response Conversion Traits
// ============================================================================

/// Trait for converting internal response types to standardized API responses
pub trait ToApiResponse {
    type Output;
    
    fn to_api_response(self, message: &str, status_code: StatusCode) -> Json<ApiResponse<Self::Output>>;
}

/// Trait for converting internal error types to standardized API error responses
pub trait ToApiErrorResponse {
    fn to_api_error_response(self, status_code: StatusCode) -> Json<ApiErrorResponse>;
}

// ============================================================================
// Content Type Validation
// ============================================================================

/// Validate content type for multipart requests
pub fn validate_content_type(content_type: &str) -> Result<(), String> {
    if content_type.starts_with("multipart/form-data") {
        Ok(())
    } else {
        Err("Invalid content type. Expected multipart/form-data".to_string())
    }
}

/// Validate content type for JSON requests
pub fn validate_json_content_type(content_type: &str) -> Result<(), String> {
    if content_type == "application/json" {
        Ok(())
    } else {
        Err("Invalid content type. Expected application/json".to_string())
    }
}

// ============================================================================
// File Validation Helpers
// ============================================================================

/// Validate file size
pub fn validate_file_size(size: usize, max_size: usize) -> Result<(), String> {
    if size > max_size {
        Err(format!("File size exceeds maximum allowed size of {} bytes", max_size))
    } else if size == 0 {
        Err("Uploaded file is empty".to_string())
    } else {
        Ok(())
    }
}

/// Validate JSON content
pub fn validate_json_content(content: &[u8]) -> Result<(), String> {
    if content.is_empty() {
        return Err("Uploaded file is empty".to_string());
    }
    
    if let Err(_) = serde_json::from_slice::<serde_json::Value>(content) {
        return Err("Uploaded file is not valid JSON".to_string());
    }
    
    Ok(())
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;

    #[test]
    fn test_create_success_response() {
        let data = json!({"key": "value"});
        let response = create_success_response(data, "Success", StatusCode::OK);
        
        assert_eq!(response.success, true);
        assert_eq!(response.message, "Success");
        assert_eq!(response.status_code, 200);
        assert!(response.data.is_some());
    }

    #[test]
    fn test_create_error_response() {
        let response = create_error_response(
            "VALIDATION_ERROR",
            "Invalid input",
            StatusCode::BAD_REQUEST,
            None,
        );
        
        assert_eq!(response.success, false);
        assert_eq!(response.error, "VALIDATION_ERROR");
        assert_eq!(response.message, "Invalid input");
        assert_eq!(response.status_code, 400);
    }

    #[test]
    fn test_calculate_pagination_meta() {
        let meta = calculate_pagination_meta(1, 10, 25);
        
        assert_eq!(meta.page, 1);
        assert_eq!(meta.limit, 10);
        assert_eq!(meta.total, 25);
        assert_eq!(meta.total_pages, 3);
        assert_eq!(meta.has_next, true);
        assert_eq!(meta.has_prev, false);
    }

    #[test]
    fn test_validate_content_type_valid() {
        let result = validate_content_type("multipart/form-data; boundary=----WebKitFormBoundary");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_content_type_invalid() {
        let result = validate_content_type("application/json");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_json_content_type_valid() {
        let result = validate_json_content_type("application/json");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_json_content_type_invalid() {
        let result = validate_json_content_type("multipart/form-data");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_file_size_valid() {
        let result = validate_file_size(1024, 2048);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_file_size_too_large() {
        let result = validate_file_size(4096, 2048);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_file_size_empty() {
        let result = validate_file_size(0, 2048);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_json_content_valid() {
        let content = r#"{"key": "value"}"#.as_bytes();
        let result = validate_json_content(content);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_json_content_invalid() {
        let content = r#"invalid json"#.as_bytes();
        let result = validate_json_content(content);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_json_content_empty() {
        let content = b"";
        let result = validate_json_content(content);
        assert!(result.is_err());
    }
} 