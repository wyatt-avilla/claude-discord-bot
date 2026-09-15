use std::sync::Arc;
use std::{num::NonZeroU64, path::PathBuf};

use crate::claude::Model;

use super::record::Record;
use thiserror::Error;

use redb::{Database, ReadableTable, TableDefinition};

const TABLE: TableDefinition<u64, Record> = TableDefinition::new("claude_discord_bot");

#[derive(Debug, Error)]
pub enum DatabaseClientError {
    #[error("Couldn't create table ({0})")]
    FileCreation(Box<redb::DatabaseError>),

    #[error("Couldn't perform transaction ({0})")]
    Transaction(Box<redb::TransactionError>),

    #[error("Couldn't open table ({0})")]
    TableOpen(Box<redb::TableError>),

    #[error("Couldn't insert ({0})")]
    Write(Box<redb::StorageError>),

    #[error("Couldn't read ({0})")]
    Read(Box<redb::StorageError>),

    #[error("Couldn't commit transaction ({0})")]
    Commit(Box<redb::CommitError>),
}

#[derive(Clone)]
pub struct Client {
    db: Arc<Database>,
}

impl Client {
    pub fn new(db_path: &PathBuf) -> Result<Self, DatabaseClientError> {
        let db = redb::Database::create(db_path)
            .map_err(|error| DatabaseClientError::FileCreation(Box::new(error)))?;

        let write_txn = db
            .begin_write()
            .map_err(|error| DatabaseClientError::Transaction(Box::new(error)))?;
        {
            let _table = write_txn
                .open_table(TABLE)
                .map_err(|error| DatabaseClientError::TableOpen(Box::new(error)))?;
        }
        write_txn
            .commit()
            .map_err(|error| DatabaseClientError::Commit(Box::new(error)))?;

        Ok(Self { db: Arc::new(db) })
    }

    pub fn get_config(&self, server_id: u64) -> Result<Record, DatabaseClientError> {
        let read_txn = self
            .db
            .begin_read()
            .map_err(|error| DatabaseClientError::Transaction(Box::new(error)))?;
        let table = read_txn
            .open_table(TABLE)
            .map_err(|error| DatabaseClientError::TableOpen(Box::new(error)))?;

        Ok(table
            .get(server_id)
            .map_err(|error| DatabaseClientError::Read(Box::new(error)))?
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

    pub fn set_model(&self, server_id: u64, model: Model) -> Result<(), DatabaseClientError> {
        self.modify_config(server_id, move |rec| {
            rec.model = model;
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

    pub fn remove_active_channel_id(
        &self,
        server_id: u64,
        channel_id: u64,
    ) -> Result<(), DatabaseClientError> {
        self.modify_config(server_id, move |rec| {
            rec.active_channel_ids.remove(&channel_id);
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
            .map_err(|error| DatabaseClientError::Transaction(Box::new(error)))?;
        {
            let mut table = write_txn
                .open_table(TABLE)
                .map_err(|error| DatabaseClientError::TableOpen(Box::new(error)))?;

            let mut config = table
                .get(server_id)
                .map_err(|error| DatabaseClientError::Read(Box::new(error)))?
                .map_or(Record::default(), |v| v.value());
            update_config(&mut config);

            table
                .insert(server_id, config)
                .map_err(|error| DatabaseClientError::Write(Box::new(error)))?;
        }
        write_txn
            .commit()
            .map_err(|error| DatabaseClientError::Commit(Box::new(error)))?;

        Ok(())
    }
}
