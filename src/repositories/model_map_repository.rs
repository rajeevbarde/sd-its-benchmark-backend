use async_trait::async_trait;
use sqlx::{Error, SqlitePool, Transaction, Sqlite};

use crate::models::model_map::ModelMap;
use crate::repositories::traits::{Repository, TransactionRepository};

pub struct ModelMapRepository {
    pool: SqlitePool,
}

impl ModelMapRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Find model map by model_name
    pub async fn find_by_model_name(&self, model_name: &str) -> Result<Vec<ModelMap>, Error> {
        let results = sqlx::query_as!(
            ModelMap,
            r#"
            SELECT id, model_name, base_model
            FROM ModelMap
            WHERE model_name = ?
            ORDER BY id DESC
            "#,
            model_name
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(results)
    }

    /// Find model map by base_model
    pub async fn find_by_base_model(&self, base_model: &str) -> Result<Vec<ModelMap>, Error> {
        let results = sqlx::query_as!(
            ModelMap,
            r#"
            SELECT id, model_name, base_model
            FROM ModelMap
            WHERE base_model = ?
            ORDER BY id DESC
            "#,
            base_model
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(results)
    }
}

#[async_trait]
impl Repository<ModelMap, i64> for ModelMapRepository {
    async fn create(&self, entity: ModelMap) -> Result<ModelMap, Error> {
        let id = sqlx::query!(
            r#"
            INSERT INTO ModelMap (model_name, base_model)
            VALUES (?, ?)
            "#,
            entity.model_name,
            entity.base_model
        )
        .execute(&self.pool)
        .await?
        .last_insert_rowid() as i64;

        Ok(ModelMap {
            id: Some(id),
            ..entity
        })
    }

    async fn find_by_id(&self, id: i64) -> Result<Option<ModelMap>, Error> {
        let result = sqlx::query_as!(
            ModelMap,
            r#"
            SELECT id, model_name, base_model
            FROM ModelMap
            WHERE id = ?
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(result)
    }

    async fn find_all(&self) -> Result<Vec<ModelMap>, Error> {
        let results = sqlx::query_as!(
            ModelMap,
            r#"
            SELECT id, model_name, base_model
            FROM ModelMap
            ORDER BY id DESC
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(results)
    }

    async fn update(&self, entity: ModelMap) -> Result<ModelMap, Error> {
        let id = entity.id.ok_or(Error::RowNotFound)?;
        
        sqlx::query!(
            r#"
            UPDATE ModelMap
            SET model_name = ?, base_model = ?
            WHERE id = ?
            "#,
            entity.model_name,
            entity.base_model,
            id
        )
        .execute(&self.pool)
        .await?;

        Ok(entity)
    }

    async fn delete(&self, id: i64) -> Result<(), Error> {
        sqlx::query!("DELETE FROM ModelMap WHERE id = ?", id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn count(&self) -> Result<i64, Error> {
        let count = sqlx::query!("SELECT COUNT(*) as count FROM ModelMap")
            .fetch_one(&self.pool)
            .await?
            .count;
        Ok(count)
    }
}

#[async_trait]
impl<'a> TransactionRepository<'a, ModelMap, i64> for ModelMapRepository {
    async fn create_tx(&self, entity: ModelMap, tx: &mut Transaction<'a, Sqlite>) -> Result<ModelMap, Error> {
        let id = sqlx::query!(
            r#"
            INSERT INTO ModelMap (model_name, base_model)
            VALUES (?, ?)
            "#,
            entity.model_name,
            entity.base_model
        )
        .execute(&mut **tx)
        .await?
        .last_insert_rowid() as i64;

        Ok(ModelMap {
            id: Some(id),
            ..entity
        })
    }

    async fn update_tx(&self, entity: ModelMap, tx: &mut Transaction<'a, Sqlite>) -> Result<ModelMap, Error> {
        let id = entity.id.ok_or(Error::RowNotFound)?;
        
        sqlx::query!(
            r#"
            UPDATE ModelMap
            SET model_name = ?, base_model = ?
            WHERE id = ?
            "#,
            entity.model_name,
            entity.base_model,
            id
        )
        .execute(&mut **tx)
        .await?;

        Ok(entity)
    }

    async fn delete_tx(&self, id: i64, tx: &mut Transaction<'a, Sqlite>) -> Result<(), Error> {
        sqlx::query!("DELETE FROM ModelMap WHERE id = ?", id)
            .execute(&mut **tx)
            .await?;
        Ok(())
    }
} 