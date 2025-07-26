use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use tracing::{error, warn, info};

use crate::error::AppError;
use crate::error::AppResult;

/// Global error handler for unhandled errors
pub async fn handle_error(err: axum::Error) -> Response {
    error!("Unhandled error: {:?}", err);
    
    let error_response = json!({
        "error": {
            "code": "INTERNAL_ERROR",
            "message": "An unexpected error occurred",
            "status": 500
        }
    });

    (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)).into_response()
}

/// Log error with context
pub fn log_error(err: &AppError, context: &str) {
    match err {
        AppError::Database(db_err) => {
            error!("Database error in {}: {:?}", context, db_err);
        }
        AppError::Validation(msg) => {
            warn!("Validation error in {}: {}", context, msg);
        }
        AppError::NotFound(resource) => {
            info!("Resource not found in {}: {}", context, resource);
        }
        AppError::Internal(msg) => {
            error!("Internal error in {}: {}", context, msg);
        }
        AppError::BadRequest(msg) => {
            warn!("Bad request in {}: {}", context, msg);
        }
        AppError::Unauthorized(msg) => {
            warn!("Unauthorized access in {}: {}", context, msg);
        }
        AppError::FileUpload(msg) => {
            warn!("File upload error in {}: {}", context, msg);
        }
        AppError::JsonParsing(json_err) => {
            warn!("JSON parsing error in {}: {:?}", context, json_err);
        }
        AppError::Io(io_err) => {
            error!("IO error in {}: {:?}", context, io_err);
        }
        AppError::Config(msg) => {
            error!("Configuration error in {}: {}", context, msg);
        }
    }
}

/// Convert anyhow errors to AppError
pub fn handle_anyhow_error(err: anyhow::Error, context: &str) -> AppError {
    error!("Anyhow error in {}: {:?}", context, err);
    AppError::Internal(format!("Error in {}: {}", context, err))
}

/// Validate required fields
pub fn validate_required_field<T>(field: Option<T>, field_name: &str) -> AppResult<T> {
    field.ok_or_else(|| AppError::validation(format!("{} is required", field_name)))
}

/// Validate string length
pub fn validate_string_length(s: &str, min: usize, max: usize, field_name: &str) -> AppResult<()> {
    if s.len() < min {
        return Err(AppError::validation(format!("{} must be at least {} characters", field_name, min)));
    }
    if s.len() > max {
        return Err(AppError::validation(format!("{} must be at most {} characters", field_name, max)));
    }
    Ok(())
}

/// Validate numeric range
pub fn validate_numeric_range<T: PartialOrd + std::fmt::Display>(
    value: T,
    min: T,
    max: T,
    field_name: &str,
) -> AppResult<()> {
    if value < min || value > max {
        return Err(AppError::validation(format!(
            "{} must be between {} and {}",
            field_name, min, max
        )));
    }
    Ok(())
} 