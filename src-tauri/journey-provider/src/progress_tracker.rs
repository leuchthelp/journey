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
}

pub type ProgressTrackerResult<T> = Result<T, ProgressTrackerError>;

#[derive(Debug, Actor)]
pub struct ProgressTracker {
    pub comm: broadcast::Sender<usize>,
    pub in_progress: usize,
}

pub struct IncProgress {
    pub amount: usize,
}

pub struct DecProgress {
    pub amount: usize,
}

pub struct ProgressRecv;

impl Message<IncProgress> for ProgressTracker {
    type Reply = ProgressTrackerResult<()>;

    async fn handle(
        &mut self,
        msg: IncProgress,
        _: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.in_progress += msg.amount;
        match self.comm.send(self.in_progress) {
            Ok(_) => Ok(()),
            Err(err) => Err(ProgressTrackerError::FailedCommSendError(err.to_string())),
        }
    }
}

impl Message<DecProgress> for ProgressTracker {
    type Reply = ProgressTrackerResult<()>;

    async fn handle(
        &mut self,
        msg: DecProgress,
        _: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.in_progress -= msg.amount;
        match self.comm.send(self.in_progress) {
            Ok(_) => Ok(()),
            Err(err) => Err(ProgressTrackerError::FailedCommSendError(err.to_string())),
        }
    }
}

impl Message<ProgressRecv> for ProgressTracker {
    type Reply = broadcast::Receiver<usize>;

    async fn handle(&mut self, _: ProgressRecv, _: &mut Context<Self, Self::Reply>) -> Self::Reply {
        self.comm.subscribe()
    }
}
