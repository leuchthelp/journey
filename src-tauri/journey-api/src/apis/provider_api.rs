use anyhow::Result;
use journey_db::entity::{ProviderDTO, ProviderVariant};
use journey_provider::{ProviderError, ProviderManagerError, ProviderManagerFn};
use serde::Serialize;
use specta::Type;
use thiserror::Error;
use tokio::sync::TryLockError;
use uuid::Uuid;

use crate::AppState;

#[derive(Debug, Error, Serialize, Type)]
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
    async fn get_provider(key: (Uuid, Uuid)) -> ProviderApiResult<ProviderDTO>;
    async fn password_auth(
        url: String,
        ty: ProviderVariant,
        uname: String,
        psw: String,
    ) -> ProviderApiResult<(Uuid, Uuid)>;
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
    async fn get_provider(self, key: (Uuid, Uuid)) -> ProviderApiResult<ProviderDTO> {
        let lock = self.state.read().await;
        let provider = lock.provider_manager.get_provider(&key)?;
        Ok(provider)
    }
    async fn password_auth(
        self,
        url: String,
        ty: ProviderVariant,
        uname: String,
        psw: String,
    ) -> ProviderApiResult<(Uuid, Uuid)> {
        let mut lock = self.state.write().await;
        let key = lock
            .provider_manager
            .password_auth(url, ty, uname, psw)
            .await?;
        Ok(key)
    }
}
