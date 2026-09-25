use anyhow::Result;
use kameo::{Actor, message::Message, prelude::*};
use serde::Serialize;
use specta::Type;
use thiserror::Error;
use tokio::{
    sync::{
        broadcast,
        mpsc::{self, UnboundedReceiver, UnboundedSender},
    },
    task::JoinHandle,
};
use tracing::warn;

use crate::{
    IndexerMsg,
    indexer::IndexerResult,
    progress_tracker::{DecProgress, IncProgress, ProgressTracker, ProgressTrackerError},
};

#[derive(Debug, Error, Serialize, Type)]
pub enum IndexRunnerError {
    #[error("Failed to register task with runner: {0}")]
    FailedRegisterTaskError(String),
    #[error("Failed to run indexer task: {0}")]
    FailedTaskError(String),
    #[error(transparent)]
    ProgressTrackerError(#[from] ProgressTrackerError),
}

pub type IndexRunnerResult<T> = Result<T, IndexRunnerError>;

/*
    This should become (!) NEVER as soon as it is stabilized. This can also NEVER fail, ever.
    All errors encountered during indexing need to be handled within the respective indexer &
    send to the frontend via an IndexerMsg::Failure().
*/
async fn runner_task(
    mut recv: UnboundedReceiver<(JoinHandle<IndexerResult<()>>, UnboundedSender<IndexerMsg>)>,
    progress_comm: ActorRef<ProgressTracker>,
) -> IndexRunnerResult<()> {
    while let Some((task, indexer_comm)) = recv.recv().await {
        let res = match task.await {
            Ok(res) => Some(res),
            Err(err) => {
                match indexer_comm.send(IndexerMsg::FullTaskFailure {
                    reason: IndexRunnerError::FailedTaskError(err.to_string()).into(),
                }) {
                    Ok(_) => (),
                    Err(err) => {
                        warn!("failed to send failure in to frontend: {}", err.to_string())
                    }
                };
                None
            }
        };

        if let Some(res) = res {
            match res {
                Ok(_) => match progress_comm.tell(DecProgress { amount: 1 }).await {
                    Ok(_) => Ok(()),
                    Err(err) => Err(ProgressTrackerError::FailedCommSendError(err.to_string())),
                }?,
                Err(err) => match indexer_comm.send(IndexerMsg::FullTaskFailure { reason: err }) {
                    Ok(_) => (),
                    Err(err) => warn!("failed to send failure in to frontend: {}", err.to_string()),
                },
            }
        }
    }

    Ok(())
}

#[derive(Debug)]
pub struct IndexRunner {
    runner: JoinHandle<IndexRunnerResult<()>>,
    pub task_comm: UnboundedSender<(JoinHandle<IndexerResult<()>>, UnboundedSender<IndexerMsg>)>,
    pub progress: ActorRef<ProgressTracker>,
}

impl Default for IndexRunner {
    fn default() -> Self {
        let (task_comm, task_recv): (
            UnboundedSender<(JoinHandle<IndexerResult<()>>, UnboundedSender<IndexerMsg>)>,
            UnboundedReceiver<(JoinHandle<IndexerResult<()>>, UnboundedSender<IndexerMsg>)>,
        ) = mpsc::unbounded_channel();

        let (progress_comm, progress_recv): (broadcast::Sender<i32>, broadcast::Receiver<i32>) =
            broadcast::channel(20);

        let progress = ProgressTracker::spawn(ProgressTracker {
            comm: progress_comm,
            _recv: progress_recv,
            in_progress: 0,
        });

        let runner = tokio::spawn(runner_task(task_recv, progress.clone()));

        IndexRunner {
            runner: runner,
            task_comm,
            progress: progress,
        }
    }
}

impl Actor for IndexRunner {
    type Args = ();
    type Error = IndexRunnerError;

    async fn on_start(_: Self::Args, _: ActorRef<Self>) -> IndexRunnerResult<Self> {
        Ok(IndexRunner::default())
    }

    async fn on_stop(
        &mut self,
        actor_ref: WeakActorRef<Self>,
        _: ActorStopReason,
    ) -> IndexRunnerResult<()> {
        self.runner.abort();
        actor_ref.kill();
        Ok(())
    }
}

pub struct NewTask {
    task: JoinHandle<IndexerResult<()>>,
    comm: UnboundedSender<IndexerMsg>,
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
