use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use inherent::inherent;
use journey_db::{entity::ProviderVariant, get_conn, sea_orm::TransactionTrait};
use rapidhash::RapidHashMap;
use serde::Serialize;
use specta::Type;
use thiserror::Error;
use tokio::{
    sync::mpsc::{self, Receiver, Sender},
    task::JoinHandle,
};
use uuid::Uuid;

use crate::indexer::{Indexer, IndexerError, IndexerMsg};

#[derive(Debug, Error, Serialize, Type)]
pub enum IndexerManagerError {
    #[error("Failed to run transaction")]
    FailedTransactionError(String),
    #[error(transparent)]
    IndexerError(#[from] IndexerError),
    #[error(transparent)]
    JourneyDbError(#[from] journey_db::JourneyDbError),
}

pub type IndexerManagerResult<T> = Result<T, IndexerManagerError>;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct IndexerKey {
    pub variant: ProviderVariant,
    pub server_id: Uuid,
}

#[async_trait]
pub trait RequiredForIndexerManager {
    fn register(&mut self, indexer: Box<dyn Indexer + Send + Sync>) -> IndexerManagerResult<()>;
}

#[async_trait]
pub trait IndexerManagerFn: RequiredForIndexerManager {}

#[derive(Default, Clone, Debug)]
pub struct IndexerManager {
    tasks: RapidHashMap<IndexerKey, Arc<JoinHandle<IndexerManagerResult<()>>>>,
    comms: RapidHashMap<IndexerKey, Arc<Receiver<IndexerMsg>>>,
}

#[async_trait]
#[inherent]
impl RequiredForIndexerManager for IndexerManager {
    pub fn register(
        &mut self,
        indexer: Box<dyn Indexer + Send + Sync>,
    ) -> IndexerManagerResult<()> {
        let (comm, recv): (Sender<IndexerMsg>, Receiver<IndexerMsg>) = mpsc::channel(100);

        let index_background_op = async |indexer: Box<dyn Indexer + Send + Sync>,
                                         comm: Sender<IndexerMsg>|
               -> IndexerManagerResult<()> {
            let conn = get_conn().await?;

            match conn
                .transaction::<_, _, IndexerManagerError>(|txn| {
                    Box::pin(async move { Ok(indexer.index(txn, comm).await?) })
                })
                .await
            {
                Ok(_) => Ok(()),
                Err(err) => Err(IndexerManagerError::FailedTransactionError(err.to_string())),
            }
        };

        let key = indexer.key()?;
        let task = tokio::spawn(index_background_op(indexer, comm));

        self.tasks.insert(key, task.into());
        self.comms.insert(key, recv.into());
        Ok(())
    }
}

impl IndexerManagerFn for IndexerManager {}
