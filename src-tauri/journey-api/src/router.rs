use std::sync::Arc;

use anyhow::Result;
use journey_media_item::MediaItemManager;
use journey_provider::{ProviderManager, ProviderManagerFn};
use tauri::Wry;
use taurpc::Router;
use tokio::sync::RwLock;

use crate::apis::{
    content_api::{ContentApi, ContentApiImpl},
    image_api::{ImageApi, ImageApiImpl},
    media_item_api::{MediaItemApi, MediaItemApiImpl},
    provider_api::{ProviderApi, ProviderApiImpl},
    source_api::{SourceApi, SourceApiImpl},
};

pub async fn get_router() -> Result<Router<Wry>> {
    let mut provider_manager = ProviderManager::default();
    provider_manager.init().await?;

    let media_item_manager = MediaItemManager::default();

    let state = AppState::new(RwLock::new(AppStateInner {
        provider_manager,
        media_item_manager,
    }));

    let router = taurpc::Router::new()
        .merge(
            MediaItemApiImpl {
                state: state.clone(),
            }
            .into_handler(),
        )
        .merge(
            ContentApiImpl {
                state: state.clone(),
            }
            .into_handler(),
        )
        .merge(
            ImageApiImpl {
                state: state.clone(),
            }
            .into_handler(),
        )
        .merge(
            ProviderApiImpl {
                state: state.clone(),
            }
            .into_handler(),
        )
        .merge(
            SourceApiImpl {
                state: state.clone(),
            }
            .into_handler(),
        );

    Ok(router)
}

#[derive(Debug)]
pub struct AppStateInner {
    pub provider_manager: ProviderManager,
    pub media_item_manager: MediaItemManager,
}

pub type AppState = Arc<RwLock<AppStateInner>>;
