use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AppDetails {
    pub id: Option<i64>,
    pub run_id: Option<i64>,
    pub app_name: Option<String>,
    pub updated: Option<String>,
    pub hash: Option<String>,
    pub url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateAppDetails {
    pub run_id: i64,
    pub app_name: String,
    pub updated: String,
    pub hash: String,
    pub url: String,
}
