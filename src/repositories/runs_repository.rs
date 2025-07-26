use async_trait::async_trait;
use sqlx::{Error, SqlitePool, Transaction, Sqlite};

use crate::models::runs::Run;
use crate::repositories::traits::{Repository, TransactionRepository};

pub struct RunsRepository {
    pool: SqlitePool,
}

impl RunsRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl Repository<Run, i64> for RunsRepository {
    async fn create(&self, entity: Run) -> Result<Run, Error> {
        let id = sqlx::query!(
            r#"
            INSERT INTO runs (timestamp, vram_usage, info, system_info, model_info, device_info, xformers, model_name, user, notes)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            entity.timestamp,
            entity.vram_usage,
            entity.info,
            entity.system_info,
            entity.model_info,
            entity.device_info,
            entity.xformers,
            entity.model_name,
            entity.user,
            entity.notes
        )
        .execute(&self.pool)
        .await?
        .last_insert_rowid() as i64;

        Ok(Run {
            id: Some(id),
            ..entity
        })
    }

    async fn find_by_id(&self, id: i64) -> Result<Option<Run>, Error> {
        let run = sqlx::query_as!(
            Run,
            r#"
            SELECT id, timestamp, vram_usage, info, system_info, model_info, device_info, xformers, model_name, user, notes
            FROM runs
            WHERE id = ?
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(run)
    }

    async fn find_all(&self) -> Result<Vec<Run>, Error> {
        let runs = sqlx::query_as!(
            Run,
            r#"
            SELECT id, timestamp, vram_usage, info, system_info, model_info, device_info, xformers, model_name, user, notes
            FROM runs
            ORDER BY id DESC
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(runs)
    }

    async fn update(&self, entity: Run) -> Result<Run, Error> {
        let id = entity.id.ok_or(Error::RowNotFound)?;
        
        sqlx::query!(
            r#"
            UPDATE runs
            SET timestamp = ?, vram_usage = ?, info = ?, system_info = ?, model_info = ?, device_info = ?, xformers = ?, model_name = ?, user = ?, notes = ?
            WHERE id = ?
            "#,
            entity.timestamp,
            entity.vram_usage,
            entity.info,
            entity.system_info,
            entity.model_info,
            entity.device_info,
            entity.xformers,
            entity.model_name,
            entity.user,
            entity.notes,
            id
        )
        .execute(&self.pool)
        .await?;

        Ok(entity)
    }

    async fn delete(&self, id: i64) -> Result<(), Error> {
        sqlx::query!("DELETE FROM runs WHERE id = ?", id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn count(&self) -> Result<i64, Error> {
        let count = sqlx::query!("SELECT COUNT(*) as count FROM runs")
            .fetch_one(&self.pool)
            .await?
            .count;
        Ok(count)
    }
}

#[async_trait]
impl<'a> TransactionRepository<'a, Run, i64> for RunsRepository {
    async fn create_tx(&self, entity: Run, tx: &mut Transaction<'a, Sqlite>) -> Result<Run, Error> {
        let id = sqlx::query!(
            r#"
            INSERT INTO runs (timestamp, vram_usage, info, system_info, model_info, device_info, xformers, model_name, user, notes)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            entity.timestamp,
            entity.vram_usage,
            entity.info,
            entity.system_info,
            entity.model_info,
            entity.device_info,
            entity.xformers,
            entity.model_name,
            entity.user,
            entity.notes
        )
        .execute(&mut **tx)
        .await?
        .last_insert_rowid() as i64;

        Ok(Run {
            id: Some(id),
            ..entity
        })
    }

    async fn update_tx(&self, entity: Run, tx: &mut Transaction<'a, Sqlite>) -> Result<Run, Error> {
        let id = entity.id.ok_or(Error::RowNotFound)?;
        
        sqlx::query!(
            r#"
            UPDATE runs
            SET timestamp = ?, vram_usage = ?, info = ?, system_info = ?, model_info = ?, device_info = ?, xformers = ?, model_name = ?, user = ?, notes = ?
            WHERE id = ?
            "#,
            entity.timestamp,
            entity.vram_usage,
            entity.info,
            entity.system_info,
            entity.model_info,
            entity.device_info,
            entity.xformers,
            entity.model_name,
            entity.user,
            entity.notes,
            id
        )
        .execute(&mut **tx)
        .await?;

        Ok(entity)
    }

    async fn delete_tx(&self, id: i64, tx: &mut Transaction<'a, Sqlite>) -> Result<(), Error> {
        sqlx::query!("DELETE FROM runs WHERE id = ?", id)
            .execute(&mut **tx)
            .await?;
        Ok(())
    }
} 