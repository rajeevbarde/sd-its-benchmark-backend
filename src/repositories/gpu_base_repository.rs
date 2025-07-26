use async_trait::async_trait;
use sqlx::{Error, SqlitePool, Transaction, Sqlite};

use crate::models::gpu_base::{GpuBase, CreateGpuBase};
use crate::repositories::traits::{Repository, TransactionRepository};

pub struct GpuBaseRepository {
    pool: SqlitePool,
}

impl GpuBaseRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Find GPU base by name
    pub async fn find_by_name(&self, name: &str) -> Result<Vec<GpuBase>, Error> {
        let results = sqlx::query_as!(
            GpuBase,
            r#"
            SELECT id, name, brand
            FROM GPUBase
            WHERE name = ?
            ORDER BY id DESC
            "#,
            name
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(results)
    }

    /// Find GPU base by brand
    pub async fn find_by_brand(&self, brand: &str) -> Result<Vec<GpuBase>, Error> {
        let results = sqlx::query_as!(
            GpuBase,
            r#"
            SELECT id, name, brand
            FROM GPUBase
            WHERE brand = ?
            ORDER BY id DESC
            "#,
            brand
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(results)
    }
}

#[async_trait]
impl Repository<GpuBase, i64> for GpuBaseRepository {
    async fn create(&self, entity: GpuBase) -> Result<GpuBase, Error> {
        let id = sqlx::query!(
            r#"
            INSERT INTO GPUBase (name, brand)
            VALUES (?, ?)
            "#,
            entity.name,
            entity.brand
        )
        .execute(&self.pool)
        .await?
        .last_insert_rowid() as i64;

        Ok(GpuBase {
            id: Some(id),
            ..entity
        })
    }

    async fn find_by_id(&self, id: i64) -> Result<Option<GpuBase>, Error> {
        let result = sqlx::query_as!(
            GpuBase,
            r#"
            SELECT id, name, brand
            FROM GPUBase
            WHERE id = ?
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(result)
    }

    async fn find_all(&self) -> Result<Vec<GpuBase>, Error> {
        let results = sqlx::query_as!(
            GpuBase,
            r#"
            SELECT id, name, brand
            FROM GPUBase
            ORDER BY id DESC
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(results)
    }

    async fn update(&self, entity: GpuBase) -> Result<GpuBase, Error> {
        let id = entity.id.ok_or(Error::RowNotFound)?;
        
        sqlx::query!(
            r#"
            UPDATE GPUBase
            SET name = ?, brand = ?
            WHERE id = ?
            "#,
            entity.name,
            entity.brand,
            id
        )
        .execute(&self.pool)
        .await?;

        Ok(entity)
    }

    async fn delete(&self, id: i64) -> Result<(), Error> {
        sqlx::query!("DELETE FROM GPUBase WHERE id = ?", id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn count(&self) -> Result<i64, Error> {
        let count = sqlx::query!("SELECT COUNT(*) as count FROM GPUBase")
            .fetch_one(&self.pool)
            .await?
            .count;
        Ok(count)
    }
}

#[async_trait]
impl<'a> TransactionRepository<'a, GpuBase, i64> for GpuBaseRepository {
    async fn create_tx(&self, entity: GpuBase, tx: &mut Transaction<'a, Sqlite>) -> Result<GpuBase, Error> {
        let id = sqlx::query!(
            r#"
            INSERT INTO GPUBase (name, brand)
            VALUES (?, ?)
            "#,
            entity.name,
            entity.brand
        )
        .execute(&mut **tx)
        .await?
        .last_insert_rowid() as i64;

        Ok(GpuBase {
            id: Some(id),
            ..entity
        })
    }

    async fn update_tx(&self, entity: GpuBase, tx: &mut Transaction<'a, Sqlite>) -> Result<GpuBase, Error> {
        let id = entity.id.ok_or(Error::RowNotFound)?;
        
        sqlx::query!(
            r#"
            UPDATE GPUBase
            SET name = ?, brand = ?
            WHERE id = ?
            "#,
            entity.name,
            entity.brand,
            id
        )
        .execute(&mut **tx)
        .await?;

        Ok(entity)
    }

    async fn delete_tx(&self, id: i64, tx: &mut Transaction<'a, Sqlite>) -> Result<(), Error> {
        sqlx::query!("DELETE FROM GPUBase WHERE id = ?", id)
            .execute(&mut **tx)
            .await?;
        Ok(())
    }
} 