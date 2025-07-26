use async_trait::async_trait;
use sqlx::{Error, Transaction, Sqlite};

/// Base trait for CRUD operations on a repository.
#[async_trait]
pub trait Repository<T, Id> {
    async fn create(&self, entity: T) -> Result<T, Error>;
    async fn find_by_id(&self, id: Id) -> Result<Option<T>, Error>;
    async fn find_all(&self) -> Result<Vec<T>, Error>;
    async fn update(&self, entity: T) -> Result<T, Error>;
    async fn delete(&self, id: Id) -> Result<(), Error>;
    async fn count(&self) -> Result<i64, Error>;
}

/// Trait for repositories that support transactions.
#[async_trait]
pub trait TransactionRepository<'a, T, Id> {
    async fn create_tx(&self, entity: T, tx: &mut Transaction<'a, Sqlite>) -> Result<T, Error>;
    async fn update_tx(&self, entity: T, tx: &mut Transaction<'a, Sqlite>) -> Result<T, Error>;
    async fn delete_tx(&self, id: Id, tx: &mut Transaction<'a, Sqlite>) -> Result<(), Error>;
} 