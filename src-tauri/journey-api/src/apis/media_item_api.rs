use anyhow::Result;
use journey_db::entity::MediaItemDTO;
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::State;
use thiserror::Error;

use crate::AppData;

#[derive(Debug, Error, Serialize, Deserialize, Type)]
#[specta(type = String)]
pub enum MediaItemApiError {}

type MediaItemApiResult<T> = Result<T, MediaItemApiError>;

#[taurpc::procedures(path = "mediaItem")]
pub trait MediaItemApi {
    async fn get_media_items(state: State<AppData>) -> MediaItemApiResult<MediaItemDTO>;
}

#[derive(Clone, Debug)]
pub struct MediaItemApiImpl;

#[taurpc::resolvers]
impl MediaItemApi for MediaItemApiImpl {
    async fn get_media_items(self, state: State<AppData>) -> MediaItemApiResult<MediaItemDTO> {
        Ok(MediaItemDTO::default())
    }
}
