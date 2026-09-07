use std::fmt::{Display, Formatter};

use anyhow::Result;
use async_trait::async_trait;
use inherent::inherent;
use journey_db::{entity::ProviderVariant, get_conn};
use rapidhash::RapidHashMap;
use serde::{Deserialize, Serialize};
use specta::Type;
use thiserror::Error;
use tokio::{
    sync::mpsc::{self, UnboundedReceiver, UnboundedSender},
    task::JoinHandle,
};
use uuid::Uuid;

use crate::indexer::{Indexer, IndexerError, IndexerMsg};

#[derive(Debug, Error, Serialize, Type)]
pub enum IndexerManagerError {
    #[error("Failed to run transaction: {0}")]
    FailedTransactionError(String),
    #[error("Failed to run indexer task: {0}")]
    FailedTaskError(String),
    #[error("Message channel for {0} does not exist.")]
    NoSuchCommError(String),
    #[error("Task for {0} does not exist.")]
    NoSuchTaskError(String),
    #[error(transparent)]
    IndexerError(#[from] IndexerError),
    #[error(transparent)]
    JourneyDbError(#[from] journey_db::JourneyDbError),
}

pub type IndexerManagerResult<T> = Result<T, IndexerManagerError>;

#[taurpc::ipc_type]
#[derive(Debug, PartialEq, Eq, Hash, Copy)]
pub struct IndexerKey {
    pub variant: ProviderVariant,
    pub provider_id: Uuid,
}

impl Display for IndexerKey {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(
            f,
            "Key for server: {}, provider: {}",
            self.provider_id, self.variant
        )
    }
}

#[async_trait]
pub trait RequiredForIndexerManager {
    fn register(&mut self, indexer: Box<dyn Indexer + Send + Sync>) -> IndexerManagerResult<()>;
    fn consume_status(
        &mut self,
        key: &IndexerKey,
    ) -> IndexerManagerResult<UnboundedReceiver<IndexerMsg>>;
    async fn consume_task(
        &mut self,
        key: &IndexerKey,
    ) -> IndexerManagerResult<JoinHandle<IndexerManagerResult<Vec<Option<IndexerError>>>>>;
}

#[async_trait]
pub trait IndexerManagerFn: RequiredForIndexerManager {}

#[derive(Default, Debug)]
pub struct IndexerManager {
    tasks: RapidHashMap<IndexerKey, JoinHandle<IndexerManagerResult<Vec<Option<IndexerError>>>>>,
    comms: RapidHashMap<IndexerKey, UnboundedReceiver<IndexerMsg>>,
}

#[async_trait]
#[inherent]
impl RequiredForIndexerManager for IndexerManager {
    pub fn register(
        &mut self,
        indexer: Box<dyn Indexer + Send + Sync>,
    ) -> IndexerManagerResult<()> {
        let (comm, recv): (UnboundedSender<IndexerMsg>, UnboundedReceiver<IndexerMsg>) =
            mpsc::unbounded_channel();

        let index_background_op =
            async |indexer: Box<dyn Indexer + Send + Sync>,
                   comm: UnboundedSender<IndexerMsg>|
                   -> IndexerManagerResult<Vec<Option<IndexerError>>> {
                let conn = get_conn().await?;
                Ok(indexer.index(&conn, comm).await?)
            };

        let key = indexer.key()?;
        let task = tokio::spawn(index_background_op(indexer, comm));

        self.tasks.insert(key, task);
        self.comms.insert(key, recv);
        Ok(())
    }
    pub fn consume_status(
        &mut self,
        key: &IndexerKey,
    ) -> IndexerManagerResult<UnboundedReceiver<IndexerMsg>> {
        match self.comms.remove(key) {
            Some(comm) => Ok(comm),
            None => Err(IndexerManagerError::NoSuchCommError(key.to_string())),
        }
    }
    pub async fn consume_task(
        &mut self,
        key: &IndexerKey,
    ) -> IndexerManagerResult<JoinHandle<IndexerManagerResult<Vec<Option<IndexerError>>>>> {
        match self.tasks.remove(key) {
            Some(task) => Ok(task),
            None => Err(IndexerManagerError::NoSuchTaskError(key.to_string())),
        }
    }
}

impl IndexerManagerFn for IndexerManager {}
