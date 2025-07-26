use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct GpuMap {
    pub id: Option<i64>,
    pub gpu_name: Option<String>,
    pub base_gpu_id: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateGpuMap {
    pub gpu_name: String,
    pub base_gpu_id: i64,
}
