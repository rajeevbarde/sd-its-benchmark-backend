use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ModelMap {
    pub id: Option<i64>,
    pub model_name: Option<String>,
    pub base_model: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateModelMap {
    pub model_name: String,
    pub base_model: String,
}
