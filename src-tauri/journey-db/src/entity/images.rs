use crate::{
    db::Convertible,
    entity::{MediaItemDTO, ProviderDTO},
};
use anyhow::Result;
use inherent::inherent;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use specta::Type;
use strum_macros::{Display, EnumString};
use url::Url;
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
#[non_exhaustive]
pub enum ImageType {
    #[default]
    Unknown,
    Primary,
    Art,
    Backdrop,
    Banner,
    Logo,
    Thumb,
    Disc,
    Box,
    Screenshot,
    Menu,
    Chapter,
    BoxRear,
    Profile,
}

#[sea_orm::model]
#[derive(Default, Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "images")]
pub struct Model {
    #[sea_orm(primary_key)]
    id: i32,
    #[sea_orm(unique)]
    pub url: String,
    pub server_id: Option<Uuid>,
    #[sea_orm(belongs_to, from = "server_id", to = "server_id")]
    pub provider: BelongsTo<Option<super::providers::Entity>>,
    pub ty: ImageType,
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
    pub ty: ImageType,
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

        let media_items = MediaItemDTO::to_dto_vec(item.media_items)?;

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
            ty: ImageType::Primary,
            server_id: None,
            provider: None,
            media_items: None,
        };
    }
}
