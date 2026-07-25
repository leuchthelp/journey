use anyhow::Result;
use journey_db::entity::ContentDTO;
use serde::{Deserialize, Serialize};
use specta::Type;
use thiserror::Error;

use crate::AppState;

#[derive(Debug, Error, Serialize, Deserialize, Type)]
#[specta(type = String)]
pub enum ContentApiError {}

type ContentApiResult<T> = Result<T, ContentApiError>;

#[taurpc::procedures(path = "content")]
pub trait ContentApi {
    async fn get_content() -> ContentApiResult<ContentDTO>;
}

#[derive(Clone, Debug)]
pub struct ContentApiImpl {
    pub state: AppState,
}

#[taurpc::resolvers]
impl ContentApi for ContentApiImpl {
    async fn get_content(self) -> ContentApiResult<ContentDTO> {
        Ok(ContentDTO::default())
    }
}
