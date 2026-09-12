use anyhow::Result;
use journey_db::entity::SourceDTO;
use serde::Serialize;
use specta::Type;
use thiserror::Error;

use crate::AppState;

#[derive(Debug, Error, Serialize, Type)]
#[serde(tag = "error", content = "data")]
pub enum SourceApiError {}

type SourceResult<T> = Result<T, SourceApiError>;

#[taurpc::procedures(path = "Source")]
pub trait SourceApi {
    async fn get_source() -> SourceResult<SourceDTO>;
}

#[derive(Clone, Debug)]
pub struct SourceApiImpl {
    pub state: AppState,
}

#[taurpc::resolvers]
impl SourceApi for SourceApiImpl {
    async fn get_source(self) -> SourceResult<SourceDTO> {
        Ok(SourceDTO::default())
    }
}
