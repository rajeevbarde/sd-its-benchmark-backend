use async_trait::async_trait;
use sqlx::{Error, SqlitePool, Transaction, Sqlite};

use crate::models::app_details::AppDetails;
use crate::repositories::traits::{Repository, TransactionRepository};

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