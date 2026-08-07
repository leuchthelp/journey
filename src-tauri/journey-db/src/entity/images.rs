use crate::{
    db::Convertible,
    entity::{MediaItemDTO, ProviderDTO},
};
use anyhow::Result;
use inherent::inherent;
use sea_orm::entity::prelude::*;
use url::Url;
use uuid::Uuid;

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "images")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[sea_orm(unique)]
    pub url: String,
    pub server_id: Option<Uuid>,
    #[sea_orm(belongs_to, from = "server_id", to = "server_id")]
    pub provider: BelongsTo<Option<super::providers::Entity>>,
    pub ty: String,
    #[sea_orm(has_many, via = "jt_media_item_to_image")]
    pub media_items: HasMany<super::media_items::Entity>,
}

impl ActiveModelBehavior for ActiveModel {}

#[taurpc::ipc_type]
#[derive(Debug)]
#[serde(rename_all = "camelCase")]
pub struct ImageDTO {
    pub url: Url,
    pub server_id: Option<Uuid>,
    pub provider: Option<ProviderDTO>,
    #[serde(rename = "type")]
    pub ty: String,
    pub media_items: Option<Vec<MediaItemDTO>>,
}

#[inherent]
impl Convertible<ModelEx> for ImageDTO {
    type DTO = ImageDTO;

    pub fn from_model(item: ModelEx) -> Result<Self> {
        let provider = match item.provider.into_option() {
            Some(provider) => Some(ProviderDTO::from_model(provider)?),
            None => None,
        };

        let media_items = MediaItemDTO::to_vec(item.media_items)?;

        Ok(ImageDTO {
            url: Url::parse(&item.url)?,
            server_id: item.server_id,
            provider: provider,
            ty: item.ty,
            media_items: media_items,
        })
    }
}

impl ImageDTO {
    pub fn new() -> Self {
        return ImageDTO {
            url: Url::parse("https://example.net").unwrap(),
            ty: "Primary".into(),
            server_id: None,
            provider: None,
            media_items: None,
        };
    }
}
