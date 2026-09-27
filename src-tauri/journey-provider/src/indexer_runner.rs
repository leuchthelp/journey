use anyhow::Result;
use journey_db::JourneyDbError;
use kameo::{Actor, message::Message, prelude::*};
use serde::Serialize;
use specta::Type;
use tauri::ipc::Channel;
use thiserror::Error;
use tokio::{
    sync::mpsc::{self, UnboundedReceiver, UnboundedSender},
    task::JoinHandle,
};

use crate::{
    IndexerMsg,
    indexer::IndexerError,
    progress_tracker::{
        DecProgress, IncProgress, ProgressRecv, ProgressTracker, ProgressTrackerError,
        ProgressTrackerMsg,
    },
};

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
    ProgressTrackerError(#[from] ProgressTrackerError),
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
    progress: ActorRef<ProgressTracker>,
) -> IndexerRunnerResult<()> {
    while let Some((task, indexer_comm)) = recv.recv().await {
        let res = match task.await {
            Ok(res) => res,
            Err(err) => match indexer_comm.send(IndexerMsg::FullTaskFailure {
                reason: IndexerRunnerError::FailedTaskError(err.to_string()),
            }) {
                Ok(_) => Ok(()),
                Err(err) => Err(ProgressTrackerError::FailedCommSendError(err.to_string()).into()),
            },
        };

        match res {
            Ok(_) => match progress.tell(DecProgress { amount: 1 }).await {
                Ok(_) => Ok(()),
                Err(err) => Err(ProgressTrackerError::FailedCommSendError(err.to_string())),
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
    progress: ActorRef<ProgressTracker>,
    ui_callback_tasks: Vec<JoinHandle<IndexerRunnerResult<()>>>,
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

        let progress = ProgressTracker::spawn_default();

        let _runner = tokio::spawn(runner_task(task_recv, progress.clone()));

        IndexerRunner {
            _runner,
            task_comm,
            progress,
            ui_callback_tasks: vec![],
        }
    }
}

pub struct NewTask {
    pub task: JoinHandle<IndexerRunnerResult<()>>,
    pub comm: UnboundedSender<IndexerMsg>,
}

impl Message<NewTask> for IndexerRunner {
    type Reply = IndexerRunnerResult<()>;

    async fn handle(&mut self, msg: NewTask, _: &mut Context<Self, Self::Reply>) -> Self::Reply {
        match self.task_comm.send((msg.task, msg.comm)) {
            Ok(_) => Ok(()),
            Err(err) => Err(IndexerRunnerError::FailedRegisterTaskError(err.to_string())),
        }?;

        match self.progress.tell(IncProgress { amount: 1 }).await {
            Ok(_) => Ok(()),
            Err(err) => Err(ProgressTrackerError::FailedCommSendError(err.to_string()).into()),
        }
    }
}

pub struct GetProgress {
    pub callback: Channel<ProgressTrackerMsg>,
}

impl Message<GetProgress> for IndexerRunner {
    type Reply = IndexerRunnerResult<()>;

    async fn handle(
        &mut self,
        msg: GetProgress,
        _: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        let on_event = msg.callback;

        let recv = match self.progress.ask(ProgressRecv).await {
            Ok(recv) => recv,
            Err(err) => {
                return Err(ProgressTrackerError::FailedCommAskError(err.to_string()).into());
            }
        };

        let op = async |mut recv: tokio::sync::broadcast::Receiver<ProgressTrackerMsg>,
                        on_event: Channel<ProgressTrackerMsg>|
               -> IndexerRunnerResult<()> {
            while let Ok(msg) = recv.recv().await {
                match on_event.send(msg) {
                    Ok(_) => Ok(()),
                    Err(err) => Err(IndexerRunnerError::FailedChannelSendError(err.to_string())),
                }?;
            }

            Ok(())
        };

        self.ui_callback_tasks
            .push(tokio::spawn(op(recv, on_event)));

        Ok(())
    }
}
