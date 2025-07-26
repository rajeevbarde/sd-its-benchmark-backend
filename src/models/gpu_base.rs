use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct GpuBase {
    pub id: Option<i32>,
    pub name: String,
    pub brand: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateGpuBase {
    pub name: String,
    pub brand: Option<String>,
}
