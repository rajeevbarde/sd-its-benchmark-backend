use async_trait::async_trait;
use sqlx::{Error, SqlitePool, Transaction, Sqlite};

use crate::models::system_info::{SystemInfo, CreateSystemInfo};
use crate::repositories::traits::{Repository, TransactionRepository};

pub struct SystemInfoRepository {
    pool: SqlitePool,
}

impl SystemInfoRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Find system info by run_id
    pub async fn find_by_run_id(&self, run_id: i64) -> Result<Vec<SystemInfo>, Error> {
        let results = sqlx::query_as!(
            SystemInfo,
            r#"
            SELECT id, run_id, arch, cpu, system, release, python
            FROM SystemInfo
            WHERE run_id = ?
            ORDER BY id DESC
            "#,
            run_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(results)
    }

    /// Find system info by architecture
    pub async fn find_by_arch(&self, arch: &str) -> Result<Vec<SystemInfo>, Error> {
        let results = sqlx::query_as!(
            SystemInfo,
            r#"
            SELECT id, run_id, arch, cpu, system, release, python
            FROM SystemInfo
            WHERE arch = ?
            ORDER BY id DESC
            "#,
            arch
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(results)
    }

    /// Find system info by system type
    pub async fn find_by_system(&self, system: &str) -> Result<Vec<SystemInfo>, Error> {
        let results = sqlx::query_as!(
            SystemInfo,
            r#"
            SELECT id, run_id, arch, cpu, system, release, python
            FROM SystemInfo
            WHERE system = ?
            ORDER BY id DESC
            "#,
            system
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(results)
    }
}

#[async_trait]
impl Repository<SystemInfo, i64> for SystemInfoRepository {
    async fn create(&self, entity: SystemInfo) -> Result<SystemInfo, Error> {
        let id = sqlx::query!(
            r#"
            INSERT INTO SystemInfo (run_id, arch, cpu, system, release, python)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
            entity.run_id,
            entity.arch,
            entity.cpu,
            entity.system,
            entity.release,
            entity.python
        )
        .execute(&self.pool)
        .await?
        .last_insert_rowid() as i64;

        Ok(SystemInfo {
            id: Some(id),
            ..entity
        })
    }

    async fn find_by_id(&self, id: i64) -> Result<Option<SystemInfo>, Error> {
        let result = sqlx::query_as!(
            SystemInfo,
            r#"
            SELECT id, run_id, arch, cpu, system, release, python
            FROM SystemInfo
            WHERE id = ?
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(result)
    }

    async fn find_all(&self) -> Result<Vec<SystemInfo>, Error> {
        let results = sqlx::query_as!(
            SystemInfo,
            r#"
            SELECT id, run_id, arch, cpu, system, release, python
            FROM SystemInfo
            ORDER BY id DESC
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(results)
    }

    async fn update(&self, entity: SystemInfo) -> Result<SystemInfo, Error> {
        let id = entity.id.ok_or(Error::RowNotFound)?;
        
        sqlx::query!(
            r#"
            UPDATE SystemInfo
            SET run_id = ?, arch = ?, cpu = ?, system = ?, release = ?, python = ?
            WHERE id = ?
            "#,
            entity.run_id,
            entity.arch,
            entity.cpu,
            entity.system,
            entity.release,
            entity.python,
            id
        )
        .execute(&self.pool)
        .await?;

        Ok(entity)
    }

    async fn delete(&self, id: i64) -> Result<(), Error> {
        sqlx::query!("DELETE FROM SystemInfo WHERE id = ?", id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn count(&self) -> Result<i64, Error> {
        let count = sqlx::query!("SELECT COUNT(*) as count FROM SystemInfo")
            .fetch_one(&self.pool)
            .await?
            .count;
        Ok(count)
    }
}

#[async_trait]
impl<'a> TransactionRepository<'a, SystemInfo, i64> for SystemInfoRepository {
    async fn create_tx(&self, entity: SystemInfo, tx: &mut Transaction<'a, Sqlite>) -> Result<SystemInfo, Error> {
        let id = sqlx::query!(
            r#"
            INSERT INTO SystemInfo (run_id, arch, cpu, system, release, python)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
            entity.run_id,
            entity.arch,
            entity.cpu,
            entity.system,
            entity.release,
            entity.python
        )
        .execute(&mut **tx)
        .await?
        .last_insert_rowid() as i64;

        Ok(SystemInfo {
            id: Some(id),
            ..entity
        })
    }

    async fn update_tx(&self, entity: SystemInfo, tx: &mut Transaction<'a, Sqlite>) -> Result<SystemInfo, Error> {
        let id = entity.id.ok_or(Error::RowNotFound)?;
        
        sqlx::query!(
            r#"
            UPDATE SystemInfo
            SET run_id = ?, arch = ?, cpu = ?, system = ?, release = ?, python = ?
            WHERE id = ?
            "#,
            entity.run_id,
            entity.arch,
            entity.cpu,
            entity.system,
            entity.release,
            entity.python,
            id
        )
        .execute(&mut **tx)
        .await?;

        Ok(entity)
    }

    async fn delete_tx(&self, id: i64, tx: &mut Transaction<'a, Sqlite>) -> Result<(), Error> {
        sqlx::query!("DELETE FROM SystemInfo WHERE id = ?", id)
            .execute(&mut **tx)
            .await?;
        Ok(())
    }
} 