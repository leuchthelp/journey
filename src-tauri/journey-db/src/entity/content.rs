use crate::entity::{MediaItemsDTO, media_items::ConvertableMediaItems};
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

pub trait ConvertableContent {
    fn from_model(item: ModelEx) -> ContentDTO;
}

#[taurpc::ipc_type]
#[derive(Debug)]
pub struct ContentDTO {
    pub id: i32,
    pub parent_id: Option<Uuid>,
    pub parent: Option<MediaItemsDTO>,
    pub kind: String,
    pub description: String,
}

impl ConvertableContent for ContentDTO {
    fn from_model(item: ModelEx) -> ContentDTO {
        let parent = MediaItemsDTO::from_model(item.parent.unwrap().clone());

        return ContentDTO {
            id: item.id,
            parent_id: item.parent_id,
            parent: Some(parent),
            kind: item.kind,
            description: item.description,
        };
    }
}
