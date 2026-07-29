use anyhow::Result;
use journey_db::entity::ProviderDTO;
use journey_provider::{ProviderError, ProviderManagerError, ProviderManagerFn};
use serde::{Deserialize, Serialize};
use specta::Type;
use thiserror::Error;
use tokio::sync::TryLockError;

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
    async fn get_providers() -> ProviderApiResult<Vec<ProviderDTO>>;
    async fn get_provider(key: u64) -> ProviderApiResult<ProviderDTO>;
    async fn password_auth(
        url: String,
        kind: String,
        uname: String,
        psw: String,
    ) -> ProviderApiResult<()>;
}

#[derive(Clone, Debug)]
pub struct ProviderApiImpl {
    pub state: AppState,
}

#[taurpc::resolvers]
impl ProviderApi for ProviderApiImpl {
    async fn get_providers(self) -> ProviderApiResult<Vec<ProviderDTO>> {
        let lock = self.state.read().await;
        let providers = lock.provider_manager.get_providers().await?;
        Ok(providers)
    }
    async fn get_provider(self, key: u64) -> ProviderApiResult<ProviderDTO> {
        let lock = self.state.read().await;
        let provider = lock.provider_manager.get_provider(&key)?;
        Ok(provider)
    }
    async fn password_auth(
        self,
        url: String,
        kind: String,
        uname: String,
        psw: String,
    ) -> ProviderApiResult<()> {
        let mut lock = self.state.write().await;
        lock.provider_manager
            .password_auth(url, kind, uname, psw)
            .await?;
        Ok(())
    }
}
