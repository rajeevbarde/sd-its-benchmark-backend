use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Gpu {
    pub id: Option<i64>,
    pub run_id: Option<i64>,
    pub device: Option<String>,
    pub driver: Option<String>,
    pub gpu_chip: Option<String>,
    pub brand: Option<String>,
    pub is_laptop: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateGpu {
    pub run_id: i64,
    pub device: String,
    pub driver: String,
    pub gpu_chip: String,
    pub brand: Option<String>,
    pub is_laptop: Option<bool>,
}
