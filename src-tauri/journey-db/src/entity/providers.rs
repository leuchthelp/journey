use anyhow::Result;
use bon::Builder;
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
    pub hash: u64,
    pub kind: String,
    pub url: String,
    #[sea_orm(has_many, via = "jt_media_item_to_provider")]
    pub media_items: HasMany<super::media_items::Entity>,
    #[sea_orm(has_many)]
    pub images: HasMany<super::images::Entity>,
}

impl ActiveModelBehavior for ActiveModel {}

#[taurpc::ipc_type]
#[derive(Debug, Builder)]
#[serde(rename_all = "camelCase")]
pub struct ProviderDTO {
    pub authenticated: bool,
    pub hash: u64,
    pub server_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
    #[serde(rename = "type")]
    pub kind: String,
    pub url: Option<Url>,
    pub media_items: Option<Vec<MediaItemDTO>>,
    pub images: Option<Vec<ImageDTO>>,
}

#[inherent]
impl Convertible<ModelEx> for ProviderDTO {
    type DTO = ProviderDTO;

    pub fn from_model(item: ModelEx) -> Result<Self> {
        let parents = MediaItemDTO::to_vec(item.media_items)?;
        let images = ImageDTO::to_vec(item.images)?;

        Ok(ProviderDTO {
            authenticated: false,
            hash: item.hash,
            user_id: Some(item.user_id),
            server_id: Some(item.server_id),
            kind: item.kind,
            url: Some(Url::parse(&item.url)?),
            media_items: parents,
            images: images,
        })
    }
}
