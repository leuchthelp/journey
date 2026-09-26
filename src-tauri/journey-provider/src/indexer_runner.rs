use anyhow::Result;
use journey_db::JourneyDbError;
use kameo::{Actor, message::Message, prelude::*};
use serde::Serialize;
use specta::Type;
use thiserror::Error;
use tokio::{
    sync::mpsc::{self, UnboundedReceiver, UnboundedSender},
    task::JoinHandle,
};

use crate::{
    IndexerMsg,
    indexer::IndexerError,
    progress_tracker::{DecProgress, IncProgress, ProgressTracker, ProgressTrackerError},
};

#[derive(Debug, Error, Serialize, Type)]
pub enum IndexRunnerError {
    #[error("Failed to register task with runner: {0}")]
    FailedRegisterTaskError(String),
    #[error("Failed to run indexer task: {0}")]
    FailedTaskError(String),
    #[error("Accidentally killed the runner. This should never happen: {0}")]
    KilledRunnerError(String),
    #[error(transparent)]
    IndexerError(#[from] IndexerError),
    #[error(transparent)]
    ProgressTrackerError(#[from] ProgressTrackerError),
    #[error(transparent)]
    JourneyDbError(#[from] JourneyDbError),
}

pub type IndexRunnerResult<T> = Result<T, IndexRunnerError>;

/*
    This should become (!) NEVER as soon as it is stabilized. This can also NEVER fail, ever.
    All errors encountered during indexing need to be handled within the respective indexer &
    send to the frontend via an IndexerMsg::Failure().
*/
async fn runner_task(
    mut recv: UnboundedReceiver<(
        JoinHandle<IndexRunnerResult<()>>,
        UnboundedSender<IndexerMsg>,
    )>,
    progress_comm: ActorRef<ProgressTracker>,
) -> IndexRunnerResult<()> {
    while let Some((task, indexer_comm)) = recv.recv().await {
        let res = match task.await {
            Ok(res) => res,
            Err(err) => match indexer_comm.send(IndexerMsg::FullTaskFailure {
                reason: IndexRunnerError::FailedTaskError(err.to_string()),
            }) {
                Ok(_) => Ok(()),
                Err(err) => Err(ProgressTrackerError::FailedCommSendError(err.to_string()).into()),
            },
        };

        match res {
            Ok(_) => match progress_comm.tell(DecProgress { amount: 1 }).await {
                Ok(_) => Ok(()),
                Err(err) => Err(ProgressTrackerError::FailedCommSendError(err.to_string())),
            }?,
            Err(err) => match indexer_comm.send(IndexerMsg::FullTaskFailure { reason: err }) {
                Ok(_) => Ok(()),
                Err(err) => Err(IndexRunnerError::KilledRunnerError(err.to_string())),
            }?,
        }
    }

    Ok(())
}

#[derive(Debug, Actor)]
pub struct IndexRunner {
    _runner: JoinHandle<IndexRunnerResult<()>>,
    task_comm: UnboundedSender<(
        JoinHandle<IndexRunnerResult<()>>,
        UnboundedSender<IndexerMsg>,
    )>,
    progress: ActorRef<ProgressTracker>,
}

impl Default for IndexRunner {
    fn default() -> Self {
        let (task_comm, task_recv): (
            UnboundedSender<(
                JoinHandle<IndexRunnerResult<()>>,
                UnboundedSender<IndexerMsg>,
            )>,
            UnboundedReceiver<(
                JoinHandle<IndexRunnerResult<()>>,
                UnboundedSender<IndexerMsg>,
            )>,
        ) = mpsc::unbounded_channel();

        let progress = ProgressTracker::spawn_default();

        let _runner = tokio::spawn(runner_task(task_recv, progress.clone()));

        IndexRunner {
            _runner,
            task_comm,
            progress,
        }
    }
}

pub struct NewTask {
    pub task: JoinHandle<IndexRunnerResult<()>>,
    pub comm: UnboundedSender<IndexerMsg>,
}

impl Message<NewTask> for IndexRunner {
    type Reply = IndexRunnerResult<()>;

    async fn handle(&mut self, msg: NewTask, _: &mut Context<Self, Self::Reply>) -> Self::Reply {
        match self.task_comm.send((msg.task, msg.comm)) {
            Ok(_) => Ok(()),
            Err(err) => Err(IndexRunnerError::FailedRegisterTaskError(err.to_string())),
        }?;

        match self.progress.tell(IncProgress { amount: 1 }).await {
            Ok(_) => Ok(()),
            Err(err) => Err(ProgressTrackerError::FailedCommSendError(err.to_string()).into()),
        }
    }
}
