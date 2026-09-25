use anyhow::Result;
use journey_db::entity::{
    ProviderDTO, ProviderKey, ProviderVariant, providers::ProviderAuthSchema,
};
use journey_provider::{
    IndexerKey, IndexerManagerError, IndexerMsg, ProviderError, ProviderManagerError,
    ProviderManagerFn,
};
use serde::Serialize;
use specta::Type;
use tauri::ipc::Channel;
use thiserror::Error;

use crate::AppState;

#[derive(Debug, Error, Serialize, Type)]
pub enum ProviderApiError {
    #[error("Failed to send msg via channel: {0}")]
    FailedChannelSendError(String),
    #[error(transparent)]
    ProviderManagerError(#[from] ProviderManagerError),
    #[error(transparent)]
    ProviderError(#[from] ProviderError),
    #[error(transparent)]
    IndexerManagerError(#[from] IndexerManagerError),
}

type ProviderApiResult<T> = Result<T, ProviderApiError>;

#[taurpc::procedures(path = "Provider")]
pub trait ProviderApi {
    async fn get_supported_variants() -> Vec<ProviderVariant>;
    async fn get_supported_auth_schema(
        variant: ProviderVariant,
    ) -> ProviderApiResult<Vec<ProviderAuthSchema>>;
    async fn get_providers() -> ProviderApiResult<Vec<ProviderDTO>>;
    async fn get_provider(key: ProviderKey) -> ProviderApiResult<ProviderDTO>;
    async fn password_auth(
        url: String,
        ty: ProviderVariant,
        uname: String,
        psw: String,
    ) -> ProviderApiResult<ProviderKey>;
    async fn deregister(key: ProviderKey) -> ProviderApiResult<()>;
    async fn indexer_status(
        key: IndexerKey,
        on_event: Channel<IndexerMsg>,
    ) -> ProviderApiResult<()>;
}

#[derive(Clone, Debug)]
pub struct ProviderApiImpl {
    pub state: AppState,
}

#[taurpc::resolvers]
impl ProviderApi for ProviderApiImpl {
    async fn get_supported_variants(self) -> Vec<ProviderVariant> {
        let lock = self.state.read().await;
        lock.provider_manager.get_supported_variants()
    }
    async fn get_supported_auth_schema(
        self,
        variant: ProviderVariant,
    ) -> ProviderApiResult<Vec<ProviderAuthSchema>> {
        let lock = self.state.read().await;
        Ok(lock.provider_manager.get_supported_auth_schema(variant)?)
    }
    async fn get_providers(self) -> ProviderApiResult<Vec<ProviderDTO>> {
        let lock = self.state.read().await;
        let providers = lock.provider_manager.get_providers()?;
        Ok(providers)
    }
    async fn get_provider(self, key: ProviderKey) -> ProviderApiResult<ProviderDTO> {
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
    ) -> ProviderApiResult<ProviderKey> {
        let mut lock = self.state.write().await;
        let key = lock
            .provider_manager
            .password_auth(url, ty, uname, psw)
            .await?;
        Ok(key)
    }
    async fn deregister(self, key: ProviderKey) -> ProviderApiResult<()> {
        let mut lock = self.state.write().await;
        Ok(lock.provider_manager.deregister(&key).await?)
    }
    async fn indexer_status(
        self,
        key: IndexerKey,
        on_event: Channel<IndexerMsg>,
    ) -> ProviderApiResult<()> {
        let mut comm = self
            .state
            .write()
            .await
            .provider_manager
            .get_indexer_manager()
            .consume_status(&key)?;

        while let Some(msg) = comm.recv().await {
            match on_event.send(msg) {
                Ok(_) => Ok(()),
                Err(err) => Err(ProviderApiError::FailedChannelSendError(err.to_string())),
            }?;
        }

        Ok(())
    }
}
