use async_trait::async_trait;
use sqlx::{Error, SqlitePool, Transaction, Sqlite};

use crate::models::app_details::AppDetails;
use crate::repositories::traits::{Repository, TransactionRepository, BulkRepository, BulkTransactionRepository};

pub struct AppDetailsRepository {
    pool: SqlitePool,
}

impl AppDetailsRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Find app details by run_id
    pub async fn find_by_run_id(&self, run_id: i64) -> Result<Vec<AppDetails>, Error> {
        let results = sqlx::query_as!(
            AppDetails,
            r#"
            SELECT id, run_id, app_name, updated, hash, url
            FROM AppDetails
            WHERE run_id = ?
            ORDER BY id DESC
            "#,
            run_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(results)
    }

    /// Find app details by app_name
    pub async fn find_by_app_name(&self, app_name: &str) -> Result<Vec<AppDetails>, Error> {
        let results = sqlx::query_as!(
            AppDetails,
            r#"
            SELECT id, run_id, app_name, updated, hash, url
            FROM AppDetails
            WHERE app_name = ?
            ORDER BY id DESC
            "#,
            app_name
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(results)
    }

    /// Clear all app details
    pub async fn clear_all(&self) -> Result<(), Error> {
        sqlx::query!("DELETE FROM AppDetails")
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Clear all app details within a transaction
    pub async fn clear_all_tx(&self, tx: &mut Transaction<'_, Sqlite>) -> Result<(), Error> {
        sqlx::query!("DELETE FROM AppDetails")
            .execute(&mut **tx)
            .await?;
        Ok(())
    }

    /// Count records where both app_name and url are NULL
    pub async fn count_null_app_name_null_url(&self) -> Result<i64, Error> {
        let count = sqlx::query!(
            r#"
            SELECT COUNT(*) as count
            FROM AppDetails
            WHERE app_name IS NULL AND url IS NULL
            "#
        )
        .fetch_one(&self.pool)
        .await?
        .count;

        Ok(count)
    }

    /// Count records where app_name is NULL but url is NOT NULL
    pub async fn count_null_app_name_non_null_url(&self) -> Result<i64, Error> {
        let count = sqlx::query!(
            r#"
            SELECT COUNT(*) as count
            FROM AppDetails
            WHERE app_name IS NULL AND url IS NOT NULL
            "#
        )
        .fetch_one(&self.pool)
        .await?
        .count;

        Ok(count)
    }

    /// Update app names for AUTOMATIC1111 URLs
    pub async fn update_automatic1111_names(&self, app_name: &str) -> Result<i64, Error> {
        let result = sqlx::query!(
            r#"
            UPDATE AppDetails
            SET app_name = ?
            WHERE url LIKE '%AUTOMATIC1111%'
            "#,
            app_name
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() as i64)
    }

    /// Update app names for vladmandic URLs (only if app_name is NULL or empty)
    pub async fn update_vladmandic_names(&self, app_name: &str) -> Result<i64, Error> {
        let result = sqlx::query!(
            r#"
            UPDATE AppDetails
            SET app_name = ?
            WHERE url LIKE '%vladmandic%' AND (app_name IS NULL OR app_name = '')
            "#,
            app_name
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() as i64)
    }

    /// Update app names for stable-diffusion-webui URLs (only if app_name is NULL)
    pub async fn update_stable_diffusion_names(&self, app_name: &str) -> Result<i64, Error> {
        let result = sqlx::query!(
            r#"
            UPDATE AppDetails
            SET app_name = ?
            WHERE url LIKE '%stable-diffusion-webui%' AND app_name IS NULL
            "#,
            app_name
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() as i64)
    }

    /// Update app names for records with both app_name and url as NULL
    pub async fn update_null_app_name_null_url_names(&self, app_name: &str) -> Result<i64, Error> {
        let result = sqlx::query!(
            r#"
            UPDATE AppDetails
            SET app_name = ?
            WHERE app_name IS NULL AND url IS NULL
            "#,
            app_name
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() as i64)
    }
}

#[async_trait]
impl Repository<AppDetails, i64> for AppDetailsRepository {
    async fn create(&self, entity: AppDetails) -> Result<AppDetails, Error> {
        let id = sqlx::query!(
            r#"
            INSERT INTO AppDetails (run_id, app_name, updated, hash, url)
            VALUES (?, ?, ?, ?, ?)
            "#,
            entity.run_id,
            entity.app_name,
            entity.updated,
            entity.hash,
            entity.url
        )
        .execute(&self.pool)
        .await?
        .last_insert_rowid() as i64;

        Ok(AppDetails {
            id: Some(id),
            ..entity
        })
    }

    async fn find_by_id(&self, id: i64) -> Result<Option<AppDetails>, Error> {
        let result = sqlx::query_as!(
            AppDetails,
            r#"
            SELECT id, run_id, app_name, updated, hash, url
            FROM AppDetails
            WHERE id = ?
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(result)
    }

    async fn find_all(&self) -> Result<Vec<AppDetails>, Error> {
        let results = sqlx::query_as!(
            AppDetails,
            r#"
            SELECT id, run_id, app_name, updated, hash, url
            FROM AppDetails
            ORDER BY id DESC
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(results)
    }

    async fn update(&self, entity: AppDetails) -> Result<AppDetails, Error> {
        let id = entity.id.ok_or(Error::RowNotFound)?;
        
        sqlx::query!(
            r#"
            UPDATE AppDetails
            SET run_id = ?, app_name = ?, updated = ?, hash = ?, url = ?
            WHERE id = ?
            "#,
            entity.run_id,
            entity.app_name,
            entity.updated,
            entity.hash,
            entity.url,
            id
        )
        .execute(&self.pool)
        .await?;

        Ok(entity)
    }

    async fn delete(&self, id: i64) -> Result<(), Error> {
        sqlx::query!("DELETE FROM AppDetails WHERE id = ?", id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn count(&self) -> Result<i64, Error> {
        let count = sqlx::query!("SELECT COUNT(*) as count FROM AppDetails")
            .fetch_one(&self.pool)
            .await?
            .count;
        Ok(count)
    }
}

#[async_trait]
impl<'a> TransactionRepository<'a, AppDetails, i64> for AppDetailsRepository {
    async fn create_tx(&self, entity: AppDetails, tx: &mut Transaction<'a, Sqlite>) -> Result<AppDetails, Error> {
        let id = sqlx::query!(
            r#"
            INSERT INTO AppDetails (run_id, app_name, updated, hash, url)
            VALUES (?, ?, ?, ?, ?)
            "#,
            entity.run_id,
            entity.app_name,
            entity.updated,
            entity.hash,
            entity.url
        )
        .execute(&mut **tx)
        .await?
        .last_insert_rowid() as i64;

        Ok(AppDetails {
            id: Some(id),
            ..entity
        })
    }

    async fn update_tx(&self, entity: AppDetails, tx: &mut Transaction<'a, Sqlite>) -> Result<AppDetails, Error> {
        let id = entity.id.ok_or(Error::RowNotFound)?;
        
        sqlx::query!(
            r#"
            UPDATE AppDetails
            SET run_id = ?, app_name = ?, updated = ?, hash = ?, url = ?
            WHERE id = ?
            "#,
            entity.run_id,
            entity.app_name,
            entity.updated,
            entity.hash,
            entity.url,
            id
        )
        .execute(&mut **tx)
        .await?;

        Ok(entity)
    }

    async fn delete_tx(&self, id: i64, tx: &mut Transaction<'a, Sqlite>) -> Result<(), Error> {
        sqlx::query!("DELETE FROM AppDetails WHERE id = ?", id)
            .execute(&mut **tx)
            .await?;
        Ok(())
    }
} 

#[async_trait]
impl BulkRepository<AppDetails, i64> for AppDetailsRepository {
    async fn bulk_create(&self, entities: Vec<AppDetails>) -> Result<Vec<AppDetails>, Error> {
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

    async fn bulk_update(&self, entities: Vec<AppDetails>) -> Result<Vec<AppDetails>, Error> {
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
impl<'a> BulkTransactionRepository<'a, AppDetails, i64> for AppDetailsRepository {
    async fn bulk_create_tx(&self, entities: Vec<AppDetails>, tx: &mut Transaction<'a, Sqlite>) -> Result<Vec<AppDetails>, Error> {
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

    async fn bulk_update_tx(&self, entities: Vec<AppDetails>, tx: &mut Transaction<'a, Sqlite>) -> Result<Vec<AppDetails>, Error> {
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
        let result = sqlx::query!("DELETE FROM AppDetails")
            .execute(&mut **tx)
            .await?;
        
        Ok(result.rows_affected() as usize)
    }
} 