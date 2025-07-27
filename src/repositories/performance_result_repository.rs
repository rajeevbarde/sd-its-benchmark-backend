use async_trait::async_trait;
use sqlx::{Error, SqlitePool, Transaction, Sqlite};

use crate::models::performance_result::PerformanceResult;
use crate::repositories::traits::{Repository, TransactionRepository, BulkRepository, BulkTransactionRepository};

pub struct PerformanceResultRepository {
    pool: SqlitePool,
}

impl PerformanceResultRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Find performance results by run_id
    pub async fn find_by_run_id(&self, run_id: i64) -> Result<Vec<PerformanceResult>, Error> {
        let results = sqlx::query_as!(
            PerformanceResult,
            r#"
            SELECT id, run_id, its, avg_its
            FROM performanceResult
            WHERE run_id = ?
            ORDER BY id DESC
            "#,
            run_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(results)
    }

    /// Clear all performance results
    pub async fn clear_all(&self) -> Result<(), Error> {
        sqlx::query!("DELETE FROM performanceResult")
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Clear all performance results within a transaction
    pub async fn clear_all_tx(&self, tx: &mut Transaction<'_, Sqlite>) -> Result<(), Error> {
        sqlx::query!("DELETE FROM performanceResult")
            .execute(&mut **tx)
            .await?;
        Ok(())
    }
}

#[async_trait]
impl Repository<PerformanceResult, i64> for PerformanceResultRepository {
    async fn create(&self, entity: PerformanceResult) -> Result<PerformanceResult, Error> {
        let id = sqlx::query!(
            r#"
            INSERT INTO performanceResult (run_id, its, avg_its)
            VALUES (?, ?, ?)
            "#,
            entity.run_id,
            entity.its,
            entity.avg_its
        )
        .execute(&self.pool)
        .await?
        .last_insert_rowid() as i64;

        Ok(PerformanceResult {
            id: Some(id),
            ..entity
        })
    }

    async fn find_by_id(&self, id: i64) -> Result<Option<PerformanceResult>, Error> {
        let result = sqlx::query_as!(
            PerformanceResult,
            r#"
            SELECT id, run_id, its, avg_its
            FROM performanceResult
            WHERE id = ?
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(result)
    }

    async fn find_all(&self) -> Result<Vec<PerformanceResult>, Error> {
        let results = sqlx::query_as!(
            PerformanceResult,
            r#"
            SELECT id, run_id, its, avg_its
            FROM performanceResult
            ORDER BY id DESC
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(results)
    }

    async fn update(&self, entity: PerformanceResult) -> Result<PerformanceResult, Error> {
        let id = entity.id.ok_or(Error::RowNotFound)?;
        
        sqlx::query!(
            r#"
            UPDATE performanceResult
            SET run_id = ?, its = ?, avg_its = ?
            WHERE id = ?
            "#,
            entity.run_id,
            entity.its,
            entity.avg_its,
            id
        )
        .execute(&self.pool)
        .await?;

        Ok(entity)
    }

    async fn delete(&self, id: i64) -> Result<(), Error> {
        sqlx::query!("DELETE FROM performanceResult WHERE id = ?", id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn count(&self) -> Result<i64, Error> {
        let count = sqlx::query!("SELECT COUNT(*) as count FROM performanceResult")
            .fetch_one(&self.pool)
            .await?
            .count;
        Ok(count)
    }
}

#[async_trait]
impl BulkRepository<PerformanceResult, i64> for PerformanceResultRepository {
    async fn bulk_create(&self, entities: Vec<PerformanceResult>) -> Result<Vec<PerformanceResult>, Error> {
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

    async fn bulk_update(&self, entities: Vec<PerformanceResult>) -> Result<Vec<PerformanceResult>, Error> {
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
impl<'a> TransactionRepository<'a, PerformanceResult, i64> for PerformanceResultRepository {
    async fn create_tx(&self, entity: PerformanceResult, tx: &mut Transaction<'a, Sqlite>) -> Result<PerformanceResult, Error> {
        let id = sqlx::query!(
            r#"
            INSERT INTO performanceResult (run_id, its, avg_its)
            VALUES (?, ?, ?)
            "#,
            entity.run_id,
            entity.its,
            entity.avg_its
        )
        .execute(&mut **tx)
        .await?
        .last_insert_rowid() as i64;

        Ok(PerformanceResult {
            id: Some(id),
            ..entity
        })
    }

    async fn update_tx(&self, entity: PerformanceResult, tx: &mut Transaction<'a, Sqlite>) -> Result<PerformanceResult, Error> {
        let id = entity.id.ok_or(Error::RowNotFound)?;
        
        sqlx::query!(
            r#"
            UPDATE performanceResult
            SET run_id = ?, its = ?, avg_its = ?
            WHERE id = ?
            "#,
            entity.run_id,
            entity.its,
            entity.avg_its,
            id
        )
        .execute(&mut **tx)
        .await?;

        Ok(entity)
    }

    async fn delete_tx(&self, id: i64, tx: &mut Transaction<'a, Sqlite>) -> Result<(), Error> {
        sqlx::query!("DELETE FROM performanceResult WHERE id = ?", id)
            .execute(&mut **tx)
            .await?;
        Ok(())
    }
}

#[async_trait]
impl<'a> BulkTransactionRepository<'a, PerformanceResult, i64> for PerformanceResultRepository {
    async fn bulk_create_tx(&self, entities: Vec<PerformanceResult>, tx: &mut Transaction<'a, Sqlite>) -> Result<Vec<PerformanceResult>, Error> {
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

    async fn bulk_update_tx(&self, entities: Vec<PerformanceResult>, tx: &mut Transaction<'a, Sqlite>) -> Result<Vec<PerformanceResult>, Error> {
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
        let result = sqlx::query!("DELETE FROM performanceResult")
            .execute(&mut **tx)
            .await?;
        
        Ok(result.rows_affected() as usize)
    }
} 