use crate::{
    db::Convertible,
    entity::{MediaItemDTO, ProviderDTO},
};
use anyhow::Result;
use inherent::inherent;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use url::Url;
use uuid::Uuid;

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "images")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[sea_orm(unique)]
    pub url: String,
    pub server_id: Option<Uuid>,
    #[sea_orm(belongs_to, from = "server_id", to = "server_id")]
    pub provider: BelongsTo<Option<super::providers::Entity>>,
    pub kind: String,
    #[sea_orm(has_many, via = "jt_media_item_to_image")]
    pub media_items: HasMany<super::media_items::Entity>,
}

impl ActiveModelBehavior for ActiveModel {}

pub trait ConvertableImage {
    fn from_model(item: ModelEx) -> ImageDTO;
}

#[taurpc::ipc_type]
#[derive(Debug)]
#[serde(rename_all = "camelCase")]
pub struct ImageDTO {
    pub url: Url,
    pub server_id: Option<Uuid>,
    pub provider: Option<ProviderDTO>,
    #[serde(rename = "type")]
    pub kind: String,
    pub media_items: Option<Vec<MediaItemDTO>>,
}

#[inherent]
impl Convertible<ModelEx> for ImageDTO {
    type DTO = ImageDTO;

    pub fn from_model(item: ModelEx) -> Result<Self> {
        let provider = ProviderDTO::from_model(item.provider.unwrap().clone())?;

        let mut media_items: Vec<MediaItemDTO> = vec![];
        for item in item.media_items {
            let dto = MediaItemDTO::from_model(item)?;
            media_items.push(dto);
        }

        Ok(ImageDTO {
            url: Url::parse(&item.url)?,
            server_id: item.server_id,
            provider: Some(provider),
            kind: item.kind,
            media_items: Some(media_items),
        })
    }
}
