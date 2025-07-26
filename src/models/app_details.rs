use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct AppDetails {
    pub id: Option<i32>,
    pub run_id: i32,
    pub app_name: Option<String>,
    pub updated: Option<String>,
    pub hash: Option<String>,
    pub url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateAppDetails {
    pub run_id: i32,
    pub app_name: String,
    pub updated: String,
    pub hash: String,
    pub url: String,
}
