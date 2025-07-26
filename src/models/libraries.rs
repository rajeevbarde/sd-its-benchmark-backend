use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Libraries {
    pub id: Option<i64>,
    pub run_id: Option<i64>,
    pub torch: Option<String>,
    pub xformers: Option<String>,
    pub xformers1: Option<String>,
    pub diffusers: Option<String>,
    pub transformers: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateLibraries {
    pub run_id: i64,
    pub torch: String,
    pub xformers: String,
    pub xformers1: String,
    pub diffusers: String,
    pub transformers: String,
}
