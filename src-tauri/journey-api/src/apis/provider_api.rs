use anyhow::Result;
use journey_db::{entity::ProviderDTO, sea_orm::sqlx::types::Uuid};
use journey_provider::{ProviderError, ProviderManagerError, ProviderManagerFn};
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::{Url, ipc::Channel};
use thiserror::Error;
use tokio::sync::TryLockError;
use tracing::warn;

use crate::AppState;

#[derive(Debug, Error, Serialize, Deserialize, Type)]
#[specta(type = String)]
pub enum ProviderApiError {
    #[error(transparent)]
    ProviderManagerError(#[from] ProviderManagerError),
    #[error(transparent)]
    ProviderError(#[from] ProviderError),
    #[error(transparent)]
    #[serde(skip)]
    ChannelSendFailureError(#[from] tauri::Error),
    #[error(transparent)]
    #[serde(skip)]
    TryLockError(#[from] TryLockError),
}

type ProviderApiResult<T> = Result<T, ProviderApiError>;

#[taurpc::procedures(path = "provider")]
pub trait ProviderApi {
    async fn get_existing_keys() -> ProviderApiResult<Vec<ProviderDTO>>;
}

#[derive(Clone, Debug)]
pub struct ProviderApiImpl {
    pub state: AppState,
}

#[taurpc::resolvers]
impl ProviderApi for ProviderApiImpl {
    async fn get_existing_keys(self) -> ProviderApiResult<Vec<ProviderDTO>> {
        let locked = self.state.lock().await;
        let providers = locked.provider_manager.get_existing_keys().await?;
        Ok(providers)
    }
}
