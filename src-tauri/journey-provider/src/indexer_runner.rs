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
    indexer_manager::IndexerManagerResult,
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

#[derive(Debug)]
pub struct IndexRunner {
    runner: JoinHandle<IndexRunnerResult<()>>,
    pub task_comm: UnboundedSender<JoinHandle<IndexerManagerResult<()>>>,
    pub progress: ActorRef<ProgressTracker>,
}

pub struct NewTask {
    task: JoinHandle<IndexerManagerResult<()>>,
}

impl Message<NewTask> for IndexRunner {
    type Reply = IndexRunnerResult<()>;

    async fn handle(&mut self, msg: NewTask, _: &mut Context<Self, Self::Reply>) -> Self::Reply {
        match self.task_comm.send(msg.task) {
            Ok(_) => Ok(()),
            Err(err) => Err(IndexRunnerError::FailedRegisterTaskError(err.to_string())),
        }?;

        match self.progress.tell(IncProgress { amount: 1 }).await {
            Ok(_) => Ok(()),
            Err(err) => Err(ProgressTrackerError::FailedCommSendError(err.to_string()).into()),
        }
    }
}

impl Default for IndexRunner {
    fn default() -> Self {
        /*
            This should become (!) NEVER as soon as it is stabilized. This can also NEVER fail, ever.
            All errors encountered during indexing need to be handled within the respective indexer &
            send to the frontend via an IndexerMsg::Failure().
        */

        let runner_op = async |mut recv: UnboundedReceiver<
            JoinHandle<IndexerManagerResult<()>>,
        >,
                               progress_comm: ActorRef<ProgressTracker>|
               -> IndexRunnerResult<()> {
            while let Some(task) = recv.recv().await {
                match task.await {
                    Ok(_) => match progress_comm.tell(DecProgress { amount: 1 }).await {
                        Ok(_) => Ok(()),
                        Err(err) => Err(ProgressTrackerError::FailedCommSendError(err.to_string())),
                    }?,
                    Err(err) => warn!("indexer task failed: {}", err.to_string()),
                };
            }

            Ok(())
        };

        let (task_comm, task_recv): (
            UnboundedSender<JoinHandle<IndexerManagerResult<()>>>,
            UnboundedReceiver<JoinHandle<IndexerManagerResult<()>>>,
        ) = mpsc::unbounded_channel();

        let (progress_comm, _): (broadcast::Sender<usize>, broadcast::Receiver<usize>) =
            broadcast::channel(20);

        let progress = ProgressTracker::spawn(ProgressTracker {
            comm: progress_comm,
            in_progress: 0,
        });

        let runner = tokio::spawn(runner_op(task_recv, progress.clone()));

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
