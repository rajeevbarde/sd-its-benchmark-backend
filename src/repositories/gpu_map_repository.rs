use async_trait::async_trait;
use sqlx::{Error, SqlitePool, Transaction, Sqlite};

use crate::models::gpu_map::{GpuMap, CreateGpuMap};
use crate::repositories::traits::{Repository, TransactionRepository};

pub struct GpuMapRepository {
    pool: SqlitePool,
}

impl GpuMapRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Find GPU map by gpu_name
    pub async fn find_by_gpu_name(&self, gpu_name: &str) -> Result<Vec<GpuMap>, Error> {
        let results = sqlx::query_as!(
            GpuMap,
            r#"
            SELECT id, gpu_name, base_gpu_id
            FROM GPUMap
            WHERE gpu_name = ?
            ORDER BY id DESC
            "#,
            gpu_name
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(results)
    }

    /// Find GPU map by base_gpu_id
    pub async fn find_by_base_gpu_id(&self, base_gpu_id: i64) -> Result<Vec<GpuMap>, Error> {
        let results = sqlx::query_as!(
            GpuMap,
            r#"
            SELECT id, gpu_name, base_gpu_id
            FROM GPUMap
            WHERE base_gpu_id = ?
            ORDER BY id DESC
            "#,
            base_gpu_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(results)
    }
}

#[async_trait]
impl Repository<GpuMap, i64> for GpuMapRepository {
    async fn create(&self, entity: GpuMap) -> Result<GpuMap, Error> {
        let id = sqlx::query!(
            r#"
            INSERT INTO GPUMap (gpu_name, base_gpu_id)
            VALUES (?, ?)
            "#,
            entity.gpu_name,
            entity.base_gpu_id
        )
        .execute(&self.pool)
        .await?
        .last_insert_rowid() as i64;

        Ok(GpuMap {
            id: Some(id),
            ..entity
        })
    }

    async fn find_by_id(&self, id: i64) -> Result<Option<GpuMap>, Error> {
        let result = sqlx::query_as!(
            GpuMap,
            r#"
            SELECT id, gpu_name, base_gpu_id
            FROM GPUMap
            WHERE id = ?
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(result)
    }

    async fn find_all(&self) -> Result<Vec<GpuMap>, Error> {
        let results = sqlx::query_as!(
            GpuMap,
            r#"
            SELECT id, gpu_name, base_gpu_id
            FROM GPUMap
            ORDER BY id DESC
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(results)
    }

    async fn update(&self, entity: GpuMap) -> Result<GpuMap, Error> {
        let id = entity.id.ok_or(Error::RowNotFound)?;
        
        sqlx::query!(
            r#"
            UPDATE GPUMap
            SET gpu_name = ?, base_gpu_id = ?
            WHERE id = ?
            "#,
            entity.gpu_name,
            entity.base_gpu_id,
            id
        )
        .execute(&self.pool)
        .await?;

        Ok(entity)
    }

    async fn delete(&self, id: i64) -> Result<(), Error> {
        sqlx::query!("DELETE FROM GPUMap WHERE id = ?", id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn count(&self) -> Result<i64, Error> {
        let count = sqlx::query!("SELECT COUNT(*) as count FROM GPUMap")
            .fetch_one(&self.pool)
            .await?
            .count;
        Ok(count)
    }
}

#[async_trait]
impl<'a> TransactionRepository<'a, GpuMap, i64> for GpuMapRepository {
    async fn create_tx(&self, entity: GpuMap, tx: &mut Transaction<'a, Sqlite>) -> Result<GpuMap, Error> {
        let id = sqlx::query!(
            r#"
            INSERT INTO GPUMap (gpu_name, base_gpu_id)
            VALUES (?, ?)
            "#,
            entity.gpu_name,
            entity.base_gpu_id
        )
        .execute(&mut **tx)
        .await?
        .last_insert_rowid() as i64;

        Ok(GpuMap {
            id: Some(id),
            ..entity
        })
    }

    async fn update_tx(&self, entity: GpuMap, tx: &mut Transaction<'a, Sqlite>) -> Result<GpuMap, Error> {
        let id = entity.id.ok_or(Error::RowNotFound)?;
        
        sqlx::query!(
            r#"
            UPDATE GPUMap
            SET gpu_name = ?, base_gpu_id = ?
            WHERE id = ?
            "#,
            entity.gpu_name,
            entity.base_gpu_id,
            id
        )
        .execute(&mut **tx)
        .await?;

        Ok(entity)
    }

    async fn delete_tx(&self, id: i64, tx: &mut Transaction<'a, Sqlite>) -> Result<(), Error> {
        sqlx::query!("DELETE FROM GPUMap WHERE id = ?", id)
            .execute(&mut **tx)
            .await?;
        Ok(())
    }
} 