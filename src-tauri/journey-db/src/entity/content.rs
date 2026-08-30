use crate::{db::Convertible, entity::MediaItemDTO};
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
pub enum ContentType {
    #[default]
    Unknown,
    Album,
    Artists,
    Container,
    ReleaseDate,
}

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "content")]
pub struct Model {
    #[sea_orm(primary_key)]
    id: i32,
    pub parent_id: Option<Uuid>,
    #[sea_orm(belongs_to, from = "parent_id", to = "uuid")]
    pub parent: BelongsTo<Option<super::media_items::Entity>>,
    pub ty: ContentType,
    pub description: Option<String>,
}

impl ActiveModelBehavior for ActiveModel {}

#[taurpc::ipc_type]
#[derive(Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ContentDTO {
    pub parent_id: Option<Uuid>,
    pub parent: Option<MediaItemDTO>,
    #[serde(rename = "type")]
    pub ty: ContentType,
    pub description: Option<String>,
}

#[inherent]
impl Convertible<ModelEx> for ContentDTO {
    type DTO = ContentDTO;

    pub fn from_model(item: ModelEx) -> Result<Self> {
        let parent = match item.parent.into_option() {
            Some(parent) => Some(MediaItemDTO::from_model(parent)?),
            None => None,
        };

        Ok(ContentDTO {
            parent_id: item.parent_id,
            parent: parent,
            ty: ContentType::Unknown,
            description: item.description,
        })
    }
}
