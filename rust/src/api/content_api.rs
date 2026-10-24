use anyhow::Result;
use journey_db::entity::ContentDTO;
use serde::Serialize;
use thiserror::Error;

use crate::AppState;

#[derive(Debug, Error, Serialize)]
pub enum ContentApiError {}

type ContentApiResult<T> = Result<T, ContentApiError>;

pub trait ContentApi {
    async fn get_content(self) -> ContentApiResult<ContentDTO>;
}

#[derive(Clone)]
pub struct ContentApiImpl {
    pub state: AppState,
}

impl ContentApi for ContentApiImpl {
    async fn get_content(self) -> ContentApiResult<ContentDTO> {
        Ok(ContentDTO::default())
    }
}
