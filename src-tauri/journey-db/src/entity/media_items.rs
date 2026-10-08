use std::collections::HashMap;

use anyhow::Result;
use inherent::inherent;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use specta::Type;
use strum_macros::{Display, EnumString};
use uuid::Uuid;

use crate::db::{ConversionResult, Convertible};
use crate::entity::content::{self, ContentType};
use crate::entity::images::{self, ImageType};
use crate::entity::{
    ContentDTO, ImageDTO, ProviderDTO, ProviderType, SourceDTO, media_items, providers,
};

#[derive(
    Display,
    Debug,
    Default,
    Serialize,
    Deserialize,
    Type,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    EnumIter,
    EnumString,
    DeriveValueType,
)]
#[sea_orm(value_type = "String")]
pub enum MediaItemType {
    #[default]
    Unknown,
    Audio,
    Playlist,
    Artist,
    Album,
    Genre,
}

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "media_items")]
pub struct Model {
    #[sea_orm(primary_key)]
    id: i32,
    #[sea_orm(unique)]
    pub music_brainz_id: Uuid,
    pub weak_id: String,
    pub is_tmp: bool,
    pub ty: MediaItemType,
    #[sea_orm(default = "#ff000000")]
    pub outline_gradient: String,
    #[sea_orm(has_many)]
    pub sources: HasMany<super::sources::Entity>,
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

#[taurpc::ipc_type]
#[derive(Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct MediaItemDTO {
    pub uuid: Uuid,
    pub is_tmp: bool,
    #[serde(rename = "type")]
    pub ty: MediaItemType,
    pub outline_gradient: Option<String>,
    pub sources: Option<Vec<SourceDTO>>,
    pub content: Option<HashMap<ContentType, ContentDTO>>,
    pub providers: Option<HashMap<ProviderType, Vec<ProviderDTO>>>,
    pub images: Option<HashMap<ImageType, Vec<ImageDTO>>>,
    pub children: Option<HashMap<MediaItemType, Vec<MediaItemDTO>>>,
    pub parents: Option<HashMap<MediaItemType, Vec<MediaItemDTO>>>,
}

#[inherent]
impl Convertible<ModelEx> for MediaItemDTO {
    type DTO = MediaItemDTO;

    pub fn from_model(item: ModelEx) -> ConversionResult<Self> {
        let sources = SourceDTO::to_vec(item.sources)?;
        let content = ContentDTO::to_hashmap(item.content, content::Column::Ty)?;
        let providers = ProviderDTO::to_hashmap_vec(item.providers, providers::Column::Ty)?;
        let images = ImageDTO::to_hashmap_vec(item.images, images::Column::Ty)?;
        let children = MediaItemDTO::to_hashmap_vec(item.children, media_items::Column::Ty)?;
        let parents = MediaItemDTO::to_hashmap_vec(item.parents, media_items::Column::Ty)?;

        Ok(MediaItemDTO {
            uuid: item.music_brainz_id,
            is_tmp: item.is_tmp,
            ty: item.ty,
            outline_gradient: Some(item.outline_gradient),
            sources,
            content,
            providers,
            images,
            children,
            parents,
        })
    }
}
