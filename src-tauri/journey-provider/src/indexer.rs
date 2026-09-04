use std::fmt::Debug;

use anyhow::Result;
use async_trait::async_trait;
use dyn_clone::{DynClone, clone_trait_object};
use serde::Serialize;
use specta::Type;
use thiserror::Error;
use tokio::sync::mpsc::UnboundedSender;
use url::Url;
use uuid::Uuid;

use crate::{indexer_manager::IndexerKey, jellyfin::jellyfin_indexer::JellyfinIndexerError};
use journey_db::{
    entity::{ProviderVariant, providers},
    sea_orm::DatabaseTransaction,
};

#[derive(Debug)]
pub struct IndexerMsg {
    pub item: Option<String>,
    pub success: bool,
    pub already_exists: bool,
}

#[derive(Debug, Error, Serialize, Type)]
pub enum IndexerError {
    #[error("Failed to parse the given String to an Url: {0}")]
    FailedParseUrlError(String),
    #[error("Failed to retrieve Jellyfin API response entry.")]
    ApiEntryRetrievalError(Option<String>),
    #[error("Failed to insert into database: {0}")]
    FailedDbInsertError(String),
    #[error("Failed to send update message over channel: {0}")]
    FailedMsgSendError(String),
    #[error("Failed to run transaction: {0}")]
    FailedTransactionError(String),
    #[error(
        "ProviderVariant has not been set. This cannot be done here. Check the original Provider implementation"
    )]
    MissingVariantError,
    #[error("server_id has not been set yet, try authenticating first.")]
    MissingServerIdError,
    #[error("user_id has not been set yet, try authenticating first.")]
    MissingUserIdError,
    #[error("Url has not been set yet, provide one first.")]
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
    fn get_model(&self) -> &providers::ActiveModelEx;
    async fn index(
        &self,
        txn: &DatabaseTransaction,
        comm: UnboundedSender<IndexerMsg>,
    ) -> IndexerResult<()>;
}

#[async_trait]
pub trait Indexer: RequiredForIndexer + DynClone + Debug + Send {
    fn ty(&self) -> IndexerResult<ProviderVariant> {
        match self.get_model().ty.try_as_ref() {
            Some(variant) => Ok(*variant),
            _ => Err(IndexerError::MissingVariantError),
        }
    }
    fn user_id(&self) -> IndexerResult<Uuid> {
        match self.get_model().user_id.try_as_ref() {
            Some(user_id) if *user_id != Uuid::nil() => Ok(*user_id),
            _ => Err(IndexerError::MissingServerIdError),
        }
    }
    fn server_id(&self) -> IndexerResult<Uuid> {
        match self.get_model().server_id.try_as_ref() {
            Some(server_id) if *server_id != Uuid::nil() => Ok(*server_id),
            _ => Err(IndexerError::MissingServerIdError),
        }
    }
    fn url(&self) -> IndexerResult<Url> {
        match self.get_model().url.try_as_ref() {
            Some(url) => Ok(match Url::parse(url) {
                Ok(url) => url,
                Err(err) => return Err(IndexerError::FailedParseUrlError(err.to_string())),
            }),
            _ => Err(IndexerError::MissingUrlError),
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
