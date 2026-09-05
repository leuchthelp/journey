use crate::db::Convertible;
use crate::entity::{ContentDTO, ImageDTO, SourceDTO, ProviderDTO};
use anyhow::Result;
use inherent::inherent;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use specta::Type;
use strum_macros::{Display, EnumString};
use uuid::Uuid;

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
    pub uuid: Uuid,
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
    pub content: Option<Vec<ContentDTO>>,
    pub providers: Option<Vec<ProviderDTO>>,
    pub images: Option<Vec<ImageDTO>>,
    pub children: Option<Vec<MediaItemDTO>>,
    pub parents: Option<Vec<MediaItemDTO>>,
}

#[inherent]
impl Convertible<ModelEx> for MediaItemDTO {
    type DTO = MediaItemDTO;

    pub fn from_model(item: ModelEx) -> Result<Self> {
        let sources = SourceDTO::to_dto_vec(item.sources)?;
        let content = ContentDTO::to_dto_vec(item.content)?;
        let providers = ProviderDTO::to_dto_vec(item.providers)?;
        let images = ImageDTO::to_dto_vec(item.images)?;
        let children = MediaItemDTO::to_dto_vec(item.children)?;
        let parents = MediaItemDTO::to_dto_vec(item.parents)?;

        Ok(MediaItemDTO {
            uuid: item.uuid,
            is_tmp: item.is_tmp,
            ty: item.ty,
            outline_gradient: Some(item.outline_gradient),
            sources: sources,
            content: content,
            providers: providers,
            images: images,
            children: children,
            parents: parents,
        })
    }
}
