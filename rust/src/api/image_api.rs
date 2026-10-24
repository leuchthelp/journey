use anyhow::Result;
use journey_db::entity::ImageDTO;
use serde::Serialize;
use thiserror::Error;

use crate::AppState;

#[derive(Debug, Error, Serialize)]
pub enum ImageApiError {}

type ImageApiResult<T> = Result<T, ImageApiError>;


pub trait ImageApi {
    async fn get_images(self) -> ImageApiResult<ImageDTO>;
}

#[derive(Clone)]
pub struct ImageApiImpl {
    pub state: AppState,
}


impl ImageApi for ImageApiImpl {
    async fn get_images(self) -> ImageApiResult<ImageDTO> {
        Ok(ImageDTO::default())
    }
}
