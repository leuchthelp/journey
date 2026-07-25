use crate::{db::Convertible, entity::MediaItemDTO};
use anyhow::Result;
use inherent::inherent;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "content")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub parent_id: Option<Uuid>,
    #[sea_orm(belongs_to, from = "parent_id", to = "uuid")]
    pub parent: BelongsTo<Option<super::media_items::Entity>>,
    #[sea_orm(unique)]
    pub kind: String,
    pub description: String,
}

impl ActiveModelBehavior for ActiveModel {}

#[taurpc::ipc_type]
#[derive(Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ContentDTO {
    pub id: i32,
    pub parent_id: Option<Uuid>,
    pub parent: Option<MediaItemDTO>,
    #[serde(rename = "type")]
    pub kind: String,
    pub description: String,
}

#[inherent]
impl Convertible<ModelEx> for ContentDTO {
    type DTO = ContentDTO;

    pub fn from_model(item: ModelEx) -> Result<Self> {
        let parent = MediaItemDTO::from_model(item.parent.unwrap().clone())?;

        Ok(ContentDTO {
            id: item.id,
            parent_id: item.parent_id,
            parent: Some(parent),
            kind: item.kind,
            description: item.description,
        })
    }
}
