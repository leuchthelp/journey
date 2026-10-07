use anyhow::Result;
use journey_db::entity::{
    ProviderDTO, ProviderKey, ProviderVariant, providers::ProviderAuthSchema,
};
use journey_playback::audio_player::AppendStream;
use journey_provider::{
    IndexerKey, IndexerManagerError, IndexerMsg, ProviderError, ProviderManagerError,
    ProviderManagerFn,
};
use serde::Serialize;
use specta::Type;
use tauri::ipc::Channel;
use thiserror::Error;
use uuid::Uuid;

use crate::AppState;

#[derive(Debug, Error, Serialize, Type)]
pub enum ProviderApiError {
    #[error("Failed to send msg via channel: {0}")]
    FailedChannelSendError(String),
    #[error("Failed to send msg via channel: {0}")]
    FailedAppendStreamError(String),
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
    async fn append_stream(key: ProviderKey, uuid: Uuid) -> ProviderApiResult<()>;
}

#[derive(Clone)]
pub struct ProviderApiImpl {
    pub state: AppState,
}

#[taurpc::resolvers]
impl ProviderApi for ProviderApiImpl {
    async fn get_supported_variants(self) -> Vec<ProviderVariant> {
        let lock = self.state.provider_manager.read().await;
        lock.get_supported_variants()
    }
    async fn get_supported_auth_schema(
        self,
        variant: ProviderVariant,
    ) -> ProviderApiResult<Vec<ProviderAuthSchema>> {
        let lock = self.state.provider_manager.read().await;
        Ok(lock.get_supported_auth_schema(variant)?)
    }
    async fn get_providers(self) -> ProviderApiResult<Vec<ProviderDTO>> {
        let lock = self.state.provider_manager.read().await;
        let providers = lock.get_providers()?;
        Ok(providers)
    }
    async fn get_provider(self, key: ProviderKey) -> ProviderApiResult<ProviderDTO> {
        let lock = self.state.provider_manager.read().await;
        let provider = lock.get_provider(&key)?;
        Ok(provider)
    }
    async fn password_auth(
        self,
        url: String,
        ty: ProviderVariant,
        uname: String,
        psw: String,
    ) -> ProviderApiResult<ProviderKey> {
        let mut lock = self.state.provider_manager.write().await;
        let key = lock.password_auth(url, ty, uname, psw).await?;
        Ok(key)
    }
    async fn deregister(self, key: ProviderKey) -> ProviderApiResult<()> {
        let mut lock = self.state.provider_manager.write().await;
        Ok(lock.deregister(&key).await?)
    }
    async fn indexer_status(
        self,
        key: IndexerKey,
        on_event: Channel<IndexerMsg>,
    ) -> ProviderApiResult<()> {
        let mut recv = self
            .state
            .provider_manager
            .write()
            .await
            .get_mut_indexer_manager()
            .consume_status(&key)?;

        while let Some(msg) = recv.recv().await {
            match on_event.send(msg) {
                Ok(_) => Ok(()),
                Err(err) => Err(ProviderApiError::FailedChannelSendError(err.to_string())),
            }?;
        }

        Ok(())
    }
    async fn append_stream(self, key: ProviderKey, uuid: Uuid) -> ProviderApiResult<()> {
        let lock = self.state.provider_manager.read().await;
        let response = lock.get_audio_stream(key, uuid).await?;

        match self.state.audio_player.ask(AppendStream { response }).await {
            Ok(_) => Ok(()),
            Err(err) => Err(ProviderApiError::FailedAppendStreamError(err.to_string())),
        }
    }
}
