use std::fmt::Debug;

use anyhow::Result;
use async_trait::async_trait;
use dyn_clone::{DynClone, clone_trait_object};
use serde::Serialize;
use specta::Type;
use thiserror::Error;
use tokio::sync::mpsc::Sender;

use crate::{jellyfin::jellyfin_indexer::JellyfinIndexerError, provider_manager::IndexerMsg};
use journey_db::{entity::providers, sea_orm::DatabaseTransaction};

#[derive(Debug, Error, Serialize, Type)]
pub enum IndexerError {
    #[error("Failed to parse the given String to an Url.")]
    FailedParseUrlError(String),
    #[error("Failed to retrieve Jellyfin API response entry.")]
    ApiEntryRetrievalError(Option<String>),
    #[error("Failed to insert provider to database.")]
    FailedDbInsertError(String),
    #[error(transparent)]
    JellyfinIndexerError(#[from] JellyfinIndexerError),
    #[error(transparent)]
    JourneyDbError(#[from] journey_db::JourneyDbError),
}

pub type IndexerResult<T> = Result<T, IndexerError>;

pub trait NewIndexer {
    type Indexer;
    type Config;

    fn new(model: providers::ActiveModelEx, config: Option<Self::Config>) -> Box<Self::Indexer>;
}

#[async_trait]
pub trait RequiredForIndexer {
    fn index(&self, txn: &DatabaseTransaction, comm: Sender<IndexerMsg>) -> IndexerResult<()>;
}

#[async_trait]
pub trait Indexer: RequiredForIndexer + DynClone + Debug + Send {}

clone_trait_object!(Indexer);
