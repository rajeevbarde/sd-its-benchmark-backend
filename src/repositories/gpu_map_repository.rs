use async_trait::async_trait;
use sqlx::{Error, SqlitePool, Transaction, Sqlite};

use crate::models::gpu_map::GpuMap;
use crate::repositories::traits::{Repository, TransactionRepository, BulkRepository, BulkTransactionRepository};

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

#[async_trait]
impl BulkRepository<GpuMap, i64> for GpuMapRepository {
    async fn bulk_create(&self, entities: Vec<GpuMap>) -> Result<Vec<GpuMap>, Error> {
        if entities.is_empty() {
            return Ok(vec![]);
        }

        let mut tx = self.pool.begin().await?;
        
        let result = self.bulk_create_tx(entities, &mut tx).await;
        
        match result {
            Ok(results) => {
                tx.commit().await?;
                Ok(results)
            }
            Err(e) => {
                tx.rollback().await?;
                Err(e)
            }
        }
    }

    async fn bulk_update(&self, entities: Vec<GpuMap>) -> Result<Vec<GpuMap>, Error> {
        if entities.is_empty() {
            return Ok(vec![]);
        }

        let mut tx = self.pool.begin().await?;
        
        let result = self.bulk_update_tx(entities, &mut tx).await;
        
        match result {
            Ok(results) => {
                tx.commit().await?;
                Ok(results)
            }
            Err(e) => {
                tx.rollback().await?;
                Err(e)
            }
        }
    }

    async fn delete_all(&self) -> Result<usize, Error> {
        let mut tx = self.pool.begin().await?;
        
        let result = self.delete_all_tx(&mut tx).await;
        
        match result {
            Ok(count) => {
                tx.commit().await?;
                Ok(count)
            }
            Err(e) => {
                tx.rollback().await?;
                Err(e)
            }
        }
    }
}

#[async_trait]
impl<'a> BulkTransactionRepository<'a, GpuMap, i64> for GpuMapRepository {
    async fn bulk_create_tx(&self, entities: Vec<GpuMap>, tx: &mut Transaction<'a, Sqlite>) -> Result<Vec<GpuMap>, Error> {
        if entities.is_empty() {
            return Ok(vec![]);
        }

        let mut created_results = Vec::with_capacity(entities.len());

        // Use batch processing for better performance
        for entity in entities {
            let created_result = self.create_tx(entity, tx).await?;
            created_results.push(created_result);
        }

        Ok(created_results)
    }

    async fn bulk_update_tx(&self, entities: Vec<GpuMap>, tx: &mut Transaction<'a, Sqlite>) -> Result<Vec<GpuMap>, Error> {
        if entities.is_empty() {
            return Ok(vec![]);
        }

        let mut updated_results = Vec::with_capacity(entities.len());

        // Use batch processing for better performance
        for entity in entities {
            let updated_result = self.update_tx(entity, tx).await?;
            updated_results.push(updated_result);
        }

        Ok(updated_results)
    }

    async fn delete_all_tx(&self, tx: &mut Transaction<'a, Sqlite>) -> Result<usize, Error> {
        let result = sqlx::query!("DELETE FROM GPUMap")
            .execute(&mut **tx)
            .await?;
        
        Ok(result.rows_affected() as usize)
    }
} 