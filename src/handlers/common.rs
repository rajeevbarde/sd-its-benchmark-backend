use axum::{
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde::{Deserialize, Serialize};

/// Unified response type that can handle both success and error cases
#[derive(Debug, Serialize, Deserialize)]
pub struct UnifiedResponse {
    pub success: bool,
    pub message: String,
    pub data: Option<serde_json::Value>,
    pub errors: Option<Vec<String>>,
}

/// Unified success response helper
pub fn unified_success_response<T: serde::Serialize>(data: T, message: &str) -> UnifiedResponse {
    UnifiedResponse {
        success: true,
        message: message.to_string(),
        data: Some(serde_json::to_value(data).unwrap_or_default()),
        errors: None,
    }
}

/// Unified error response helper
pub fn unified_error_response(message: &str, errors: Vec<String>) -> UnifiedResponse {
    UnifiedResponse {
        success: false,
        message: message.to_string(),
        data: None,
        errors: Some(errors),
    }
}

/// File upload response data
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileUploadResponse {
    pub filename: String,
    pub size: usize,
    pub content_type: String,
    pub processed: bool,
}

/// File validation result
#[derive(Debug)]
pub struct FileValidationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub file_size: usize,
    pub content_type: String,
}

/// Validate file size against configured limits
pub fn validate_file_size(file_size: usize, max_size_mb: usize) -> Result<(), String> {
    let max_size_bytes = max_size_mb * 1024 * 1024;
    if file_size > max_size_bytes {
        return Err(format!(
            "File size {} bytes exceeds maximum allowed size of {} MB",
            file_size, max_size_mb
        ));
    }
    Ok(())
}

/// Validate file content type
pub fn validate_content_type(content_type: &str, allowed_types: &[String]) -> Result<(), String> {
    if !allowed_types.iter().any(|t| content_type.starts_with(t)) {
        return Err(format!(
            "Content type '{}' is not allowed. Allowed types: {:?}",
            content_type, allowed_types
        ));
    }
    Ok(())
}

/// Validate JSON content
pub fn validate_json_content(content: &str) -> Result<serde_json::Value, String> {
    serde_json::from_str(content).map_err(|e| {
        format!("Invalid JSON content: {}", e)
    })
}

impl IntoResponse for UnifiedResponse {
    fn into_response(self) -> axum::response::Response {
        let status = if self.success {
            StatusCode::OK
        } else {
            StatusCode::BAD_REQUEST
        };
        
        (status, Json(self)).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_file_size_valid() {
        let result = validate_file_size(1024 * 1024, 10); // 1MB file, 10MB limit
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_file_size_too_large() {
        let result = validate_file_size(20 * 1024 * 1024, 10); // 20MB file, 10MB limit
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("exceeds maximum"));
    }

    #[test]
    fn test_validate_content_type_valid() {
        let allowed_types = vec!["application/json".to_string(), "text/json".to_string()];
        let result = validate_content_type("application/json", &allowed_types);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_content_type_invalid() {
        let allowed_types = vec!["application/json".to_string(), "text/json".to_string()];
        let result = validate_content_type("text/plain", &allowed_types);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not allowed"));
    }

    #[test]
    fn test_validate_json_content_valid() {
        let valid_json = r#"{"key": "value"}"#;
        let result = validate_json_content(valid_json);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_json_content_invalid() {
        let invalid_json = r#"{"key": "value"#;
        let result = validate_json_content(invalid_json);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid JSON"));
    }

    #[test]
    fn test_unified_success_response() {
        let data = vec!["test1", "test2"];
        let response = unified_success_response(data, "Success message");
        
        assert!(response.success);
        assert_eq!(response.message, "Success message");
        assert!(response.data.is_some());
        assert!(response.errors.is_none());
    }

    #[test]
    fn test_unified_error_response() {
        let errors = vec!["Error 1".to_string(), "Error 2".to_string()];
        let response = unified_error_response("Error message", errors.clone());
        
        assert!(!response.success);
        assert_eq!(response.message, "Error message");
        assert!(response.data.is_none());
        assert_eq!(response.errors, Some(errors));
    }
} 