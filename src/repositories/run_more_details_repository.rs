use async_trait::async_trait;
use sqlx::{Error, SqlitePool, Transaction, Sqlite};

use crate::models::run_more_details::RunMoreDetails;
use crate::repositories::traits::{Repository, TransactionRepository, BulkRepository, BulkTransactionRepository};

pub struct RunMoreDetailsRepository {
    pool: SqlitePool,
}

impl RunMoreDetailsRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Find run more details by run_id
    pub async fn find_by_run_id(&self, run_id: i64) -> Result<Vec<RunMoreDetails>, Error> {
        let results = sqlx::query_as!(
            RunMoreDetails,
            r#"
            SELECT id, run_id, timestamp, model_name, user, notes, ModelMapId as "model_map_id"
            FROM RunMoreDetails
            WHERE run_id = ?
            ORDER BY id DESC
            "#,
            run_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(results)
    }

    /// Find run more details by model_name
    pub async fn find_by_model_name(&self, model_name: &str) -> Result<Vec<RunMoreDetails>, Error> {
        let results = sqlx::query_as!(
            RunMoreDetails,
            r#"
            SELECT id, run_id, timestamp, model_name, user, notes, ModelMapId as "model_map_id"
            FROM RunMoreDetails
            WHERE model_name = ?
            ORDER BY id DESC
            "#,
            model_name
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(results)
    }

    /// Find run more details by user
    pub async fn find_by_user(&self, user: &str) -> Result<Vec<RunMoreDetails>, Error> {
        let results = sqlx::query_as!(
            RunMoreDetails,
            r#"
            SELECT id, run_id, timestamp, model_name, user, notes, ModelMapId as "model_map_id"
            FROM RunMoreDetails
            WHERE user = ?
            ORDER BY id DESC
            "#,
            user
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(results)
    }

    /// Clear all records from the RunMoreDetails table
    pub async fn clear_all(&self) -> Result<(), Error> {
        sqlx::query!("DELETE FROM RunMoreDetails")
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Clear all records from the RunMoreDetails table within a transaction
    pub async fn clear_all_tx(&self, tx: &mut Transaction<'_, Sqlite>) -> Result<(), Error> {
        sqlx::query!("DELETE FROM RunMoreDetails")
            .execute(&mut **tx)
            .await?;
        Ok(())
    }

    /// Find all RunMoreDetails records that don't have ModelMapId filled
    pub async fn find_without_modelmapid(&self) -> Result<Vec<RunMoreDetails>, Error> {
        let results = sqlx::query_as!(
            RunMoreDetails,
            r#"
            SELECT id, run_id, timestamp, model_name, user, notes, ModelMapId as "model_map_id"
            FROM RunMoreDetails
            WHERE ModelMapId IS NULL
            ORDER BY id DESC
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(results)
    }
}

#[async_trait]
impl Repository<RunMoreDetails, i64> for RunMoreDetailsRepository {
    async fn create(&self, entity: RunMoreDetails) -> Result<RunMoreDetails, Error> {
        let id = sqlx::query!(
            r#"
            INSERT INTO RunMoreDetails (run_id, timestamp, model_name, user, notes, ModelMapId)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
            entity.run_id,
            entity.timestamp,
            entity.model_name,
            entity.user,
            entity.notes,
            entity.model_map_id
        )
        .execute(&self.pool)
        .await?
        .last_insert_rowid() as i64;

        Ok(RunMoreDetails {
            id: Some(id),
            ..entity
        })
    }

    async fn find_by_id(&self, id: i64) -> Result<Option<RunMoreDetails>, Error> {
        let result = sqlx::query_as!(
            RunMoreDetails,
            r#"
            SELECT id, run_id, timestamp, model_name, user, notes, ModelMapId as "model_map_id"
            FROM RunMoreDetails
            WHERE id = ?
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(result)
    }

    async fn find_all(&self) -> Result<Vec<RunMoreDetails>, Error> {
        let results = sqlx::query_as!(
            RunMoreDetails,
            r#"
            SELECT id, run_id, timestamp, model_name, user, notes, ModelMapId as "model_map_id"
            FROM RunMoreDetails
            ORDER BY id DESC
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(results)
    }

    async fn update(&self, entity: RunMoreDetails) -> Result<RunMoreDetails, Error> {
        let id = entity.id.ok_or(Error::RowNotFound)?;
        
        sqlx::query!(
            r#"
            UPDATE RunMoreDetails
            SET run_id = ?, timestamp = ?, model_name = ?, user = ?, notes = ?, ModelMapId = ?
            WHERE id = ?
            "#,
            entity.run_id,
            entity.timestamp,
            entity.model_name,
            entity.user,
            entity.notes,
            entity.model_map_id,
            id
        )
        .execute(&self.pool)
        .await?;

        Ok(entity)
    }

    async fn delete(&self, id: i64) -> Result<(), Error> {
        sqlx::query!("DELETE FROM RunMoreDetails WHERE id = ?", id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn count(&self) -> Result<i64, Error> {
        let count = sqlx::query!("SELECT COUNT(*) as count FROM RunMoreDetails")
            .fetch_one(&self.pool)
            .await?
            .count;
        Ok(count)
    }
}

#[async_trait]
impl<'a> TransactionRepository<'a, RunMoreDetails, i64> for RunMoreDetailsRepository {
    async fn create_tx(&self, entity: RunMoreDetails, tx: &mut Transaction<'a, Sqlite>) -> Result<RunMoreDetails, Error> {
        let id = sqlx::query!(
            r#"
            INSERT INTO RunMoreDetails (run_id, timestamp, model_name, user, notes, ModelMapId)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
            entity.run_id,
            entity.timestamp,
            entity.model_name,
            entity.user,
            entity.notes,
            entity.model_map_id
        )
        .execute(&mut **tx)
        .await?
        .last_insert_rowid() as i64;

        Ok(RunMoreDetails {
            id: Some(id),
            ..entity
        })
    }

    async fn update_tx(&self, entity: RunMoreDetails, tx: &mut Transaction<'a, Sqlite>) -> Result<RunMoreDetails, Error> {
        let id = entity.id.ok_or(Error::RowNotFound)?;
        
        sqlx::query!(
            r#"
            UPDATE RunMoreDetails
            SET run_id = ?, timestamp = ?, model_name = ?, user = ?, notes = ?, ModelMapId = ?
            WHERE id = ?
            "#,
            entity.run_id,
            entity.timestamp,
            entity.model_name,
            entity.user,
            entity.notes,
            entity.model_map_id,
            id
        )
        .execute(&mut **tx)
        .await?;

        Ok(entity)
    }

    async fn delete_tx(&self, id: i64, tx: &mut Transaction<'a, Sqlite>) -> Result<(), Error> {
        sqlx::query!("DELETE FROM RunMoreDetails WHERE id = ?", id)
            .execute(&mut **tx)
            .await?;
        Ok(())
    }
} 

#[async_trait]
impl BulkRepository<RunMoreDetails, i64> for RunMoreDetailsRepository {
    async fn bulk_create(&self, entities: Vec<RunMoreDetails>) -> Result<Vec<RunMoreDetails>, Error> {
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

    async fn bulk_update(&self, entities: Vec<RunMoreDetails>) -> Result<Vec<RunMoreDetails>, Error> {
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
impl<'a> BulkTransactionRepository<'a, RunMoreDetails, i64> for RunMoreDetailsRepository {
    async fn bulk_create_tx(&self, entities: Vec<RunMoreDetails>, tx: &mut Transaction<'a, Sqlite>) -> Result<Vec<RunMoreDetails>, Error> {
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

    async fn bulk_update_tx(&self, entities: Vec<RunMoreDetails>, tx: &mut Transaction<'a, Sqlite>) -> Result<Vec<RunMoreDetails>, Error> {
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
        let result = sqlx::query!("DELETE FROM RunMoreDetails")
            .execute(&mut **tx)
            .await?;
        
        Ok(result.rows_affected() as usize)
    }
} 