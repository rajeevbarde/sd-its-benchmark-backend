use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct PerformanceResult {
    pub id: Option<i32>,
    pub run_id: i32,
    pub its: Option<String>,
    pub avg_its: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreatePerformanceResult {
    pub run_id: i32,
    pub its: String,
    pub avg_its: Option<f64>,
}
