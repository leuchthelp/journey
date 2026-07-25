use anyhow::Result;
use inherent::inherent;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use url::Url;
use uuid::Uuid;

use crate::{
    db::Convertible,
    entity::{ImageDTO, MediaItemDTO},
};

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "providers")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub server_id: Uuid,
    #[sea_orm(unique)]
    pub user_id: Uuid,
    pub kind: String,
    pub url: String,
    #[sea_orm(has_many, via = "jt_media_item_to_provider")]
    pub media_items: HasMany<super::media_items::Entity>,
    #[sea_orm(has_many)]
    pub images: HasMany<super::images::Entity>,
}

impl ActiveModelBehavior for ActiveModel {}

#[taurpc::ipc_type]
#[derive(Debug)]
#[serde(rename_all = "camelCase")]
pub struct ProviderDTO {
    pub server_id: Uuid,
    pub user_id: Uuid,
    #[serde(rename = "type")]
    pub kind: String,
    pub url: Url,
    pub media_items: Option<Vec<MediaItemDTO>>,
    pub images: Option<Vec<ImageDTO>>,
}

#[inherent]
impl Convertible<ModelEx> for ProviderDTO {
    type DTO = ProviderDTO;

    pub fn from_model(item: ModelEx) -> Result<Self> {
        let mut parents: Vec<MediaItemDTO> = vec![];
        for item in item.media_items {
            let dto = MediaItemDTO::from_model(item)?;
            parents.push(dto);
        }

        let mut images: Vec<ImageDTO> = vec![];
        for item in item.images {
            let dto = ImageDTO::from_model(item)?;
            images.push(dto);
        }

        Ok(ProviderDTO {
            user_id: item.user_id,
            server_id: item.server_id,
            kind: item.kind,
            url: Url::parse(&item.url)?,
            media_items: Some(parents),
            images: Some(images),
        })
    }
}
