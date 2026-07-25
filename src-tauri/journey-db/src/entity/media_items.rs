use crate::db::Convertible;
use crate::entity::{ContentDTO, ImageDTO, OriginalDTO, ProviderDTO};
use anyhow::Result;
use inherent::inherent;
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

#[taurpc::ipc_type]
#[derive(Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct MediaItemDTO {
    pub uuid: Uuid,
    #[serde(rename = "type")]
    pub kind: String,
    pub outline_gradient: String,
    pub loaded: bool,
    pub local: String,
    pub original: Option<Vec<OriginalDTO>>,
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
        let mut providers: Vec<ProviderDTO> = vec![];
        for item in item.providers {
            let dto = ProviderDTO::from_model(item)?;
            providers.push(dto);
        }
        let mut images: Vec<ImageDTO> = vec![];
        for item in item.images {
            let dto = ImageDTO::from_model(item)?;
            images.push(dto);
        }
        let mut children: Vec<MediaItemDTO> = vec![];
        for item in item.children {
            let dto = MediaItemDTO::from_model(item)?;
            children.push(dto);
        }
        let mut parents: Vec<MediaItemDTO> = vec![];
        for item in item.parents {
            let dto = MediaItemDTO::from_model(item)?;
            parents.push(dto);
        }

        Ok(MediaItemDTO {
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
        })
    }
}
