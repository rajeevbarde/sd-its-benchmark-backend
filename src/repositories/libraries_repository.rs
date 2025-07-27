use async_trait::async_trait;
use sqlx::{Error, SqlitePool, Transaction, Sqlite};

use crate::models::libraries::Libraries;
use crate::repositories::traits::{Repository, TransactionRepository, BulkRepository, BulkTransactionRepository};

pub struct LibrariesRepository {
    pool: SqlitePool,
}

impl LibrariesRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Find libraries by run_id
    pub async fn find_by_run_id(&self, run_id: i64) -> Result<Vec<Libraries>, Error> {
        let results = sqlx::query_as!(
            Libraries,
            r#"
            SELECT id, run_id, torch, xformers, xformers1, diffusers, transformers
            FROM Libraries
            WHERE run_id = ?
            ORDER BY id DESC
            "#,
            run_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(results)
    }

    /// Clear all libraries records
    pub async fn clear_all(&self) -> Result<(), Error> {
        sqlx::query!("DELETE FROM Libraries")
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Clear all libraries records within a transaction
    pub async fn clear_all_tx(&self, tx: &mut Transaction<'_, Sqlite>) -> Result<(), Error> {
        sqlx::query!("DELETE FROM Libraries")
            .execute(&mut **tx)
            .await?;
        Ok(())
    }
}

#[async_trait]
impl Repository<Libraries, i64> for LibrariesRepository {
    async fn create(&self, entity: Libraries) -> Result<Libraries, Error> {
        let id = sqlx::query!(
            r#"
            INSERT INTO Libraries (run_id, torch, xformers, xformers1, diffusers, transformers)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
            entity.run_id,
            entity.torch,
            entity.xformers,
            entity.xformers1,
            entity.diffusers,
            entity.transformers
        )
        .execute(&self.pool)
        .await?
        .last_insert_rowid() as i64;

        Ok(Libraries {
            id: Some(id),
            ..entity
        })
    }

    async fn find_by_id(&self, id: i64) -> Result<Option<Libraries>, Error> {
        let result = sqlx::query_as!(
            Libraries,
            r#"
            SELECT id, run_id, torch, xformers, xformers1, diffusers, transformers
            FROM Libraries
            WHERE id = ?
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(result)
    }

    async fn find_all(&self) -> Result<Vec<Libraries>, Error> {
        let results = sqlx::query_as!(
            Libraries,
            r#"
            SELECT id, run_id, torch, xformers, xformers1, diffusers, transformers
            FROM Libraries
            ORDER BY id DESC
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(results)
    }

    async fn update(&self, entity: Libraries) -> Result<Libraries, Error> {
        let id = entity.id.ok_or(Error::RowNotFound)?;
        
        sqlx::query!(
            r#"
            UPDATE Libraries
            SET run_id = ?, torch = ?, xformers = ?, xformers1 = ?, diffusers = ?, transformers = ?
            WHERE id = ?
            "#,
            entity.run_id,
            entity.torch,
            entity.xformers,
            entity.xformers1,
            entity.diffusers,
            entity.transformers,
            id
        )
        .execute(&self.pool)
        .await?;

        Ok(entity)
    }

    async fn delete(&self, id: i64) -> Result<(), Error> {
        sqlx::query!("DELETE FROM Libraries WHERE id = ?", id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn count(&self) -> Result<i64, Error> {
        let count = sqlx::query!("SELECT COUNT(*) as count FROM Libraries")
            .fetch_one(&self.pool)
            .await?
            .count;
        Ok(count)
    }
}

#[async_trait]
impl<'a> TransactionRepository<'a, Libraries, i64> for LibrariesRepository {
    async fn create_tx(&self, entity: Libraries, tx: &mut Transaction<'a, Sqlite>) -> Result<Libraries, Error> {
        let id = sqlx::query!(
            r#"
            INSERT INTO Libraries (run_id, torch, xformers, xformers1, diffusers, transformers)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
            entity.run_id,
            entity.torch,
            entity.xformers,
            entity.xformers1,
            entity.diffusers,
            entity.transformers
        )
        .execute(&mut **tx)
        .await?
        .last_insert_rowid() as i64;

        Ok(Libraries {
            id: Some(id),
            ..entity
        })
    }

    async fn update_tx(&self, entity: Libraries, tx: &mut Transaction<'a, Sqlite>) -> Result<Libraries, Error> {
        let id = entity.id.ok_or(Error::RowNotFound)?;
        
        sqlx::query!(
            r#"
            UPDATE Libraries
            SET run_id = ?, torch = ?, xformers = ?, xformers1 = ?, diffusers = ?, transformers = ?
            WHERE id = ?
            "#,
            entity.run_id,
            entity.torch,
            entity.xformers,
            entity.xformers1,
            entity.diffusers,
            entity.transformers,
            id
        )
        .execute(&mut **tx)
        .await?;

        Ok(entity)
    }

    async fn delete_tx(&self, id: i64, tx: &mut Transaction<'a, Sqlite>) -> Result<(), Error> {
        sqlx::query!("DELETE FROM Libraries WHERE id = ?", id)
            .execute(&mut **tx)
            .await?;
        Ok(())
    }
} 

#[async_trait]
impl BulkRepository<Libraries, i64> for LibrariesRepository {
    async fn bulk_create(&self, entities: Vec<Libraries>) -> Result<Vec<Libraries>, Error> {
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

    async fn bulk_update(&self, entities: Vec<Libraries>) -> Result<Vec<Libraries>, Error> {
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
impl<'a> BulkTransactionRepository<'a, Libraries, i64> for LibrariesRepository {
    async fn bulk_create_tx(&self, entities: Vec<Libraries>, tx: &mut Transaction<'a, Sqlite>) -> Result<Vec<Libraries>, Error> {
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

    async fn bulk_update_tx(&self, entities: Vec<Libraries>, tx: &mut Transaction<'a, Sqlite>) -> Result<Vec<Libraries>, Error> {
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
        let result = sqlx::query!("DELETE FROM Libraries")
            .execute(&mut **tx)
            .await?;
        
        Ok(result.rows_affected() as usize)
    }
} 