#![allow(clippy::result_large_err)]

use std::num::NonZeroU64;

use super::record::Record;
use thiserror::Error;

use redb::{Database, ReadableTable, TableDefinition};

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
    Write(redb::StorageError),

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

    pub fn set_claude_api_key(
        &self,
        server_id: u64,
        api_key: &str,
    ) -> Result<(), DatabaseClientError> {
        self.modify_config(server_id, move |rec| {
            rec.claude_api_key = Some(api_key.to_string());
        })
    }

    pub fn set_random_interaction_denominator(
        &self,
        server_id: u64,
        denominator: Option<NonZeroU64>,
    ) -> Result<(), DatabaseClientError> {
        self.modify_config(server_id, move |rec| {
            rec.random_interaction_chance_denominator = denominator;
        })
    }

    pub fn add_active_channel_id(
        &self,
        server_id: u64,
        channel_id: u64,
    ) -> Result<(), DatabaseClientError> {
        self.modify_config(server_id, move |rec| {
            rec.active_channel_ids.insert(channel_id);
        })
    }

    pub fn clear_active_channel_ids(&self, server_id: u64) -> Result<(), DatabaseClientError> {
        self.modify_config(server_id, move |rec| {
            rec.active_channel_ids.clear();
        })
    }

    fn modify_config<F>(&self, server_id: u64, update_config: F) -> Result<(), DatabaseClientError>
    where
        F: FnOnce(&mut Record),
    {
        let write_txn = self
            .db
            .begin_write()
            .map_err(DatabaseClientError::Transaction)?;
        {
            let mut table = write_txn
                .open_table(TABLE)
                .map_err(DatabaseClientError::TableOpen)?;

            let mut config = table
                .get(server_id)
                .map_err(DatabaseClientError::Read)?
                .map_or(Record::default(), |v| v.value());
            update_config(&mut config);

            table
                .insert(server_id, config)
                .map_err(DatabaseClientError::Write)?;
        }
        write_txn.commit().map_err(DatabaseClientError::Commit)?;

        Ok(())
    }
}
