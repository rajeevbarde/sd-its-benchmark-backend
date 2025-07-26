use sqlx::{SqlitePool, Transaction, Sqlite, Error};

/// Wrapper for managing SQLx Sqlite transactions.
pub struct DatabaseTransaction<'a> {
    pub tx: Transaction<'a, Sqlite>,
}

impl<'a> DatabaseTransaction<'a> {
    /// Begin a new transaction from the pool
    pub async fn begin(pool: &'a SqlitePool) -> Result<Self, Error> {
        let tx = pool.begin().await?;
        Ok(Self { tx })
    }

    /// Commit the transaction
    pub async fn commit(self) -> Result<(), Error> {
        self.tx.commit().await
    }

    /// Rollback the transaction
    pub async fn rollback(self) -> Result<(), Error> {
        self.tx.rollback().await
    }

    /// Get a mutable reference to the underlying transaction
    pub fn as_mut(&mut self) -> &mut Transaction<'a, Sqlite> {
        &mut self.tx
    }
} 