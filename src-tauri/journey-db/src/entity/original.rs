use crate::{db::Convertible, entity::MediaItemDTO};
use Uuid;
use inherent::inherent;
use anyhow::Result;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use url::Url;

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
#[derive(Debug)]
#[serde(rename_all = "camelCase")]
pub struct OriginalDTO {
    pub id: i32,
    pub parent_id: Option<Uuid>,
    pub parent: Option<MediaItemDTO>,
    pub server_id: Uuid,
    pub uuid: Uuid,
    pub url: Url,
}

#[inherent]
impl Convertible<ModelEx> for OriginalDTO {
    type DTO = OriginalDTO;

    pub fn from_model(item: ModelEx) -> Result<Self> {
        let parent = MediaItemDTO::from_model(item.parent.unwrap().clone())?;

        Ok(OriginalDTO {
            id: item.id,
            parent_id: item.parent_id,
            parent: Some(parent),
            server_id: item.server_id,
            uuid: item.uuid,
            url: Url::parse(&item.url)?,
        })
    }
}
