use anyhow::Result;
use journey_db::entity::{MediaItemDTO, media_items::MediaItemType};
use journey_media_item::{MediaItemManagerError, MediaItemManagerFn};
use serde::Serialize;
use specta::Type;
use thiserror::Error;

use crate::AppState;

#[derive(Debug, Error, Serialize, Type)]
pub enum MediaItemApiError {
    #[error(transparent)]
    MediaItemManagerError(#[from] MediaItemManagerError),
}

type MediaItemApiResult<T> = Result<T, MediaItemApiError>;

#[taurpc::procedures(path = "MediaItem")]
pub trait MediaItemApi {
    async fn get_media_items(
        ty: MediaItemType,
        amount: u64,
    ) -> MediaItemApiResult<Vec<MediaItemDTO>>;
}

#[derive(Clone, Debug)]
pub struct MediaItemApiImpl {
    pub state: AppState,
}

#[taurpc::resolvers]
impl MediaItemApi for MediaItemApiImpl {
    async fn get_media_items(
        self,
        ty: MediaItemType,
        amount: u64,
    ) -> MediaItemApiResult<Vec<MediaItemDTO>> {
        let lock = self.state.read().await;
        let items = lock.media_item_manager.get_items(ty, amount).await?;
        Ok(items)
    }
}
