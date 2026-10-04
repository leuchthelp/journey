use std::time::Duration;

use anyhow::Result;
use journey_playback::audio_player::{Pause, Play};
use serde::Serialize;
use specta::Type;
use thiserror::Error;

use crate::AppState;

#[derive(Debug, Error, Serialize, Type)]
pub enum PlayerApiError {
    #[error("Failed to send msg: {msg} to player: {err}")]
    FailedMessageSendError { msg: &'static str, err: String },
}

type PlayerApiResult<T> = Result<T, PlayerApiError>;

#[taurpc::procedures(path = "Player")]
pub trait PlayerApi {
    async fn play(immediately: bool) -> PlayerApiResult<Duration>;
    async fn pause() -> PlayerApiResult<Duration>;
}

#[derive(Clone)]
pub struct PlayerApiImpl {
    pub state: AppState,
}

#[taurpc::resolvers]
impl PlayerApi for PlayerApiImpl {
    async fn play(self, immediately: bool) -> PlayerApiResult<Duration> {
        let lock = self.state.read().await;
        let duration = match lock
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
        let lock = self.state.read().await;
        let duration = match lock.audio_player.ask(Pause).await {
            Ok(duration) => Ok(duration),
            Err(err) => Err(PlayerApiError::FailedMessageSendError {
                msg: "Pause",
                err: err.to_string(),
            }),
        }?;

        Ok(duration)
    }
}
