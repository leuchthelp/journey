use crate::entity::{
    ContentDTO, ImagesDTO, OriginalDTO, ProviderDTO, content::ConvertableContent,
    images::ConvertableImage, original::ConvertableOriginal, providers::ConvertableProvider,
};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "media_items")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    #[sea_orm(unique)]
    pub uuid: Uuid,
    pub kind: String,
    pub outline_gradient: String,
    pub loaded: bool,
    pub local: String,
    #[sea_orm(has_many)]
    pub original: HasMany<super::original::Entity>,
    #[sea_orm(has_many)]
    pub content: HasMany<super::content::Entity>,
    #[sea_orm(has_many, via = "jt_media_item_to_provider")]
    pub providers: HasMany<super::providers::Entity>,
    #[sea_orm(has_many, via = "jt_media_item_to_image")]
    pub images: HasMany<super::images::Entity>,
    #[sea_orm(
        self_ref,
        via = "jt_parent_to_child",
        from = "MediaItems",
        to = "Child"
    )]
    pub children: HasMany<Entity>,
    #[sea_orm(self_ref, via = "jt_parent_to_child", reverse)]
    pub parents: HasMany<Entity>,
}

impl ActiveModelBehavior for ActiveModel {}

pub trait ConvertableMediaItems {
    fn from_model(item: ModelEx) -> MediaItemsDTO;
}

#[taurpc::ipc_type]
#[derive(Debug)]
pub struct MediaItemsDTO {
    pub uuid: Uuid,
    pub kind: String,
    pub outline_gradient: String,
    pub loaded: bool,
    pub local: String,
    pub original: Option<Vec<OriginalDTO>>,
    pub content: Option<Vec<ContentDTO>>,
    pub providers: Option<Vec<ProviderDTO>>,
    pub images: Option<Vec<ImagesDTO>>,
    pub children: Option<Vec<MediaItemsDTO>>,
    pub parents: Option<Vec<MediaItemsDTO>>,
}

impl ConvertableMediaItems for MediaItemsDTO {
    fn from_model(item: ModelEx) -> MediaItemsDTO {
        let original = item
            .original
            .iter()
            .map(|f| OriginalDTO::from_model(f.clone()))
            .collect::<Vec<_>>();
        let content = item
            .content
            .iter()
            .map(|f| ContentDTO::from_model(f.clone()))
            .collect::<Vec<_>>();
        let providers = item
            .providers
            .iter()
            .map(|f| ProviderDTO::from_model(f.clone()))
            .collect::<Vec<_>>();
        let images = item
            .images
            .iter()
            .map(|f| ImagesDTO::from_model(f.clone()))
            .collect::<Vec<_>>();
        let children = item
            .children
            .iter()
            .map(|f| MediaItemsDTO::from_model(f.clone()))
            .collect::<Vec<_>>();
        let parents = item
            .children
            .iter()
            .map(|f| MediaItemsDTO::from_model(f.clone()))
            .collect::<Vec<_>>();

        return MediaItemsDTO {
            uuid: item.uuid,
            kind: item.kind,
            outline_gradient: item.outline_gradient,
            loaded: item.loaded,
            local: item.local,
            original: Some(original),
            content: Some(content),
            providers: Some(providers),
            images: Some(images),
            children: Some(children),
            parents: Some(parents),
        };
    }
}
