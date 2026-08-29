use std::fmt::Debug;

use anyhow::Result;
use async_trait::async_trait;
use dyn_clone::{DynClone, clone_trait_object};
use serde::Serialize;
use specta::Type;
use thiserror::Error;
use tokio::sync::mpsc::Sender;
use uuid::Uuid;

use crate::{indexer_manager::IndexerKey, jellyfin::jellyfin_indexer::JellyfinIndexerError};
use journey_db::{
    entity::{ProviderVariant, providers},
    sea_orm::DatabaseTransaction,
};

pub struct IndexerMsg {
    pub item: Option<String>,
    pub success: bool,
}

#[derive(Debug, Error, Serialize, Type)]
pub enum IndexerError {
    #[error("Failed to parse the given String to an Url.")]
    FailedParseUrlError(String),
    #[error("Failed to retrieve Jellyfin API response entry.")]
    ApiEntryRetrievalError(Option<String>),
    #[error("Failed to insert provider to database.")]
    FailedDbInsertError(String),
    #[error("server_id hasn't been set yet, try authenticating first.")]
    MissingServerIdError,
    #[error("user_id hasn't been set yet, try authenticating first.")]
    MissingUserIdError,
    #[error("Url hasn't been set yet, provide one first.")]
    MissingUrlError,
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
    fn get_model(&self) -> &providers::ActiveModelEx;
}

#[async_trait]
pub trait Indexer: RequiredForIndexer + DynClone + Debug + Send {
    fn ty(&self) -> IndexerResult<ProviderVariant> {
        match self.get_model().ty.try_as_ref() {
            Some(user_id) => Ok(*user_id),
            _ => Err(IndexerError::MissingServerIdError),
        }
    }
    fn server_id(&self) -> IndexerResult<Uuid> {
        match self.get_model().server_id.try_as_ref() {
            Some(server_id) if *server_id != Uuid::nil() => Ok(*server_id),
            _ => Err(IndexerError::MissingServerIdError),
        }
    }
    fn key(&self) -> IndexerResult<IndexerKey> {
        Ok(IndexerKey {
            variant: self.ty()?,
            server_id: self.server_id()?,
        })
    }
}

clone_trait_object!(Indexer);
