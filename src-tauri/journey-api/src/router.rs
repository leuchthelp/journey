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
        .merge(MediaItemApiImpl.into_handler())
        .merge(ContentApiImpl.into_handler())
        .merge(ImageApiImpl.into_handler())
        .merge(ProviderApiImpl.into_handler())
        .merge(OriginalApiImpl.into_handler());

    Ok(router)
}
