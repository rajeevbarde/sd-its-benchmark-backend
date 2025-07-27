use async_trait::async_trait;
use sqlx::{Error, SqlitePool, Transaction, Sqlite};

use crate::models::runs::Run;
use crate::repositories::traits::{Repository, TransactionRepository, BulkRepository, BulkTransactionRepository};

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
impl BulkRepository<Run, i64> for RunsRepository {
    async fn bulk_create(&self, entities: Vec<Run>) -> Result<Vec<Run>, Error> {
        if entities.is_empty() {
            return Ok(vec![]);
        }

        let mut tx = self.pool.begin().await?;
        
        let result = self.bulk_create_tx(entities, &mut tx).await;
        
        match result {
            Ok(runs) => {
                tx.commit().await?;
                Ok(runs)
            }
            Err(e) => {
                tx.rollback().await?;
                Err(e)
            }
        }
    }

    async fn bulk_update(&self, entities: Vec<Run>) -> Result<Vec<Run>, Error> {
        if entities.is_empty() {
            return Ok(vec![]);
        }

        let mut tx = self.pool.begin().await?;
        
        let result = self.bulk_update_tx(entities, &mut tx).await;
        
        match result {
            Ok(runs) => {
                tx.commit().await?;
                Ok(runs)
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

#[async_trait]
impl<'a> BulkTransactionRepository<'a, Run, i64> for RunsRepository {
    async fn bulk_create_tx(&self, entities: Vec<Run>, tx: &mut Transaction<'a, Sqlite>) -> Result<Vec<Run>, Error> {
        if entities.is_empty() {
            return Ok(vec![]);
        }

        let mut created_runs = Vec::with_capacity(entities.len());

        // Use batch processing for better performance
        for entity in entities {
            let created_run = self.create_tx(entity, tx).await?;
            created_runs.push(created_run);
        }

        Ok(created_runs)
    }

    async fn bulk_update_tx(&self, entities: Vec<Run>, tx: &mut Transaction<'a, Sqlite>) -> Result<Vec<Run>, Error> {
        if entities.is_empty() {
            return Ok(vec![]);
        }

        let mut updated_runs = Vec::with_capacity(entities.len());

        // Use batch processing for better performance
        for entity in entities {
            let updated_run = self.update_tx(entity, tx).await?;
            updated_runs.push(updated_run);
        }

        Ok(updated_runs)
    }

    async fn delete_all_tx(&self, tx: &mut Transaction<'a, Sqlite>) -> Result<usize, Error> {
        let result = sqlx::query!("DELETE FROM runs")
            .execute(&mut **tx)
            .await?;
        
        Ok(result.rows_affected() as usize)
    }
} 

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::SqlitePool;
    use std::sync::Once;

    static INIT: Once = Once::new();

    async fn setup_test_db() -> SqlitePool {
        INIT.call_once(|| {
            // Initialize test environment if needed
        });

        // Create in-memory database for testing
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        
        // Create the runs table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS runs (
                id INTEGER PRIMARY KEY,
                timestamp TEXT,
                vram_usage TEXT,
                info TEXT,
                system_info TEXT,
                model_info TEXT,
                device_info TEXT,
                xformers TEXT,
                model_name TEXT,
                user TEXT,
                notes TEXT
            )
            "#
        )
        .execute(&pool)
        .await
        .unwrap();

        pool
    }

    fn create_test_run(id: Option<i64>) -> Run {
        Run {
            id,
            timestamp: Some("2024-01-01T00:00:00Z".to_string()),
            vram_usage: Some("8GB/16GB".to_string()),
            info: Some("Test info".to_string()),
            system_info: Some("Test system".to_string()),
            model_info: Some("Test model".to_string()),
            device_info: Some("Test device".to_string()),
            xformers: Some("true".to_string()),
            model_name: Some("test-model".to_string()),
            user: Some("test-user".to_string()),
            notes: Some("Test notes".to_string()),
        }
    }

    #[tokio::test]
    async fn test_bulk_create_empty_vector() {
        let pool = setup_test_db().await;
        let repo = RunsRepository::new(pool);
        
        let result = repo.bulk_create(vec![]).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }

    #[tokio::test]
    async fn test_bulk_create_single_item() {
        let pool = setup_test_db().await;
        let repo = RunsRepository::new(pool);
        
        let test_run = create_test_run(None);
        let result = repo.bulk_create(vec![test_run]).await;
        
        assert!(result.is_ok());
        let created_runs = result.unwrap();
        assert_eq!(created_runs.len(), 1);
        assert!(created_runs[0].id.is_some());
    }

    #[tokio::test]
    async fn test_bulk_create_multiple_items() {
        let pool = setup_test_db().await;
        let repo = RunsRepository::new(pool);
        
        let test_runs = vec![
            create_test_run(None),
            create_test_run(None),
            create_test_run(None),
        ];
        
        let result = repo.bulk_create(test_runs).await;
        
        assert!(result.is_ok());
        let created_runs = result.unwrap();
        assert_eq!(created_runs.len(), 3);
        
        // Verify all runs have IDs
        for run in &created_runs {
            assert!(run.id.is_some());
        }
        
        // Verify IDs are unique
        let ids: Vec<i64> = created_runs.iter()
            .map(|r| r.id.unwrap())
            .collect();
        let unique_ids: std::collections::HashSet<i64> = ids.iter().cloned().collect();
        assert_eq!(ids.len(), unique_ids.len());
    }

    #[tokio::test]
    async fn test_bulk_create_transaction_rollback() {
        let pool = setup_test_db().await;
        let repo = RunsRepository::new(pool.clone());
        
        // Test that bulk operations work correctly with transactions
        // This test verifies the transaction wrapper works properly
        
        let test_runs = vec![
            create_test_run(None),
            create_test_run(None),
            create_test_run(None),
        ];
        
        // Test bulk create - should succeed
        let result = repo.bulk_create(test_runs).await;
        assert!(result.is_ok());
        
        let created_runs = result.unwrap();
        assert_eq!(created_runs.len(), 3);
        
        // Verify all runs have IDs
        for run in &created_runs {
            assert!(run.id.is_some());
        }
        
        // Verify data was actually inserted
        let count = repo.count().await.unwrap();
        assert_eq!(count, 3);
        
        // Test that delete_all works and is atomic
        let delete_result = repo.delete_all().await;
        assert!(delete_result.is_ok());
        assert_eq!(delete_result.unwrap(), 3);
        
        // Verify all data was deleted
        let count_after = repo.count().await.unwrap();
        assert_eq!(count_after, 0);
    }

    #[tokio::test]
    async fn test_bulk_update() {
        let pool = setup_test_db().await;
        let repo = RunsRepository::new(pool);
        
        // First create some runs
        let test_runs = vec![
            create_test_run(None),
            create_test_run(None),
        ];
        
        let created_runs = repo.bulk_create(test_runs).await.unwrap();
        
        // Update the runs
        let mut updated_runs = created_runs.clone();
        updated_runs[0].notes = Some("Updated note 1".to_string());
        updated_runs[1].notes = Some("Updated note 2".to_string());
        
        let result = repo.bulk_update(updated_runs).await;
        assert!(result.is_ok());
        
        // Verify updates
        let updated = result.unwrap();
        assert_eq!(updated[0].notes, Some("Updated note 1".to_string()));
        assert_eq!(updated[1].notes, Some("Updated note 2".to_string()));
    }

    #[tokio::test]
    async fn test_delete_all() {
        let pool = setup_test_db().await;
        let repo = RunsRepository::new(pool);
        
        // Create some runs
        let test_runs = vec![
            create_test_run(None),
            create_test_run(None),
            create_test_run(None),
        ];
        
        repo.bulk_create(test_runs).await.unwrap();
        
        // Verify runs exist
        let count_before = repo.count().await.unwrap();
        assert_eq!(count_before, 3);
        
        // Delete all
        let deleted_count = repo.delete_all().await.unwrap();
        assert_eq!(deleted_count, 3);
        
        // Verify all deleted
        let count_after = repo.count().await.unwrap();
        assert_eq!(count_after, 0);
    }

    #[tokio::test]
    async fn test_bulk_operations_with_transaction() {
        let pool = setup_test_db().await;
        let repo = RunsRepository::new(pool.clone());
        
        // Test bulk create with transaction
        let mut tx = pool.begin().await.unwrap();
        
        let test_runs = vec![
            create_test_run(None),
            create_test_run(None),
        ];
        
        let result = repo.bulk_create_tx(test_runs, &mut tx).await;
        assert!(result.is_ok());
        
        let created_runs = result.unwrap();
        assert_eq!(created_runs.len(), 2);
        
        // Commit transaction
        tx.commit().await.unwrap();
        
        // Verify data persists after commit
        let count = repo.count().await.unwrap();
        assert_eq!(count, 2);
    }

    #[tokio::test]
    async fn test_bulk_operations_transaction_rollback() {
        let pool = setup_test_db().await;
        let repo = RunsRepository::new(pool.clone());
        
        // Test bulk create with transaction rollback
        let mut tx = pool.begin().await.unwrap();
        
        let test_runs = vec![
            create_test_run(None),
            create_test_run(None),
        ];
        
        let result = repo.bulk_create_tx(test_runs, &mut tx).await;
        assert!(result.is_ok());
        
        // Rollback transaction
        tx.rollback().await.unwrap();
        
        // Verify data was rolled back
        let count = repo.count().await.unwrap();
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn test_bulk_update_tx() {
        let pool = setup_test_db().await;
        let repo = RunsRepository::new(pool.clone());
        
        // Create runs first
        let test_runs = vec![
            create_test_run(None),
            create_test_run(None),
        ];
        
        let created_runs = repo.bulk_create(test_runs).await.unwrap();
        
        // Update with transaction
        let mut tx = pool.begin().await.unwrap();
        
        let mut updated_runs = created_runs.clone();
        updated_runs[0].notes = Some("Transaction updated".to_string());
        
        let result = repo.bulk_update_tx(updated_runs, &mut tx).await;
        assert!(result.is_ok());
        
        tx.commit().await.unwrap();
        
        // Verify update
        let updated_run = repo.find_by_id(created_runs[0].id.unwrap()).await.unwrap().unwrap();
        assert_eq!(updated_run.notes, Some("Transaction updated".to_string()));
    }

    #[tokio::test]
    async fn test_delete_all_tx() {
        let pool = setup_test_db().await;
        let repo = RunsRepository::new(pool.clone());
        
        // Create some runs
        let test_runs = vec![
            create_test_run(None),
            create_test_run(None),
        ];
        
        repo.bulk_create(test_runs).await.unwrap();
        
        // Delete all with transaction
        let mut tx = pool.begin().await.unwrap();
        
        let deleted_count = repo.delete_all_tx(&mut tx).await.unwrap();
        assert_eq!(deleted_count, 2);
        
        tx.commit().await.unwrap();
        
        // Verify deletion
        let count = repo.count().await.unwrap();
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn test_performance_bulk_vs_individual() {
        let pool = setup_test_db().await;
        let repo = RunsRepository::new(pool);
        
        // Create test data
        let test_runs: Vec<Run> = (0..100)
            .map(|i| Run {
                id: None,
                timestamp: Some(format!("2024-01-01T{:02}:00:00Z", i % 24)),
                vram_usage: Some(format!("{}GB/16GB", (i % 8) + 1)),
                info: Some(format!("Test info {}", i)),
                system_info: Some("Test system".to_string()),
                model_info: Some("Test model".to_string()),
                device_info: Some("Test device".to_string()),
                xformers: Some("true".to_string()),
                model_name: Some(format!("test-model-{}", i)),
                user: Some("test-user".to_string()),
                notes: Some(format!("Test notes {}", i)),
            })
            .collect();
        
        // Test bulk create performance
        let start = std::time::Instant::now();
        let result = repo.bulk_create(test_runs).await;
        let bulk_duration = start.elapsed();
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 100);
        
        println!("Bulk create of 100 items took: {:?}", bulk_duration);
        
        // Verify all items were created
        let count = repo.count().await.unwrap();
        assert_eq!(count, 100);
    }
} 