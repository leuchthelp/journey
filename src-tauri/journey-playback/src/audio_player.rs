use std::{io::Cursor, time::Duration};

use anyhow::Result;
use kameo::{
    Actor,
    message::{Context, Message},
};
use reqwest::Response;
use rodio::{Decoder, DeviceSinkBuilder, MixerDeviceSink, Player, Source};
use serde::Serialize;
use specta::Type;
use thiserror::Error;

#[derive(Debug, Error, Serialize, Type)]
pub enum AudioPlayerError {
    #[error("Failed to build device sink: {0}")]
    FailedBuildSinkError(String),
    #[error("Failed to build stream reader: {0}")]
    FailedBuildStreamReaderError(String),
    #[error("Failed to build decoder: {0}")]
    FailedBuildDecoderError(String),
}

pub type AudioPlayerResult<T> = Result<T, AudioPlayerError>;

#[derive(Actor)]
pub struct AudioPlayer {
    player: Player,
    _sink: MixerDeviceSink,
}

impl AudioPlayer {
    pub fn new() -> AudioPlayerResult<Self> {
        let _sink = match DeviceSinkBuilder::open_default_sink() {
            Ok(sink) => Ok(sink),
            Err(err) => Err(AudioPlayerError::FailedBuildSinkError(err.to_string())),
        }?;

        let player = Player::connect_new(_sink.mixer());
        Ok(AudioPlayer { player, _sink })
    }
}

pub struct AppendStream {
    pub response: Response,
}

impl Message<AppendStream> for AudioPlayer {
    type Reply = AudioPlayerResult<()>;

    async fn handle(
        &mut self,
        msg: AppendStream,
        ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        let cursor = match msg.response.bytes().await {
            Ok(cursor) => Ok(Cursor::new(cursor)),
            Err(err) => Err(AudioPlayerError::FailedBuildStreamReaderError(
                err.to_string(),
            )),
        }?;

        let decoder = match Decoder::new(cursor) {
            Ok(decoder) => Ok(decoder),
            Err(err) => Err(AudioPlayerError::FailedBuildDecoderError(err.to_string())),
        }?;

        decoder.periodic_access(Duration::from_millis(5), move |src| {
            futures::executor::block_on(async { () })
        });
        let actor_ref = ctx.actor_ref();
        Ok(())
    }
}

pub struct Play {
    pub immediately: bool,
}
impl Message<Play> for AudioPlayer {
    type Reply = AudioPlayerResult<Duration>;

    async fn handle(&mut self, msg: Play, _: &mut Context<Self, Self::Reply>) -> Self::Reply {
        match msg.immediately {
            true => {
                self.player.skip_one();
                self.player.play()
            }
            false => self.player.play(),
        }
        Ok(self.player.get_pos())
    }
}

pub struct Pause;
impl Message<Pause> for AudioPlayer {
    type Reply = AudioPlayerResult<Duration>;

    async fn handle(&mut self, _: Pause, _: &mut Context<Self, Self::Reply>) -> Self::Reply {
        self.player.pause();
        Ok(self.player.get_pos())
    }
}
