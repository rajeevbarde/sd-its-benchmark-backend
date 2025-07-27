use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
    body::to_bytes,
};
use validator::{Validate, ValidationErrors};
use tracing::{error, warn};

use crate::error::types::AppError;

/// Validation middleware that can be applied to any endpoint
pub async fn validate_request<T>(
    request: Request,
    next: Next,
) -> Result<Response, AppError>
where
    T: for<'de> serde::Deserialize<'de> + Validate,
{
    // Extract the body from the request
    let (parts, body) = request.into_parts();
    
    // Try to deserialize the body
    let bytes = to_bytes(body, usize::MAX)
        .await
        .map_err(|e| {
            error!("Failed to read request body: {}", e);
            AppError::BadRequest("Invalid request body".to_string())
        })?;

    // Deserialize the request
    let data: T = serde_json::from_slice(&bytes).map_err(|e| {
        error!("Failed to deserialize request: {}", e);
        AppError::BadRequest("Invalid JSON format".to_string())
    })?;

    // Validate the request
    if let Err(validation_errors) = data.validate() {
        let error_messages = format_validation_errors(&validation_errors);
        warn!("Validation failed: {}", error_messages);
        return Err(AppError::Validation(error_messages));
    }

    // Reconstruct the request with the validated data
    let body = axum::body::Body::from(bytes);
    let request = Request::from_parts(parts, body);
    
    Ok(next.run(request).await)
}

/// Format validation errors into a user-friendly message
fn format_validation_errors(errors: &ValidationErrors) -> String {
    let mut messages = Vec::new();
    
    for (field, field_errors) in errors.field_errors() {
        for error in field_errors {
            let message = error.message.as_ref()
                .map(|m| m.to_string())
                .unwrap_or_else(|| format!("Invalid value for field '{}'", field));
            messages.push(format!("{}: {}", field, message));
        }
    }
    
    messages.join("; ")
}

/// Validation error response
#[derive(Debug, serde::Serialize)]
pub struct ValidationErrorResponse {
    pub error: String,
    pub details: Vec<String>,
    pub status: u16,
}

impl ValidationErrorResponse {
    pub fn new(error: String, details: Vec<String>) -> Self {
        Self {
            error,
            details,
            status: StatusCode::BAD_REQUEST.as_u16(),
        }
    }
}

/// Helper function to create validation error response
pub fn create_validation_error_response(errors: &ValidationErrors) -> Response {
    let details: Vec<String> = errors
        .field_errors()
        .iter()
        .flat_map(|(field, field_errors)| {
            let field = field.to_string();
            field_errors.iter().map(move |error| {
                error.message.as_ref()
                    .map(|m| format!("{}: {}", field, m))
                    .unwrap_or_else(|| format!("Invalid value for field '{}'", field))
            })
        })
        .collect();

    let response = ValidationErrorResponse::new(
        "Validation failed".to_string(),
        details,
    );

    (StatusCode::BAD_REQUEST, Json(response)).into_response()
}

/// File upload validation helper
pub fn validate_file_upload(
    file_content: &[u8],
    file_name: &str,
    max_size: usize,
    allowed_extensions: &[&str],
) -> Result<(), AppError> {
    // Check file size
    if file_content.len() > max_size {
        return Err(AppError::BadRequest(format!(
            "File size {} exceeds maximum allowed size of {} bytes",
            file_content.len(),
            max_size
        )));
    }

    if file_content.is_empty() {
        return Err(AppError::BadRequest("Uploaded file is empty".to_string()));
    }

    // Check file extension
    if let Some(extension) = file_name.split('.').last() {
        if !allowed_extensions.contains(&extension.to_lowercase().as_str()) {
            return Err(AppError::BadRequest(format!(
                "File extension '{}' is not allowed. Allowed extensions: {:?}",
                extension,
                allowed_extensions
            )));
        }
    } else {
        return Err(AppError::BadRequest("File must have an extension".to_string()));
    }

    // Validate JSON content
    if let Err(_) = serde_json::from_slice::<serde_json::Value>(file_content) {
        return Err(AppError::BadRequest("Uploaded file is not valid JSON".to_string()));
    }

    Ok(())
}

/// Query parameter validation helper
pub fn validate_query_params<T>(params: &T) -> Result<(), AppError>
where
    T: Validate,
{
    if let Err(validation_errors) = params.validate() {
        let error_messages = format_validation_errors(&validation_errors);
        return Err(AppError::Validation(error_messages));
    }
    Ok(())
} 