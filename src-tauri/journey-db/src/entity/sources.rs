use crate::{
    db::{ConversionResult, Convertible},
    entity::MediaItemDTO,
};
use Uuid;
use anyhow::Result;
use inherent::inherent;
use sea_orm::entity::prelude::*;

#[sea_orm::model]
#[derive(Default, Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "sources")]
pub struct Model {
    #[sea_orm(primary_key)]
    id: i32,
    #[sea_orm(unique)]
    pub source_id: Uuid,
    pub parent_id: Option<Uuid>,
    #[sea_orm(belongs_to, from = "parent_id", to = "music_brainz_id")]
    pub parent: BelongsTo<Option<super::media_items::Entity>>,
    pub provider_id: Uuid,
}

impl ActiveModelBehavior for ActiveModel {}

#[taurpc::ipc_type]
#[derive(Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct SourceDTO {
    pub source_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub parent: Option<MediaItemDTO>,
    pub provider_id: Uuid,
}

#[inherent]
impl Convertible<ModelEx> for SourceDTO {
    type DTO = SourceDTO;

    pub fn from_model(item: ModelEx) -> ConversionResult<Self> {
        Ok(SourceDTO {
            source_id: item.source_id,
            parent_id: item.parent_id,
            parent: MediaItemDTO::option_from_option(item.parent)?,
            provider_id: item.provider_id,
        })
    }
}
