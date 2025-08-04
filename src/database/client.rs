#![allow(clippy::result_large_err)]

use super::record::Record;
use thiserror::Error;

use redb::{Database, TableDefinition};

const REDB_FILE_NAME: &str = "claude_discord_bot.redb";
const TABLE: TableDefinition<u64, Record> = TableDefinition::new("claude_discord_bot");

#[derive(Debug, Error)]
pub enum DatabaseClientError {
    #[error("Couldn't create table ({0})")]
    FileCreation(redb::DatabaseError),

    #[error("Couldn't perform transaction ({0})")]
    Transaction(redb::TransactionError),

    #[error("Couldn't open table ({0})")]
    TableOpen(redb::TableError),

    #[error("Couldn't insert ({0})")]
    Insert(redb::StorageError),

    #[error("Couldn't read ({0})")]
    Read(redb::StorageError),

    #[error("Couldn't commit transaction ({0})")]
    Commit(redb::CommitError),
}

pub struct Client {
    db: Database,
}

impl Client {
    pub fn new() -> Result<Self, DatabaseClientError> {
        let db =
            redb::Database::create(REDB_FILE_NAME).map_err(DatabaseClientError::FileCreation)?;

        let write_txn = db.begin_write().map_err(DatabaseClientError::Transaction)?;
        {
            let _table = write_txn
                .open_table(TABLE)
                .map_err(DatabaseClientError::TableOpen)?;
        }
        write_txn.commit().map_err(DatabaseClientError::Commit)?;

        Ok(Self { db })
    }

    pub fn get_config(&self, server_id: u64) -> Result<Record, DatabaseClientError> {
        let read_txn = self
            .db
            .begin_read()
            .map_err(DatabaseClientError::Transaction)?;
        let table = read_txn
            .open_table(TABLE)
            .map_err(DatabaseClientError::TableOpen)?;

        Ok(table
            .get(server_id)
            .map_err(DatabaseClientError::Read)?
            .map_or(Record::default(), |a| a.value()))
    }

    pub fn set_config(&self, server_id: u64, config: &Record) -> Result<(), DatabaseClientError> {
        let write_txn = self
            .db
            .begin_write()
            .map_err(DatabaseClientError::Transaction)?;
        {
            let mut table = write_txn
                .open_table(TABLE)
                .map_err(DatabaseClientError::TableOpen)?;
            table
                .insert(server_id, config)
                .map_err(DatabaseClientError::Insert)?;
        }
        write_txn.commit().map_err(DatabaseClientError::Commit)?;

        Ok(())
    }
}
