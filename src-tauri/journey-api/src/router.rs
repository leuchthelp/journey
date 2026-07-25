use anyhow::Result;
use journey_provider::{ProviderManager, ProviderManagerFn};
use std::sync::Arc;
use tauri::Wry;
use taurpc::Router;
use tokio::sync::Mutex;

use crate::apis::{
    content_api::{ContentApi, ContentApiImpl},
    image_api::{ImageApi, ImageApiImpl},
    media_item_api::{MediaItemApi, MediaItemApiImpl},
    original_api::{OriginalApi, OriginalApiImpl},
    provider_api::{ProviderApi, ProviderApiImpl},
};

pub async fn get_router() -> Result<Router<Wry>> {
    let mut provider_manager = ProviderManager::default();
    provider_manager.init().await?;

    let state = AppState::new(Mutex::new(AppStateInner { provider_manager }));

    let router = taurpc::Router::new()
        .merge(MediaItemApiImpl.into_handler())
        .merge(ContentApiImpl.into_handler())
        .merge(ImageApiImpl.into_handler())
        .merge(ProviderApiImpl { state }.into_handler())
        .merge(OriginalApiImpl.into_handler());

    Ok(router)
}

#[derive(Clone, Debug)]
pub struct AppStateInner {
    pub provider_manager: ProviderManager,
}

pub type AppState = Arc<Mutex<AppStateInner>>;
