use anyhow::Result;
use journey_db::entity::ImageDTO;
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::State;
use thiserror::Error;

use crate::AppData;

#[derive(Debug, Error, Serialize, Deserialize, Type)]
#[specta(type = String)]
pub enum ImageApiError {}

type ImageApiResult<T> = Result<T, ImageApiError>;

#[taurpc::procedures(path = "image")]
pub trait ImageApi {
    async fn get_images(state: State<AppData>) -> ImageApiResult<ImageDTO>;
}

#[derive(Clone, Debug)]
pub struct ImageApiImpl;

#[taurpc::resolvers]
impl ImageApi for ImageApiImpl {
    async fn get_images(self, state: State<AppData>) -> ImageApiResult<ImageDTO> {
        Ok(ImageDTO::default())
    }
}
