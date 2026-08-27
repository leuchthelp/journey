use anyhow::Result;
use journey_db::entity::OriginalDTO;
use serde::{Deserialize, Serialize};
use specta::Type;
use thiserror::Error;

use crate::AppState;

#[derive(Debug, Error, Serialize, Deserialize, Type)]
#[specta(type = String)]
pub enum OriginalApiError {}

type OriginalResult<T> = Result<T, OriginalApiError>;

#[taurpc::procedures(path = "original")]
pub trait OriginalApi {
    async fn get_original() -> OriginalResult<OriginalDTO>;
}

#[derive(Clone, Debug)]
pub struct OriginalApiImpl {
    pub state: AppState,
}

#[taurpc::resolvers]
impl OriginalApi for OriginalApiImpl {
    async fn get_original(self) -> OriginalResult<OriginalDTO> {
        Ok(OriginalDTO::default())
    }
}
