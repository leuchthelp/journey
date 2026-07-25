use anyhow::Result;
use journey_db::entity::ProviderDTO;
use journey_provider::{ProviderManagerError, ProviderManagerFn};
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::State;
use thiserror::Error;

use crate::AppData;

#[derive(Debug, Error, Serialize, Deserialize, Type)]
#[specta(type = String)]
pub enum ProviderApiError {
    #[error(transparent)]
    ProviderManagerError(#[from] ProviderManagerError),
}

type ProviderApiResult<T> = Result<T, ProviderApiError>;

#[taurpc::procedures(path = "provider")]
pub trait ProviderApi {
    async fn get_providers(state: State<AppData>) -> ProviderApiResult<Vec<ProviderDTO>>;
}

#[derive(Clone, Debug)]
pub struct ProviderApiImpl;

#[taurpc::resolvers]
impl ProviderApi for ProviderApiImpl {
    async fn get_providers(self, state: State<AppData>) -> ProviderApiResult<Vec<ProviderDTO>> {
        let providers = state
            .provider_manager
            .get_providers()?
            .values()
            .collect::<Vec<&Box<_>>>();

        Ok()
    }
}
