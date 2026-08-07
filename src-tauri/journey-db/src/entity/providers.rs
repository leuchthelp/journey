use crate::{
    db::Convertible,
    entity::{ImageDTO, MediaItemDTO},
};
use anyhow::Result;
use bon::Builder;
use inherent::inherent;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use specta::Type;
use url::Url;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Type, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "i32", db_type = "Integer")]
pub enum ProviderVariant {
    #[sea_orm(num_value = 0)]
    JellyfinProvider,
}

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "providers")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub server_id: Uuid,
    #[sea_orm(unique)]
    pub user_id: Uuid,
    pub ty: ProviderVariant,
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
    pub server_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
    #[serde(rename = "type")]
    pub ty: ProviderVariant,
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
            user_id: Some(item.user_id),
            server_id: Some(item.server_id),
            ty: item.ty,
            url: Some(Url::parse(&item.url)?),
            media_items: parents,
            images: images,
        })
    }
}
