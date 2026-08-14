use anyhow::Result;
use journey_db::entity::{ProviderDTO, ProviderVariant};
use journey_provider::{ProviderError, ProviderManagerError, ProviderManagerFn};
use serde::Serialize;
use specta::Type;
use thiserror::Error;
use uuid::Uuid;

use crate::AppState;

#[derive(Debug, Error, Serialize, Type)]
pub enum ProviderApiError {
    #[error(transparent)]
    ProviderManagerError(#[from] ProviderManagerError),
    #[error(transparent)]
    ProviderError(#[from] ProviderError),
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
    async fn deregister(key: (Uuid, Uuid)) -> ProviderApiResult<()>;
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
    async fn deregister(self, key: (Uuid, Uuid)) -> ProviderApiResult<()> {
        let mut lock = self.state.write().await;
        Ok(lock.provider_manager.deregister(&key).await?)
    }
}
