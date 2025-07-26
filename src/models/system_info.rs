use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SystemInfo {
    pub id: Option<i64>,
    pub run_id: Option<i64>,
    pub arch: Option<String>,
    pub cpu: Option<String>,
    pub system: Option<String>,
    pub release: Option<String>,
    pub python: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateSystemInfo {
    pub run_id: i64,
    pub arch: String,
    pub cpu: String,
    pub system: String,
    pub release: String,
    pub python: String,
}
