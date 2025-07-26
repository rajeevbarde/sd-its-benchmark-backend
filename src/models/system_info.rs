use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct SystemInfo {
    pub id: Option<i32>,
    pub run_id: i32,
    pub arch: Option<String>,
    pub cpu: Option<String>,
    pub system: Option<String>,
    pub release: Option<String>,
    pub python: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateSystemInfo {
    pub run_id: i32,
    pub arch: String,
    pub cpu: String,
    pub system: String,
    pub release: String,
    pub python: String,
}
