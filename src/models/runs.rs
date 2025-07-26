use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Run {
    pub id: Option<i64>,
    pub timestamp: Option<String>,
    pub vram_usage: Option<String>,
    pub info: Option<String>,
    pub system_info: Option<String>,
    pub model_info: Option<String>,
    pub device_info: Option<String>,
    pub xformers: Option<String>,
    pub model_name: Option<String>,
    pub user: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateRun {
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
