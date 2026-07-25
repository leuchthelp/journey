use anyhow::Result;
use journey_db::entity::OriginalDTO;
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::State;
use thiserror::Error;

use crate::AppState;

#[derive(Debug, Error, Serialize, Deserialize, Type)]
#[specta(type = String)]
pub enum OriginalApiError {}

type OriginalResult<T> = Result<T, OriginalApiError>;

#[taurpc::procedures(path = "original")]
pub trait OriginalApi {
    async fn get_original(state: State<AppState>) -> OriginalResult<OriginalDTO>;
}

#[derive(Clone, Debug)]
pub struct OriginalApiImpl;

#[taurpc::resolvers]
impl OriginalApi for OriginalApiImpl {
    async fn get_original(self, state: State<AppState>) -> OriginalResult<OriginalDTO> {
        Ok(OriginalDTO::default())
    }
}
