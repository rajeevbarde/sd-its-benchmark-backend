use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct RunMoreDetails {
    pub id: Option<i64>,
    pub run_id: Option<i64>,
    pub timestamp: Option<String>,
    pub model_name: Option<String>,
    pub user: Option<String>,
    pub notes: Option<String>,
    pub model_map_id: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateRunMoreDetails {
    pub run_id: i64,
    pub timestamp: String,
    pub model_name: String,
    pub user: String,
    pub notes: String,
    pub model_map_id: Option<i64>,
}
