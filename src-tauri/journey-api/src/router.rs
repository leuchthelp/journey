use anyhow::Result;
use tauri::Wry;
use taurpc::Router;

use crate::apis::{
    content_api::{ContentApi, ContentApiImpl},
    image_api::{ImageApi, ImageApiImpl},
    media_item_api::{MediaItemApi, MediaItemApiImpl},
    original_api::{OriginalApi, OriginalApiImpl},
    provider_api::{ProviderApi, ProviderApiImpl},
};

pub fn get_router() -> Result<Router<Wry>> {
    let router = taurpc::Router::new()
        .merge(MediaItemApi.into_handler())
        .merge(ContentApi.into_handler())
        .merge(ImageApi.into_handler())
        .merge(ProviderApi.into_handler())
        .merge(OriginalApi.into_handler());

    Ok(router)
}
