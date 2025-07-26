use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PerformanceResult {
    pub id: Option<i64>,
    pub run_id: Option<i64>,
    pub its: Option<String>,
    pub avg_its: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreatePerformanceResult {
    pub run_id: i64,
    pub its: String,
    pub avg_its: Option<f64>,
}
