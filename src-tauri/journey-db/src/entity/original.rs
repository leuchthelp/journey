use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

use crate::entity::{MediaItemsDTO, media_items::ConvertableMediaItems};

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "original")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub parent_id: Option<uuid::Uuid>,
    #[sea_orm(belongs_to, from = "parent_id", to = "uuid")]
    pub parent: BelongsTo<Option<super::media_items::Entity>>,
    pub server_id: uuid::Uuid,
    pub uuid: uuid::Uuid,
    #[sea_orm(unique)]
    pub url: String,
}

impl ActiveModelBehavior for ActiveModel {}

pub trait ConvertableOriginal {
    fn from_model(item: ModelEx) -> OriginalDTO;
}

#[taurpc::ipc_type]
#[derive(Debug)]
pub struct OriginalDTO {
    pub id: i32,
    pub parent_id: Option<uuid::Uuid>,
    pub parent: Option<MediaItemsDTO>,
    pub server_id: uuid::Uuid,
    pub uuid: uuid::Uuid,
    pub url: String,
}

impl ConvertableOriginal for OriginalDTO {
    fn from_model(item: ModelEx) -> OriginalDTO {
        let parent = MediaItemsDTO::from_model(item.parent.unwrap().clone());

        return OriginalDTO {
            id: item.id,
            parent_id: item.parent_id,
            parent: Some(parent),
            server_id: item.server_id,
            uuid: item.uuid,
            url: item.url,
        };
    }
}
