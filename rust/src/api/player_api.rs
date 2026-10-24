use std::time::Duration;

use anyhow::Result;
use journey_playback::audio_player::{Pause, Play};
use serde::Serialize;
use thiserror::Error;

use crate::AppState;

#[derive(Debug, Error, Serialize)]
pub enum PlayerApiError {
    #[error("Failed to send msg: {msg} to player: {err}")]
    FailedMessageSendError { msg: &'static str, err: String },
}

type PlayerApiResult<T> = Result<T, PlayerApiError>;

pub trait PlayerApi {
    async fn play(self, immediately: bool) -> PlayerApiResult<Duration>;
    async fn pause(self) -> PlayerApiResult<Duration>;
}

#[derive(Clone)]
pub struct PlayerApiImpl {
    pub state: AppState,
}


impl PlayerApi for PlayerApiImpl {
    async fn play(self, immediately: bool) -> PlayerApiResult<Duration> {
        let duration = match self
            .state
            .audio_player
            .ask(Play {
                immediately: immediately,
            })
            .await
        {
            Ok(duration) => Ok(duration),
            Err(err) => Err(PlayerApiError::FailedMessageSendError {
                msg: "Play",
                err: err.to_string(),
            }),
        }?;

        Ok(duration)
    }
    async fn pause(self) -> PlayerApiResult<Duration> {
        let duration = match self.state.audio_player.ask(Pause).await {
            Ok(duration) => Ok(duration),
            Err(err) => Err(PlayerApiError::FailedMessageSendError {
                msg: "Pause",
                err: err.to_string(),
            }),
        }?;

        Ok(duration)
    }
}
