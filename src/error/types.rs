use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Internal server error: {0}")]
    Internal(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("File upload error: {0}")]
    FileUpload(String),

    #[error("JSON parsing error: {0}")]
    JsonParsing(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Configuration error: {0}")]
    Config(String),
}

impl AppError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            AppError::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Validation(_) => StatusCode::BAD_REQUEST,
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AppError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            AppError::FileUpload(_) => StatusCode::BAD_REQUEST,
            AppError::JsonParsing(_) => StatusCode::BAD_REQUEST,
            AppError::Io(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Config(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    pub fn error_code(&self) -> &'static str {
        match self {
            AppError::Database(_) => "DATABASE_ERROR",
            AppError::Validation(_) => "VALIDATION_ERROR",
            AppError::NotFound(_) => "NOT_FOUND",
            AppError::Internal(_) => "INTERNAL_ERROR",
            AppError::BadRequest(_) => "BAD_REQUEST",
            AppError::Unauthorized(_) => "UNAUTHORIZED",
            AppError::FileUpload(_) => "FILE_UPLOAD_ERROR",
            AppError::JsonParsing(_) => "JSON_PARSING_ERROR",
            AppError::Io(_) => "IO_ERROR",
            AppError::Config(_) => "CONFIG_ERROR",
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let error_response = json!({
            "error": {
                "code": self.error_code(),
                "message": self.to_string(),
                "status": status.as_u16()
            }
        });

        (status, Json(error_response)).into_response()
    }
}

// Convenience constructors
impl AppError {
    pub fn validation<T: Into<String>>(message: T) -> Self {
        AppError::Validation(message.into())
    }

    pub fn not_found<T: Into<String>>(resource: T) -> Self {
        AppError::NotFound(resource.into())
    }

    pub fn internal<T: Into<String>>(message: T) -> Self {
        AppError::Internal(message.into())
    }

    pub fn bad_request<T: Into<String>>(message: T) -> Self {
        AppError::BadRequest(message.into())
    }

    pub fn unauthorized<T: Into<String>>(message: T) -> Self {
        AppError::Unauthorized(message.into())
    }

    pub fn file_upload<T: Into<String>>(message: T) -> Self {
        AppError::FileUpload(message.into())
    }

    pub fn config<T: Into<String>>(message: T) -> Self {
        AppError::Config(message.into())
    }
}

// Result type alias for convenience
pub type AppResult<T> = Result<T, AppError>; 