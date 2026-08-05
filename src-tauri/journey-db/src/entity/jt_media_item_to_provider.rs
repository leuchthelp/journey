use Uuid;
use sea_orm::entity::prelude::*;

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "jt_media_item_to_provider")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub media_item_id: Uuid,
    #[sea_orm(primary_key, auto_increment = false)]
    pub provider_id: Uuid,
    #[sea_orm(belongs_to, from = "media_item_id", to = "uuid")]
    pub media_item: BelongsTo<super::media_items::Entity>,
    #[sea_orm(belongs_to, from = "provider_id", to = "user_id")]
    pub provider: BelongsTo<super::providers::Entity>,
}

impl ActiveModelBehavior for ActiveModel {}
