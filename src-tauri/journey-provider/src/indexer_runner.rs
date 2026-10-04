use anyhow::Result;
use jiff::Timestamp;
use journey_db::JourneyDbError;
use kameo::{Actor, message::Message, prelude::*};
use serde::Serialize;
use specta::Type;
use thiserror::Error;
use tokio::{
    sync::mpsc::{self, UnboundedReceiver, UnboundedSender},
    task::JoinHandle,
};

use crate::{IndexerMsg, indexer::IndexerError};

#[derive(Debug, Error, Serialize, Type)]
pub enum IndexerRunnerError {
    #[error("Failed to register task with runner: {0}")]
    FailedRegisterTaskError(String),
    #[error("Failed to run indexer task: {0}")]
    FailedTaskError(String),
    #[error("Failed to send msg via channel: {0}")]
    FailedChannelSendError(String),
    #[error("Accidentally killed the runner. This should never happen: {0}")]
    KilledRunnerError(String),
    #[error(transparent)]
    IndexerError(#[from] IndexerError),
    #[error(transparent)]
    JourneyDbError(#[from] JourneyDbError),
}

pub type IndexerRunnerResult<T> = Result<T, IndexerRunnerError>;

/*
    This should become (!) NEVER as soon as it is stabilized. This can also NEVER fail, ever.
    All errors encountered during indexing need to be handled within the respective indexer &
    send to the frontend via an IndexerMsg::Failure().
*/
async fn runner_task(
    mut recv: UnboundedReceiver<(
        JoinHandle<IndexerRunnerResult<()>>,
        UnboundedSender<IndexerMsg>,
    )>,
) -> IndexerRunnerResult<()> {
    while let Some((task, indexer_comm)) = recv.recv().await {
        let res = match task.await {
            Ok(res) => res,
            Err(err) => match indexer_comm.send(IndexerMsg::FullTaskFailure {
                reason: IndexerRunnerError::FailedTaskError(err.to_string()),
            }) {
                Ok(_) => Ok(()),
                Err(err) => Err(IndexerRunnerError::FailedChannelSendError(err.to_string())),
            },
        };

        match res {
            Ok(_) => match indexer_comm.send(IndexerMsg::Finished {
                time: Timestamp::now(),
            }) {
                Ok(_) => Ok(()),
                Err(err) => Err(IndexerRunnerError::FailedRegisterTaskError(err.to_string())),
            }?,

            Err(err) => match indexer_comm.send(IndexerMsg::FullTaskFailure { reason: err }) {
                Ok(_) => Ok(()),
                Err(err) => Err(IndexerRunnerError::KilledRunnerError(err.to_string())),
            }?,
        }
    }

    Ok(())
}

#[derive(Debug, Actor)]
pub struct IndexerRunner {
    _runner: JoinHandle<IndexerRunnerResult<()>>,
    task_comm: UnboundedSender<(
        JoinHandle<IndexerRunnerResult<()>>,
        UnboundedSender<IndexerMsg>,
    )>,
}

impl Default for IndexerRunner {
    fn default() -> Self {
        let (task_comm, task_recv): (
            UnboundedSender<(
                JoinHandle<IndexerRunnerResult<()>>,
                UnboundedSender<IndexerMsg>,
            )>,
            UnboundedReceiver<(
                JoinHandle<IndexerRunnerResult<()>>,
                UnboundedSender<IndexerMsg>,
            )>,
        ) = mpsc::unbounded_channel();

        let _runner = tokio::spawn(runner_task(task_recv));

        IndexerRunner { _runner, task_comm }
    }
}

pub struct NewTask {
    pub task: JoinHandle<IndexerRunnerResult<()>>,
    pub comm: UnboundedSender<IndexerMsg>,
}

impl Message<NewTask> for IndexerRunner {
    type Reply = IndexerRunnerResult<()>;

    async fn handle(&mut self, msg: NewTask, _: &mut Context<Self, Self::Reply>) -> Self::Reply {
        match msg.comm.send(IndexerMsg::Started {
            time: Timestamp::now(),
        }) {
            Ok(_) => Ok(()),
            Err(err) => Err(IndexerRunnerError::FailedRegisterTaskError(err.to_string())),
        }?;

        match self.task_comm.send((msg.task, msg.comm)) {
            Ok(_) => Ok(()),
            Err(err) => Err(IndexerRunnerError::FailedRegisterTaskError(err.to_string())),
        }
    }
}
