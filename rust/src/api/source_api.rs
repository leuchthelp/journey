use anyhow::Result;
use journey_db::entity::SourceDTO;
use serde::Serialize;
use thiserror::Error;

use crate::AppState;

#[derive(Debug, Error, Serialize)]
//#[serde(tag = "error", content = "data")]
pub enum SourceApiError {}

type SourceResult<T> = Result<T, SourceApiError>;

pub trait SourceApi {
    async fn get_source(self) -> SourceResult<SourceDTO>;
}

#[derive(Clone)]
pub struct SourceApiImpl {
    pub state: AppState,
}

impl SourceApi for SourceApiImpl {
    async fn get_source(self) -> SourceResult<SourceDTO> {
        Ok(SourceDTO::default())
    }
}
