use std::{collections::HashMap, hash::Hash};

use crate::{
    ConversionError,
    db::{ConversionResult, Convertible},
    entity::{
        ImageDTO, MediaItemDTO,
        images::{self, ImageType},
        media_items::{self, MediaItemType},
    },
};
use anyhow::Result;
use bon::Builder;
use inherent::inherent;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use specta::Type;
use strum_macros::{Display, EnumString};
use url::Url;
use uuid::Uuid;

#[derive(
    Default,
    Display,
    Debug,
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
pub enum ProviderType {
    #[default]
    Unknown,
    JellyfinProvider,
}

#[derive(
    Default,
    Display,
    Debug,
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
pub enum ProviderAuthSchema {
    #[default]
    Unknown,
    Password,
    OTP,
    Oauth,
}

#[sea_orm::model]
#[derive(Default, Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "providers")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub provider_id: Uuid,
    #[sea_orm(unique)]
    pub user_id: Uuid,
    pub ty: ProviderType,
    pub url: String,
    #[sea_orm(has_many, via = "jt_media_item_to_provider")]
    pub media_items: HasMany<super::media_items::Entity>,
    #[sea_orm(has_many)]
    pub images: HasMany<super::images::Entity>,
}

impl ActiveModelBehavior for ActiveModel {}

#[taurpc::ipc_type]
#[derive(Debug, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct ProviderKey {
    pub user_id: Uuid,
    pub provider_id: Uuid,
}

#[taurpc::ipc_type]
#[derive(Debug, Builder)]
#[serde(rename_all = "camelCase")]
pub struct ProviderDTO {
    pub authenticated: bool,
    pub key: ProviderKey,
    #[serde(rename = "type")]
    pub ty: ProviderType,
    pub url: Url,
    pub media_items: Option<HashMap<MediaItemType, Vec<MediaItemDTO>>>,
    pub images: Option<HashMap<ImageType, Vec<ImageDTO>>>,
}

#[inherent]
impl Convertible<ModelEx> for ProviderDTO {
    type DTO = ProviderDTO;

    pub fn from_model(item: ModelEx) -> ConversionResult<Self> {
        let media_items = MediaItemDTO::to_hashmap_vec(item.media_items, media_items::Column::Ty)?;
        let images = ImageDTO::to_hashmap_vec(item.images, images::Column::Ty)?;

        let url = match Url::parse(&item.url) {
            Ok(url) => Ok(url),
            Err(err) => Err(ConversionError::FailedParseUrlError(err.to_string())),
        }?;

        Ok(ProviderDTO {
            authenticated: false,
            key: ProviderKey {
                user_id: item.user_id,
                provider_id: item.provider_id,
            },
            ty: item.ty,
            url,
            media_items,
            images,
        })
    }
}
