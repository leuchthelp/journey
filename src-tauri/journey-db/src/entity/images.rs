use std::collections::HashMap;

use anyhow::Result;
use inherent::inherent;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use specta::Type;
use strum_macros::{Display, EnumString};
use url::Url;
use uuid::Uuid;

use crate::{
    db::{ConversionError, ConversionResult, Convertible},
    entity::{
        MediaItemDTO, ProviderDTO,
        media_items::{self, MediaItemType},
    },
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
    pub ty: ImageType,
    pub provider_id: Option<Uuid>,
    #[sea_orm(belongs_to, from = "provider_id", to = "provider_id")]
    pub provider: BelongsTo<Option<super::providers::Entity>>,
    #[sea_orm(has_many, via = "jt_media_item_to_image")]
    pub media_items: HasMany<super::media_items::Entity>,
}

impl ActiveModelBehavior for ActiveModel {}

#[taurpc::ipc_type]
#[derive(Debug)]
#[serde(rename_all = "camelCase")]
pub struct ImageDTO {
    pub url: Url,
    #[serde(rename = "type")]
    pub ty: ImageType,
    pub provider_id: Option<Uuid>,
    pub provider: Option<ProviderDTO>,
    pub media_items: Option<HashMap<MediaItemType, Vec<MediaItemDTO>>>,
}

#[inherent]
impl Convertible<ModelEx> for ImageDTO {
    type DTO = ImageDTO;

    pub fn from_model(item: ModelEx) -> ConversionResult<Self> {
        let provider = ProviderDTO::option_from_option(item.provider)?;
        let media_items = MediaItemDTO::to_hashmap_vec(item.media_items, media_items::Column::Ty)?;

        let url = match Url::parse(&item.url) {
            Ok(url) => Ok(url),
            Err(err) => Err(ConversionError::FailedParseUrlError(err.to_string())),
        }?;

        Ok(ImageDTO {
            url,
            ty: item.ty,
            provider_id: item.provider_id,
            provider,
            media_items,
        })
    }
}

impl Default for ImageDTO {
    fn default() -> Self {
        ImageDTO {
            url: Url::parse("https://example.net").unwrap(),
            ty: ImageType::Unknown,
            provider_id: None,
            provider: None,
            media_items: None,
        }
    }
}
