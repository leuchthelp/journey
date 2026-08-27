use crate::{db::Convertible, entity::MediaItemDTO};
use Uuid;
use anyhow::Result;
use inherent::inherent;
use sea_orm::entity::prelude::*;

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "original")]
pub struct Model {
    #[sea_orm(primary_key)]
    id: i32,
    #[sea_orm(unique)]
    pub uuid: Uuid,
    pub parent_id: Option<Uuid>,
    #[sea_orm(belongs_to, from = "parent_id", to = "uuid")]
    pub parent: BelongsTo<Option<super::media_items::Entity>>,
    pub server_id: Uuid,
}

impl ActiveModelBehavior for ActiveModel {}

#[taurpc::ipc_type]
#[derive(Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct OriginalDTO {
    pub parent_id: Option<Uuid>,
    pub parent: Option<MediaItemDTO>,
    pub server_id: Uuid,
}

#[inherent]
impl Convertible<ModelEx> for OriginalDTO {
    type DTO = OriginalDTO;

    pub fn from_model(item: ModelEx) -> Result<Self> {
        let parent = match item.parent.into_option() {
            Some(parent) => Some(MediaItemDTO::from_model(parent)?),
            None => None,
        };

        Ok(OriginalDTO {
            parent_id: item.parent_id,
            parent: parent,
            server_id: item.server_id,
        })
    }
}
