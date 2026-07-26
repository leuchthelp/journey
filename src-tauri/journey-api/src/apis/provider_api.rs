use anyhow::Result;
use journey_db::entity::ProviderDTO;
use journey_provider::{ProviderError, ProviderManagerError};
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::ipc::Channel;
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
    async fn get_providers(on_event: Channel<ProviderDTO>) -> ProviderApiResult<()>;
}

#[derive(Clone, Debug)]
pub struct ProviderApiImpl {
    pub state: AppState,
}

#[taurpc::resolvers]
impl ProviderApi for ProviderApiImpl {
    async fn get_providers(self, on_event: Channel<ProviderDTO>) -> ProviderApiResult<()> {
        warn!("passing providers to frontend");
        let locked = self.state.lock().await;

        let providers = locked
            .provider_manager
            .get_providers()?
            .values()
            .collect::<Vec<&Box<_>>>();

        for provider in providers {
            let dto = ProviderDTO {
                url: provider.url().clone(),
                server_id: provider.server_id()?,
                user_id: provider.user_id()?,
                kind: provider.type_(),
                media_items: None,
                images: None,
            };

            on_event.send(dto)?;
        }

        Ok(())
    }
}
