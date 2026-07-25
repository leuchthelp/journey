use anyhow::Result;
use journey_db::entity::MediaItemDTO;
use serde::{Deserialize, Serialize};
use specta::Type;
use thiserror::Error;

use crate::AppState;

#[derive(Debug, Error, Serialize, Deserialize, Type)]
#[specta(type = String)]
pub enum MediaItemApiError {}

type MediaItemApiResult<T> = Result<T, MediaItemApiError>;

#[taurpc::procedures(path = "mediaItem")]
pub trait MediaItemApi {
    async fn get_media_items() -> MediaItemApiResult<MediaItemDTO>;
}

#[derive(Clone, Debug)]
pub struct MediaItemApiImpl {
    pub state: AppState,
}

#[taurpc::resolvers]
impl MediaItemApi for MediaItemApiImpl {
    async fn get_media_items(self) -> MediaItemApiResult<MediaItemDTO> {
        Ok(MediaItemDTO::default())
    }
}
