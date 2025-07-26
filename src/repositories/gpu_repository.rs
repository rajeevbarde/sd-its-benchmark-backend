use async_trait::async_trait;
use sqlx::{Error, SqlitePool, Transaction, Sqlite};

use crate::models::gpu::Gpu;
use crate::repositories::traits::{Repository, TransactionRepository};

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