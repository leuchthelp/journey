use anyhow::Result;
use journey_db::entity::ImageDTO;
use serde::{Deserialize, Serialize};
use specta::Type;
use thiserror::Error;

use crate::AppState;

#[derive(Debug, Error, Serialize, Deserialize, Type)]
#[specta(type = String)]
pub enum ImageApiError {}

type ImageApiResult<T> = Result<T, ImageApiError>;

#[taurpc::procedures(path = "image")]
pub trait ImageApi {
    async fn get_images() -> ImageApiResult<ImageDTO>;
}

#[derive(Clone, Debug)]
pub struct ImageApiImpl {
    pub state: AppState,
}

#[taurpc::resolvers]
impl ImageApi for ImageApiImpl {
    async fn get_images(self) -> ImageApiResult<ImageDTO> {
        Ok(ImageDTO::new())
    }
}
