use crate::{db::Convertible, entity::MediaItemDTO};
use Uuid;
use inherent::inherent;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "original")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub parent_id: Option<Uuid>,
    #[sea_orm(belongs_to, from = "parent_id", to = "uuid")]
    pub parent: BelongsTo<Option<super::media_items::Entity>>,
    pub server_id: Uuid,
    pub uuid: Uuid,
    #[sea_orm(unique)]
    pub url: String,
}

impl ActiveModelBehavior for ActiveModel {}

#[taurpc::ipc_type]
#[derive(Debug, Default)]
pub struct OriginalDTO {
    pub id: i32,
    pub parent_id: Option<Uuid>,
    pub parent: Option<MediaItemDTO>,
    pub server_id: Uuid,
    pub uuid: Uuid,
    pub url: String,
}

#[inherent]
impl Convertible<ModelEx> for OriginalDTO {
    pub fn from_model(item: ModelEx) -> Self {
        let parent = MediaItemDTO::from_model(item.parent.unwrap().clone());

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
