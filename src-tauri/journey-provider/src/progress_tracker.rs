use anyhow::Result;
use kameo::{Actor, message::Message, prelude::*};
use serde::Serialize;
use specta::Type;
use thiserror::Error;
use tokio::sync::broadcast;

#[derive(Debug, Error, Serialize, Type)]
pub enum ProgressTrackerError {
    #[error("Failed to send message to broadcast channel: {0}")]
    FailedCommSendError(String),
    #[error("Failed to ask progress tracker: {0}")]
    FailedCommAskError(String),
}

pub type ProgressTrackerResult<T> = Result<T, ProgressTrackerError>;

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all_fields = "camelCase", tag = "event", content = "data")]
pub enum ProgressTrackerMsg {
    Progress { amount: i32 },
}

#[derive(Debug, Actor)]
pub struct ProgressTracker {
    comm: broadcast::Sender<ProgressTrackerMsg>,
    _recv: broadcast::Receiver<ProgressTrackerMsg>,
    in_progress: i32,
}

impl Default for ProgressTracker {
    fn default() -> Self {
        let (comm, _recv): (
            broadcast::Sender<ProgressTrackerMsg>,
            broadcast::Receiver<ProgressTrackerMsg>,
        ) = broadcast::channel(20);

        ProgressTracker {
            comm,
            _recv,
            in_progress: 0,
        }
    }
}

pub struct IncProgress {
    pub amount: i32,
}

impl Message<IncProgress> for ProgressTracker {
    type Reply = ProgressTrackerResult<()>;

    async fn handle(
        &mut self,
        msg: IncProgress,
        _: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.in_progress += msg.amount;
        match self.comm.send(ProgressTrackerMsg::Progress {
            amount: self.in_progress,
        }) {
            Ok(_) => Ok(()),
            Err(err) => Err(ProgressTrackerError::FailedCommSendError(err.to_string())),
        }
    }
}

pub struct DecProgress {
    pub amount: i32,
}

impl Message<DecProgress> for ProgressTracker {
    type Reply = ProgressTrackerResult<()>;

    async fn handle(
        &mut self,
        msg: DecProgress,
        _: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.in_progress -= msg.amount;
        match self.comm.send(ProgressTrackerMsg::Progress {
            amount: self.in_progress,
        }) {
            Ok(_) => Ok(()),
            Err(err) => Err(ProgressTrackerError::FailedCommSendError(err.to_string())),
        }
    }
}

pub struct ProgressRecv;

impl Message<ProgressRecv> for ProgressTracker {
    type Reply = broadcast::Receiver<ProgressTrackerMsg>;

    async fn handle(&mut self, _: ProgressRecv, _: &mut Context<Self, Self::Reply>) -> Self::Reply {
        self.comm.subscribe()
    }
}
