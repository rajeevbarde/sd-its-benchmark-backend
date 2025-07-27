use async_trait::async_trait;
use sqlx::{Error, SqlitePool, Transaction, Sqlite};

use crate::models::gpu::Gpu;
use crate::repositories::traits::{Repository, TransactionRepository, BulkRepository, BulkTransactionRepository};

pub struct GpuRepository {
    pool: SqlitePool,
}

impl GpuRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Find GPUs by run_id
    pub async fn find_by_run_id(&self, run_id: i64) -> Result<Vec<Gpu>, Error> {
        let results = sqlx::query_as!(
            Gpu,
            r#"
            SELECT id, run_id, device, driver, gpu_chip, brand, isLaptop as "is_laptop"
            FROM GPU
            WHERE run_id = ?
            ORDER BY id DESC
            "#,
            run_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(results)
    }

    /// Find GPUs by brand
    pub async fn find_by_brand(&self, brand: &str) -> Result<Vec<Gpu>, Error> {
        let results = sqlx::query_as!(
            Gpu,
            r#"
            SELECT id, run_id, device, driver, gpu_chip, brand, isLaptop as "is_laptop"
            FROM GPU
            WHERE brand = ?
            ORDER BY id DESC
            "#,
            brand
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(results)
    }

    /// Find GPUs by laptop status
    pub async fn find_by_laptop_status(&self, is_laptop: bool) -> Result<Vec<Gpu>, Error> {
        let results = sqlx::query_as!(
            Gpu,
            r#"
            SELECT id, run_id, device, driver, gpu_chip, brand, isLaptop as "is_laptop"
            FROM GPU
            WHERE isLaptop = ?
            ORDER BY id DESC
            "#,
            is_laptop
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(results)
    }

    /// Clear all GPU records
    pub async fn clear_all(&self) -> Result<(), Error> {
        sqlx::query!("DELETE FROM GPU")
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Clear all GPU records within a transaction
    pub async fn clear_all_tx(&self, tx: &mut Transaction<'_, Sqlite>) -> Result<(), Error> {
        sqlx::query!("DELETE FROM GPU")
            .execute(&mut **tx)
            .await?;
        Ok(())
    }
}

#[async_trait]
impl Repository<Gpu, i64> for GpuRepository {
    async fn create(&self, entity: Gpu) -> Result<Gpu, Error> {
        let id = sqlx::query!(
            r#"
            INSERT INTO GPU (run_id, device, driver, gpu_chip, brand, isLaptop)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
            entity.run_id,
            entity.device,
            entity.driver,
            entity.gpu_chip,
            entity.brand,
            entity.is_laptop
        )
        .execute(&self.pool)
        .await?
        .last_insert_rowid() as i64;

        Ok(Gpu {
            id: Some(id),
            ..entity
        })
    }

    async fn find_by_id(&self, id: i64) -> Result<Option<Gpu>, Error> {
        let result = sqlx::query_as!(
            Gpu,
            r#"
            SELECT id, run_id, device, driver, gpu_chip, brand, isLaptop as "is_laptop"
            FROM GPU
            WHERE id = ?
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(result)
    }

    async fn find_all(&self) -> Result<Vec<Gpu>, Error> {
        let results = sqlx::query_as!(
            Gpu,
            r#"
            SELECT id, run_id, device, driver, gpu_chip, brand, isLaptop as "is_laptop"
            FROM GPU
            ORDER BY id DESC
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(results)
    }

    async fn update(&self, entity: Gpu) -> Result<Gpu, Error> {
        let id = entity.id.ok_or(Error::RowNotFound)?;
        
        sqlx::query!(
            r#"
            UPDATE GPU
            SET run_id = ?, device = ?, driver = ?, gpu_chip = ?, brand = ?, isLaptop = ?
            WHERE id = ?
            "#,
            entity.run_id,
            entity.device,
            entity.driver,
            entity.gpu_chip,
            entity.brand,
            entity.is_laptop,
            id
        )
        .execute(&self.pool)
        .await?;

        Ok(entity)
    }

    async fn delete(&self, id: i64) -> Result<(), Error> {
        sqlx::query!("DELETE FROM GPU WHERE id = ?", id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn count(&self) -> Result<i64, Error> {
        let count = sqlx::query!("SELECT COUNT(*) as count FROM GPU")
            .fetch_one(&self.pool)
            .await?
            .count;
        Ok(count)
    }
}

#[async_trait]
impl<'a> TransactionRepository<'a, Gpu, i64> for GpuRepository {
    async fn create_tx(&self, entity: Gpu, tx: &mut Transaction<'a, Sqlite>) -> Result<Gpu, Error> {
        let id = sqlx::query!(
            r#"
            INSERT INTO GPU (run_id, device, driver, gpu_chip, brand, isLaptop)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
            entity.run_id,
            entity.device,
            entity.driver,
            entity.gpu_chip,
            entity.brand,
            entity.is_laptop
        )
        .execute(&mut **tx)
        .await?
        .last_insert_rowid() as i64;

        Ok(Gpu {
            id: Some(id),
            ..entity
        })
    }

    async fn update_tx(&self, entity: Gpu, tx: &mut Transaction<'a, Sqlite>) -> Result<Gpu, Error> {
        let id = entity.id.ok_or(Error::RowNotFound)?;
        
        sqlx::query!(
            r#"
            UPDATE GPU
            SET run_id = ?, device = ?, driver = ?, gpu_chip = ?, brand = ?, isLaptop = ?
            WHERE id = ?
            "#,
            entity.run_id,
            entity.device,
            entity.driver,
            entity.gpu_chip,
            entity.brand,
            entity.is_laptop,
            id
        )
        .execute(&mut **tx)
        .await?;

        Ok(entity)
    }

    async fn delete_tx(&self, id: i64, tx: &mut Transaction<'a, Sqlite>) -> Result<(), Error> {
        sqlx::query!("DELETE FROM GPU WHERE id = ?", id)
            .execute(&mut **tx)
            .await?;
        Ok(())
    }
} 

#[async_trait]
impl BulkRepository<Gpu, i64> for GpuRepository {
    async fn bulk_create(&self, entities: Vec<Gpu>) -> Result<Vec<Gpu>, Error> {
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

    async fn bulk_update(&self, entities: Vec<Gpu>) -> Result<Vec<Gpu>, Error> {
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
impl<'a> BulkTransactionRepository<'a, Gpu, i64> for GpuRepository {
    async fn bulk_create_tx(&self, entities: Vec<Gpu>, tx: &mut Transaction<'a, Sqlite>) -> Result<Vec<Gpu>, Error> {
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

    async fn bulk_update_tx(&self, entities: Vec<Gpu>, tx: &mut Transaction<'a, Sqlite>) -> Result<Vec<Gpu>, Error> {
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
        let result = sqlx::query!("DELETE FROM GPU")
            .execute(&mut **tx)
            .await?;
        
        Ok(result.rows_affected() as usize)
    }
} 