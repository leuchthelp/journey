use std::fmt::{Display, Formatter};

use anyhow::Result;
use async_trait::async_trait;
use inherent::inherent;
use journey_db::{entity::ProviderVariant, get_conn};
use kameo::prelude::*;
use rapidhash::RapidHashMap;
use serde::Serialize;
use specta::Type;
use thiserror::Error;
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};
use uuid::Uuid;

use crate::{
    indexer::{Indexer, IndexerError, IndexerMsg},
    indexer_runner::{IndexerRunner, IndexerRunnerError, IndexerRunnerResult, NewTask},
};

#[derive(Debug, Error, Serialize, Type)]
pub enum IndexerManagerError {
    #[error("Message channel for {0} does not exist.")]
    NoSuchCommError(String),
    #[error("Task for {0} does not exist.")]
    NoSuchTaskError(String),
    #[error(transparent)]
    IndexerError(#[from] IndexerError),
    #[error(transparent)]
    IndexerRunnerError(#[from] IndexerRunnerError),
}

pub type IndexerManagerResult<T> = Result<T, IndexerManagerError>;

#[taurpc::ipc_type]
#[derive(Debug, PartialEq, Eq, Hash, Copy)]
#[serde(rename_all = "camelCase")]
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
    async fn register(
        &mut self,
        indexer: Box<dyn Indexer + Send + Sync>,
    ) -> IndexerManagerResult<()>;
    fn consume_status(
        &mut self,
        key: &IndexerKey,
    ) -> IndexerManagerResult<UnboundedReceiver<IndexerMsg>>;
}

#[derive(Debug)]
pub struct IndexerManager {
    runner: ActorRef<IndexerRunner>,
    comms: RapidHashMap<IndexerKey, UnboundedReceiver<IndexerMsg>>,
}

impl Default for IndexerManager {
    fn default() -> Self {
        IndexerManager {
            runner: IndexerRunner::spawn_default(),
            comms: RapidHashMap::default(),
        }
    }
}

#[async_trait]
#[inherent]
impl RequiredForIndexerManager for IndexerManager {
    pub async fn register(
        &mut self,
        indexer: Box<dyn Indexer + Send + Sync>,
    ) -> IndexerManagerResult<()> {
        let (comm, recv): (UnboundedSender<IndexerMsg>, UnboundedReceiver<IndexerMsg>) =
            mpsc::unbounded_channel();

        let index_background_op = async |indexer: Box<dyn Indexer + Send + Sync>,
                                         comm: UnboundedSender<IndexerMsg>|
               -> IndexerRunnerResult<()> {
            let conn = get_conn().await?;

            indexer.index(&conn, comm).await?;
            Ok(())
        };

        let key = indexer.key()?;
        let task = tokio::spawn(index_background_op(indexer, comm.clone()));

        match self.runner.tell(NewTask { task, comm }).await {
            Ok(_) => Ok(()),
            Err(err) => Err(IndexerRunnerError::FailedRegisterTaskError(err.to_string())),
        }?;

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
}
