use crate::{db::Convertible, entity::MediaItemDTO};
use Uuid;
use anyhow::Result;
use inherent::inherent;
use sea_orm::entity::prelude::*;
use url::Url;

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "original")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub parent_id: Option<Uuid>,
    #[sea_orm(belongs_to, from = "parent_id", to = "uuid")]
    pub parent: BelongsTo<Option<super::media_items::Entity>>,
    pub server_id: Uuid,
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
    pub url: Url,
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
            id: item.id,
            parent_id: item.parent_id,
            parent: parent,
            server_id: item.server_id,
            url: Url::parse(&item.url)?,
        })
    }
}

impl OriginalDTO {
    pub fn new() -> Self {
        return OriginalDTO {
            url: Url::parse("https://example.net").unwrap(),
            id: -1,
            parent_id: None,
            parent: None,
            server_id: Uuid::nil(),
        };
    }
}
